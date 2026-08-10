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

**[RISOLTO]:**
Aggiunto `self_type_keyword` alla `LanguageConfig` (impostato a `Self` per Rust). `scope.rs` ora inietta automaticamente questo alias come `TypeAlias` al tipo della struct/impl block in cui ci si trova. Questo permette a `Self::Target` di risolversi correttamente al nodo `Target` all'interno della struct. Inolte è stato disattivato il resolving implicito per i `return_type` in `executor.rs` in modo da preservare l'edge verso l'alias invece che verso il tipo risolto.

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

**[RISOLTO]:**
Implementata la struttura `TypeRef::EvaluatedAccess` in `executor.rs` (sia in `find_global` che `symbol_to_typeref`) per mantenere la storia delle risoluzioni intermedie. Durante la valutazione di un alias o di un valore (`Symbol::Value` o `Symbol::TypeAlias`), l'analizzatore ora avvolge il tipo risolto con il path originale del simbolo. Questo permette a `type_ref_targets` di esplorare l'intero albero di risoluzione e di produrre correttamente un arco verso il nodo radice `STATIC_SA`.

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

**[RISOLTO]:**
Aggiunto `scoped_identifier` e `qualified_identifier` al filtro del blocco `Accesses` in `body_extraction.rs`. L'estrattore ora aggiunge correttamente gli identificatori di scope alla lista di `accesses`, permettendo all'engine di emettere la dipendenza (es. `EnumA::FIRST`). L'attraversamento ricorsivo di `find_behavioral_deps` si assicura inoltre che gli attributi all'interno di tuple o struct expression vengano trovati e aggiunti agli accessi interni della dichiarazione, propagandosi fino alla radice.

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

**[PARZIALMENTE RISOLTO / POSTICIPATO]:**
- **Deref Coercion (Risolto):** Il metodo `register_impl_block` in `scope.rs` è stato esteso per intercettare l'implementazione del trait `Deref`. Se il target name di un impl è `Deref` e l'impl è associato a una struct (es. `StructC`), il tipo contenuto nel corpo del trait `Target = StructA` viene mappato estraendo il path e viene aggiunto ricorsivamente alla lista dei `super_types` del record. In questo modo il processo di lexical climbing standard (che ispeziona `super_types`) eredita automaticamente i metodi di `StructA` all'interno di `StructC`, risolvendo la chiamata a `instance_method` e la dipendenza alla struct `StructA`.
- **Generic Bounds & Traits (Posticipato):** Come discusso con l'utente, la gestione dei bounds per i tipi generici e dell'ereditarietà complessa dei Trait (es. `TraitB` che estende `TraitA`) è attualmente in fase di revisione architetturale. L'implementazione definitiva per mappare la type equality e le dipendenze dinamiche dei trait richiederà l'uso di un costrutto language-agnostic trasversale ai linguaggi (C++ concepts/templates, Java bounded generics, Rust Traits), e per ora la risoluzione degli archi associati (`TraitA`, `trait_method`, ecc.) è rimandata.

---

## 5. Decisioni di Design Architetturale: Tipi di Ritorno vs Alias
**Archi:**
- `structs.StructC.deref -> structs.StructC.Target`

**Causa originale:**
In Rust, il metodo `deref` restituisce `&Self::Target`. In passato il risolutore, a causa della direttiva `resolve_type = true`, valutava completamente l'alias `Target` trovando il suo tipo base `StructA`, omettendo l'arco all'alias intermedio.

**[RISOLTO]:**
Grazie all'introduzione di `TypeRef::EvaluatedAccess` in `executor.rs` (che mantiene l'intera catena di risoluzione), il graph exporter ora emette archi sia verso l'alias intermedio (`Target`) sia verso il tipo base (`StructA`). Questa soluzione elegante permette di soddisfare le aspettative specifiche del benchmark di Rust (che traccia l'uso dell'alias) senza compromettere la flessibilità agnostica del core, che continua a tracciare anche il tipo reale sottostante.

---

## Riepilogo Attuale (71/75 Archi Trovati)
Allo stato attuale dello sviluppo, tutte le discrepanze strutturali e architetturali del benchmark Rust sono state risolte. I **4 archi rimanenti** (su un totale di 75) appartengono esclusivamente alla categoria **Generic Bounds & Traits** (Punto 4), che è stata formalmente posticipata in attesa di un design language-agnostic per i vincoli di tipo. L'infrastruttura di analisi per C++, Python, Java e Rust risulta stabile e allineata.

### Tabella di Riepilogo

| # | Causa Radice | Archi Coinvolti | Componente | Stato |
|---|--------------|-----------------|------------|-------|
| 1 | Manca l'alias `Self` per la keyword | 1 | `scope.rs` / Config | **Risolto** |
| 2 | Hop intermedi persi (Variabili globali statiche) | 1 | `executor.rs` | **Risolto** (con `EvaluatedAccess`) |
| 3 | `scoped_identifier` ignorati (Enum variants) | 4 | `body_extraction.rs` | **Risolto** |
| 4a | Ereditarietà Deref Coercion mancante | 1 | `scope.rs` | **Risolto** (tramite super_types) |
| 4b | Generic Bounds e Trait Inheritance | 4 | `executor.rs` / `scope.rs` | **Posticipato** |
| 5 | Design Architetturale (Tipi Ritorno vs Alias) | 1 | `executor.rs` / `graph.rs` | **Risolto** (con `EvaluatedAccess`) |
