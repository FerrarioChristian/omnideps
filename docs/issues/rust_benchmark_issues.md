# Rust Benchmark Issues

Questo documento analizza gli archi mancanti rilevati durante i benchmark del linguaggio Rust (`benchmark_all`), raggruppandoli per causa radice e fornendo per ciascuno una potenziale soluzione implementativa.

## 1. La keyword `Self` e l'Alias Target (1 arco mancante)
**Archi:**
- `structs.StructC.deref -> structs.StructC.Target`

**Causa:**
In Rust, la keyword `self` (minuscolo) si riferisce all'istanza, mentre `Self` (maiuscolo) si riferisce al tipo strutturato in cui ci si trova. Attualmente, `AnalyzerConfig` per Rust inietta `self` come `self_keyword`, ma l'analizzatore non inietta `Self` come alias del tipo corrente nello `ScopeTree`. 
Quando il metodo `deref()` restituisce `&Self::Target`, l'engine tenta di risolvere la query `Extract(Find("Self"), "Target")`. Poiché `Self` non esiste nello scope, la risoluzione fallisce.

**Soluzione Potenziale:**
Aggiornare `register_structured_type` e `register_impl_block` in `scope.rs` per iniettare esplicitamente un `Symbol::TypeAlias` chiamato `Self` che punti al nodo corrente, analogamente a come viene già fatto per il parametro `this`/`self` nei metodi.

---

## 2. Variabili Globali Statiche (1 arco mancante)
**Archi:**
- `functions.function_with_local_variables -> functions.STATIC_SA`

**Causa:**
`STATIC_SA` è una variabile globale (`FreeVariable`) definita a livello di modulo. All'interno della funzione viene fatto l'accesso `STATIC_SA.x`.
L'estrattore di codice intercetta correttamente `STATIC_SA.x` generando una query `Extract(Find("STATIC_SA"), "x")`. 
Durante l'esecuzione (in `executor.rs`), `Find("STATIC_SA")` viene valutato con successo in `StructA`, dopodiché viene estratto il campo `x` ottenendo `StructA.x`.
Tuttavia, il builder del grafo emette un arco di dipendenza *solo* verso il nodo finale risolto (`StructA.x`), dimenticandosi del nodo intermedio `STATIC_SA` che è stato attraversato per arrivarci.

**Soluzione Potenziale:**
Modificare il Query Engine (`executor.rs`) affinché tenga traccia degli "hop" intermedi durante la risoluzione (ad esempio, quando si valuta una variabile o un alias), oppure forzare l'emissione di un arco `AccessesField` per la radice della query (`STATIC_SA`) prima di scendere nei suoi campi.

---

## 3. Varianti Enum e Scoped Identifiers ignorati (4 archi mancanti)
**Archi:**
- `functions.function_with_local_variables -> enums.EnumA.FIRST`
- `functions.function_with_instance_methods -> enums.EnumA.FIRST`
- `functions.function_with_return_types -> enums.EnumA.SECOND`
- `functions.function_with_inherited_trait_methods -> enums.EnumA.FIRST`

**Causa:**
In Rust, l'accesso alle varianti di un enum (es. `EnumA::FIRST`) viene modellato da Tree-sitter come uno `scoped_identifier`.
Attualmente `body_extraction.rs` non estrae questi nodi in due situazioni critiche:
1. Quando si trovano sul lato destro (RHS) di una dichiarazione o nell'inizializzatore di un campo in una struct (`StructB { y: EnumA::FIRST }`), perché l'estrattore fa un "ritorno anticipato" sulla dichiarazione per evitare falsi positivi.
2. Quando passati come argomenti nidificati di una chiamata a funzione (es. `EnumB::FIRST(EnumA::SECOND)`).

Di conseguenza, gli identificatori non finiscono nella lista di `accesses` del blocco e l'engine non sa di doverli risolvere.

**Soluzione Potenziale:**
Rivedere la funzione `find_behavioral_deps` in `body_extraction.rs` per assicurarsi che i nodi `scoped_identifier` vengano correttamente attraversati ed estratti come `AccessesField` anche quando si trovano dentro i rami `value` di un `field_initializer` o dentro gli argomenti (`arguments`) di un `call_expression`.

---

## 4. Ereditarietà via Trait, Bound Generici e Deref (5 archi mancanti)
**Archi:**
- `functions.function_with_deref_inheritance -> structs.StructA`
- `functions.function_with_trait_methods -> traits.TraitA`
- `functions.function_with_trait_methods -> traits.TraitA.trait_method`
- `functions.function_with_inherited_trait_methods -> traits.TraitA.trait_method`
- `functions.function_with_inherited_trait_methods -> traits.TraitB.new_trait_method`

**Causa:**
Questa è la categoria più complessa ed è causata da feature avanzate del type system di Rust:
1. **Generic Bounds:** In `<T: TraitA>`, il tipo parametrico `T` non viene registrato con il bound `TraitA` nel contesto locale della funzione. Per cui `ta.trait_method()` fallisce perché l'engine non sa che `T` implementa `TraitA`.
2. **Trait Inheritance:** Anche se la funzione riceve `TraitB`, le chiamate ai metodi ereditati dal super-trait `TraitA` falliscono perché `TraitB` non "fonde" i metodi di `TraitA` al suo interno durante la costruzione dell'IR.
3. **Deref Coercion:** Il metodo `instance_method()` viene chiamato su `StructC`, ma in realtà appartiene a `StructA`. Questo in Rust funziona perché `StructC` implementa il trait `Deref<Target=StructA>`. L'analizzatore non implementa questa risoluzione implicita.

**Soluzione Potenziale:**
1. Aggiornare `extract_function_signature` per estrarre i bound generici (`<T: Trait>`) e iniettare `T` come `Symbol::TypeAlias` o alias generico nello scope della funzione.
2. Estendere il flattening (attualmente fatto solo sugli `impl_block`) ai tratti base (es. in `execute_structured_type` o `register_structured_type` fondere i metodi dei super trait).
3. (Per Deref) Aggiungere una logica di fallback in `execute_extract`: se l'estrazione di un metodo fallisce per una struct, controllare se questa possiede un alias `Target` e riprovare la query su quel target (simulando la deref coercion).
