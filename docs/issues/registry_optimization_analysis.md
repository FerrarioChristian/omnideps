# Analisi di Ottimizzazione: Global Registry

Questo documento analizza la necessità reale e le possibili implementazioni per ottimizzare il `GlobalRegistry`, l'indice centrale del Query Engine.

## 1. Il Problema è Reale?

Attualmente il `GlobalRegistry` è implementato come una mappa piatta:
```rust
pub struct GlobalRegistry {
    pub paths: HashMap<Vec<String>, RegistryEntry>,
}
```

### Come viene allocata la memoria oggi
Ogni `QualifiedName` è un `Vec<String>`. In Rust, una singola `String` occupa 24 byte sullo stack (pointer, capacity, length) più la memoria heap per i caratteri (ad esempio 5 byte per `"core"`). 
Un percorso come `["root", "core", "utils", "Database"]` richiede:
1. Allocazione del `Vec` (24 byte)
2. Allocazione di 4 `String` (4 * 24 = 96 byte) sullo stack del Vec
3. Allocazione dei 4 frammenti di testo sull'heap (es. ~20 byte)
**Totale: ~140 byte per chiave.**

### Impatto su progetti di dimensioni diverse
- **Progetto Piccolo (1.000 componenti)**: ~140 KB di chiavi. Irrilevante.
- **Progetto Medio (50.000 componenti)**: ~7 MB di chiavi. Assolutamente accettabile per i PC moderni.
- **Progetto Enorme (Linux Kernel, Chromium - ~5 milioni di componenti)**: ~700 MB solo per le chiavi dell'HashMap (senza contare i valori `RegistryEntry` e il layout di memoria frammentato).

### Il problema occulto: La frammentazione della Cache della CPU
Il problema reale in Rust non è tanto la RAM totale (avere un analizzatore che consuma 1GB di RAM per un progetto enorme è nella norma per tool come `rust-analyzer`), quanto il **Cache Miss Rate**. Poiché ogni segmento del path vive in un'area di heap diversa, calcolare l'hash di `["root", "core", "utils", "Database"]` costringe la CPU a "saltare" continuamente nella memoria heap per leggere i byte delle stringhe, invalidando la cache L1/L2 e rallentando enormemente i lookup.

### Verdetto
**Vale la pena risolverlo?** 
*Ni.* Se l'obiettivo dell'analizzatore è processare progetti da 10-100k linee di codice, la struttura attuale è perfetta: il codice è leggibile (Clean Code) e abbastanza veloce. 
Se invece l'obiettivo della tua tesi è dimostrare un'architettura **"Enterprise-Ready" e iper-scalabile**, ottimizzare questo aspetto darà una marcia in più al progetto da un punto di vista ingegneristico.

---

## 2. Come si potrebbe risolvere (Le 2 Strategie)

Esistono due pattern architetturali standard nei compilatori per risolvere questo problema: lo **String Interning** e il **Prefix Trie**.

### Soluzione A: String Interning (Consigliata)
Invece di clonare le stringhe ovunque, usiamo una "piscina" centrale (Interner) che salva ogni parola univoca una sola volta e restituisce un numero intero (`SymbolId` a 32-bit).

```rust
// Prima:
type QualifiedName = Vec<String>; // Molto costoso

// Dopo:
type SymbolId = u32; // Un intero da 4 byte
type QualifiedName = Vec<SymbolId>; // Un vettore di interi (contiguo in memoria!)
```

**Vantaggi:**
- Veloce da implementare (basta usare il crate standard de-facto `ustr` oppure `string_interner`).
- Drastica riduzione della memoria e abbattimento dei Cache Miss: fare l'hash di `[10, 45, 99, 102]` (4 interi contigui) è pressoché istantaneo per la CPU.
- Non devi stravolgere la logica dell'Executor, l'algoritmo algebrico rimane identico, si confrontano solo interi invece di stringhe.

### Soluzione B: Prefix Trie (Albero dei Suffissi/Prefissi)
Invece di avere una mappa piatta, modelliamo il registro come un albero dei namespace, dove ogni nodo rappresenta una cartella o un modulo:

```rust
struct TrieNode {
    name: String, // Es: "utils"
    entry: Option<RegistryEntry>,
    children: HashMap<String, TrieNode>,
}
```
Per cercare `["root", "core", "utils"]`, si parte dalla radice, si cerca `"root"`, si scende nel figlio `"core"`, e così via.

**Vantaggi:**
- Modella perfettamente e intrinsecamente il *Lexical Scoping* (la risalita degli scope `Query::Find` diventerebbe letteralmente una risalita tra puntatori parent dell'albero).
- Nessuna duplicazione dei prefissi in memoria (il prefisso `"root" -> "core"` esiste una volta sola per mille funzioni al suo interno).

**Svantaggi:**
- L'implementazione è complessa, specialmente in Rust a causa del Borrow Checker (gli alberi con puntatori `parent` sono ostili in Rust puro, richiedono `Rc<RefCell<T>>` o id basati su indici/arene).
- Il lookup richiede molti salti di puntatore (un salto per ogni segmento del path).

## Conclusione

Dal punto di vista dell'ingegneria del software, **lo String Interning è nettamente superiore** come rapporto costo/beneficio. Mantiene l'eleganza algebrica attuale del tuo Query Engine fornendo però prestazioni da compilatore di livello industriale. Il Trie, seppur elegante concettualmente, complicherebbe inutilmente le query algebriche.
