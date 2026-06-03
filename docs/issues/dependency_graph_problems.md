# Problemi Identificati nella Fase 3 — Costruzione del Dependency Graph

Analisi dei file `src/export/graph.rs`, `src/export/cytoscape.rs`, `src/export/summary.rs`.

---

## 🔴 Problema 1: Il flattening non include gli ImplBlock come nodi, ma gli archi li referenziano come sorgente

**Gravità: Alta** — Genera archi orfani (senza nodo sorgente nel grafo).

### Descrizione
La funzione `flatten_modules` (`graph.rs:201-212`) raccoglie i nodi del grafo ma **non include gli ImplBlock**:

```rust
fn flatten_modules(modules: &[Module]) -> Vec<Component> {
    let mut flat = vec![];
    for m in modules {
        flat.push(Component::Module(m.clone()));
        for st in &m.structured_types {
            flat.extend(flatten_structured_type(st));
        }
        flat.extend(m.free_functions.iter().cloned().map(Component::Function));
        flat.extend(flatten_modules(&m.sub_modules));
        // ← Nessun processing di m.impl_blocks!
    }
    flat
}
```

Tuttavia, `traverse_module_for_edges` (`graph.rs:46-53`) **genera archi** che partono dagli impl blocks:

```rust
for ib in &m.impl_blocks {
    edges.push(Dependency {
        from: m.name.clone(),
        to: ib.name.clone(),               // ← arco VERSO l'impl block
        kind: DependencyEdgeKind::ModuleContainment,
    });
    add_impl_edges(ib, edges);              // ← archi CHE PARTONO dall'impl block
}
```

### Conseguenza
Se dopo la Fase 2 esistono ancora impl blocks nel modulo (il che non dovrebbe accadere nel flusso normale perché vengono svuotati dal flattening della Fase 2, ma potrebbe accadere se la pipeline cambiasse), il grafo conterrebbe archi con `from` o `to` che puntano a nodi inesistenti nella lista `nodes`.

L'esportazione Cytoscape mitiga il problema creando nodi "External" per i target mancanti, ma nel formato JSON diretto il grafo sarebbe incoerente.

### Mitigazione attuale
Nella pipeline corrente, `module.impl_blocks` è sempre vuoto dopo la Fase 2 (`resolver.rs:411` — `module.impl_blocks = vec![]`), quindi il codice morto non viene mai raggiunto. Tuttavia, il fatto che `traverse_module_for_edges` processi gli impl blocks mentre `flatten_modules` li ignora è una inconsistenza architetturale.

### Soluzione suggerita
Allineare i due percorsi: o aggiungere gli impl blocks al flattening, o rimuovere il processing degli impl blocks dalla generazione degli archi (dato che sono sempre vuoti):

```rust
// Opzione A: Aggiungere al flattening
// (richiede che Component supporti ImplBlock, attualmente non lo fa)

// Opzione B: Rimuovere codice morto dalla generazione archi
// Eliminare il blocco for ib in &m.impl_blocks { ... } da traverse_module_for_edges
```

**[Risolto]**: Sistemato in `src/export/graph.rs`. La funzione fantasma `add_impl_edges` e l'intero iteratore sui blocchi di implementazione all'interno di `traverse_module_for_edges` sono stati rimossi, dato che architetturalmente i blocchi implementativi non fanno più parte dei componenti emessi dopo la Fase 2 di Flattening.

---

## 🟡 Problema 2: Possibili archi duplicati nella generazione

**Gravità: Media** — Archi ridondanti nel grafo.

### Descrizione
La generazione degli archi non effettua deduplicazione. Se lo stesso tipo appare più volte nello stesso contesto, verranno generati archi duplicati. Esempi:

1. **Stesso tipo in più campi:**
   ```rust
   struct Config {
       name: String,    // → UsesFieldType → String (ma Primitive, ignorato)
       db1: Database,   // → UsesFieldType → Database
       db2: Database,   // → UsesFieldType → Database  ← DUPLICATO
   }
   ```

2. **Stesso tipo come parametro e ritorno:**
   ```rust
   fn transform(input: User) -> User {  // UsesParamType → User + UsesReturnType → User
       // Questi non sono duplicati (kind diverso), ma nella pratica
       // rappresentano la stessa dipendenza architetturale
   }
   ```

3. **Stessa funzione chiamata più volte:**
   ```rust
   fn process() {
       helper();    // → Calls → helper
       helper();    // → Calls → helper  ← DUPLICATO
   }
   ```

### Conseguenza
Il grafo contiene archi ridondanti che possono inflazionare le metriche di accoppiamento e rendere la visualizzazione più confusa. L'esportazione Cytoscape deduplica gli archi (`graph.rs:97-99`), ma il `DependencyGraph` in sé no.

### Soluzione suggerita
Deduplicare gli archi dopo la generazione, o durante:

```rust
// Post-generazione:
edges.sort();
edges.dedup();

// Oppure usare un HashSet durante la generazione
let mut edge_set: HashSet<(QualifiedName, QualifiedName, DependencyEdgeKind)> = HashSet::new();
```

**[Risolto]**: Inserito il tratto `Eq, PartialOrd, Ord, Hash` su `DependencyEdgeKind` e `Dependency` all'interno di `src/ir.rs`. In `src/export/graph.rs` è stato aggiunto un `edges.sort()` seguito da `edges.dedup()` prima di ritornare il `DependencyGraph`, pulendo efficacemente tutte le misurazioni di accoppiamento dai duplicati.

---

## 🟡 Problema 3: `Inherits` vs `Implements` — semantica incoerente tra Fase 2 e Fase 3

**Gravità: Media** — Confusione semantica nell'output.

### Descrizione
Nella Fase 2, il flattening degli impl blocks (`resolver.rs:405-407`) aggiunge il trait implementato ai `super_types` del tipo target:

```rust
if let Some(trait_ref) = ib.implements_trait.clone() {
    target_st.super_types.push(trait_ref);  // ← Il trait finisce in super_types
}
```

Nella Fase 3, `add_super_edges` (`graph.rs:83-96`) genera archi `Inherits` per **tutti** gli elementi di `super_types`:

```rust
for sup in &st.super_types {
    match sup {
        TypeRef::Resolved(to) | TypeRef::External(to) => {
            edges.push(Dependency {
                from: st.name.clone(), to: to.clone(),
                kind: DependencyEdgeKind::Inherits,  // ← Sempre Inherits!
            });
        }
        _ => {}
    }
}
```

### Conseguenza
Un trait implementato tramite `impl Trait for Struct` in Rust viene etichettato come `Inherits` nel grafo, non come `Implements`. Questo è semanticamente scorretto — "implementare" un trait è diverso da "ereditare" da una classe.

L'enum `DependencyEdgeKind` ha sia `Inherits` che `Implements`, ma `Implements` viene usato solo in `add_impl_edges` (che processa impl blocks vuoti dopo la Fase 2).

### Soluzione suggerita
Distinguere i super_types in base alla loro origine, o aggiungere un flag al `TypeRef` quando viene inserito dal flattening degli impl blocks:

```rust
// Opzione: usare un wrapper nel vettore super_types
pub enum SuperTypeRelation {
    Inherits(TypeRef),
    Implements(TypeRef),
}
```

**[Design Choice]**: Discusso esplicitamente a livello teorico: la distinzione è stata fusa nel nuovo arco astratto universale `IsA` per garantire l'approccio language-agnostic, abbracciando una semantica puramente basata sul contratto senza impigliarsi nei costrutti specifici dei linguaggi OOP.

---

## 🟡 Problema 4: `flatten_modules` non include i metodi delle funzioni libere che contengono componenti annidati

**Gravità: Media** — Limitazione minore dato il modello IR attuale.

### Descrizione
La funzione `flatten_structured_type` (`graph.rs:214-221`) appiattisce i metodi di un tipo strutturato come nodi `Function`. Tuttavia, `flatten_modules` non fa lo stesso per i metodi/componenti che potrebbero essere annidati nelle funzioni libere (se un giorno l'IR supportasse tale annidamento).

Più concretamente, l'IR non ha un modo per le funzioni libere di contenere tipi annidati (non c'è `nested_types` in `Function`), ma nella Fase 1 (`analyzer.rs:48-56`) le struct definite nel body delle funzioni vengono promosse a livello di modulo. Se questa promozione non avvenisse, i tipi annidati nelle funzioni verrebbero persi dal flattening.

### Conseguenza
Attualmente non è un bug grazie alla promozione nel Fase 1, ma è un'assunzione implicita tra le fasi che potrebbe rompersi con modifiche future.

**[Design Choice]**: Analogamente all'Extraction, le funzioni non sono trattate come costrutti di isolamento scope per i tipi strutturati (Hoisting), quindi l'appiattimento a livello Modulo rimane la pratica corretta e design-compliant.

---

## 🟡 Problema 5: Il summary non conta metodi né archi risolti/falliti

**Gravità: Media** — Informazione incompleta per analisi di qualità.

### Descrizione
La funzione `build_analysis_summary` (`summary.rs:5-23`) conta solo moduli, tipi strutturati e funzioni libere:

```rust
pub struct AnalysisSummary {
    pub total_modules: usize,
    pub total_structured_types: usize,
    pub total_free_functions: usize,
    pub resolved_refs: usize,    // ← MAI POPOLATO
    pub unknown_refs: usize,     // ← MAI POPOLATO
}
```

### Conseguenza
I campi `resolved_refs` e `unknown_refs` sono dichiarati nella struct ma **mai popolati** — rimangono sempre a 0. Questi campi sarebbero utilissimi per valutare la qualità della risoluzione (es. "su 100 riferimenti, 85 risolti, 10 esterni, 5 falliti").

Inoltre, i metodi (funzioni dentro struct) non vengono contati — solo le `free_functions` a livello di modulo.

### Soluzione suggerita
Attraversare ricorsivamente tutti i `TypeRef` nell'IR e contare:

```rust
fn count_refs_in_module(m: &Module, resolved: &mut usize, failed: &mut usize) {
    for st in &m.structured_types {
        for f in &st.fields {
            match &f.ty {
                TypeRef::Resolved(_) | TypeRef::External(_) => *resolved += 1,
                TypeRef::Failed(_) => *failed += 1,
                _ => {}
            }
        }
        // ... parametri, return types, calls, instantiates, super_types ...
    }
}
```

**[Risolto]**: In `src/export/summary.rs` è stata sviluppata una traversata esaustiva (`count_refs_in_st` e `count_refs_in_func`) che itera su `super_types`, `fields`, `methods`, `nested_types`, `parameters`, `return_type`, `calls` e `instantiates`, calcolando con altissima precisione il bilancio tra ref falliti e risolti per una chiara diagnostica a runtime.

---

## 🟢 Problema 6: L'esportazione Cytoscape crea nodi "External" senza distinguere il tipo reale

**Gravità: Bassa** — Informazione persa nella visualizzazione.

### Descrizione
In `cytoscape.rs:87-95`, quando un arco punta a un nodo che non esiste nella lista dei componenti, viene creato un nodo generico:

```rust
if !added_nodes.contains(&target_id) {
    let node_label = target_id.split("::").last().unwrap_or("").to_string();
    add_node(&mut elements, &mut added_nodes,
        target_id.clone(), node_label, "External".to_string());
}
```

### Conseguenza
Tutti i nodi mancanti vengono etichettati come `"External"`, senza distinzione tra:
- Tipi esterni effettivi (librerie)
- Nodi creati per archi orfani (bug del grafo)
- Tipi `Failed` che hanno comunque generato archi (non dovrebbe accadere, ma per sicurezza)

### Soluzione suggerita
Passare informazioni aggiuntive quando si crea il nodo, ad esempio dal tipo di `TypeRef` che ha generato l'arco.

**[Design Choice]**: Trattandosi di Cytoscape (un semplice visualizzatore), marcare tutti i nodi fantasma genericamente con l'etichetta `External` si dimostra la soluzione più pulita a livello UI per evitare un sovraccarico visivo di colori e shape. L'informazione tecnica vera e propria resta nel JSON nativo.

---

## 🟢 Problema 7: Pattern match ripetitivo per TypeRef nei generatori di archi

**Gravità: Bassa** — Code smell, nessun impatto funzionale.

### Descrizione
Il pattern `match &ref { TypeRef::Resolved(to) | TypeRef::External(to) => { ... } _ => {} }` è ripetuto **11 volte** in `graph.rs`. Questo rende il codice verboso e soggetto a errori di copia-incolla.

### Soluzione suggerita
Estrarre una funzione helper:

```rust
fn type_ref_target(tr: &TypeRef) -> Option<&QualifiedName> {
    match tr {
        TypeRef::Resolved(to) | TypeRef::External(to) => Some(to),
        _ => None,
    }
}

// Uso:
if let Some(to) = type_ref_target(&f.ty) {
    edges.push(Dependency { from: st.name.clone(), to: to.clone(), kind: ... });
}
```

**[Risolto]**: Eseguito il refactoring in `src/export/graph.rs` estraendo il match nel blocco helper `type_ref_target(tr: &TypeRef) -> Option<&QualifiedName>`. Ciò ha dimezzato la lunghezza di metodi critici come `add_function_edges` minimizzando drasticamente il rischio di errori.

---

## 🟡 Problema 8 (NUOVO): `add_block_edges` usa `UsesFieldType` per le dichiarazioni di variabili locali

**Gravità: Media** — Semantica dell'arco errata.

### Descrizione
La nuova funzione `add_block_edges` (`graph.rs`) genera archi `DependencyEdgeKind::UsesFieldType` per le dichiarazioni di variabili locali dentro i blocchi:

```rust
for decl in &block.declarations {
    if let Some(to) = type_ref_target(&decl.ty) {
        edges.push(Dependency {
            from: ff.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::UsesFieldType,  // ← Concepito per campi struct!
        });
    }
}
```

### Conseguenza
`UsesFieldType` è concepito per i campi delle struct (dipendenza strutturale), non per le variabili locali delle funzioni (dipendenza comportamentale/locale). Questo mescola due semantiche diverse nello stesso tipo di arco, rendendo impossibile distinguere nel grafo se una dipendenza `UsesFieldType` proviene da un campo struct o da una variabile locale di una funzione.

### Soluzione suggerita
Creare un nuovo `DependencyEdgeKind::UsesLocalVariable` per le dichiarazioni locali, separando la semantica strutturale da quella comportamentale.

---

## ✅ Miglioramenti nel refactoring recente

### Nuovo `add_block_edges` ricorsivo
La generazione degli archi per le funzioni non itera più su `ff.calls`/`ff.instantiates` direttamente. Ora delega a `add_block_edges` che:
1. Genera archi per le dichiarazioni locali (`declarations`)
2. Genera archi `Calls` e `Instantiates` dal blocco
3. Ricorsivamente processa i `sub_blocks` (if, while, scope anonimi)

Tutti gli archi dai sotto-blocchi vengono attribuiti alla funzione contenitrice (`ff.name`).

### `count_refs_in_block` ricorsivo
Anche il summary ora attraversa ricorsivamente i blocchi per contare accuratamente tutti i `TypeRef` risolti e falliti.

---

## Riepilogo

| # | Problema | Gravità | File | Impatto | Status |
|---|----------|---------|------|---------|--------|
| 1 | ImplBlock non flattened ma referenziati negli archi | 🔴 Alta | `graph.rs` | Archi orfani (attualmente codice morto) | **[Risolto]** |
| 2 | Archi duplicati non deduplicati | 🟡 Media | `graph.rs` | Metriche inflazionate | **[Risolto]** |
| 3 | `Inherits` vs `Implements` incoerente | 🟡 Media | `graph.rs` | Trait marcati come Inherits | **[Design Choice]** |
| 4 | Assunzione implicita sulla promozione delle struct annidate | 🟡 Media | `graph.rs` | Fragilità inter-fase | **[Design Choice]** |
| 5 | Summary non conta refs risolti/falliti né metodi | 🟡 Media | `summary.rs` | Campi sempre a 0 | **[Risolto]** |
| 6 | Nodi Cytoscape External generici | 🟢 Bassa | `cytoscape.rs` | Perdita di informazione | **[Design Choice]** |
| 7 | Pattern match ripetitivo per TypeRef | 🟢 Bassa | `graph.rs` | Manutenibilità | **[Risolto]** |
| 8 | `UsesFieldType` per variabili locali | 🟡 Media | `graph.rs` | Semantica arco errata | **Nuovo** |

