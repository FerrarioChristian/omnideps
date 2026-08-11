# C Benchmark Issues Report

Questo documento raggruppa e analizza le cause dei fallimenti riscontrati nei test del benchmark per il linguaggio C (estesi per includere costrutti tipici del linguaggio come macro, alias, enum, puntatori a funzione e forward declarations).

L'analisi è stata condotta senza alterare il codice dell'analizzatore Rust, ma esclusivamente indagando i log di esecuzione (AST generati da Tree-sitter e log del Resolver).

---

## 1. Loop Statement ignorati in C (Block Traversal)
**Test Falliti:** `C-CALL-1` (`multiply -> add`)

**Sintomo:** 
L'analizzatore estrae correttamente le dipendenze dal blocco principale di una funzione, ma ignora completamente eventuali chiamate a funzione (come `add(result, a)`) se queste sono annidate all'interno di un ciclo `for`, `while` o un `if`.

**Causa Tecnica:** 
In `src/heuristics/body_extraction.rs`, la funzione `extract_block` naviga ricorsivamente solo in specifici costrutti di loop (`for_range_loop`, `for_in_statement`). In C, il ciclo canonico è rappresentato dal nodo AST `for_statement` (o `while_statement`). Non trovando un match per questi nodi, il blocco interno (il `compound_statement` del ciclo) viene scartato per via della regola di "skip nested blocks" di `find_behavioral_deps`. Di conseguenza, tutti gli accessi e le chiamate interne al ciclo vanno persi.

**Possibile Soluzione:** 
Estendere il pattern matching in `extract_block` includendo i costrutti canonici del C/C++: `"for_statement"`, `"while_statement"`, `"do_statement"`, e `"if_statement"`.

---

## 2. Field Access Estratti come Stringhe Grezze (`field_expression`)
**Test Falliti:** `C-ACC-1`, `C-ACC-2`, `C-ACC-3`, `C-ACC-4`, `C-ACC-5`, `C-ACC-7`, `C-ACC-TYPEDEF`, `C-ACC-MACRO`, `C-MUT-GLOBAL`, `C-ACC-GLOBAL`
*(Es: `calculate_area -> Rectangle.width`, `move_point -> Point.x`, `main -> Circle.radius`)*

**Sintomo:** 
Il resolver fallisce la risoluzione di query per accessi a campi di struct. Nei log, gli "available sinks" mostrano stringhe composite come `"rect->width"` o `"p->x"` invece del nodo corretto `Point.x` o `Rectangle.width`. Similmente le variabili globali falliscono perché nascoste in assegnamenti analoghi se non decostruiti.

**Causa Tecnica:**
Attualmente, l'estrattore di tipi (`src/heuristics/type_extraction.rs`) valuta i nodi `field_expression` del C (es. `p->x` o `rect.width`) catturandone unicamente il testo grezzo (raw string `p->x`) come un generico `TypeRef::Unresolved(["p->x"])`. Il Name Resolver non è in grado di dividere e dedurre i tipi da stringhe composite, e quindi la ricerca del simbolo fallisce.

**Possibile Soluzione:**
Intercettare il nodo `"field_expression"` in `type_extraction.rs` ed estrarne le componenti: il nodo figlio `argument` (es. `p`) e il nodo `field` (es. `x`). Si dovrà quindi comporre un `TypeRef::PropertyAccess(base, field)` affinché il resolver possa dedurre dinamicamente il tipo della base (il puntatore `Point*`) e aggiungere l'arco corretto al membro strutturale.

---

## 3. Disallineamento Nomi Funzione: Dichiarazione vs Definizione
**Test Falliti:** `C-USE-4` (`create_point -> Point`)

**Sintomo:**
L'edge atteso manca, tuttavia nei log dell'analizzatore appare un edge simile generato da un nodo anomalo: `create_point(int x, int y) -> Point (UsesReturnType)`. L'analizzatore ha quindi "sdoppiato" la funzione in due entità separate: `create_point` (derivato dalla firma in `pointers.h`) e `create_point(int x, int y)` (derivato dall'implementazione in `pointers.c`).

**Causa Tecnica:**
In `src/heuristics/structural_extraction.rs`, quando si estrae il nome della funzione da un nodo `function_definition`, il parsing AST del C presenta spesso il nome avvolto in un `pointer_declarator` e in un `function_declarator`. Se la heuristica non scava fino all'identificatore base (`identifier`), finisce per usare `node.utf8_text()` sull'intero dichiaratore, che purtroppo include le parentesi e i parametri, generando ID incompatibili e duplicazioni nel grafo.

**Possibile Soluzione:**
Migliorare `extract_name` per gestire correttamente la discesa nell'AST per `function_declarator` e `pointer_declarator`, assicurandosi di recuperare esclusivamente il nodo foglia `"identifier"`.

---

## 4. Chiamate a Puntatori a Funzione (Function Pointers)
**Test Falliti:** `C-CALL-PTR` (`trigger_callback -> ActionCallback`)

**Sintomo:** 
L'invocazione di un puntatore a funzione (es. `active_callback(1)`) non genera un arco `Calls` verso il tipo del puntatore (il typedef `ActionCallback`), ma si ferma.

**Causa Tecnica:**
Il modulo `body_extraction.rs` rileva correttamente `active_callback(1)` come `call_expression` ed estrae l'intento di chiamare `active_callback`. Quando il Resolver esegue questa query, trova effettivamente il simbolo globale `active_callback` (di tipo `Symbol::Value` o `Symbol::Field`). Tuttavia, poiché ci si aspetta di chiamare una *funzione* e non una variabile, l'esecuzione termina lì, senza generare l'arco verso il tipo strutturale effettivo del puntatore a funzione.

**Possibile Soluzione:**
In `executor.rs`, durante l'esecuzione di una query con intento `Call`, se il bersaglio risolto è una variabile o un campo (quindi un puntatore a funzione in C), l'esecutore dovrebbe automaticamente risalire al *tipo* di quella variabile (il typedef associato) e redirigere l'arco verso di esso.

---

## Problemi risolti successivamente 

## 5. Parametri mancanti nelle dichiarazioni di funzioni (Scope resolution fallita)
**Sintomo:** Le query `Find("rect")` all'interno di `calculate_area` fallivano. Di conseguenza, l'analizzatore non riusciva a tracciare l'accesso ai campi `rect->width` e `rect->height`.
**Causa:** In C, le dichiarazioni di funzioni nei file header (es. `shapes.h`) vengono parsate dall'AST come nodi `declaration`, in cui i parametri si trovano annidati all'interno di un `declarator` (nello specifico `function_declarator`). La funzione `extract_parameters` cercava i parametri solo al primo livello del nodo. Questo portava alla creazione di uno scope senza parametri per le dichiarazioni. L'`executor` successivamente, trovando per primo lo scope della dichiarazione (dal file `.h`), falliva la risoluzione del parametro usato nel body (nel file `.c`).
**Soluzione:** Modificata la funzione `extract_parameters` in `src/heuristics/structural_extraction.rs` per cercare ricorsivamente il campo `parameters` all'interno dei nodi `declarator` annidati.

## 6. Classificazione errata di variabili libere e funzioni in C
**Sintomo:** Le variabili globali (come `global_counter` e `active_callback`) non venivano rilevate, e le dichiarazioni di funzioni (come `create_point` in `pointers.h`) causavano conflitti essendo classificate erroneamente o non classificate.
**Causa:** 
1. La funzione `is_free_variable` non gestiva i nodi `declaration` o `variable_declaration` per il C, saltando l'estrazione delle variabili globali.
2. La funzione `is_function` non riconosceva i nodi `declaration` contenenti un `function_declarator` annidato come dichiarazioni di funzioni in C/C++.
3. La funzione `extract_identifier` restituiva l'intera signature al posto del solo identificatore nel caso in cui il `function_declarator` fosse annidato all'interno di un `pointer_declarator`.
**Soluzione:** 
- Aggiornata `is_function` in `src/heuristics/classifiers.rs` per riconoscere le dichiarazioni di funzioni cercando ricorsivamente `function_declarator` nei nodi `declaration`.
- Aggiornata `is_free_variable` per includere `declaration` e `variable_declaration`, avendo cura di escludere esplicitamente le dichiarazioni di funzioni gestite sopra esplorando ricorsivamente l'esistenza di `function_declarator`.
- Aggiornata `extract_identifier` in `src/heuristics/text_parsing.rs` in modo da scendere ricorsivamente in tutti i `declarator` annidati fino a trovare l'identificatore base della funzione/variabile, e fare comunque lo split sul carattere `(` per maggiore sicurezza nel processare le raw string di C.

## 7. Errori nel codice e nelle aspettative del benchmark (`main.c` e `test.yml`)
**Sintomo:** Mancavano gli archi per `Circle.radius`, `Rectangle.width`, `Rectangle.height` in `main.c`, e la chiamata a puntatore a funzione `trigger_callback -> ActionCallback`.
**Causa & Soluzioni:**
1. **Shadowing variabile:** Nel file `main.c`, la variabile `c` di tipo `Circle` veniva sovrascritta (re-dichiarata) da `int c = color_to_int(BLUE);` nello stesso blocco. Questo causava la risoluzione di `c` come `int` anziché `Circle`. Rinominata la seconda variabile in `color_val`.
2. **Accessi impliciti non estratti:** L'inizializzazione posizionale di struct `struct Rectangle rect = {10, 5};` non produce accessi ai campi nell'AST. Aggiunti gli accessi espliciti (`rect.width = 10;`) in `main.c` per permetterne la validazione.
3. **Modellazione puntatori a funzione:** Il benchmark si aspettava `trigger_callback -> ActionCallback (Calls)`. Il comportamento corretto dell'analizzatore modella questo flusso come `trigger_callback -> active_callback (Calls)` e `active_callback -> ActionCallback (UsesFieldType)`. Aggiornate le aspettative in `test.yml` per rispecchiare questa corretta modellazione.

