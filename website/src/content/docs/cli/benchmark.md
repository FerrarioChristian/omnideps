---
title: omnideps benchmark
description: Measure extraction and resolution precision, recall, and false positives.
order: 4
---

# `omnideps benchmark`

Omnideps includes a built-in benchmarking harness to quantitatively measure analysis accuracy, precision, and recall against curated test suites.

```bash
omnideps benchmark <SUBCOMMAND> [OPTIONS]
```

---

## Subcommands

### 1. `omnideps benchmark run`

Runs a benchmark against a single test directory containing a `test.yml` ground-truth specification:

```bash
omnideps benchmark run <TESTDIR> [OPTIONS]
```

#### Arguments & Options
* `<TESTDIR>` *(required)*: Path to the benchmark directory (e.g., `tests/benchmarks/benchmark-java`).
* `-o, --output <DIR>`: Directory to save generated reports (defaults to the benchmark directory).
* `-c, --config <FILE>`: Path to a custom `omnideps.json` configuration file.

#### Generated Reports
Generates two artifact files:
* `report.md`: Human-readable Markdown summary detailing found, missed, and extra nodes/edges.
* `report.json`: Machine-readable adherence metrics including precision, recall, and F1-score.

```text
Analyzing test directory: tests/benchmarks/benchmark-java
Markdown report saved to tests/benchmarks/benchmark-java/report.md
JSON report saved to tests/benchmarks/benchmark-java/report.json
Statistiche nodi: Trovati 42, Non Trovati 0, Totali 42
Statistiche archi: Trovati 128, Non Trovati 1, Totali 129
```

---

### 2. `omnideps benchmark all`

Executes all benchmarks across all language suites in a single pass and aggregates the results:

```bash
omnideps benchmark all [OPTIONS]
```

#### Options
* `-o, --output <DIR>`: Destination directory for all reports and `results.csv` (defaults to `tests/benchmarks`).
* `-c, --config <FILE>`: Custom configuration file path.

#### Output
Aggregates metrics across suites into a unified `results.csv`:
```text
Benchmark,TotalNodes,FoundNodes,PrecisionNodes,TotalEdges,FoundEdges,PrecisionEdges
benchmark-java,42,42,1.0,129,128,0.992
benchmark-rust,38,36,0.947,94,90,0.957
```
