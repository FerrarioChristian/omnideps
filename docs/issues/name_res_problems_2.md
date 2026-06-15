# Problemi Identificati nella Name Resolution v3 — Query Engine Architecture

Analisi dei file `src/resolver/mod.rs`, `src/resolver/builder.rs`, `src/resolver/executor.rs`, `src/resolver/registry.rs`, `src/resolver/stack.rs`, `src/resolver/primitives.rs`.

---

## ✅ Problemi della v2 RISOLTI nella v3

| # v2 | Problema | Stato v3 |
|------|----------|----------|
| 1 | Global Registry non registra nested_types delle struct | ✅ **Risolto** — `register_structured_type` è ricorsiva |
| 2 | Global Registry non registra nested_types degli impl blocks | ✅ **Risolto** — il loop sugli impl chiama `register_structured_type` per i nested |
| 3 | Wildcard import solo per Local | ⚠️ **Rimosso** — gli import wildcard non sono gestiti nell'Executor (vedi Problema 1 sotto) |
| 4 | resolve_function push/pop di scope vuoto | ✅ **Risolto** — i parametri vengono registrati come `Query` nel Builder |
| 5 | validate_import catena di fallback asimmetrica | ✅ **Risolto** — logica semplificata nell'Executor (`Find` climbing + import check) |
| 6 | Impl blocks non hoisted nello scope | ✅ **N/A** — il Builder non fa hoisting di componenti, solo sostituzione lessicale |
| 7 | scope_key con "::" collisione | ✅ **Risolto** — la Cache è stata completamente rimossa |
| 8 | Inherits vs Implements | ⚠️ **Aperto** — invariato, ereditato dalla v2 |
| 9 | RefCell fragile | ⚠️ **Parzialmente risolto** — RefCell usato solo nel Builder; l'Executor usa contesti immutabili |
| 10 | Dichiarazioni registrate solo se Resolved | ⚠️ **Aperto** — stesso pattern nel Builder (vedi sotto) |
| 11 | add_block_edges usa UsesFieldType per variabili locali | ⚠️ **Aperto** — riguarda `graph.rs`, non il resolver |

---

## 🟡 Problema 1 (NUOVO): Import wildcard non gestiti nell'Executor

**Gravità: Media** — `use std::io::*` non produce risoluzioni.

### Descrizione
In `evaluate_query` (`executor.rs:188-201`), durante la fase di check degli import, vengono controllati **solo** gli alias espliciti e l'ultimo segmento del path:

```rust
for imp in level_imports {
    if let Some(alias) = &imp.alias {
        if alias == name { return Some(imp.path.clone()); }
    } else if let Some(last) = imp.path.last() {
        if last == name { return Some(imp.path.clone()); }
    }
}
```

Non c'è nessun branch per `imp.is_wildcard`. Un import come `use std::io::*` non produrrà mai un match.

### Conseguenza
Tipi importati tramite wildcard non vengono risolti e finiranno come `Failed` o `Primitive` (se il nome coincide con un primitivo).

### Possibile soluzione
Aggiungere un branch per i wildcard che tenti `imp.path + [name]` e verifichi nel registry:
```rust
if imp.is_wildcard {
    let mut candidate = imp.path.clone();
    candidate.push(name.clone());
    // Verificare se esiste o trattare come External
}
```

---

## 🟡 Problema 2 (NUOVO): Dichiarazioni locali e parametri registrati solo se `ResolutionQuery`

**Gravità: Media** — Variabili con tipi primitivi o già risolti non diventano regole di sostituzione.

### Descrizione
In `build_function_queries` (`builder.rs:73-77`) e `build_block_queries` (`builder.rs:92-97`):

```rust
if let TypeRef::ResolutionQuery(ref q) = p.ty {
    ctx.stack.borrow_mut().define_symbol(name.clone(), q.clone());
}
```

Se il tipo è `Primitive(String)` o `Unresolved` che non viene trasformato (es. vuoto), la variabile non viene registrata nello stack.

### Conseguenza
Se `let s: String = ...` e poi si usa `s.len()`, la variabile `s` non sarà nello stack perché `String` potrebbe non generare una `ResolutionQuery` (dipende dal flusso). In pratica, il pattern `Unresolved → substitute_type → ResolutionQuery` copre la maggior parte dei casi, ma edge cases con tipi primitivi rimangono.

---

## 🟡 Problema 3 (NUOVO): `Extract` non verifica esistenza nel registry

**Gravità: Media** — Possibili path fantasma.

### Descrizione
In `evaluate_query`, il branch `Extract` (`executor.rs:222-231`) non verifica se il path costruito esiste:

```rust
Query::Extract(parent_query, member_name) => {
    if let Some(mut parent_path) = evaluate_query(ctx, parent_query, true) {
        parent_path.push(member_name.clone());
        Some(parent_path)  // ← Restituisce SEMPRE, senza verificare!
    } else { None }
}
```

Il commento nel codice dice: "We don't check existence immediately, because it might be an external chain". Tuttavia, questo significa che path completamente inventati come `["std", "nonexistent", "Fake"]` verranno restituiti come risultato valido e diventeranno `External` in `evaluate_typeref`.

### Conseguenza
Potenziali falsi `External` per catene multi-livello con nomi errati. Il trade-off è accettabile per supportare catene esterne come `std::vec::Vec`, ma produce rumore nell'output.

---

## 🟡 Problema 4 (NUOVO): `Call` con `resolve_call_return: false` non effettua inferenza

**Gravità: Media** — Le chiamate nella generazione degli archi non risolvono il tipo di ritorno.

### Descrizione
`evaluate_typeref` chiama `evaluate_query(ctx, &query, false)`. Questo significa che per una query come `Call(Find("f"))`:
- `resolve_call_return = false` → restituisce il path della funzione stessa, non il suo tipo di ritorno
- Il risultato diventa `Resolved(["root", "f"])` → un arco verso la funzione

Questo è **corretto** per gli archi di dipendenza (vogliamo sapere che *chiama* `f`), ma non è corretto se qualcuno volesse il tipo di ritorno dell'espressione.

### Nota
Questo è il design intenzionale: `evaluate_typeref` deve produrre archi di dipendenza, non inferire tipi di ritorno. Il flag `resolve_call_return: true` è usato solo internamente dall'`Extract` per le catene. Non è un bug ma una scelta documentata.

---

## 🟢 Problema 5: Il Builder non registra import nello SymbolStack

**Gravità: Bassa** — Design intenzionale, ma non ovvio.

### Descrizione
Lo `StackFrame` ha un campo `imports: Vec<Import>`, ma il Builder non chiama mai `add_import`. Gli import sono gestiti **solo** nell'Executor tramite `imports_stack`.

### Conseguenza
Il campo `imports` in `StackFrame` è sempre vuoto. Il codice è pulito ma il campo è vestigiale.

### Suggerimento
Rimuovere il campo `imports` da `StackFrame` se non viene mai usato, oppure documentare esplicitamente che gli import sono gestiti dall'Executor.

---

## 🟢 Problema 6: `Inherits` vs `Implements` ereditato dalla v2

**Gravità: Bassa** — Il flattening degli impl blocks aggiunge i trait a `super_types`, che poi generano tutti archi `Inherits`. Nessuna distinzione semantica tra ereditarietà e implementazione.

---

## ✅ Miglioramenti v2 → v3

| Aspetto | v2 (Symbol Stack monofase) | v3 (Query Engine bifase) |
|---|---|---|
| Fasi | Singolo pass | Due fasi: Substitution → Navigation |
| SymbolStack mappa | `String → QualifiedName` | `String → Query` |
| Rappresentazione intermedia | Nessuna | `TypeRef::ResolutionQuery(Query)` |
| Forward references | Problematici | Risolti nativamente |
| GlobalRegistry | `HashSet<QN>` | `HashMap<QN, RegistryEntry>` con type inference |
| Cache | `ResolutionCache` con scope_key | Rimossa — non necessaria |
| Catene di chiamate | Non supportate | Supportate via `Call(Extract(...))` |
| self/this | Non gestiti | Iniettati automaticamente |
| Primitivi | Marcati come Failed | Intercettati dal PrimitiveRegistry data-driven |
| nested_types nel registry | ❌ Bug | ✅ Registrati ricorsivamente |

---

## Riepilogo

| # | Problema | Gravità | File | Stato |
|---|----------|---------|------|-------|
| 1 | Import wildcard non gestiti nell'Executor | 🟡 Media | `executor.rs` | Nuovo |
| 2 | Dichiarazioni registrate solo se ResolutionQuery | 🟡 Media | `builder.rs` | Nuovo |
| 3 | Extract non verifica esistenza nel registry | 🟡 Media | `executor.rs` | Nuovo (trade-off) |
| 4 | Call con resolve_call_return=false | 🟡 Media | `executor.rs` | Design intenzionale |
| 5 | Campo imports vestigiale in StackFrame | 🟢 Bassa | `stack.rs` | Nuovo |
| 6 | Inherits vs Implements ereditato | 🟢 Bassa | `executor.rs` | Ereditato dalla v2 |
