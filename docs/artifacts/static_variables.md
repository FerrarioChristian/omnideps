# Architettura dell'Estrattore e Variabili Statiche

Questo documento documenta l'architettura modulare dell'estrattore di dipendenze (ispirata al Single Responsibility Principle) e l'implementazione del tracciamento per le Variabili Statiche e Globali.

---

## 1. Modularizzazione dell'Estrattore (Clean Code Refactoring)

Per garantire la manutenibilità e l'estendibilità dell'analizzatore, la logica di estrazione è stata segmentata in unità funzionali isolate.

### Modulo `body_extraction.rs`
Le procedure storicamente centralizzate in `find_behavioral_deps` sono state scomposte in funzioni specializzate:

- **`parse_token_tree_macro`**: Isola l'automa a stati finito responsabile del Token Coalescing. Utilizzato per analizzare i nodi `token_tree` (es. chiamate a macro in Rust come `println!`), scorre l'albero sintattico per ricostruire i path qualificati e registrarne gli accessi e le chiamate, sollevando la funzione principale dalle ricorsioni più profonde.
- **`extract_call_dependency`**: Incapsula la logica di estrazione per le chiamate a funzione e a metodi (inclusi i metodi statici tramite `scoped_identifier`). L'astrazione di un nodo `call_expression` viene interamente delegata a questa funzione.

### Modulo `type_extraction.rs`
- **`try_extract_from_type_field`**: Isola l'ispezione dei field di Tree-sitter associati storicamente alla definizione dei tipi (`type`, `return_type`, `field_type`, `value_type`). Al rilevamento di tali campi, la funzione avvia l'estrazione ricorsiva, semplificando notevolmente il control flow primario in `extract_type_ref`.

> [!TIP]
> Questa struttura modulare agevola drasticamente l'introduzione di regole di parsing specifiche per nuovi linguaggi e l'integrazione di nuove euristiche, circoscrivendo il raggio d'azione di ciascuna modifica.

---

## 2. Tracciamento delle Variabili Statiche e Globali (Free Variables)

L'identificazione e l'estrazione delle variabili statiche sono state implementate in modo interamente *language-agnostic*.

### Architettura e Implementazione
1. **Model** (`src/model/components.rs`): Il modello `Module` include il vettore `free_variables: Vec<Field>`. Questo costrutto modella la capacità del modulo di possedere variabili globali proprie, esterne a tipi strutturati (es. classi o struct).
2. **Heuristics e Parsing**:
   - In `classifiers.rs`, la funzione `is_free_variable` intercetta nodi sintattici come `static_item` e `const_item` (Rust) o `global_variable_declaration`.
   - In `parsers.rs`, la procedura `try_parse_free_variable` mappa queste entità nella struct logica `Field`.
3. **Core Analyzer** (`src/analyzer.rs`): Le istanze di `ParsedItem::Component(Component::Field)` vengono iniettate direttamente nel pool delle variabili globali del `Module` contenitore.
4. **Resolution Phase** (`src/resolver/`):
   - Le *free_variables* sono integrate nell'albero gerarchico `ScopeTree` alla pari di moduli, struct e funzioni.
   - `builder.rs` ed `executor.rs` gestiscono la risoluzione algebrica. Un'istruzione che accede a una variabile globale innesca il *Lexical Climbing*: l'executor risale lo `ScopeTree` fino al modulo d'origine, identifica il campo e genera le dipendenze in modo del tutto trasparente.
5. **Esportazione Grafo** (`export/graph.rs`): Le query vengono srotolate in vertici Cytoscape (con classificazione Type = *Field*). Archi di tipo `UsesFieldType` vengono generati per associare il campo alla sua definizione tipizzata.

> [!NOTE]
> Grazie all'unificazione architettonica tramite ScopeTree, l'analizzatore mappa costrutti complessi (es. `STATIC_SA.x`) instradando coerentemente l'accesso prima alla variabile globale `STATIC_SA`, poi alla definizione della struct base, e infine al campo `x`. Questa pipeline logica garantisce zero perdite di precisione ed è coperta esaustivamente dalla suite di benchmark automatizzata.
