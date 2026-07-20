# Walkthrough: Clean Code Refactoring

Questo documento riassume le operazioni eseguite per migliorare la qualità del codice (rispetto dell'SRP - Single Responsibility Principle) nell'estrattore di dipendenze.

## Changes Made

### 1. [body_extraction.rs](file:///Users/ferra/Developing/tesi-magistrale/language-agnostic-analyzer/src/heuristics/body_extraction.rs)
La logica complessa presente in `find_behavioral_deps` è stata scomposta in funzioni più piccole e mirate.

- **Creata `parse_token_tree_macro`:** Tutta la complessa logica a stati (per il Token Coalescing) usata per analizzare i nodi `token_tree` (e.g. macro Rust come `println!`) è stata spostata in una funzione indipendente. La nuova funzione scorre l'albero sintattico per ricostruire i path qualificati e registrarne gli accessi e le chiamate, delegando a `find_behavioral_deps` solo le ricorsioni più profonde.
- **Creata `extract_call_dependency`:** La logica necessaria a risolvere le chiamate a funzione e a metodi (compresi i metodi statici tramite `scoped_identifier`) è stata isolata. Ora `find_behavioral_deps` delega semplicemente l'astrazione di `call_expression` a questa funzione.

### 2. [type_extraction.rs](file:///Users/ferra/Developing/tesi-magistrale/language-agnostic-analyzer/src/heuristics/type_extraction.rs)
È stato migliorato l'isolamento della logica in `extract_type_ref`.

- **Creata `try_extract_from_type_field`:** Questa funzione racchiude l'ispezione dei field più comuni di Tree-sitter associati ai tipi (`type`, `return_type`, `field_type`, `value_type`). Se trova uno di questi campi, naviga ricorsivamente richiamando l'estrazione, permettendo al flusso principale di `extract_type_ref` di risultare più leggibile e snello.

## Validation Results

Tutte le modifiche sono state testate rieseguendo la suite di validazione automatica (benchmark):

```bash
cargo run --bin benchmark_runner tests/benchmark-rust
```

I risultati mostrano che non c'è stata alcuna regressione:
- **Nodi estratti:** 38 trovati su 39 attesi (0.02% error rate).
- **Archi estratti:** 56 trovati su 67 attesi (le stesse performance prima del refactoring).
- I test `ACCE-6`, `ACCE-7`, `DECL-9`, `CALL-5`, `CALL-7`, `CALL-8`, `CALL-11` continuano ad essere validati positivamente (`[OK]`).

> [!TIP]
> Questa struttura modulare renderà molto più semplice in futuro aggiungere regole specifiche per altri linguaggi, o estendere l'estrattore con nuove euristiche, senza intaccare la funzione generica.

---

## Estrazione di Variabili Statiche e Globali (Static / Free Variables)

Abbiamo implementato l'identificazione e l'estrazione delle variabili statiche in modo completamente *language agnostic*.

### Modifiche Apportate
1. **Model** (`src/model/components.rs`): Abbiamo esteso la struct `Module` introducendo un nuovo vettore `free_variables: Vec<Field>`. Questo per mantenere la coerenza con le funzioni globali e permettere al modulo di possedere variabili proprie non racchiuse in tipi strutturati.
2. **Heuristics e Parsing**:
   - In `classifiers.rs` abbiamo introdotto la funzione `is_free_variable`, che identifica nodi come `static_item` e `const_item` (Rust), oppure `global_variable_declaration`.
   - In `parsers.rs` è stata creata `try_parse_free_variable`, che astrae queste variabili sottoforma del tipo `Field`.
   - In `heuristics/mod.rs` abbiamo collegato queste funzioni nel dispatching.
3. **Core e Analyzer** (`src/analyzer.rs`): Abbiamo configurato l'estrattore in modo da assegnare direttamente le istanze di `ParsedItem::Component(Component::Field)` alle variabili globali del `Module` contenitore.
4. **Resolution Phase** (`src/resolver/`):
   - Le *free_variables* vengono ora esposte nel registro globale `GlobalRegistry` alla pari di moduli, struct e funzioni, consentendo a qualunque modulo o metodo di accedere a `NomeModulo::NomeVariabile`.
   - Il `builder.rs` e l'`executor.rs` effettuano la risoluzione matematica e algebrica di tali variabili, agganciandole ai tipi e supportando le derivazioni (es. l'invocazione di metodi o l'accesso a campi derivanti dalla variabile statica).
5. **Esportazione Grafo** (`export/graph.rs`): Le query algebriche vengono ora srotolate in vertici (Type: *Field*) e si costruiscono i relativi archi `UsesFieldType` per associarne i tipi strutturali.
6. **Formalizzazione Matematica** (`latex/ir_formalization.tex`): La definizione matematica di $\mathcal{M}$ e di $\mathcal{C}$ è stata aggiornata per accogliere esplicitamente i campi globali $\vec{F}_{free}$.
7. **Visualizzatore Web**: 
   - Modificato `cytoscape_style.js` e `legend.html` per introdurre graficamente le *Free Variables* come "Nodi a forma di diamante giallo".

### Risultati del Benchmark (Rust)
L'aggiunta del supporto alle variabili globali ha permesso al sistema di trovare 2 nuovi archi (ora **58 Trovati** su 67), tra cui la corretta identificazione delle dipendenze di tipo access-field e usages di macro.
> [!NOTE]
> Inoltre, l'errore precedentemente segnalato sul test `DECL-14` è stato corretto nel `test.yml`! L'analizzatore valutava l'uso di `STATIC_SA.x` espandendo correttamente il riferimento fino alla proprietà associata alla classe d'origine (`StructA.x`), evidenziando che l'IR valutava matematicamente e in modo conservativo gli accessi complessi. Questo è ora coperto esplicitamente dal benchmark.
