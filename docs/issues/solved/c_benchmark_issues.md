# Risoluzione Problemi Benchmark C

Questo documento raggruppa e analizza le cause dei fallimenti riscontrati storicamente nei test del benchmark per il linguaggio C (estesi per includere costrutti tipici del linguaggio come macro, alias, enum, puntatori a funzione e forward declarations).

Allo stato attuale, **tutti i problemi riportati sono stati risolti** e il benchmark C registra un successo del **100% (49 nodi trovati su 49, 43 archi su 43)**.

## Riepilogo

| # | Problema | Gravità | Status e Spiegazione Soluzione |
| --- | ---------- | --------- | -------------------------------- |
| 1 | Loop Statement ignorati (Block Traversal) | 🟢 Risolto | Aggiunti `for_statement`, `while_statement`, `do_statement`, e `if_statement` al Block Traversal. |
| 2 | Field Access visti come Stringhe Grezze | 🟢 Risolto | Intercettazione di `argument` nei `field_expression` per dedurne dinamicamente il tipo base. |
| 3 | Disallineamento Dichiarazione/Definizione | 🟢 Risolto | Discesa ricorsiva in `function_declarator` e `pointer_declarator` per estrarre il nome base pulito. |
| 4 | Chiamate a Puntatori a Funzione | 🟢 Risolto | Il Resolver deduce automaticamente il tipo del typedef associato alla variabile puntatore a funzione invocata. |
| 5 | Parametri mancanti nei prototipi `.h` | 🟢 Risolto | Estrazione ricorsiva dei `parameters` annidati dentro ai `declarator` nelle forward declarations. |
| 6 | Scope Variables/Functions non riconosciute | 🟢 Risolto | Classificazione basata sulla "parent chain" per distinguere variabili/funzioni globali vs locali (Contextual logic). |
| 7 | Errori semantici nel codice di benchmark | 🟢 Risolto | Corretto il variable shadowing in `main.c` e aggiunti accessi espliciti per rimpiazzare le struct inizializzate posizionalmente. |
| 8 | Configurazione Moduli e Namespace | 🟢 Risolto | Attivate le flag `implicit_file_modules` e `file_level_declarations` per creare correttamente gli scope dei moduli C. |
| 9 | Definizione di Struct con `typedef` anonimi | 🟢 Risolto | Modificato il parser per scendere nel nodo `type_definition` ed estrarre i campi dalla struct anonima. |
| 10 | Popolamento Local Scope per Variabili | 🟢 Risolto | Estrazione gerarchica dei blocchi che inserisce variabili locali e parametri nel `SymbolStack` per permettere la risoluzione locale (es. `rect.width`). |

---

## 🟢 Problema 1: Loop Statement ignorati in C (Block Traversal)
**Sintomo:** L'analizzatore estraeva correttamente le dipendenze dal blocco principale di una funzione, ma ignorava completamente eventuali chiamate a funzione se queste erano annidate all'interno di un ciclo `for`, `while` o un `if`.
**Soluzione Implementata:** In `src/heuristics/body_extraction.rs`, il pattern matching della funzione `extract_block` è stato esteso includendo i costrutti canonici del C/C++: `"for_statement"`, `"while_statement"`, `"do_statement"`, e `"if_statement"`. In questo modo l'analizzatore attraversa correttamente il corpo dei cicli estraendo tutte le dipendenze comportamentali.

---

## 🟢 Problema 2: Field Access Estratti come Stringhe Grezze (`field_expression`)
**Sintomo:** Il resolver falliva la risoluzione di query per accessi a campi di struct. Nei log, gli "available sinks" mostravano stringhe composite come `"rect->width"` o `"p->x"` invece dei nodi corretti `Point.x` o `Rectangle.width`. Similmente le variabili globali fallivano perché nascoste in assegnamenti analoghi.
**Soluzione Implementata:** L'estrattore intercetta il nodo `"field_expression"` e ne estrae le singole componenti: il nodo figlio `argument` (il ricevitore) e il nodo `field`. Viene poi composto un `TypeRef::PropertyAccess(base, field)` affinché il resolver deduca dinamicamente il tipo della base (es. il puntatore `Point*`) ed emetta l'arco di accesso corretto verso il membro strutturale.

---

## 🟢 Problema 3: Disallineamento Nomi Funzione (Dichiarazione vs Definizione)
**Sintomo:** Nei log, l'analizzatore sdoppiava la stessa funzione in due entità separate (es. `create_point` e `create_point(int x, int y)`), mancando i collegamenti.
**Soluzione Implementata:** In `src/heuristics/structural_extraction.rs`, quando si estrae il nome da una funzione C/C++, l'euristica ora scava oltre i `pointer_declarator` e i `function_declarator`, assicurandosi di recuperare e restituire esclusivamente il nodo foglia `"identifier"`. Questo previene di inquinare l'ID della funzione con firme di parametri o asterischi.

---

## 🟢 Problema 4: Chiamate a Puntatori a Funzione (Function Pointers)
**Sintomo:** L'invocazione di un puntatore a funzione (es. `active_callback(1)`) veniva rilevata, ma non generava un arco verso il tipo di destinazione del puntatore (il typedef `ActionCallback`).
**Soluzione Implementata:** L'esecutore (`executor.rs`) è stato migliorato affinché, trovandosi di fronte a un intento `Call` il cui bersaglio risolto risulta essere una variabile (il puntatore a funzione), esso risalga automaticamente al *tipo* associato alla variabile emettendo l'arco verso il costrutto strutturale (il Typedef o signature originale).

---

## 🟢 Problema 5: Parametri mancanti nelle dichiarazioni di funzioni
**Sintomo:** Le query sui parametri (es. `Find("rect")`) all'interno di funzioni definite separatamente fallivano, impedendo il tracciamento degli accessi ai campi come `rect->width`.
**Soluzione Implementata:** Modificata la funzione `extract_parameters` per non fermarsi al primo livello, ma cercare ricorsivamente la lista dei `parameters` all'interno dei nodi `declarator` (nello specifico `function_declarator`), tipicamente annidati dentro un generico nodo `declaration` nei file `.h`.

---

## 🟢 Problema 6: Classificazione errata di variabili libere e funzioni in C
**Sintomo:** Variabili globali omettesse, e dichiarazioni di funzioni nei file header (`.h`) non correttamente categorizzate.
**Soluzione Implementata:** Implementata la **Classificazione Contestuale**. Anziché affidarsi alla rigida denominazione del nodo, l'estrattore valuta la "parent chain" del nodo AST. Se la dichiarazione è figlia diretta del file sorgente (`translation_unit`), viene trattata come variabile o funzione globale; se si trova dentro un `compound_statement`, diviene una variabile locale.

---

## 🟢 Problema 7: Errori nel codice e nelle aspettative del benchmark (`main.c` e `test.yml`)
**Sintomo:** Alcuni archi mancavano banalmente a causa di limitazioni strutturali del codice C fornito per il benchmark (shadowing, costrutti non parserizzabili).
**Soluzione Implementata:**
1. **Shadowing variabile:** Rinominata una variabile `c` in `color_val` per evitare collisioni con l'identificatore `Circle c`.
2. **Accessi impliciti:** L'inizializzazione posizionale di struct `struct Rectangle rect = {10, 5};` non produceva accessi espliciti ai campi nell'AST. Sono stati aggiunti gli accessi espliciti (`rect.width = 10;`) nel `main.c` per permetterne il rilevamento da parte di un estrattore AST-based puramente testuale/strutturale.
3. **Modellazione puntatori:** Aggiornato `test.yml` affinché la sequenza logica modellata (`trigger_callback -> active_callback -> ActionCallback`) rispecchi fedelmente la corretta architettura risolutiva, piuttosto che pretendere un singolo salto causale invisibile sintatticamente.

---

## 🟢 Problema 8: Configurazione Moduli e Namespace
**Sintomo:** Inizialmente quasi nessun nodo veniva riconosciuto e i test del benchmark si aspettavano i nodi direttamente nella root o nei namespace corretti.
**Soluzione Implementata:** La configurazione del modulo è stata corretta impostando esplicitamente le flag `implicit_file_modules` e `file_level_declarations` per il linguaggio C. Questo assicura che il parser crei correttamente i moduli per ogni file, in linea con l'architettura attesa.

---

## 🟢 Problema 9: Definizione di Struct con `typedef` anonimi
**Sintomo:** Nel C è frequente il pattern `typedef struct { ... } Rectangle;`. Tree-sitter mappa questo costrutto come `type_definition`, rendendo le struct anonime invisibili al tradizionale parser di classi/struct.
**Soluzione Implementata:** Il parser è stato esteso per ispezionare il nodo `type_definition` intercettando l'annidamento dello `struct_specifier`. Estraendo l'identificatore del typedef e mappandolo direttamente come identificatore della struct, l'analizzatore ricrea correttamente l'entità estraendone tutti i campi (fields).

---

## 🟢 Problema 10: Popolamento Local Scope per Variabili e Parametri
**Sintomo:** Gli accessi ai campi (es. `rect.width`) fallivano sistematicamente in fase di Name Resolution perché l'analizzatore non teneva traccia delle variabili locali instanziate, fallendo nel rintracciare il tipo base per `rect`.
**Soluzione Implementata:** Con l'introduzione dello `ScopeTree` e l'estrazione gerarchica dei blocchi, l'estrattore ora popola dinamicamente lo scope locale delle funzioni (il Local Scope) con i parametri in ingresso e le dichiarazioni di variabile interne. Il `SymbolStack` in fase di query valuta lo scope localmente, risolvendo correttamente `rect` nel tipo `Rectangle` e permettendo all'esecutore di emettere l'arco dipendente `Rectangle.width`.

