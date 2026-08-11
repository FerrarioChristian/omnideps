# Analisi delle Dipendenze da Controllo di Flusso (Flow Control Dependencies)

La struttura corrente dell'Intermediate Representation (IR), nello specifico la struct `Block`, non possiede alcun campo per tracciare esplicitamente le eccezioni catturate o lanciate. L'implementazione attuale è quindi:

- **Exception Handling (catch)**: ❌ Non Supportato
- **Throw**: ⚠️ Parzialmente Supportato (l'instanziazione dell'eccezione viene catturata ma mescolata genericamente nell'array delle `instantiates`).

Con la maturità raggiunta dall'architettura del *Query Engine V3*, implementare queste dipendenze in modo pulito e language-agnostic risulta un'estensione diretta. Di seguito è definita la strategia architetturale per l'implementazione.

---

## Piano di Implementazione (Language-Agnostic)

### 1. Modifica del Modello IR (`src/model/components.rs`)
Il primo passo consiste nell'espandere la struct `Block` per ospitare i nuovi vettori semantici:

```rust
pub struct Block {
    // ... campi esistenti (declarations, calls, accesses, ecc.) ...
    
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub catches: Vec<TypeRef>,
    
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub throws: Vec<TypeRef>,
}
```

### 2. Estrazione Euristica (`src/heuristics/body_extraction.rs`)
La logica di estrazione si appoggia all'albero sintattico di Tree-sitter. All'interno di `extract_block` (o `find_behavioral_deps`), è necessario intercettare i nodi sintattici legati alle eccezioni:

- **Per il `catch`**:
  Identificare i nodi con nomi language-agnostic come `catch_clause` (Java), `catch_handler` (C++), o `except_clause` (Python).
  All'interno di questi nodi, estrarre il parametro (il tipo dell'eccezione catturata) utilizzando la funzione esistente `extract_type_ref` e inserire il `TypeRef::Unresolved` risultante nell'array `catches`.

- **Per il `throw`**:
  Identificare nodi come `throw_statement` o `raise_statement`. Invece di delegare la normale ricorsione che inserirebbe l'eccezione nell'array `instantiates` (es. `throw new IOException()`), estrarre il tipo dell'espressione e spingerlo nell'array `throws`.

### 3. Aggiornamento della Pipeline di Risoluzione (`src/resolver/`)
Poiché il motore di risoluzione opera uniformemente sui `TypeRef`, l'aggiornamento sfrutta nativamente la sostituzione lessicale e il lexical climbing del Query Engine V3:

- **In `builder.rs` (`build_block_queries`)**:
  Applicare `substitute_type` ai vettori `catches` e `throws`. Questo passaggio è fondamentale: se il codice esegue `throw e;` (dove `e` è una variabile locale), il builder lo trasformerà in `Query::Find("e")`, ereditandone automaticamente il tipo logico.
  
- **In `executor.rs` (`execute_block`)**:
  Applicare `evaluate_typeref` per risolvere il percorso globale dell'eccezione interrogando lo `ScopeTree`.

### 4. Esportazione del Grafo (`src/export/graph.rs`)
Nell'ultimo step della pipeline, la funzione `add_block_edges` leggerà i nuovi array dal blocco risolto per generare archi semantici dedicati:
- Per ogni elemento in `catches`, l'esportatore emette un arco di tipo `CatchesException`.
- Per ogni elemento in `throws`, l'esportatore emette un arco di tipo `ThrowsException`.

### Considerazioni Architetturali
Il design attuale dell'analizzatore è perfettamente predisposto per accogliere questa modifica in maniera orizzontale su tutta la pipeline. L'approccio mantiene intatta la filosofia language-agnostic e beneficia immediatamente della potenza deduttiva del Query Engine V3.
