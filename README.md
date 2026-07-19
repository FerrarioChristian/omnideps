# Language Agnostic Analyzer

Analizzatore architetturale multilingua progettato per estrarre il grafo delle dipendenze del codice sorgente (es. Java, Rust) tramite Tree-sitter. Il sistema estrae componenti e relazioni strutturali o comportamentali in modo language-agnostic e risolve i tipi per tracciare le reali dipendenze.

## Comandi Utili

### 1. Testare l'Estrazione e Preparare il Visualizer
Per validare l'estrazione e generare automaticamente i file JSON di input per il **Visualizer**, eseguire la suite di test standard:
```bash
cargo test
```
*I test esportano l'analisi sotto la cartella `tests/outputs/` (inclusi i file `cyto_*.json`), i quali verranno poi consumati dal visualizer locale.*

### 2. Eseguire l'Analizzatore su una Cartella Qualsiasi
Per lanciare l'analizzatore su una cartella specifica o file del proprio progetto, estraendone i log e il JSON risultante, eseguire:
```bash
cargo run --bin language-agnostic-analyzer -- [PERCORSO]
```
Opzioni utili:
- `--json [FILE_OUT.json]`: Salva l'output in formato JSON standard (diverso dal file Cytoscape).
- `--csv [FILE_OUT.csv]`: Genera un report CSV riassuntivo.
- `--config [FILE_CONFIG.json]`: Permette di utilizzare file di configurazione custom per l'analizzatore.
- `--debug-refs`: Abilita il debug delle reference risolte/non risolte.

### 3. Suite di Benchmark (Java / Rust / C / C++ / Python)
Il progetto include benchmark custom per misurare accuratamente i falsi positivi/negativi sull'astrazione AST di tutti i linguaggi supportati. I risultati vengono salvati nelle rispettive sottocartelle di benchmark all'interno di `tests/benchmarks/` in formato `report.md` e `report.json`.

**Eseguire un Benchmark (es. Rust):**
```bash
cargo run --release --bin benchmark_runner tests/benchmarks/benchmark-rust
```

**Eseguire un Benchmark (es. Java):**
```bash
cargo run --release --bin benchmark_runner tests/benchmarks/benchmark-java
```

## Visualizer
Il visualizer web (ora basato su SvelteKit e Node.js) permette di esplorare in tempo reale i grafi e visualizzare in modo interattivo i report di benchmark e la documentazione del progetto.

Per avviarlo, spostati nella cartella dedicata ed esegui i seguenti comandi (è richiesto Node.js e NPM):
```bash
cd visualizer-svelte
npm install
npm run dev
```
Dopodiché apri `http://localhost:5173` nel tuo browser.
