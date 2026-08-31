# Omnideps (ex Language Agnostic Analyzer)

Analizzatore architetturale multilingua progettato per estrarre il grafo delle
dipendenze del codice sorgente (es. Java, Rust) tramite Tree-sitter. Il sistema
estrae componenti e relazioni strutturali o comportamentali in modo
language-agnostic e risolve i tipi per tracciare le reali dipendenze.

## Architettura Unificata (Single Binary)

Omnideps è distribuito come un singolo file eseguibile che contiene sia il motore di analisi (scritto in Rust) sia l'interfaccia web interattiva (scritta in SvelteKit e integrata staticamente nell'eseguibile). Non è necessario installare Node.js o altre dipendenze per visualizzare i grafi!

## Comandi Utili

Il comando principale è `omnideps`, che espone diverse funzionalità tramite sottocomandi (in stile Git o Cargo).

### 1. Analisi di una Cartella o Singolo File

Per lanciare l'analizzatore su una cartella specifica o file del proprio progetto, estraendone i log e il JSON risultante, eseguire:

```bash
omnideps analyze [PERCORSO]
```

Opzioni utili:
- `--output [FILE_OUT.json]`: Salva l'output in formato JSON standard (e genera in automatico la versione Cytoscape).
- `--csv [FILE_OUT.csv]`: Genera un report CSV riassuntivo.
- `--config [FILE_CONFIG.json]`: Permette di utilizzare file di configurazione custom per l'analizzatore.
- `-d, --debug-refs`: Abilita il debug delle reference risolte/non risolte.

### 2. Avviare il Visualizer Web Integrato

Per avviare l'interfaccia web e interagire visivamente con i grafi:

```bash
omnideps serve
```

Questo avvierà un server HTTP locale ultraleggero (sulla porta 3000 di default). Aprendo `http://localhost:3000` nel browser, potrai accedere al visualizer, senza dover installare Node.js.
(Puoi cambiare la porta con `omnideps serve --port 8080`).

### 3. Suite di Benchmark (Java / Rust / C / C++ / Python)

Il progetto include benchmark custom per misurare accuratamente i falsi positivi/negativi sull'astrazione AST di tutti i linguaggi supportati.

**Eseguire un Benchmark specifico (es. Rust):**
```bash
omnideps benchmark run tests/benchmarks/benchmark-rust
```
Di default, i file `report.md` e `report.json` verranno salvati all'interno della cartella specificata. È possibile specificare un'altra cartella di destinazione usando il flag `-o` (o `--output`):
```bash
omnideps benchmark run tests/benchmarks/benchmark-rust -o cartella_di_destinazione
```

**Eseguire tutti i Benchmark:**
```bash
omnideps benchmark all
```
I risultati verranno salvati nelle rispettive sottocartelle in formato `report.md` e `report.json`, e un CSV aggregato in `tests/benchmarks/results.csv`.

### 4. Esportazione Cytoscape

Per convertire un file JSON generato in precedenza in formato compatibile con Cytoscape:

```bash
omnideps export-cyto input.json output_cyto.json
```

## Installazione (Sviluppatori)

Se hai Rust installato, puoi compilare Omnideps dal codice sorgente. Il processo di build si occuperà automaticamente di compilare anche il frontend SvelteKit (è richiesto Node.js solo in fase di compilazione).

```bash
# 1. Compila il frontend SvelteKit (genera i file statici in visualizer-svelte/build)
cd visualizer-svelte
npm install
npm run build
cd ..

# 2. Compila l'eseguibile Rust (che includerà i file statici)
cargo build --release
```

L'eseguibile finale si troverà in `target/release/omnideps`.

## Integrazione Continua (CI/CD)

Il progetto include una pipeline GitHub Actions (`.github/workflows/release.yml`) che compila automaticamente eseguibili ottimizzati per **Windows, macOS (Intel e Apple Silicon) e Linux** ad ogni nuova release (es. tag `v1.0.0`), gestendo il caching aggressivo per Rust e npm.

### 5. Generazione File di Configurazione

Se vuoi personalizzare le regole e le strategie dell'analizzatore architetturale senza dover scrivere il file JSON da zero, puoi usare il comando:

```bash
omnideps config init
```
Questo genererà un file `omnideps.json` nella cartella corrente con tutti i valori predefiniti, pronto per essere modificato e passato al flag `--config` negli altri comandi. Puoi anche specificare un percorso diverso con `omnideps config init path/to/config.json`.
