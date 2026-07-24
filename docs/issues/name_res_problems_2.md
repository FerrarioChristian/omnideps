# Problemi Identificati nella Name Resolution v3/v4 (ScopeTree + Query Engine)

Analisi dei file `src/resolver/builder.rs`, `src/resolver/executor.rs`, `src/resolver/scope.rs`.

---

## 🔴 Problema 1: Import wildcard non funzionanti nell'Executor

**Gravità: Media** — `use std::io::*` non produce risoluzioni corrette per i simboli non aliasati.

### Descrizione
In `evaluate_query` (in particolare nel controllo degli import in `executor.rs`), quando viene elaborato un import wildcard (`*`), il codice si comporta in questo modo:

```rust
for imp in &ctx.tree.arena[id].imports {
    if let Some(last) = imp.path.last() {
        if last == name || last == "*" {
            if let Some(resolved) = find_global(ctx, &imp.path) {
                return Some(resolved);
            } else {
                // If not found in the tree, it might be an external library
                return Some(TypeRef::External(imp.path.clone()));
            }
        }
    }
}
```

Quando `last == "*"`, il codice invoca `find_global(ctx, &imp.path)`. Tuttavia, `imp.path` termina letteralmente con il carattere `*` (es. `["std", "io", "*"]`). `find_global` cercherà quindi un nodo di nome `*` all'interno dello scope `std::io`, fallendo inevitabilmente e restituendo `External(["std", "io", "*"])` invece di cercare il vero `name` (es. `Read`) all'interno di `std::io`.

### Conseguenza
Tipi e funzioni importati tramite wildcard non vengono mai risolti localmente, anche se esistono nello `ScopeTree`. Verranno sempre falsamente etichettati come `External` con un path invalido che termina con `*`.

### Causa
Si tratta di un bug logico nell'implementazione attuale: quando si incontra un wildcard, bisognerebbe effettuare una ricerca prefix-based nello scope target (`std::io`) per il simbolo `name`, non inoltrare l'intero path wildcard al `find_global`.

### Soluzione Implementata
All'interno di `evaluate_query_find(name: &str, ...)`, il Query Engine sta cercando di risolvere un simbolo specifico (ovvero il parametro `name`, ad esempio `"Read"`).
Se durante la ricerca negli import validi in quello scope l'algoritmo incrocia un wildcard (es. `["std", "io", "*"]`), fa questo:
1. Rimuove il `*` dal path dell'import.
2. Vi appende il simbolo attualmente cercato (`name`), creando `["std", "io", "Read"]`.
3. Controlla globalmente se `["std", "io", "Read"]` esiste nel nostro albero. Se sì, la dipendenza è risolta.
4. Se non esiste nell'albero, controlla se per caso anche il modulo base (`["std", "io"]`) è assente dall'albero (significa che è una libreria esterna non analizzata). In quel caso, assume che `Read` provenga da lì, e genera un arco pulito `External(["std", "io", "Read"])` anziché `External(["std", "io", "*"])`.

---

## Riepilogo

| # | Problema | Gravità | File | Stato |
|---|----------|---------|------|-------|
| 1 | Import wildcard valutano letteralmente `*` | 🔴 Alta | `executor.rs` | **Risolto** — Causa path External invalidi e perdita di risoluzioni locali |
