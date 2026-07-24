# Walkthrough: Clean Code Refactoring e Variabili Statiche

Questo documento riassume le operazioni eseguite storicamente per migliorare la qualità del codice (rispetto dell'SRP - Single Responsibility Principle) nell'estrattore di dipendenze e l'introduzione delle Variabili Statiche.

## Changes Made all'Estrattore

### 1. `body_extraction.rs`
La logica complessa presente in `find_behavioral_deps` è stata scomposta in funzioni più piccole e mirate.

- **Creata `parse_token_tree_macro`:** Tutta la complessa logica a stati (per il Token Coalescing) usata per analizzare i nodi `token_tree` (e.g. macro Rust come `println!`) è stata spostata in una funzione indipendente. La nuova funzione scorre l'albero sintattico per ricostruire i path qualificati e registrarne gli accessi e le chiamate, delegando a `find_behavioral_deps` solo le ricorsioni più profonde.
- **Creata `extract_call_dependency`:** La logica necessaria a risolvere le chiamate a funzione e a metodi (compresi i metodi statici tramite `scoped_identifier`) è stata isolata. Ora `find_behavioral_deps` delega semplicemente l'astrazione di `call_expression` a questa funzione.

### 2. `type_extraction.rs`
È stato migliorato l'isolamento della logica in `extract_type_ref`.

- **Creata `try_extract_from_type_field`:** Questa funzione racchiude l'ispezione dei field più comuni di Tree-sitter associati ai tipi (`type`, `return_type`, `field_type`, `value_type`). Se trova uno di questi campi, naviga ricorsivamente richiamando l'estrazione, permettendo al flusso principale di `extract_type_ref` di risultare più leggibile e snello.

## Validation Results Storici
Queste modifiche non hanno introdotto alcuna regressione nei benchmark automatizzati, confermando che l'estrazione avviene correttamente con una modularità superiore.

> [!TIP]
> Questa struttura modulare rende molto più semplice aggiungere regole specifiche per altri linguaggi, o estendere l'estrattore con nuove euristiche, senza intaccare la funzione generica.

---

## Estrazione di Variabili Statiche e Globali (Static / Free Variables)

Abbiamo implementato l'identificazione e l'estrazione delle variabili statiche in modo completamente *language agnostic*.

### Modifiche Apportate
1. **Model** (`src/model/components.rs`): Abbiamo esteso la struct `Module` introducendo un nuovo vettore `free_variables: Vec<Field>`. Questo per mantenere la coerenza con le funzioni globali e permettere al modulo di possedere variabili proprie non racchiuse in tipi strutturati.
2. **Heuristics e Parsing**:
   - In `classifiers.rs` abbiamo introdotto la funzione `is_free_variable`, che identifica nodi come `static_item` e `const_item` (Rust), oppure `global_variable_declaration`.
   - In `parsers.rs` è stata creata `try_parse_free_variable`, che astrae queste variabili sottoforma del tipo `Field`.
3. **Core e Analyzer** (`src/analyzer.rs`): L'estrattore assegna direttamente le istanze di `ParsedItem::Component(Component::Field)` alle variabili globali del `Module` contenitore.
4. **Resolution Phase** (`src/resolver/`):
   - Le *free_variables* vengono ora incapsulate come nodi a sé stanti all'interno dell'architettura gerarchica `ScopeTree`, alla pari di moduli, struct e funzioni.
   - Il `builder.rs` e l'`executor.rs` effettuano la risoluzione algebrica di tali variabili. Quando una funzione accede a una variabile globale (es. `STATIC_SA.x`), l'executor scala lo `ScopeTree` fino al modulo genitore, trova il campo globale, e deduce le derivazioni architetturali.
5. **Esportazione Grafo** (`export/graph.rs`): Le query algebriche vengono ora srotolate in vertici (Type: *Field*) e si costruiscono i relativi archi `UsesFieldType` per associarne i tipi strutturali base.
6. **Visualizzatore Web**: 
   - Modificato `cytoscape_style.js` e `legend.html` per introdurre graficamente le *Free Variables* come "Nodi a forma di diamante giallo".

### Impatto (Benchmark Rust)
L'aggiunta del supporto alle variabili globali e al Local Scope ha permesso al sistema di intercettare le corrette dipendenze di tipo access-field e usages di macro.

> [!NOTE]
> L'analizzatore valuta l'uso di `STATIC_SA.x` espandendo correttamente il riferimento fino alla proprietà associata alla classe d'origine (`StructA.x`), evidenziando che l'IR valuta matematicamente e in modo conservativo gli accessi complessi. Questo è ora coperto esplicitamente dai benchmark.
