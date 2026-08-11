# Gestione Moduli e Namespace: Architettura `ModuleConfig`

Il concetto di "Modulo" o "Package" nell'ingegneria del software non costituisce un paradigma mutualmente esclusivo (es. *DirectoryBased* vs *PackageBased*), ma rappresenta un insieme di **meccanismi ortogonali** che i vari linguaggi possono combinare, sovrapporre o omettere del tutto.

Questo documento illustra come l'analizzatore astrae e uniforma queste logiche attraverso l'architettura `ModuleConfig`.

---

## 1. Analisi dei Meccanismi per Linguaggio

Di seguito l'elenco delle modalità di definizione dei moduli e package nei principali ecosistemi supportati dall'analizzatore:

1. **C**
   - **Meccanismo:** Nessuno. Non esistono namespace testuali o moduli. Tutte le dichiarazioni vivono in un unico *global scope*.
   - **Comportamento Atteso:** L'analizzatore inietta tutte le entità estratte direttamente nel modulo base `root`.

2. **C++**
   - **Meccanismo:** Dichiarazioni di blocchi AST inline tramite `namespace x { ... }`.
   - **Nota:** I namespace possono essere annidati e distribuiti liberamente su più file. Il file system non ha alcun impatto sul FQN (Fully Qualified Name).
   - **Comportamento Atteso:** L'estrattore ricostruisce la gerarchia esclusivamente scorrendo i blocchi `namespace` all'interno dell'AST.

3. **Java**
   - **Meccanismo:** Dichiarazione a livello di file tramite `package x.y.z;`.
   - **Nota:** Il package è definito tramite un'intestazione statica. Sebbene la convenzione Java imponga una struttura delle directory corrispondente, l'analizzatore si affida unicamente alla direttiva testuale.
   - **Comportamento Atteso:** Tutti i nodi estratti dal file convergono nel namespace `root::x::y::z`.

4. **Python**
   - **Meccanismo:** Gerarchia basata su file system (`Directory / File Hierarchy`).
   - **Nota:** Ogni file `.py` costituisce un modulo; ogni cartella con `__init__.py` è un package. La struttura delle directory e dei file *è* la struttura logica del programma.
   - **Comportamento Atteso:** Il path relativo del file sorgente determina automaticamente la posizione nell'albero dei moduli.

5. **Rust**
   - **Meccanismo Ibrido:** Combina file system e blocchi inline.
   - **Nota:** Rust permette di montare file come moduli (`mod x;` caricherà il file `x.rs`), ancorando l'FQN al file system, e parallelamente di dichiarare sottomoduli in-memory (`mod x { ... }`).
   - **Comportamento Atteso:** Il file determina il modulo base genitore, ma all'interno del file è consentito espandere l'albero tramite blocchi AST inline.

---

## 2. L'Architettura `ModuleConfig`

Per riflettere la natura ibrida descritta, il sistema adotta una struct di configurazione formata da flag booleani indipendenti, denominata `ModuleConfig`. Ogni asse attiva o disattiva un meccanismo specifico in maniera puramente ortogonale.

L'implementazione (presente in `src/config.rs`) definisce i seguenti 5 assi comportamentali:

```rust
pub struct ModuleConfig {
    /// Se true, ogni file crea implicitamente un modulo omonimo (es. Python, Rust)
    pub file_based: bool,
    
    /// Se true, l'albero delle directory modella attivamente la gerarchia (es. Python, Rust)
    pub directory_based: bool,
    
    /// Se true, il linguaggio usa intestazioni dichiarative `package x.y;` (es. Java)
    pub package_decl_based: bool,
    
    /// Se true, il linguaggio usa blocchi AST sintattici `namespace x { ... }` (es. C++)
    pub namespace_based: bool,
    
    /// Se true, il linguaggio usa blocchi AST sintattici per sottomoduli `mod x { ... }` (es. Rust)
    pub inline_mod_based: bool,
}
```

### Configurazione per Linguaggio

| Linguaggio | `file_based` | `directory_based` | `package_decl_based` | `namespace_based` | `inline_mod_based` |
|------------|--------------|-------------------|----------------------|-------------------|--------------------|
| **C**      | False        | False             | False                | False             | False              |
| **C++**    | False        | False             | False                | True              | False              |
| **Java**   | False        | False             | True                 | False             | False              |
| **Python** | True         | True              | False                | False             | False              |
| **Rust**   | True         | True              | False                | False             | True               |

### Vantaggi dell'Astrazione Ortogonale

1. **Semantica Naturale per il Global Scope:** Avendo tutti i flag impostati a `false`, il linguaggio C inietta correttamente le dichiarazioni nel *global scope* (il nodo `root`), senza necessitare di eccezioni o workaround logici.
2. **Supporto Ibrido Trasparente:** Il supporto a linguaggi complessi come Rust diviene naturale. È possibile attivare simultaneamente l'estrazione implicita dai percorsi file/directory e l'intercettazione dei blocchi inline, combinando gli effetti anziché costringerli in un enumeratore esclusivo (come una classica enum `ModuleStrategy`).
3. **Formalizzazione Teorica:** Nel contesto accademico e architetturale, questo set di assi di funzionalità booleane offre un approccio formale ed elegante, permettendo all'estrattore CST (Concrete Syntax Tree) di comporre le regole di analisi iterativamente e deterministicamente.
