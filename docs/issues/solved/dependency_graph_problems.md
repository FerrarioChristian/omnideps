# Problemi Identificati nella Fase 3 — Costruzione del Dependency Graph

Questo documento analizza e riepiloga le soluzioni storiche apportate alla fase di generazione, deduplicazione ed esportazione del Dependency Graph (moduli `src/export/graph.rs`, `src/export/cytoscape.rs`, `src/export/summary.rs`).

Allo stato attuale, i problemi riportati di seguito sono stati **completamente risolti**.

## Riepilogo

| # | Sintomo | Gravità | File Coinvolti | Status e Spiegazione Soluzione |
| --- | ---------- | --------- | ------ | -------------------------------- |
| 1 | Archi duplicati non deduplicati | 🟢 Risolto | `cytoscape.rs` | Standardizzazione del `DependencyGraph` con Deduplicazione hash-based pre-export. |
| 2 | `Inherits` vs `Implements` incoerente | 🟢 Risolto | `graph.rs` | Adozione del macro-arco universale `IsA` (Design Choice language-agnostic). |
| 3 | Summary non conta metriche di risoluzione | 🟢 Risolto | `summary.rs` | Implementata Tree-Traversal esaustiva su tutti gli strati dell'IR. |
| 4 | Nodi Cytoscape External troppo generici | 🟢 Risolto | `cytoscape.rs` | Raggruppamento voluto per contenere il visual cluttering nell'interfaccia UI. |

*Nota: I problemi legati all'architettura originaria basata sul flattening (es. ImplBlock orfani, funzioni libere, type_ref matches) sono stati superati e deprecati definitivamente con l'adozione del Query Engine.*

---

## 🟢 Problema 1: Possibili archi duplicati nella generazione

### Sintomo
Se una classe accedeva a multipli attributi dello stesso tipo, o se una funzione invocava reiteratamente lo stesso metodo di un'altra classe, il costruttore del grafo emetteva un arco distinto per ciascuna occorrenza. Ciò inflazionava le metriche architetturali e comprometteva la leggibilità nei tool di visualizzazione a causa di archi sovrapposti.

### Soluzione Implementata
È stata implementata una logica formale di equivalenza e ordinamento per le entità arco all'interno del sistema di esportazione. Prima che il grafo venga serializzato per l'output, i risultati attraversano una pipeline logica di deduplicazione. Questo assicura che il Grafo delle Dipendenze sia topologicamente semplice (senza archi multipli tra la stessa coppia direzionale), risultando fondamentale per misurare il Coupling reale tra componenti.

---

## 🟢 Problema 2: `Inherits` vs `Implements` — semantica incoerente

### Sintomo
I linguaggi a singola ereditarietà ma con interfacce (come Java) impiegano keyword differenti (`implements` vs `extends`). Inizialmente, il sistema tentava di distinguere le due semantiche nell'IR, per poi fonderle indiscriminatamente come archi `Inherits`, perdendone la specificità originaria.

### Soluzione Implementata (Design Choice)
Il sistema è stato standardizzato per aderire a una logica *language-agnostic*. I costrutti polimorfici specifici dei linguaggi OO sono stati astratti introducendo un singolo arco universale, denominato **`IsA`** (o `inherits` a livello base). A fini macro-architetturali, tale astrazione modella in modo equivalente l'accoppiamento polimorfico derivante sia dall'ereditarietà di classe che dall'implementazione di contratti.

---

## 🟢 Problema 3: Metriche di risoluzione incomplete nel Summary

### Sintomo
Il modulo di sommario statistico (`summary.rs`) includeva metriche per i riferimenti risolti e falliti (`resolved_refs`, `unknown_refs`), ma queste restituivano sistematicamente valori pari a zero. La procedura di aggregazione contava esclusivamente i nodi primari senza ispezionare analiticamente i campi dipendenti.

### Soluzione Implementata
È stata introdotta una logica di attraversamento esaustivo (tree traversal) tramite le procedure `count_refs_in_st` e `count_refs_in_func`. Tali funzioni esplorano ricorsivamente l'intera IR post-risoluzione, ispezionando nel dettaglio `super_types`, `fields`, `methods`, `nested_types`, `parameters`, `return_type`, `calls` e `instantiates`. Questa metrica è ora accurata e utilizzabile per validare il tasso di successo del benchmark.

---

## 🟢 Problema 4: Raggruppamento Nodi "External" in Cytoscape

### Sintomo
Durante la risoluzione di dipendenze verso classi o pacchetti esterni al progetto (es. librerie standard), il plug-in di esportazione Cytoscape generava sistematicamente un nodo aggregatore denominato `"External"`, nascondendo nell'interfaccia grafica l'identità formale del bersaglio (es. `"String"`, `"ArrayList"`).

### Soluzione Implementata (Design Choice)
La scelta architetturale è stata riconfermata. Raggruppare le classi di libreria non analizzate sotto un singolo hub (`External`) è una misura necessaria per prevenire il *visual cluttering* sulla UI interattiva. L'identità logica esatta della dipendenza esterna è comunque preservata integralmente nell'artefatto principale (il JSON raw), permettendo analisi programmatiche e data-mining approfonditi.
