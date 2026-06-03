# Problemi Identificati nella Name Resolution v2 — Symbol Stack Architecture

Analisi dei file `src/resolver/mod.rs`, `src/resolver/stack.rs`, `src/resolver/cache.rs`.

---

## 🔴 Problema 1: Il Global Registry non registra i `nested_types` delle struct

**Gravità: Alta** — Tipi annidati non risolvibili come `Local`.

### Descrizione
La funzione `GlobalRegistry::register_module` (`mod.rs:32-72`) registra:
- ✅ Moduli (ricorsivamente)
- ✅ Tipi strutturati (top-level nel modulo)
- ✅ Metodi dei tipi strutturati
- ✅ Funzioni libere
- ✅ Metodi degli impl blocks
- ❌ **nested_types** dei tipi strutturati

```rust
fn register_module(&mut self, m: &Module, mut prefix: QualifiedName) {
    // ...
    for st in &m.structured_types {
        let mut st_prefix = prefix.clone();
        st_prefix.extend(st.name.clone());
        self.paths.insert(st_prefix.clone());

        // Registra i metodi
        for method in &st.methods { ... }

        // ⚠️ MANCA: for nested in &st.nested_types { register_structured_type(nested, st_prefix) }
    }
}
```

### Conseguenza
Se una struct `Outer` ha un tipo annidato `Inner` (es. `Outer::Inner`), il path `["root", "Outer", "Inner"]` non verrà inserito nel registry. Quando il resolver troverà il nome `Inner` nello scope locale e costruirà `full_path`, la chiamata `registry.exists(&full_path)` restituirà `false`, e il tipo verrà erroneamente marcato come `External` o `Failed` anziché `Local`.

### Soluzione suggerita
Aggiungere una funzione ricorsiva per registrare i tipi strutturati con i loro nested_types:

```rust
fn register_structured_type(&mut self, st: &StructuredType, prefix: QualifiedName) {
    let mut st_prefix = prefix;
    st_prefix.extend(st.name.clone());
    self.paths.insert(st_prefix.clone());

    for method in &st.methods {
        let mut m_prefix = st_prefix.clone();
        m_prefix.extend(method.name.clone());
        self.paths.insert(m_prefix);
    }
    for nested in &st.nested_types {
        self.register_structured_type(nested, st_prefix.clone()); // Ricorsione!
    }
}
```

**[Risolto]**: Sistemato in `src/resolver/mod.rs`. La logica di registrazione per i `StructuredType` all'interno del `GlobalRegistry` è stata separata nella funzione ricorsiva `register_structured_type` in modo che tutti i `nested_types`, a qualsiasi profondità (e i relativi metodi), vengano correttamente indicizzati nel Passo 1 (Hoisting).

---

## 🔴 Problema 2: Il Global Registry non registra i `nested_types` degli impl blocks

**Gravità: Alta** — Stesso problema del precedente ma per gli impl blocks.

### Descrizione
In `register_module` (`mod.rs:53-68`), per gli impl blocks si registrano i metodi ma non i `nested_types`:

```rust
for ib in &m.impl_blocks {
    let target_name = match &ib.impl_for { ... };
    if !target_name.is_empty() {
        let mut target_prefix = prefix.clone();
        target_prefix.push(target_name);
        for method in &ib.methods { ... }  // ✅ Metodi registrati
        // ⚠️ MANCA: for nested in &ib.nested_types { ... }
    }
}
```

### Conseguenza
Tipi annidati definiti negli impl blocks non saranno trovati dal registry.

**[Risolto]**: Corretto in tandem con il Problema 1. Adesso in `register_module`, il loop sugli `impl_blocks` invoca ricorsivamente `register_structured_type` per ogni classe o struct annidata al loro interno.

---

## 🟡 Problema 3: Wildcard import valida solo come `Local`, mai come `External`

**Gravità: Media** — Asimmetria logica con gli import normali.

### Descrizione
Nel trattamento dei wildcard imports (`mod.rs:183-192`), il risultato viene restituito **solo** se è `Local`:

```rust
} else {
    // Wildcard import
    let mut candidate = imp.path.clone();
    candidate.extend(name.clone());
    let res = validate_import(&candidate);
    if let ResolutionResult::Local(_) = res {
        ctx.cache.borrow_mut().insert(...);
        return Some(res);
    }
    // ← Se è External, viene IGNORATO e si continua il loop
}
```

### Conseguenza
Un wildcard import come `use external_lib::*` non risolverà mai un tipo esterno. Questo è intenzionale (commento nel vecchio codice: "Non registriamo sempre external da wildcard perché causerebbe falsi positivi massicci"), ma potrebbe portare a `Failed` per tipi che in realtà sarebbero `External` tramite wildcard import.

### Possibile mitigazione
Documentare esplicitamente questa scelta nel codice e nella documentazione. Se in futuro si volesse supportare i wildcard External, si potrebbe accumulare un "candidato External migliore" e usarlo come fallback prima di restituire `None`.

---

## ✅ ~~Problema 4: Il resolve_function fa push/pop di uno scope vuoto~~ — RISOLTO

**Gravità: ~~Media~~ → Risolto nell'ultimo refactoring**

### Stato
**RISOLTO.** La nuova versione di `resolve_function` ora:
1. Registra effettivamente i parametri come simboli locali nello scope:
```rust
if let Some(name) = &p.name {
    if let TypeRef::Resolved(abs_path) = &p.ty {
        ctx.stack.borrow_mut().define_symbol(name.clone(), abs_path.clone());
    }
}
```
2. Delega l'analisi del corpo a `resolve_block`, che a sua volta fa push/pop per ogni blocco annidato e registra le dichiarazioni locali.

Lo scope delle funzioni non è più vuoto — contiene sia i parametri risolti che i blocchi del corpo.

---

## 🟡 Problema 5: `validate_import` prova i fallback in modo non simmetrico

**Gravità: Media** — Comportamento incoerente per certi path.

### Descrizione
La closure `validate_import` (`mod.rs:126-147`) segue questa catena:

```
1. registry.exists(candidate)           → Local
2. registry.exists(["root"] + candidate) → Local
3. Se candidate[0] == "crate":
   3a. registry.exists(["root"] + candidate[1..]) → Local
   3b. Altrimenti → External
4. Altrimenti → External
```

Il problema è che il branch `crate` (step 3) viene valutato **solo se step 2 fallisce**. Se un import ha path `["crate", "module", "Type"]` e il registry contiene `["root", "crate", "module", "Type"]` (improbabile ma possibile), verrebbe trovato allo step 2 con un path sbagliato.

### Conseguenza
Edge case raro, ma la logica sarebbe più chiara e robusta se il check per `crate` fosse fatto per primo quando il primo segmento è `"crate"`.

**[Risolto]**: Sistemato in `src/resolver/mod.rs`. La closure `validate_import` adesso valuta il token iniziale `crate` nel primo livello logico del costrutto `else if`, gestendolo come un ramo preferenziale anziché dipendere dal fallimento della gerarchia standard.

---

## 🟡 Problema 6: I simboli degli impl blocks non vengono hoisted nello scope del modulo

**Gravità: Media** — I metodi degli impl blocks non sono visibili per la ricerca locale.

### Descrizione
In `resolve_module_in_context` (`mod.rs:232-323`), durante la fase di hoisting vengono registrati:
- ✅ `structured_types` → `define_symbol`
- ✅ `free_functions` → `define_symbol`
- ✅ `sub_modules` → `define_symbol`
- ❌ **impl_blocks** → Non hoisted

```rust
// Hoist local components (Structs, Functions, Sub-modules)
for st in &module.structured_types { ... define_symbol ... }
for ff in &module.free_functions { ... define_symbol ... }
for sub in &module.sub_modules { ... define_symbol ... }
// ⚠️ Nessun hoisting per impl_blocks
```

### Conseguenza
Se un impl block aggiunge metodi a un tipo, quei metodi non saranno visibili nello scope del modulo per la risoluzione dei `calls`. Tuttavia, dopo il flattening, i metodi finiscono nel `StructuredType` target, quindi saranno visibili indirettamente tramite la struct. Il problema si manifesta solo se si tenta di risolvere un metodo di un impl block *prima* del flattening, ma dato che il flattening avviene dopo la risoluzione, l'ordine è corretto. Il problema è quindi più teorico che pratico.

**[Design Choice]**: Le funzioni (metodi) dichiarati all'interno di un `impl Trait for Target` o `impl Target` sono semanticamente subordinate a `Target`. Architetturalmente parlando, non hanno alcun senso esposte come free-functions nello scope globale del modulo senza essere qualificate o senza la risoluzione su un'istanza dell'oggetto. È tecnicamente corretto che lo stack del modulo le ignori all'avvio.

---

## 🟢 Problema 7: Il `scope_key` usa `prefix.join("::")` che potrebbe collidere con nomi contenenti `::`

**Gravità: Bassa** — Improbabile ma possibile.

### Descrizione
In `cache.rs:22-28`:

```rust
fn scope_key(prefix: &[String]) -> String {
    if prefix.is_empty() {
        "GLOBAL_ROOT".to_string()
    } else {
        prefix.join("::")
    }
}
```

Se un identificatore nel path contiene letteralmente `::` (es. `"module::inner"` come singolo elemento), il join produrrebbe una chiave ambigua indistinguibile da un path con due elementi.

### Conseguenza
In pratica, gli identificatori estratti dal parser non contengono mai `::` (sono nomi singoli), quindi il rischio è puramente teorico.

### Soluzione suggerita
Usare un separatore più sicuro come `\0` (null byte) o usare una struttura di chiave diversa.

---

## 🟢 Problema 8: `Inherits` vs `Implements` — stessa problematica della v1

**Gravità: Bassa** — Problematica ereditata dal vecchio codice.

### Descrizione
Il flattening degli impl blocks (`mod.rs:290-311`) aggiunge i trait implementati ai `super_types` del tipo target:

```rust
if let Some(trait_ref) = ib.implements_trait.clone() {
    target_st.super_types.push(trait_ref);
}
```

Nella Fase 3 (dependency graph), tutti i `super_types` generano archi `Inherits`, anche quelli che semanticamente sono `Implements`. Questa problematica era già presente nella v1 e non è stata affrontata nel refactoring.

---

## 🟢 Problema 9: Il RefCell per stack e cache potrebbe causare panic a runtime

**Gravità: Bassa** — Il codice attuale non presenta il problema, ma è fragile.

### Descrizione
Sia `SymbolStack` che `ResolutionCache` sono wrappati in `RefCell` per permettere mutabilità interiore:

```rust
pub struct ResolutionContext<'a> {
    pub stack: &'a RefCell<SymbolStack>,
    pub cache: &'a RefCell<ResolutionCache>,
    // ...
}
```

Se per errore una chiamata a `ctx.stack.borrow()` (immutabile) coincidesse con un `ctx.stack.borrow_mut()` (mutabile) nello stesso scope, si avrebbe un panic a runtime. Nel codice attuale questo non accade perché i borrow sono sempre in scope disgiunti, ma il pattern è intrinsecamente fragile.

### Nota
In `resolve_name_in_context`, la riga `for frame in ctx.stack.borrow().iter_frames_top_down()` tiene il borrow immutabile attivo per tutta la durata del loop, e dentro il loop si chiama `ctx.cache.borrow_mut()` (su cache, non su stack) — questo è sicuro. Ma se si aggiungesse una chiamata `ctx.stack.borrow_mut()` dentro il loop, si avrebbe un panic.

---

## 🟡 Problema 10 (NUOVO): `resolve_block` registra dichiarazioni solo se `Resolved`

**Gravità: Media** — Variabili con tipi non risolti non vengono registrate nello scope.

### Descrizione
In `resolve_block` (`mod.rs`), le dichiarazioni locali vengono registrate nello stack solo se il tipo si è risolto con successo:

```rust
block.declarations = block.declarations.into_iter().map(|mut decl| {
    decl.ty = resolve_type_ref(ctx, decl.ty);
    if let TypeRef::Resolved(abs_path) = &decl.ty {
        ctx.stack.borrow_mut().define_symbol(decl.name.clone(), abs_path.clone());
    }
    // ← Se il tipo è Failed o External, la variabile NON viene definita
    decl
}).collect();
```

Lo stesso pattern è usato per i parametri in `resolve_function`.

### Conseguenza
Se una variabile locale `let db: Db = ...` ha un tipo che si risolve come `External`, la variabile `db` non verrà registrata nello scope. Se più avanti nel corpo della funzione si chiama `db.query()`, il resolver non troverà `db` nello stack e non potrà risalire al tipo. In pratica questo ha un impatto limitato perché l'analizzatore traccia le dipendenze comportamentali (calls/instantiates) attraverso i `TypeRef` direttamente, non attraverso le variabili locali.

### Possibile miglioramento
Registrare anche i simboli `External` nello stack, così che la variabile sia comunque visibile anche se punta a un tipo esterno.

**[Risolto]**: Corretto in `src/resolver/mod.rs`. Ora in `resolve_block` e nella risoluzione dei parametri (`resolve_function`), le variabili e gli argomenti vengono registrati nel Symbol Stack chiamando `define_symbol` sia per tipi `TypeRef::Resolved` che `TypeRef::External`.

---

## 🟡 Problema 11 (NUOVO): `add_block_edges` usa `UsesFieldType` per le dichiarazioni locali

**Gravità: Media** — Semantica dell'arco errata.

### Descrizione
In `graph.rs`, la nuova funzione `add_block_edges` genera archi `UsesFieldType` per le dichiarazioni di variabili locali:

```rust
for decl in &block.declarations {
    if let Some(to) = type_ref_target(&decl.ty) {
        edges.push(Dependency {
            from: ff.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::UsesFieldType,  // ← Semantica errata!
        });
    }
}
```

`UsesFieldType` è concepito per i campi delle struct, non per le variabili locali delle funzioni. Andrebbe creato un nuovo `DependencyEdgeKind::UsesLocalVariable` o simile per distinguere le dipendenze strutturali da quelle comportamentali-locali.

**[Risolto]**: Sistemato. Introdotto matematicamente ed esplicitamente il `DependencyEdgeKind::UsesLocalType` nell'enum dell'IR e conseguentemente in `add_block_edges` all'interno di `src/export/graph.rs`.

---

## ✅ Miglioramenti rispetto alla v1

| Aspetto | v1 (Arena) | v2 (Symbol Stack) |
|---|---|---|
| Struttura dati per scope | `Vec<ScopeNode>` statico (Arena) | Stack dinamico push/pop |
| Contenuto dei simboli | `HashMap<Identifier, Component>` (clona interi Component) | `HashMap<String, QualifiedName>` (solo path) |
| Validazione path | `find_node_by_path` (traversal O(depth)) | `GlobalRegistry.exists` (HashSet O(1)) |
| Navigazione genitore | `arena[idx].parent` (indice) | Iterazione frames top-down (implicita) |
| Cache | `HashMap<(usize, QualifiedName), Result>` | Cache Tree `HashMap<String, HashMap<QN, Result>>` |
| Consumo memoria | Alto (clonazione di Component) | Basso (solo QualifiedName) |
| Allineamento teoria | Basso (Arena non è un costrutto formale) | Alto (Symbol Stack = Environment ρ classico) |
| Analisi corpo funzioni | `calls`/`instantiates` piatti sulla Function | `Block` gerarchico con `declarations`, `sub_blocks` ricorsivi |
| Scope parametri | Parametri non registrati | Parametri registrati con `define_symbol` |
| Scope blocchi interni | Non modellati | Ogni `{}` crea un push/pop di scope |

---

## Riepilogo

| # | Problema | Gravità | File | Stato |
|---|----------|---------|------|-------|
| 1 | Global Registry non registra nested_types delle struct | 🔴 Alta | `mod.rs` | **Risolto** |
| 2 | Global Registry non registra nested_types degli impl blocks | 🔴 Alta | `mod.rs` | **Risolto** |
| 3 | Wildcard import solo per Local | 🟡 Media | `mod.rs` | **Design Choice** |
| 4 | ~~resolve_function push/pop di scope vuoto~~ | ✅ | `mod.rs` | **Risolto** |
| 5 | validate_import catena di fallback asimmetrica | 🟡 Media | `mod.rs` | **Risolto** |
| 6 | Impl blocks non hoisted nello scope | 🟡 Media | `mod.rs` | **Design Choice** |
| 7 | scope_key con "::" potrebbe collidere | 🟢 Bassa | `cache.rs` | Aperto |
| 8 | Inherits vs Implements ereditato dalla v1 | 🟢 Bassa | `mod.rs` | **Design Choice** |
| 9 | RefCell fragile per borrow concorrenti | 🟢 Bassa | `mod.rs` | Aperto |
| 10 | Dichiarazioni locali registrate solo se Resolved | 🟡 Media | `mod.rs` | **Risolto** |
| 11 | add_block_edges usa UsesFieldType per variabili locali | 🟡 Media | `graph.rs` | **Risolto** |
