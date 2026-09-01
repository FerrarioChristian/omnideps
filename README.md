# Omnideps (formerly Language Agnostic Analyzer)

A language-agnostic architectural dependency analyzer designed to extract source code dependency graphs using Tree-sitter.
Omnideps extracts components and structural or behavioral relationships (for languages like Rust, Java, Python, C, C++) and resolves types to trace actual code dependencies.

## Architecture & Installation

Omnideps is distributed as a **Single Standalone Binary**. The web visualizer (SvelteKit) is pre-compiled and embedded directly inside the Rust executable.

**For End Users:**
You **do not** need to install Node.js, npm, or even Rust to use Omnideps.

1. Go to the GitHub **Releases** page.
2. Download the executable for your Operating System (Windows `.exe`, macOS, or Linux).
3. Open your terminal and run it directly!

---

## Commands

Omnideps provides several subcommands (similar to Git or Cargo).

### 1. Analyze Code

Run the analyzer on a specific directory or file to extract the dependency graph:

```bash
omnideps analyze /path/to/project
```

**Options:**

- `-o, --output <FILE>`: Save the output in standard JSON format (also automatically generates the Cytoscape version).
- `-c, --csv <FILE>`: Generate a CSV summary report.
- `--config <FILE>`: Use a custom JSON configuration file.
- `-d, --debug-refs`: Enable debug output for resolved/unresolved references.

### 2. Web Visualizer

Launch the embedded web interface to interactively explore your graphs:

```bash
omnideps serve
```

This will start an ultra-lightweight local HTTP server (default port: `3000`) and will **automatically open your default web browser** to `http://127.0.0.1:3000`. You can analyze code and visualize graphs directly from the UI.
*(Change the port with `omnideps serve --port 8080`).*

### 3. Benchmarking Suite

The project includes custom benchmarks to accurately measure false positives/negatives on the AST abstraction across supported languages.

**Run a specific Benchmark:**

```bash
omnideps benchmark run tests/benchmarks/benchmark-rust -o /path/to/output_dir
```

*(By default, reports are saved in the benchmark's folder. Use `-o` to override the destination).*

**Run all Benchmarks:**
```bash
omnideps benchmark all -o /path/to/results_dir
```
Generates aggregated results in `results.csv` and subfolders for each benchmark's report inside `/path/to/results_dir`. The `-o` flag is optional (defaults to `tests/benchmarks`).

### 4. Configuration

If you want to customize the architectural analyzer's rules and strategies, you can generate a default configuration file:

```bash
omnideps config init
```

This creates an `omnideps.json` file in your current directory. Edit it as needed, then pass it to the analyzer using the `--config` flag.

### 5. Cytoscape Export

To convert a previously generated standard JSON file into a Cytoscape-compatible format:

```bash
omnideps export-cyto input.json output_cyto.json
```

---

## Development (Building from Source)

The following instructions are **only for developers** who want to modify the source code. To compile the project yourself, you will need both **Node.js** and **Rust** installed.

1. **Build the SvelteKit frontend** (Generates static files in `visualizer-svelte/build`):

   ```bash
   cd visualizer-svelte
   npm install
   npm run build
   cd ..
   ```

2. **Compile the Rust executable** (The `rust-embed` macro will package the Svelte build inside the binary):

   ```bash
   cargo build --release
   ```

The final executable will be located in `target/release/omnideps`.

## CI/CD Pipeline

The project includes a GitHub Actions pipeline (`.github/workflows/release.yml`) that automatically builds optimized executables for **Windows, macOS (Intel & Apple Silicon), and Linux** whenever a new tag (e.g., `v1.0.0`) is pushed, using aggressive caching for both Rust and npm.
To publish a new release, simply create a new tag and push it to the repository. The pipeline will handle the rest, including uploading the binaries to the GitHub Releases page.

```bash
git tag v0.x.y
git push origin v0.x.y
```
