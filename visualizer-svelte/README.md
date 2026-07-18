# Language Agnostic Analyzer - Web Visualizer

This is a **SvelteKit-based Web Visualizer** designed specifically to interface with the `language-agnostic-analyzer` Rust backend. It uses **Cytoscape.js** to render rich, interactive dependency graphs of source code architectures in real-time.

## Features

The visualizer provides three primary workflows via the top navigation bar:

1. **Graphs (`/`)**:
   - Visualize pre-calculated JSON output graphs located in `tests/outputs/`.
   - Execute and visualize lightweight language benchmarks directly from the `benchmarks/` directory on the fly.
   - Includes a **Fuzzy Finder** (Cmd/Ctrl + K or Button) to quickly search and switch between available graphs.
   - Features a collapsible sidebar, node/edge search, and an interactive legend.

2. **Reports (`/reports`)**:
   - Run full benchmark suites (`tests/benchmark-rust` and `tests/benchmark-java`) via `cargo run --release --bin benchmark_runner`.
   - Automatically reads the generated `report.md` files and renders the Markdown tables natively into a beautiful, compact UI to easily spot adherence to expectations.

3. **Custom Input (`/custom`)**:
   - Interactively paste raw source code in the browser and instantly render the dependency graph.
   - Point the visualizer to an absolute path on your local filesystem and trigger a real-time `cargo run` analysis for immediate visualization.

## Technologies Used

- **SvelteKit** (Svelte 5 Runes)
- **Vite**
- **Cytoscape.js** (with `fcose` layout algorithm)
- **Node.js** (Backend API endpoints that spawn native `cargo` child processes)
- **Marked** (Markdown rendering for Reports)

## Getting Started

Make sure you have Node.js and Rust (`cargo`) installed on your system.

1. Install JavaScript dependencies:
   ```bash
   npm install
   ```

2. Start the Vite development server:
   ```bash
   npm run dev
   ```

3. Open your browser at `http://localhost:5173`. 
   
*(The Node.js server automatically handles communication with the Rust CLI in the parent folder, executing analysis when requested).*
