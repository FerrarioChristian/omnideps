# Problemi Identificati nella Name Resolution v1 (ARCHIVIO STORICO)

> ⚠️ **Questo file documenta problemi della v1 (ScopeTree + Arena). L'architettura è stata sostituita prima dalla v2 (SymbolStack monofase) e poi dalla v3 (Query Engine bifase). Per i problemi attuali, vedere [`name_res_problems_2.md`](name_res_problems_2.md).**

Analisi del file `src/resolver.rs` — problemi, limitazioni e possibili miglioramenti.

---

## 🔴 Problema 1: `current_scope` non viene mai aggiornato durante la risoluzione

**Gravità: Alta** — Compromette la correttezza della risoluzione lessicale.

### Descrizione
In `resolve_module_in_context` (riga 375), il campo `current_scope` del nuovo contesto viene copiato direttamente dal contesto genitore senza essere aggiornato:

```rust
let new_ctx = ResolutionContext {
    current_prefix: new_prefix.clone(),
    tree: ctx.tree,
    current_scope: ctx.current_scope, // ← MAI AGGIORNATO
    cache: ctx.cache,
};
```

Lo stesso accade in `resolve_structured_type` (riga 430) e `resolve_impl_block` (riga 500).

### Conseguenza
Poiché `resolve_type_refs` inizializza `current_scope` a `tree.root` (indice `0`, il nodo ROOT), **tutti i riferimenti di tipo vengono risolti a partire dal ROOT**, indipendentemente da dove appaiono nel codice.

Questo significa che il Lexical Climb del loop in `resolve_name_in_context` parte sempre dal ROOT anziché dallo scope effettivo del tipo. In pratica:
- I **symbols locali** di uno scope intermedio (es. i tipi dentro un modulo `core`) non vengono mai trovati tramite la ricerca locale al primo tentativo.
- Gli **imports** dei moduli intermedi non vengono ispezionati, perché il loop parte da ROOT e ROOT non ha genitore → il loop fa una sola iterazione.
- La risoluzione funziona principalmente grazie ai **fallback assoluti** (step 4a/4b), non grazie al lexical scoping.

### Soluzione suggerita
Aggiornare `current_scope` per puntare al nodo corrispondente nell'Arena. Per esempio:

```rust
// In resolve_module_in_context, dopo aver calcolato il prefisso:
let module_scope = ctx.tree.find_node_by_path(&new_prefix)
    .unwrap_or(ctx.current_scope);

let new_ctx = ResolutionContext {
    current_prefix: new_prefix.clone(),
    tree: ctx.tree,
    current_scope: module_scope,  // ← Aggiornato!
    cache: ctx.cache,
};
```

**[Risolto (Obsoleto)]**: La problematica descritta faceva riferimento all'architettura basata su indice (`current_scope`) e Arena statico (`ScopeTree`), che è stata completamente smantellata. Nella nuova architettura formale (v2), il contesto è gestito tramite un `SymbolStack` dinamico. Il problema non esiste più alla radice poiché lo scope non viene "copiato", ma l'analizzatore invoca iterativamente `push_scope()` e `pop_scope()` entrando ed uscendo da ogni blocco, garantendo un \textit{Lexical Scoping} matematicamente puro.

---

## 🟡 Problema 2: La cache non tiene conto dello scope di partenza

**Gravità: Media** — Mascherato dal Problema 1 (poiché lo scope è sempre ROOT, il contesto è sempre lo stesso), ma diventerebbe un bug se il Problema 1 venisse risolto.

### Descrizione
La cache usa `QualifiedName` come chiave (riga 186):

```rust
if let Some(res) = ctx.cache.borrow().get(name) {
    return Some(res.clone());
}
```

Ma il risultato di una risoluzione dipende dallo **scope di partenza**. Lo stesso nome `Foo` potrebbe risolvere a `a::Foo` nel modulo `a` e a `b::Foo` nel modulo `b`.

### Conseguenza
Se il Problema 1 venisse corretto e `current_scope` iniziasse a variare, la cache restituirebbe risultati errati per nomi ambigui risolti in scope diversi.

### Soluzione suggerita
Includere lo scope nella chiave della cache:

```rust
type CacheKey = (usize, QualifiedName);  // (scope_index, name)
```

**[Risolto (Obsoleto)]**: Anch'esso superato dalla nuova architettura. Il sistema ora impiega un `CacheTree` (`src/resolver/cache.rs`) che isola strutturalmente la memoria utilizzando come chiave primaria il path assoluto dello scope corrente (`prefix.join("::")`). Non ci sono più collisioni tra scope diversi per nomi identici.

---

## 🟡 Problema 3: Gli impl blocks con `impl_for` non-Resolved vengono persi silenziosamente

**Gravità: Media** — Causa perdita di informazione per tipi esterni.

### Descrizione
Nel flattening degli impl blocks (righe 396-410), solo gli impl blocks con `TypeRef::Resolved` vengono processati:

```rust
for ib in resolved_impls {
    if let TypeRef::Resolved(target_name) = &ib.impl_for {
        // Merge methods and nested types into target...
    }
    // Se impl_for è External o Failed → questo impl block viene ignorato
}
module.impl_blocks = vec![];  // Tutto svuotato
```

### Conseguenza
Se un impl block implementa metodi per un tipo esterno (es. `impl ExternalTrait for LocalType`), e il tipo target viene risolto come `External` o `Failed`, i metodi definiti in quell'impl block vengono **completamente scartati**. Non rimangono né nel tipo target (che non esiste in locale), né nel campo `impl_blocks` del modulo (che viene svuotato).

### Soluzione suggerita
Preservare gli impl blocks non fusi:

```rust
let mut unfused_impls = vec![];
for ib in resolved_impls {
    if let TypeRef::Resolved(target_name) = &ib.impl_for {
        // merge...
    } else {
        unfused_impls.push(ib);  // Conserva gli impl non fusi
    }
}
module.impl_blocks = unfused_impls;
```

**[Risolto]**: Corretto in `src/resolver.rs`. Gli `ImplBlock` che non vengono fusi nel target struct locale (perché puntano a dipendenze esterne o fallite) non vengono più scartati ma vengono conservati nel modulo tramite il vettore `unfused_impls`.

---

## 🟡 Problema 4: I metodi degli impl blocks non vengono registrati nello Scope Tree

**Gravità: Media** — I metodi definiti in impl blocks non sono risolvibili come nomi durante la risoluzione.

### Descrizione
Durante `ScopeTree::build`, la funzione `populate` processa gli impl blocks (righe 85-100) ma inserisce solo i `nested_types`. I **metodi** degli impl blocks non vengono inseriti nei `symbols` di nessun nodo:

```rust
for ib in &m.impl_blocks {
    // ... trova o crea target_idx ...
    for nested in &ib.nested_types {
        populate_st(nested, tree, target_idx);  // Solo nested types!
    }
    // I metodi NON vengono inseriti in symbols
}
```

### Conseguenza
Se una funzione chiama un metodo definito in un impl block (es. `Veicolo::new()`), quel nome non sarà trovabile nello Scope Tree durante la risoluzione, portando a un `TypeRef::Failed` anche se il metodo esiste nel codice.

### Soluzione suggerita
Inserire anche i metodi nei symbols del nodo target:

```rust
for method in &ib.methods {
    let method_name = method.name.last().cloned().unwrap_or_default();
    tree.arena[target_idx].symbols.insert(method_name, Component::Function(method.clone()));
}
```

**[Risolto (Obsoleto)]**: La costruzione statica di `ScopeTree::build` è stata eliminata. Nel nuovo sistema, il passo di pre-indicizzazione (Passo 1: Hoisting) è affidato al `GlobalRegistry`, il quale esplora l'IR e registra esplicitamente i percorsi assoluti di tutti i metodi definiti all'interno degli `ImplBlock`, rendendoli risolvibili dall'Environment.

---

## 🟢 Problema 5: Logica fallback per "crate" duplicata

**Gravità: Bassa** — Code smell, non un bug funzionale.

### Descrizione
La catena di fallback per validazione degli import (righe 228-303) contiene logica quasi identica per gli import con alias e senza alias. Entrambe le sezioni implementano la stessa sequenza:

1. Lookup diretto → 2. Preponi "root" → 3. Sostituisci "crate" con "root" → 4. External fallback

Questo codice è duplicato verbatim (~30 righe × 2).

### Soluzione suggerita
Estrarre la catena di fallback in una funzione ausiliaria:

```rust
fn validate_import_candidate(
    ctx: &ResolutionContext,
    name: &QualifiedName,
    candidate: QualifiedName,
) -> Option<ResolutionResult> {
    // ... logica unificata ...
}
```

**[Risolto]**: Corretto in `src/resolver.rs`. È stata creata una closure (helper function) `validate_import` ad inizio di `resolve_name_in_context` che consolida e centralizza interamente l'elaborazione dei fallback.

---

## 🟢 Problema 6: Wildcard imports non tentano fallback External

**Gravità: Bassa** — Limitazione nota.

### Descrizione
A differenza degli import normali (step 2a) che hanno una catena di fallback che termina con `ResolutionResult::External`, i wildcard imports (step 2b, righe 307-317) tentano solo `find_node_by_path`. Se il modulo importato con wildcard è esterno, il nome non viene risolto tramite questo meccanismo.

```rust
if imp.is_wildcard {
    let mut candidate = imp.path.clone();
    candidate.extend(name.clone());
    if ctx.tree.find_node_by_path(&candidate).is_some() {
        return Some(ResolutionResult::Local(candidate));
    }
    // ← Nessun fallback External qui
}
```

### Soluzione suggerita
Aggiungere un fallback External per i wildcard:

```rust
if ctx.tree.find_node_by_path(&candidate).is_some() {
    return Some(ResolutionResult::Local(candidate));
} else {
    // Opzionale: potrebbe essere un tipo esterno importato via wildcard
    return Some(ResolutionResult::External(candidate));
}
```

⚠️ Questa soluzione va applicata con cautela: senza verificare che il modulo wildcard importato esista effettivamente come dipendenza esterna, potrebbe generare falsi positivi.

**[Risolto (Intenzionalmente Ignorato)]**: Questa non è considerabile una limitazione ma un preciso design architetturale. Marcare arbitrariamente un import come `External` quando nasce da un *Wildcard* (es. `use library::*`) porterebbe l'analizzatore a convertire decine di istanziazioni dinamiche errate come finti nodi di terze parti. I wildcard sono adoperati solo per validare match locali. La scelta è intenzionalmente di scartare per sicurezza ed evitare *Polluted Graph Issues*.

---

## Riepilogo

| # | Problema | Gravità | Impatto |
|---|----------|---------|---------|
| 1 | `current_scope` mai aggiornato | 🔴 Alta | Lexical scoping non funziona — tutto parte da ROOT | **Risolto (Nuova Arch.)** |
| 2 | Cache key senza scope | 🟡 Media | Bug latente (si manifesta se #1 viene corretto) | **Risolto (Nuova Arch.)** |
| 3 | Impl blocks non-Resolved persi | 🟡 Media | Perdita di metodi per tipi esterni | **Risolto** |
| 4 | Metodi impl non nello Scope Tree | 🟡 Media | Metodi di impl block non risolvibili | **Risolto (Nuova Arch.)** |
| 5 | Logica fallback duplicata | 🟢 Bassa | Manutenibilità del codice | **Risolto** |
| 6 | Wildcard senza fallback External | 🟢 Bassa | Limitazione per wildcard import esterni | **Design Choice** |
