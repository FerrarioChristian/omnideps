# Gestione Moduli e Namespace: Analisi e Proposta

Il relatore ha fatto un'osservazione eccellente. Il concetto di "Modulo" o "Package" non è un'opzione mutuamente esclusiva (`DirectoryBased` o `PackageBased`), ma piuttosto un insieme di **meccanismi ortogonali** che i linguaggi possono combinare o omettere del tutto.

## Analisi dei Meccanismi per Linguaggio

Ecco una lista delle modalità di definizione dei moduli/package nei vari ecosistemi supportati:

1. **C**
   - **Meccanismo:** Nessuno. Non esistono namespace o moduli. Tutte le dichiarazioni vivono in un unico *global scope* (salvo visibilità `static` limitata alla translation unit, che l'analizzatore in genere appiattisce).
   - **Comportamento atteso:** Tutto viene iniettato nel modulo `root`.

2. **C++**
   - **Meccanismo:** `namespace x { ... }` (Dichiarazioni di blocchi inline).
   - **Nota:** I namespace possono essere annidati e distribuiti su più file. Il file system non ha alcun impatto sull'FQN (Fully Qualified Name). (Esistono anche i moduli C++20, ma il namespace è il costrutto dominante per lo scope).
   - **Comportamento atteso:** Si estrae la gerarchia esclusivamente scorrendo i blocchi `namespace` nell'AST.

3. **Java**
   - **Meccanismo:** `package x.y.z;` (Dichiarazione a livello di file).
   - **Nota:** Il package è definito tramite uno statement a inizio file. Anche se la convenzione impone che la struttura delle directory rifletta il package, il compilatore (e il nostro analizzatore) si affida all'intestazione. Non esistono package dichiarati "inline" con le parentesi graffe.
   - **Comportamento atteso:** Tutti i nodi estratti dal file finiscono in `root::x::y::z`.

4. **Python**
   - **Meccanismo:** `Directory / File Hierarchy` (Moduli impliciti basati sul file system).
   - **Nota:** Ogni file `.py` è un modulo. Ogni cartella con `__init__.py` è un package. La struttura delle directory e dei file *è* la struttura dei moduli.
   - **Comportamento atteso:** Il path del file determina automaticamente la posizione nell'albero dei moduli.

5. **Rust**
   - **Meccanismo Ibrido:** Usa sia il File System che blocchi Inline.
   - **Nota:** Rust permette sia di montare file come moduli (`mod x;` caricherà `x.rs`), facendo corrispondere file system e FQN, sia di dichiarare sottomoduli inline (`mod x { ... }`).
   - **Comportamento atteso:** Il file determina il modulo base, ma all'interno del file si possono creare ulteriori sottomoduli tramite blocchi AST.

## Refactoring Implementato: `ModuleConfig`

Come concordato, l'Enum monolitico `ModuleStrategy` è stato definitivamente sostituito con una struct di flag booleani (`ModuleConfig`), in cui ogni campo attiva o disattiva un meccanismo specifico in modo ortogonale. Questo rende l'architettura estremamente più flessibile e formalmente corretta.

L'implementazione attuale in `src/config.rs` definisce i seguenti 5 assi ortogonali:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    /// Se true, ogni file crea implicitamente un modulo con il suo nome (es. Python, Rust)
    pub file_based: bool,
    
    /// Se true, l'albero delle directory modella la gerarchia dei moduli (es. Python, Rust)
    pub directory_based: bool,
    
    /// Se true, il linguaggio usa intestazioni a livello di file come `package x.y;` (es. Java)
    pub package_decl_based: bool,
    
    /// Se true, il linguaggio usa blocchi AST interni al file del tipo `namespace x { ... }` (es. C++)
    pub namespace_based: bool,
    
    /// Se true, il linguaggio usa blocchi AST interni al file del tipo `mod x { ... }` (es. Rust)
    pub inline_mod_based: bool,
}
```

### Come mappano i linguaggi in questa nuova struttura

| Linguaggio | `file` | `directory` | `package_decl` | `namespace` | `inline_mod` |
|------------|--------|-------------|----------------|-------------|--------------|
| **C**      | False  | False       | False          | False       | False        |
| **C++**    | False  | False       | False          | True        | False        |
| **Java**   | False  | False       | True           | False       | False        |
| **Python** | True   | True        | False          | False       | False        |
| **Rust**   | True   | True        | False          | False       | True         |

### Vantaggi Ottenuti
1. **Semantica Corretta per il C:** Avendo tutti i flag a `false`, il C inserisce semplicemente tutte le dichiarazioni in `root`, il global scope atteso. Non è stato necessario introdurre alcun valore "None" speciale.
2. **Supporto Ibrido (Rust):** Supportare Rust è diventato naturale, potendo attivare sia l'estrazione implicita dai file/directory (`file_based`, `directory_based`) sia l'intercettazione dei blocchi inline (`inline_mod_based`), senza che un paradigma escluda l'altro come accadeva con il vecchio Enum.
3. **Formalizzazione Matematica (Tesi):** Nel documento LaTeX, questo set di assi di funzionalità booleane è molto più elegante e rigoroso da esprimere e schematizzare rispetto a un enumeratore arbitrario ed esclusivo. L'estrattore CST ora può semplicemente applicare le regole attivate componendole in sequenza.
