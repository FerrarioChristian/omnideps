---
title: CLI Commands
description: Complete command-line reference for all Omnideps subcommands.
order: 0
---

# Command Line Interface

Omnideps provides subcommands for extraction, visualization, intermediate representation inspection, benchmarking, and configuration.

---

## Subcommands Overview

<div class="not-prose my-6 grid gap-4 sm:grid-cols-2">
  <a href="/docs/cli/analyze" class="block rounded-lg border bg-card p-5 transition-colors hover:border-foreground/30">
    <div class="font-mono text-sm font-semibold text-foreground">omnideps analyze</div>
    <p class="text-muted-foreground text-sm mt-2">Scan code repositories and extract architectural dependency graphs in Cytoscape, raw JSON, or CSV summary metrics.</p>
  </a>
  <a href="/docs/cli/dump-ir" class="block rounded-lg border bg-card p-5 transition-colors hover:border-foreground/30">
    <div class="font-mono text-sm font-semibold text-foreground">omnideps dump-ir</div>
    <p class="text-muted-foreground text-sm mt-2">Serialize unpruned Intermediate Representation (IR) structures for deep scope and AST debugging.</p>
  </a>
  <a href="/docs/cli/serve" class="block rounded-lg border bg-card p-5 transition-colors hover:border-foreground/30">
    <div class="font-mono text-sm font-semibold text-foreground">omnideps serve</div>
    <p class="text-muted-foreground text-sm mt-2">Launch the embedded Axum HTTP server and explore graphs interactively with the pre-compiled SvelteKit UI.</p>
  </a>
  <a href="/docs/cli/benchmark" class="block rounded-lg border bg-card p-5 transition-colors hover:border-foreground/30">
    <div class="font-mono text-sm font-semibold text-foreground">omnideps benchmark</div>
    <p class="text-muted-foreground text-sm mt-2">Execute ground-truth accuracy tests, calculating precision, recall, and false positive metrics.</p>
  </a>
  <a href="/docs/cli/config" class="block rounded-lg border bg-card p-5 transition-colors hover:border-foreground/30">
    <div class="font-mono text-sm font-semibold text-foreground">omnideps config init</div>
    <p class="text-muted-foreground text-sm mt-2">Bootstrap a customizable <code>omnideps.json</code> file to tune Language Product Line heuristics.</p>
  </a>
  <a href="/docs/cli/export" class="block rounded-lg border bg-card p-5 transition-colors hover:border-foreground/30">
    <div class="font-mono text-sm font-semibold text-foreground">omnideps export-cyto</div>
    <p class="text-muted-foreground text-sm mt-2">Convert raw DependencyGraph JSON files into Cytoscape-formatted graph elements.</p>
  </a>
</div>
