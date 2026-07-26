# Risoluzione Problemi Benchmark C++

Durante l'analisi del benchmark C++, sono emersi e stati risolti alcuni bug architetturali nel motore di Name Resolution, che causavano l'assenza di ben 17 archi (di cui ne sono stati recuperati 15).

Ecco il riepilogo dei bug identificati e delle soluzioni generali adottate:

### 1. Risoluzione dei namespace non funzionante per i "using namespace" in C++
**Problema:** L'istruzione `using namespace Transport;` in C++ veniva correttamente parserizzata dall'AST, ma l'analizzatore non estraeva le regole di import perché la struttura AST di `using_declaration` era diversa da quelle tipiche (come `import` di Python o `use` di Rust) e non veniva matchata. Di conseguenza, il resolver non sapeva dove cercare i nomi brevi (es. `Car`), cercando invano nel modulo globale invece che dentro il namespace `Transport`.
**Soluzione (Generale):** Il metodo `try_parse_imports` è stato aggiornato per includere e gestire i nodi `using_declaration`. Inoltre, è stato modificato per contrassegnare `is_wildcard = true` (es. `using namespace X;`), cosicché il resolver esamini tutto il modulo importato quando cerca un simbolo.

### 2. Disallineamento dell'ereditarietà dei sub-modules (Language Context)
**Problema:** Quando si estraevano i moduli interni (es. i blocchi `namespace` in C++), il contesto del linguaggio (l'informazione che indicava "questo codice è C++") andava perso, risultando in un `lang_name` nullo. Ciò faceva fallire alcune strategie specifiche per linguaggio durante l'analisi successiva.
**Soluzione (Generale):** È stata aggiornata la firma di `try_parse_module_node` affinché riceva e mantenga il `lang_name` da propagare ai sub-modules. In questo modo tutti i namespace ereditano correttamente le impostazioni del linguaggio originario.

### 3. Modelli C++ (Templates) interpretati come tipi incompleti
**Problema:** La dichiarazione `std::vector<Car> cars` veniva troncata dall'estrattore dei tipi prima della `<`, risultando in un tipo `["std", "vector"]` ed ignorando totalmente il parametro generico `<Car>`. Ciò portava a non rilevare le dipendenze di composizione (l'arco `Fleet.cars -> Transport.Car`).
**Soluzione (Generale):** L'estrattore di tipo `extract_type_ref` è stato esteso per ispezionare opzionalmente il contenuto dei nodi `template_type`. Quando si rileva la sintassi dei generici (`<...>`), l'estrattore restituisce un `TypeRef::Union` che include sia il tipo contenitore (es. `std::vector`) sia i tipi generici (es. `Car`).

### 4. Dichiarazioni implicite (loop `for`) trascurate per i behavioral dependencies
**Problema:** In cicli iterativi range-based (es. `for (auto& car : cars)`), la variabile a sinistra (`car`) veniva correttamente estratta come variabile locale, ma non si stava analizzando la collezione di destinazione (`cars`) alla sua destra per trovare le dipendenze comportamentali (accessi ai campi/collezioni). Inoltre, si ignorava il corpo del ciclo per la ricerca di blocchi nidificati.
**Soluzione (Generale):** La funzione `extract_block` è stata estesa per considerare `for_range_loop` e `for_in_statement` come definizioni di variabili, assicurando l'ispezione della variabile iterata (nell'attributo `right`) e l'attraversamento ricorsivo del blocco `body`. 

---

### Archi Attualmente Mancanti e Motivi (2 rimasti su 17)

Attualmente, il benchmark riporta il ritrovamento di 30 archi su 32, fallendo su soli due:

1. **`Fleet.startAll -> Transport.Car.displayInfo` (Calls)**
   * **Motivo Reale:** Nel codice sorgente `for (auto& car : cars) { car.displayInfo(); }`, il tipo della variabile `car` è esplicitamente `auto`. Il nostro resolver si basa fortemente sui tipi statici espliciti per la risoluzione (Name Resolution base).
   * **Cosa Manca:** Per dedurre che `car` ha tipo `Car`, l'analizzatore dovrebbe essere in grado di implementare algoritmi di *Data-Flow Analysis / Type Inference Avanzata*, andando a vedere il tipo che popola l'array `cars` (ovvero `std::vector<Car>`) e deducendone l'identità per `auto&`. Questo eccede le capacità attuali delle euristiche stateless, per questo l'arco non può essere ricavato se non introducendo un level of analysis decisamente più complesso e dispendioso, o con un framework dedicato al typing di quel linguaggio (oltrepassando il concetto di "agnostic").
   
2. **`main -> Transport.Vehicle` (UsesType)**
   * **Motivo Reale:** Il codice della funzione `main()` instanzia un oggetto `Car`, ma non fa alcun riferimento diretto a `Vehicle`. C++ richiama internamente i costruttori della classe madre in fase di allocazione o inclusione, e il file di test sembra aspettarsi questo legame indiretto come "dipendenza di tipo" di `main`.
   * **Cosa Manca:** Trattandosi di un legame del tutto invisibile nell'AST del file `main.cpp` (essendo implicito all'architettura dei costruttori della libreria o indotto dagli header C++), l'analizzatore basato su AST generici non ha modo di desumerlo senza esplorare il file di header della classe `Car` ed emulare le regole C++ di istanziazione degli oggetti polimorfici, il che contrasterebbe con l'approccio *language-agnostic*. Probabilmente l'aspettativa del benchmark va rivista.
