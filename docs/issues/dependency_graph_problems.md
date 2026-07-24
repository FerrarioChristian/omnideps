# Problemi Identificati nella Fase 3 — Costruzione del Dependency Graph

Analisi dei file `src/export/graph.rs`, `src/export/cytoscape.rs`, `src/export/summary.rs`.

## Riepilogo Finale

| # | Problema | Gravità | File | Status e Spiegazione Soluzione |
| --- | ---------- | --------- | ------ | -------------------------------- |
| 1 | Archi duplicati non deduplicati | 🟢 Risolto | `cytoscape.rs` | Standardizzazione del `DependencyGraph` con Deduplicazione hash-based pre-export. |
| 2 | `Inherits` vs `Implements` incoerente | 🟢 Risolto | `graph.rs` | Adozione del macro-arco universale `IsA` (Design Choice language-agnostic). |
| 3 | Summary non conta metriche di risoluzione | 🟢 Risolto | `summary.rs` | Implementata Tree-Traversal esaustiva su tutti gli strati dell'IR. |
| 4 | Nodi Cytoscape External troppo generici | 🟢 Risolto | `cytoscape.rs` | Scelta voluta per contenimento visual cluttering UI; dati esatti in JSON. |

*Nota: I problemi legati alla vecchia architettura basata sul flattening (es. ImplBlock orfani, funzioni libere, type_ref matches, UsesLocalType) sono stati definitivamente eliminati con l'adozione della V4 (ScopeTree e Name Resolution su Query).*

---

## 🟢 Problema 1: Possibili archi duplicati nella generazione

**Gravità: Media** — Archi ridondanti nel grafo.

### Descrizione Storica

Se una classe aveva più attributi dello stesso tipo, o se una funzione invocava lo stesso metodo di un'altra classe multipli volte, il costruttore del grafo emetteva un nuovo arco per ciascuna occorrenza, inflazionando le metriche architetturali e intasando i tool di visualizzazione con linee sovrapposte.

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** implementando i tratti di equivalenza e ordinamento sull'entità arco all'interno del sistema di export. 
Immediatamente prima che il grafo venga restituito e serializzato per Cytoscape, viene applicata una pipeline di deduplicazione (eliminazione dei duplicati logici). Questo garantisce che l'Output rappresenti un grafo semplice, essenziale per calcolare accoppiamenti (Coupling) reali tra componenti indipendenti dalle ripetizioni locali del codice.

---

## 🟢 Problema 2: `Inherits` vs `Implements` — semantica incoerente

**Gravità: Media** — Confusione semantica nell'output.

### Descrizione Storica

Linguaggi a singola ereditarietà ma con interfacce (come Java e C#) usano keyword diverse (`implements` vs `extends`). Inizialmente, il sistema tentava di mantenere questa distinzione in `super_types`, ma poi emetteva indiscriminatamente archi `Inherits` per tutti, perdendo la semantica.

### Soluzione Implementata (Attuale / Design Choice)

Il dilemma è stato risolto tramite una precisa **Design Choice** teorica. Per mantenere l'analizzatore *language-agnostic*, i costrutti specifici dei linguaggi a oggetti sono stati astratti. È stato introdotto un singolo arco universale, denominato **`IsA`** (o `inherits` a livello base), che rappresenta indistintamente sia l'ereditarietà di classe sia l'implementazione di interfacce o trait.
Sotto il profilo del design architetturale a grana grossa, sapere che un modulo dipende da un'astrazione tramite ereditarietà o contratto comporta la medesima dipendenza polimorfica.

---

## 🟢 Problema 3: Il summary non conta metodi né archi risolti/falliti

**Gravità: Media** — Informazione incompleta per analisi di qualità.

### Descrizione Storica

Il file di sommario statistico (`summary.rs`) dichiarava contatori per i referimenti risolti e falliti (`resolved_refs`, `unknown_refs`), ma questi rimanevano inesorabilmente a 0 poiché la procedura contava solo il numero di nodi primari senza ispezionare le dichiarazioni di tipo dei loro attributi.

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** in modo completo in `src/export/summary.rs`. È stata scritta una logica di attraversamento (tree traversal) esaustiva (`count_refs_in_st` e `count_refs_in_func`). Tali procedure scendono ricorsivamente all'interno di tutti i componenti estratti (`super_types`, `fields`, `methods`, `nested_types`, `parameters`, `return_type`, `calls` e `instantiates`). 
Questa logica estrae statistiche accurate anche nella V4 post-ScopeTree Resolution, contando ogni `TypeRef::Resolved` contro quelli falliti, rendendo affidabile il benchmarking della tesi.

---

## 🟢 Problema 4: L'esportazione Cytoscape crea nodi "External" senza distinguere il tipo reale

**Gravità: Bassa** — Informazione persa nella visualizzazione.

### Descrizione Storica

Se l'analizzatore incontrava una dipendenza verso una classe esterna o sconosciuta (non presente nel progetto, es. librerie standard), il plug-in di esportazione Cytoscape disegnava un nodo nominandolo sempre e solo `"External"`, perdendo l'identità del target (es. `"String"`, `"ArrayList"`).

### Soluzione Implementata (Attuale / Design Choice)

Classificata come **Design Choice**. Cytoscape funge unicamente da visualizzatore macro-strutturale interattivo. Raggruppare tutte le innumerevoli classi delle librerie esterne sotto un singolo nodo di escape (`External`) è fondamentale per prevenire il *visual cluttering* (esplosione dei nodi sullo schermo che rende invisibile la topologia reale del progetto locale in esame). L'identità formale esatta della dipendenza esterna è comunque preservata al 100% all'interno dell'esportazione testuale/JSON, che resta il vero artefatto dell'analisi.
