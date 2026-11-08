---
title: omnideps analyze
description: Command-line reference for running architectural dependency analysis.
order: 1
---

# `omnideps analyze`

The `analyze` command is the core engine of Omnideps. It discovers source files across supported languages, parses their concrete syntax trees via Tree-sitter in parallel, performs cross-file reference resolution, and synthesizes architectural dependency graphs.

```bash
omnideps analyze <PATH> [OPTIONS]
```

---

## Arguments

* `<PATH>` *(required)*: Path to the target source file or project directory to analyze.

---

## Options & Flags

| Flag | Long Flag | Default | Description |
| :--- | :--- | :--- | :--- |
| `-o` | `--output <FILE>` | *None* | Output Cytoscape JSON file path for interactive graph rendering (e.g., `graph.json`). |
| | `--raw` | `false` | When used with `-o`, also exports the raw internal `DependencyGraph` schema alongside, saved as `raw_<output>`. |
| `-c` | `--csv <FILE>` | *None* | Export comprehensive quantitative analysis metrics to a CSV file. |
| `-s` | `--summary` | `false` | Calculate and display formatted analysis summary statistics (modules, types, resolved and failed references) in the terminal. |
| `-d` | `--debug-refs` | `false` | Print detailed debug information for all references (both resolved and unresolved). |
| `-f` | `--failed-only` | `false` | Print debug information **only** for unresolved/failed references. |
| | `--config <FILE>` | *None* | Path to a custom `omnideps.json` configuration file defining architectural strategies. |
| `-h` | `--help` | | Print command help information. |

---

## Examples

### 1. Basic Analysis & Graph Export

Extract dependencies and save the Cytoscape graph:

```bash
omnideps analyze ./my-project -o output.json
```

This generates `output.json` conforming to Cytoscape.js elements format, ready to be inspected or loaded into external tools.

### 2. Exporting Both Cytoscape and Raw Graphs

If you need the raw formal intermediate representation graph alongside Cytoscape:

```bash
omnideps analyze ./my-project -o graph.json --raw
```

This produces two files:
* `graph.json`: Cytoscape format.
* `raw_graph.json`: Canonical `DependencyGraph` structure (`nodes` and `edges` with formal `DependencyEdgeKind`).

### 3. Displaying Summary Statistics & CSV Export

```bash
omnideps analyze ./src -s -c summary.csv
```

Outputs a terminal report:
```text
=== ANALYSIS CARTELLA ./src ===
============================================================
ANALYSIS SUMMARY
------------------------------------------------------------
Total Modules:                18
Total Structured Types:       142
Total Free Functions:         64
Resolved References:          1,280
Failed References:            14
Resolution Rate:              98.92%
============================================================
```

### 4. Debugging Unresolved References

To pinpoint missing symbols or dynamic references without noise:

```bash
omnideps analyze ./src --failed-only
```

### 5. Running with Custom Architectural Strategies

```bash
omnideps analyze ./src --config ./omnideps.json -o graph.json
```
