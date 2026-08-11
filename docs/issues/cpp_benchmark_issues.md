# Risoluzione Problemi Benchmark C++

Durante l'analisi del benchmark C++, sono emersi e stati risolti alcuni bug architetturali nel motore di Name Resolution, che causavano l'assenza di ben 17 archi (di cui ne sono stati recuperati 16).

## Riepilogo

| # | Problema | Gravità | Status e Spiegazione Soluzione |
| --- | ---------- | --------- | -------------------------------- |
| 1 | Risoluzione namespace non funzionante | 🟢 Risolto | Gestione del nodo `using_declaration` come import wildcard per espandere lo scope. |
| 2 | Perdita contesto lingua nei sub-modules | 🟢 Risolto | Propagazione del `lang_name` durante la discesa nei blocchi `namespace`. |
| 3 | Templates interpretati scorrettamente | 🟢 Risolto | Discesa opzionale nei nodi `template_type` per generare `TypeRef::Union` con i tipi generici. |
| 4 | Loop for-range trascurati | 🟢 Risolto | Attraversamento esplicito di `for_range_loop` e `for_in_statement` nei behavioral deps. |
| 5 | Metodi Out-of-Line scollegati | 🟢 Risolto | Implementato Global Scope Fallback per ricollegare l'ImplBlock alla classe target, ed estrazione di `argument` per le field expressions. |
| 6 | Mancanza invocazioni implicite | 🟡 Non Risolto | Costruttori della superclasse invocati implicitamente (es. `Car` -> `Vehicle`). |
| 7 | Configurazione Moduli e Estensioni | 🟢 Risolto | Mappatura estensioni `.h` al parser C++ e attivazione flag `implicit_file_modules` per scope gerarchico. |
| 8 | Popolamento Local Scope per Variabili | 🟢 Risolto | Estrazione gerarchica dei blocchi che inserisce variabili locali nel `SymbolStack` per permettere la risoluzione locale (es. `rect.width`). |

---

## 🟢 Problema 1: Risoluzione dei namespace non funzionante per i "using namespace" in C++

**Sintomo:** L'istruzione `using namespace Transport;` in C++ veniva correttamente parserizzata dall'AST, ma l'analizzatore non estraeva le regole di import perché la struttura AST di `using_declaration` era diversa da quelle tipiche (come `import` di Python o `use` di Rust) e non veniva matchata. Di conseguenza, il resolver non sapeva dove cercare i nomi brevi (es. `Car`), cercando invano nel modulo globale invece che dentro il namespace `Transport`.

**Soluzione Implementata:** Il metodo `try_parse_imports` è stato aggiornato per includere e gestire i nodi `using_declaration`. Inoltre, è stato modificato per contrassegnare `is_wildcard = true` (es. `using namespace X;`), cosicché il resolver esamini tutto il modulo importato quando cerca un simbolo, esattamente come farebbe il compilatore C++.

---

## 🟢 Problema 2: Disallineamento dell'ereditarietà dei sub-modules (Language Context)

**Sintomo:** Quando si estraevano i moduli interni (es. i blocchi `namespace` in C++), il contesto del linguaggio (l'informazione che indicava "questo codice è C++") andava perso, risultando in un `lang_name` nullo. Ciò faceva fallire alcune strategie specifiche per linguaggio durante l'analisi successiva.

**Soluzione Implementata:** È stata aggiornata la firma di `try_parse_module_node` affinché riceva e mantenga il `lang_name` da propagare ai sub-modules. In questo modo tutti i namespace ereditano correttamente le impostazioni del linguaggio originario.

---

## 🟢 Problema 3: Modelli C++ (Templates) interpretati come tipi incompleti

**Sintomo:** La dichiarazione `std::vector<Car> cars` veniva troncata dall'estrattore dei tipi prima della `<`, risultando in un tipo `["std", "vector"]` ed ignorando totalmente il parametro generico `<Car>`. Ciò portava a non rilevare le dipendenze di composizione (l'arco `Fleet.cars -> Transport.Car`).

**Soluzione Implementata:** L'estrattore di tipo `extract_type_ref` è stato esteso per ispezionare opzionalmente il contenuto dei nodi `template_type`. Quando si rileva la sintassi dei generici (`<...>`), l'estrattore restituisce un `TypeRef::Union` che include sia il tipo contenitore (es. `std::vector`) sia i tipi generici (es. `Car`), garantendo che il Name Resolver emetta archi verso tutte le entità coinvolte.

---

## 🟢 Problema 4: Dichiarazioni implicite (loop `for`) trascurate per i behavioral dependencies

**Sintomo:** In cicli iterativi range-based (es. `for (auto& car : cars)`), la variabile a sinistra (`car`) veniva correttamente estratta come variabile locale, ma non si stava analizzando la collezione di destinazione (`cars`) alla sua destra per trovare le dipendenze comportamentali (accessi ai campi/collezioni). Inoltre, si ignorava il corpo del ciclo per la ricerca di blocchi nidificati.

**Soluzione Implementata:** La funzione `extract_block` è stata estesa per considerare `for_range_loop` e `for_in_statement` come definizioni di variabili, assicurando l'ispezione della variabile iterata (nell'attributo `right`) e l'attraversamento ricorsivo del blocco `body`. 

---

## 🟢 Problema 5: Definizione di Metodi Out-of-Line (C++)

**Sintomo:** In C++ è prassi dichiarare una classe in un file header (`.h`) e definirne i metodi in un file sorgente (`.cpp`) usando l'operatore di risoluzione di scope (es. `void Fleet::addCar() { ... }`). L'AST estrae queste definizioni come funzioni libere scollegate dalla classe originale, e gli accessi ai membri (come la lista `cars`) fallivano (mancava l'arco `Fleet.addCar -> Fleet.cars`). Inoltre, le chiamate a metodi (es. `cars.push_back()`) non estraevano l'oggetto ricevitore perché in C++ il nodo AST `field_expression` usa la chiave `argument` invece che `object`.

**Soluzione Implementata:** 
1. In `src/resolver/scope.rs`, quando l'engine processa un `ImplBlock` per un metodo out-of-line, ora esegue un **lookup globale (Global Scope Fallback)** nello `ScopeTree` per trovare la classe originale (che risiede nel modulo dell'header) e innesca il metodo direttamente all'interno dello scope di quella classe, ereditandone così tutta la visibilità lessicale.
2. In `src/heuristics/body_extraction.rs`, l'estrazione del "receiver" delle chiamate di metodo è stata estesa per ispezionare il campo `argument` nativo dei `field_expression` C/C++.

---

## 🟢 Problema 7: Configurazione Moduli e Estensioni File
**Sintomo:** Inizialmente i file header `.h` venivano mappati al parser C puro anziché C++. Questo impediva il riconoscimento corretto delle classi C++ (come `class Vehicle`). Inoltre, le configurazioni non creavano i namespace corretti.
**Soluzione Implementata:** La mappatura delle estensioni è stata aggiornata per interpretare i `.h` all'interno di progetti C++ correttamente usando il parser `cpp`. Sono state anche attivate le flag `implicit_file_modules` e `file_level_declarations` per allineare l'estrazione allo ScopeTree gerarchico.

---

## 🟢 Problema 8: Popolamento Local Scope per Variabili e Parametri
**Sintomo:** Gli accessi ai campi (es. `myCar.speed`) fallivano in fase di Name Resolution perché l'analizzatore non teneva traccia delle istanziazioni locali.
**Soluzione Implementata:** Grazie allo `ScopeTree` e all'estrazione gerarchica dei blocchi, l'estrattore popola lo scope locale della funzione con parametri e variabili. Il `SymbolStack` valuta così la variabile `myCar` risolvendola in `Car`, e innescando con successo l'arco di accesso a campi e metodi.

---

## 🟡 Problema 6: L'unico arco attualmente mancante (1/36 falliti)

Dopo l'implementazione dei fix, il benchmark C++ ha una percentuale di successo di circa il **97% (35 archi trovati su 36)**. 
L'unico arco che rimane fallito è strutturale al funzionamento di C++:

### `main -> Transport.Vehicle` (UsesType)
* **Sintomo:** Il benchmark si aspetta che la funzione `main` abbia una dipendenza (UsesType) verso la classe base `Vehicle`. 
* **Codice nel main:** `Car myCar("Toyota", 120, 4);`
* **Analisi della causa:** La funzione `main()` alloca nello stack un oggetto `Car`. Nel linguaggio C++, l'allocazione di un oggetto derivato comporta un'invocazione implicita al costruttore della superclasse (`Vehicle`). Tuttavia, nel file `main.cpp` non è presente **alcun riferimento testuale o nodo AST** che citi la parola `Vehicle`. Il parser Tree-sitter per il file `main.cpp` vede solo il token `Car`.
* **Motivo di non risoluzione:** Il nostro strumento è un analizzatore **agnostico** basato sull'estrazione dell'AST testuale e sulla risoluzione dei nomi esplicitati. Non incorpora un compilatore completo per ogni linguaggio. Risolvere questo arco richiederebbe che il resolver modifichi a posteriori il contenuto del blocco `main`, aggiungendovi chiamate che non sono presenti nel codice originale, dopo aver dedotto che `Car` eredita da `Vehicle`. Questa funzionalità emulativa ("deep inference" dei costruttori impliciti) esula dagli scopi di un analizzatore architetturale agnostico, ed è stata perciò volutamente tralasciata per preservare la semplicità dell'astrazione logica.
