# Benchmark Runner Guide

Il **Benchmark Runner** è lo strumento primario integrato in `language-agnostic-analyzer` per misurare oggettivamente le regressioni e il tasso di successo dell'estrazione architetturale. L'infrastruttura permette di confrontare il `DependencyGraph` generato dall'analizzatore con le aspettative architetturali (manifesti) scritte manualmente per vari linguaggi di programmazione.

---

## 1. Esecuzione dei Benchmark

Il sistema di benchmark offre due eseguibili principali, scritti in Rust all'interno di `src/bin/`:

### Esecuzione Singola (`benchmark_runner`)
Permette di eseguire l'analisi su una specifica cartella di test, stampando i risultati a terminale e generando report locali.
```bash
cargo run --bin benchmark_runner -- <percorso_directory_test>
```
**Cosa fa:**
- Analizza i sorgenti presenti nella directory.
- Confronta il grafo generato con il `test.yml` presente in quella cartella.
- Genera (o sovrascrive) i file locali `report.md` e `report.json` all'interno della cartella di test.

### Esecuzione Globale (`benchmark_all`)
Permette di avviare l'intera suite di benchmark su tutte le cartelle presenti sotto `tests/benchmarks/`.
```bash
cargo run --bin benchmark_all
```
**Cosa fa:**
- Itera su ogni sottocartella che possiede un `test.yml`.
- Aggrega tutti i risultati per formare statistiche globali (hit-rate, nodi trovati, archi trovati).
- Genera il file riassuntivo **`results.csv`** nella root del progetto, contenente una riga per ogni benchmark con le rispettive metriche di fallimento/successo, utilissimo per il tracciamento storico delle regressioni.

---

## 2. Il Manifesto YAML (`test.yml`)

Ogni suite di test richiede un file `test.yml` che funga da "Oracolo" architetturale. Il manifesto utilizza il crate `serde_yaml` per mappare tre sezioni fondamentali:

1. **`files`**: Lista di percorsi ai file sorgente che costituiscono il test.
2. **`nodes`**: Elenco delle entità architetturali (Classi, Funzioni, Moduli, Campi) che ci si aspetta di trovare nel codice.
   - Ogni nodo richiede un `name` (il Fully Qualified Name, es. `models.User.__init__`) e un `kind`.
3. **`edges`**: Elenco delle dipendenze architetturali che uniscono i nodi.
   - `testid`: Un identificativo univoco (es. `PY-CALL-1`).
   - `source` e `sink`: I Fully Qualified Name dei nodi coinvolti.
   - `kind`: Il tipo della dipendenza testata (es. `calls`, `inherits`, `accesses_field`).

---

## 3. Logica di Validazione (Verifica dei Risultati)

La verifica dell'aderenza del grafo generato rispetto al manifesto YAML viene effettuata tramite la funzione `verify_graph_adherence(graph, manifest)`. L'algoritmo esegue un confronto strutturale operando nei seguenti passaggi:

### 3.1. Appiattimento e Indicizzazione
Per superare la potenziale discrepanza di nomenclatura (naming mismatch) e per ottimizzare i lookup, il runner converte le entità del `DependencyGraph` reale in strutture di indicizzazione piatte (flat-maps):
- **Nodi (`nodes_map`)**: Viene creata una mappa `HashMap<String, &Component>`. Indipendentemente dal fatto che l'entità sia un `Module`, un `StructuredType` o un `Function`, la funzione `flatten_name` estrae il percorso vettoriale interno (es. `["models", "User", "__init__"]`) e lo unisce in una stringa piatta separata da punti (`"models.User.__init__"`), associandola al componente originale.
- **Archi (`edges_map`)**: Viene creata una mappa di adiacenza `HashMap<String, HashSet<String>>`. Per ogni arco logico estratto, il source e il sink vengono appiattiti in formato stringa. Il sink viene quindi aggiunto all'`HashSet` corrispondente alla chiave del source, tralasciando momentaneamente la tipologia (es. `Calls` o `Inherits`).

### 3.2. Controllo dei Nodi
Il validatore itera su tutti i nodi definiti in `manifest.nodes`:
- Viene effettuato un check di esistenza in `nodes_map` (tramite `contains_key(&node.name)`).
- **Validazione Tipo Nodo**: Allo stato attuale, per garantire compatibilità architetturale tra l'AST grezzo originale e i nuovi componenti astratti di Antigravity, il controllo stringente sul tipo del nodo (es. pretendere che `class` nel test coincida col componente `StructuredType(Class)`) è ignorato a favore di un "match strutturale" basato esclusivamente sull'FQN (Fully Qualified Name). 

### 3.3. Controllo degli Archi (Dependency Edges)
L'algoritmo verifica iterativamente la lista `edges` del manifesto:
1. Controlla prima che entrambi i nodi coinvolti (source e sink) esistano effettivamente in `nodes_map`. Se manca anche solo uno dei due, l'arco viene automaticamente considerato fallito per "Missing Endpoint".
2. Se entrambi esistono, l'algoritmo accede all'`HashSet` dei sink disponibile per il source all'interno di `edges_map`.
3. Valuta la presenza dell'arco tramite una banale operazione su insiemi (`sinks.contains(&edge.sink)`).
4. **Validazione Tipo Arco (Edge Kind)**: Per mantenere compatibilità retroattiva e comparativa con i vecchi analizzatori, la validazione *ignora* il tipo esatto dell'arco richiesto (es. `calls`), verificando solamente l'esistenza di una dipendenza generica tra i due nodi. Oggi, tuttavia, il `language-agnostic-analyzer` estrae nativamente i tipi esatti tramite l'enum `DependencyEdgeKind`, rendendo l'infrastruttura pronta per controlli stretti sul `kind` tramite semplici estensioni della logica di lookup. Se un arco fallisce, il runner stampa nei log a terminale l'elenco dei sink disponibili per quel source, facilitando enormemente il debugging delle pipeline di estrazione.

---

## 4. Generazione del Report

A seguito dell'esecuzione singola, il runner compila e serializza due output:

1. **`report.json`**: L'oggetto dati grezzo della validazione, utile per integrazioni automatizzate o script di test CI/CD.
2. **`report.md`**: Un report in formato Markdown per la lettura umana. Questo documento modella dinamicamente tabelle riepilogative identiche a quelle utilizzate storicamente:
   - Una tabella riassuntiva dei nodi trovati / mancanti.
   - Una tabella dettagliata per gli archi, con gli ID del test e lo status (✅ Passato / ❌ Fallito).

Questo meccanismo offre un feedback visuale immediato durante la fase di sviluppo, permettendo di identificare con precisione la risoluzione implementativa fallita (es. `Resolution(Missing)` vs `Extraction(Missing)`).
