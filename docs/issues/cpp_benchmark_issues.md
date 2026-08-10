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

### 5. Definizione di Metodi Out-of-Line (C++)
**Problema:** In C++ è prassi dichiarare una classe in un file header (`.h`) e definirne i metodi in un file sorgente (`.cpp`) usando l'operatore di risoluzione di scope (es. `void Fleet::addCar() { ... }`). L'AST estrae queste definizioni come funzioni libere scollegate dalla classe originale, e gli accessi ai membri (come la lista `cars`) fallivano (mancava l'arco `Fleet.addCar -> Fleet.cars`). Inoltre, le chiamate a metodi (es. `cars.push_back()`) non estraevano l'oggetto ricevitore perché in C++ il nodo AST `field_expression` usa la chiave `argument` invece che `object`.
**Soluzione (Generale):** 
1. In `src/resolver/scope.rs`, quando l'engine processa un `ImplBlock` per un metodo out-of-line, ora esegue un **lookup globale** nello `ScopeTree` per trovare la classe originale (che risiede nel modulo dell'header) e innesca il metodo direttamente all'interno dello scope di quella classe, ereditandone così tutta la visibilità.
2. In `src/heuristics/body_extraction.rs`, l'estrazione del "receiver" delle chiamate di metodo è stata estesa per leggere anche il campo `argument` dai `field_expression`.

---

### Archi Attualmente Mancanti e Motivi (1 rimasto su 32)

Dopo l'implementazione dei fix, il benchmark riporta il ritrovamento di **31 archi su 32**. 
L'arco inizialmente indicato come mancante `Fleet.startAll -> Transport.Car.displayInfo` è stato **pienamente risolto** grazie alla corretta associazione dei generics (punto 3). L'arco `Fleet.addCar -> Fleet.cars` è stato parimenti **risolto** (punto 5).

L'unico arco che rimane fallito è:
   
1. **`main -> Transport.Vehicle` (UsesType)**
   * **Motivo Reale:** Il codice della funzione `main()` istanzia un oggetto `Car`, ma non fa alcun riferimento diretto a `Vehicle`. In C++ i costruttori della superclasse vengono invocati implicitamente durante l'allocazione, e il benchmark si aspetta questo legame indiretto come "dipendenza di tipo" del `main`.
   * **Perché non possiamo risolverlo staticamente:** Essendo questo legame del tutto invisibile nell'AST del file `main.cpp` (poiché dipendente dall'albero di ereditarietà definito nell'header di `Car`), un analizzatore AST agnostico non può dedurlo senza un motore di inferenza profondo che emuli le regole C++ di costruzione polimorfica. Questa funzionalità è out-of-scope per le euristiche attuali, quindi l'arco è strutturalmente impossibile da risolvere staticamente senza snaturare l'approccio agnostico del tool.
