---
title: Quickstart
description: Analyze a codebase and launch the visualizer in under 5 minutes.
order: 2
---

# 5-Minute Quickstart

Follow this guide to run your first architectural analysis and view the resulting dependency graph.

---

## Step 1: Run an Analysis

Point Omnideps to any repository or source directory containing code in Rust, Java, Python, C, or C++:

```bash
omnideps analyze ./path-to-my-project --output dependencies.json
```

Omnideps will:
1. Discover all source files matching supported extensions.
2. Parse the AST using multi-threaded Tree-sitter grammars.
3. Extract entities (classes, structs, functions, modules) and statements.
4. Execute type and name resolution to link callers with callees and types.
5. Save the Cytoscape-formatted graph directly to `dependencies.json` (or pass `--raw` to also export `raw_dependencies.json`).

---

## Step 2: Launch the Web Visualizer

To inspect the dependencies interactively in your browser:

```bash
omnideps serve
```

This starts a local Axum server at `http://127.0.0.1:3000` and automatically opens your default browser.

<div class="rounded-lg border bg-muted/30 p-4 text-sm my-4">
  <strong>Tip:</strong> If port 3000 is occupied, supply a custom port with <code>omnideps serve --port 8080</code>.
</div>

Inside the UI you can:
* Load your freshly generated `dependencies.json` or choose a folder to analyze on-the-fly.
* Filter components by type (classes, functions, files, modules).
* Zoom, pan, and run force-directed layout simulations (fCoSE / CoSE).
* Inspect in-degree and out-degree relationships for any selected node.

---

## Step 3: Export CSV Metrics

If you are performing architectural research or code smell detection, export a tabular CSV summary:

```bash
omnideps analyze ./path-to-my-project -o deps.json -c report.csv
```

The CSV report contains summary metrics including total node counts, edge distributions, and reference resolution statistics.
