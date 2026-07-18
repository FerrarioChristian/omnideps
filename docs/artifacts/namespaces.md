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

## Proposta di Refactoring della `ModuleStrategy`

Attualmente abbiamo un Enum monolitico `ModuleStrategy`. Propongo di sostituirlo con una struct di flag booleani (es. `ModuleConfig`), in cui ogni campo attiva o disattiva un meccanismo specifico. Questo rende la formalizzazione LaTeX e l'architettura estremamente più precise.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Se true, il path del file e le directory contribuiscono al nome del modulo (es. Python, Rust)
    pub implicit_file_modules: bool,
    
    /// Se true, il linguaggio usa intestazioni a livello di file come `package x.y;` (es. Java)
    pub file_level_declarations: bool,
    
    /// Se true, il linguaggio usa blocchi AST interni al file come `namespace x {}` o `mod x {}` (es. C++, Rust)
    pub inline_module_blocks: bool,
}
```

### Come mapperebbero i linguaggi in questa nuova struttura?

| Linguaggio | `implicit_file_modules` | `file_level_declarations` | `inline_module_blocks` |
|------------|-------------------------|---------------------------|------------------------|
| **C**      | False                   | False                     | False                  |
| **C++**    | False                   | False                     | True                   |
| **Java**   | False                   | True                      | False                  |
| **Python** | True                    | False                     | False                  |
| **Rust**   | True                    | False                     | True                   |

### Vantaggi dell'approccio
1. **Semantica Corretta per il C:** Avendo tutti i flag a `false`, il C inserirà semplicemente tutto in `root`, che è esattamente ciò che ci si aspetta. Non dovremo usare un valore "None" speciale, sarà lo stato di default disattivo.
2. **Supporto Ibrido (Rust):** Supportare Rust diventerà naturale, attivando sia i file impliciti che i blocchi inline, senza che uno escluda l'altro come accadeva con l'Enum.
3. **Formalizzazione Matematica:** Anche nel documento LaTeX, questo set di booleani è molto più rigoroso da esprimere rispetto a un enum arbitrario.

### Open Questions per l'utente
Sei d'accordo con la scomposizione della `ModuleStrategy` in questi 3 flag ortogonali? 
Se sì, procederò a sostituire l'Enum nella codebase, ad aggiornare la tabella in LaTeX, e ad adattare il parser (`src/analyzer.rs`) affinché processi le regole dei package e delle directory in base a questi tre flag, piuttosto che affidarsi al match su `ModuleStrategy`.
