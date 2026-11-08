---
title: omnideps serve
description: Launch the embedded web server and explore dependency graphs interactively.
order: 3
---

# `omnideps serve`

The `serve` command starts the local web server and launches the interactive visualizer embedded directly within the Omnideps binary.

```bash
omnideps serve [OPTIONS]
```

---

## Behavior

* Initializes an embedded high-performance **Axum** HTTP server powered by **Tokio**.
* Extracts the pre-compiled **SvelteKit + Cytoscape** Single Page Application (SPA) directly from binary memory using `rust-embed`.
* Automatically invokes your default operating system browser to open `http://127.0.0.1:3000`.

---

## Options

| Flag | Long Flag | Default | Description |
| :--- | :--- | :--- | :--- |
| `-p` | `--port <PORT>` | `3000` | Custom TCP port to bind the HTTP server. |
| `-h` | `--help` | | Print help information. |

---

## Examples

### Default Launch
```bash
omnideps serve
```
Binds to `127.0.0.1:3000` and opens your default browser.

### Custom Port
```bash
omnideps serve --port 8080
```

---

## Embedded REST API

While running, `omnideps serve` provides internal HTTP endpoints used by the visualizer and external tools:

* `POST /api/analyze`: Analyzes an on-disk project directory or ad-hoc code snippet submitted as JSON.
* `GET /api/benchmark/list`: Returns available benchmark suites embedded in the binary.
* `POST /api/benchmark/run`: Triggers a benchmark run directly from the browser UI and returns adherence metrics.
