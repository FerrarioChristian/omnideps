# Analisi delle Dipendenze da Controllo di Flusso (Flow Control Dependencies)

Ho verificato lo stato attuale del codice (in particolare `src/model/components.rs`, `src/heuristics/body_extraction.rs` e il modulo `resolver/`). 
Le informazioni riportate nella tabella della roadmap al punto 6 **sono ancora veritiere**: la struttura `Block` non possiede alcun campo per tracciare le eccezioni catturate o lanciate. L'implementazione è quindi allo stato:
- **Exception Handling (catch)**: ❌ No
- **Throw**: ⚠️ Parziale (l'instanziazione dell'eccezione viene catturata ma mescolata genericamente nelle `instantiates`).

Tuttavia, con la maturità raggiunta dall'architettura (in particolare dal Query Engine V3), implementare queste dipendenze in modo pulito e language-agnostic è **estremamente semplice e diretto**. Di seguito descrivo la strategia architetturale per implementarle.

---

## Piano di Implementazione (Language-Agnostic)

### 1. Modifica del Modello IR (`src/model/components.rs`)
Il primo passo è espandere la struct `Block` per ospitare i nuovi vettori semantici:
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
La logica di estrazione si appoggia all'albero sintattico di Tree-sitter. 
In `extract_block`, quando si cicla sui figli del blocco (o all'interno di `find_behavioral_deps`), è sufficiente intercettare i nodi sintattici legati alle eccezioni:

- **Per il `catch`**:
  Cercare nodi con nomi language-agnostic (o le loro varianti) come `catch_clause` (Java), `catch_handler` (C++), o `except_clause` (Python).
  All'interno di questi nodi, estrarre il parametro (il tipo dell'eccezione catturata) utilizzando la funzione esistente `extract_type_ref`. Inserire il `TypeRef::Unresolved` risultante nell'array `catches`.

- **Per il `throw`**:
  Cercare nodi come `throw_statement` o `raise_statement`. 
  Invece di lasciare che la normale ricorsione inserisca l'eccezione lanciata nell'array generico `instantiates` (es. in `throw new IOException()`), estrarre il tipo dell'espressione e spingerlo nell'array `throws`.

### 3. Aggiornamento della Pipeline di Risoluzione (`src/resolver/`)
Poiché il motore di risoluzione è generalizzato per operare sui `TypeRef`, l'aggiornamento è banale e sfrutta tutta la potenza del motore V3 (sostituzione lessicale e lexical climbing):

- **In `builder.rs` (`build_block_queries`)**:
  Applicare `substitute_type` ai vettori `catches` e `throws`. Questo passaggio è **fondamentale**: se un programmatore fa `throw e;` (dove `e` è una variabile locale), il builder lo trasformerà in `Query::Find("e")`, ereditando automaticamente il tipo corretto!
  
- **In `executor.rs` (`execute_block`)**:
  Applicare `evaluate_typeref` per risolvere il percorso globale dell'eccezione interrogando lo ScopeTree.

### 4. Esportazione del Grafo (`src/export/graph.rs`)
Nell'ultimo step della pipeline, la funzione `add_block_edges` leggerà i nuovi array dal blocco risolto e genererà archi semantici dedicati:
- Per ogni elemento in `catches`, si emette un arco di tipo `CatchesException`.
- Per ogni elemento in `throws`, si emette un arco di tipo `ThrowsException`.

### Conclusione
Non ci sono ostacoli architetturali. Il design attuale è perfettamente predisposto per accogliere questa modifica in maniera orizzontale su tutta la pipeline, mantenendo intatta la filosofia language-agnostic dell'analizzatore e beneficiando gratuitamente della potenza deduttiva del Query Engine V3.
