# Problemi Identificati nella Name Resolution (ScopeTree + Query Engine)

Questo documento riepiloga e analizza le cause dei fallimenti riscontrati storicamente nel modulo Resolver (`src/resolver/builder.rs`, `src/resolver/executor.rs`, `src/resolver/scope.rs`) e le rispettive soluzioni integrate nell'analizzatore. 

Allo stato attuale, i problemi riportati di seguito sono stati **completamente risolti**.

## Riepilogo

| # | Sintomo | Gravità | File Coinvolti | Status e Spiegazione Soluzione |
|---|----------|---------|------|--------------------------------|
| 1 | Import wildcard valutano letteralmente `*` | 🔴 Alta | `executor.rs` | 🟢 Risolto. Ricerca prefix-based che risolve dinamicamente i percorsi degli import. |

---

## 🟢 Problema 1: Import wildcard non funzionanti nell'Executor

### Sintomo
Costrutti come `use std::io::*` non producevano risoluzioni corrette per i simboli non esplicitati. Tipi e funzioni importati tramite wildcard venivano etichettati falsamente come dipendenze `External` con un path invalido terminante con `*`, sebbene le entità esistessero effettivamente nello `ScopeTree`.

### Causa Tecnica
In `evaluate_query` (in `executor.rs`), quando veniva elaborato un import wildcard (`*`), l'algoritmo passava l'intero percorso alla funzione `find_global`. Essendo il percorso terminante con `*` (es. `["std", "io", "*"]`), il resolver cercava testualmente l'identificatore `*` all'interno dello scope, fallendo e restituendo un riferimento `External` errato.

### Soluzione Implementata
All'interno di `evaluate_query_find(name: &str, ...)`, quando la ricerca negli import incrocia un wildcard (o rileva il flag `is_wildcard`), la logica segue i seguenti passaggi:
1. Rimuove il token testuale `*` dal path dell'import.
2. Vi concatena il simbolo puntuale da risolvere (il parametro `name`, es. `"Read"`), formando il percorso effettivo `["std", "io", "Read"]`.
3. Interroga lo `ScopeTree` globalmente per verificare l'esistenza del path puntuale. Se esiste, la dipendenza viene risolta sul nodo specifico.
4. Se il nodo specifico non esiste, verifica l'assenza dell'intero modulo base (es. `["std", "io"]`) nell'albero. Se quest'ultimo è assente, l'algoritmo assume giustamente che l'entità derivi da una libreria non analizzata e genera un arco pulito `External(["std", "io", "Read"])` anziché connettere la dipendenza al carattere `*`.
