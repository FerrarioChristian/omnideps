---
title: Introduction
description: Omnideps is a fast, language-agnostic architectural dependency analyzer and visualizer built in Rust and powered by Tree-sitter.
order: 0
---

# Omnideps Overview

**Omnideps** is a language-agnostic architectural dependency analyzer designed to extract high-fidelity source code dependency graphs using Tree-sitter AST parsing and cross-file type resolution.

Unlike traditional dependency scanners that only inspect package manifests (such as `package.json` or `Cargo.toml`), Omnideps analyzes the **actual code structure**: which classes inherit from which, which functions invoke each other, which structs reference other types, and how components interact across module boundaries.

<div class="not-prose my-6 grid gap-4 sm:grid-cols-2">
  <div class="rounded-xl border bg-card p-5 text-card-foreground shadow-xs">
    <div class="font-semibold text-foreground text-base">📦 Single Standalone Executable</div>
    <p class="text-muted-foreground text-sm mt-2">Zero runtime dependencies. No Node.js, Python, or JVM required on the target machine.</p>
  </div>
  <div class="rounded-xl border bg-card p-5 text-card-foreground shadow-xs">
    <div class="font-semibold text-foreground text-base">🌐 Built-in Web Visualizer</div>
    <p class="text-muted-foreground text-sm mt-2">Run <code>omnideps serve</code> to launch an embedded SvelteKit + Cytoscape visualizer in your browser.</p>
  </div>
  <div class="rounded-xl border bg-card p-5 text-card-foreground shadow-xs">
    <div class="font-semibold text-foreground text-base">🌲 Multi-Language AST Parsing</div>
    <p class="text-muted-foreground text-sm mt-2">Native Tree-sitter grammars for Rust, Java, Python, C, and C++ with parallel processing via Rayon.</p>
  </div>
  <div class="rounded-xl border bg-card p-5 text-card-foreground shadow-xs">
    <div class="font-semibold text-foreground text-base">🎯 Deep Name & Type Resolution</div>
    <p class="text-muted-foreground text-sm mt-2">Cross-file name lookup, inheritance resolution, generic type handling, and member field tracking.</p>
  </div>
</div>

---

## Core Capabilities

Omnideps operates directly on source trees, decomposing software architectures into an intermediate representation (IR) of nodes and edges:

1. **Component Extraction**: Identifies files, modules, packages, classes, structs, traits, interfaces, and functions.
2. **Relationship Tracking**:
   * **Structural**: Inheritance (`extends`, `implements`), member fields, and return types.
   * **Behavioral**: Function and method invocations, variable assignments, and instantiations.
   * **Module-level**: Explicit imports, module inclusions, and path qualifications.
3. **Interactive Graph Exploration**: An ultra-responsive web UI featuring forced-directed Cytoscape graph layouts, component filtering, and dependency search.
4. **Rich Export Formats**: Exports to canonical JSON schemas, Cytoscape JSON, and summary CSV reports for architectural metric analysis.

---

## Next Steps

<div class="not-prose my-4 grid gap-4 sm:grid-cols-2">
  <a href="/docs/getting-started/installation" class="block rounded-lg border bg-muted/40 p-4 transition-colors hover:bg-muted">
    <div class="font-medium text-foreground">Installation →</div>
    <div class="text-xs text-muted-foreground mt-1">Download pre-compiled binaries or build from source.</div>
  </a>
  <a href="/docs/getting-started/quickstart" class="block rounded-lg border bg-muted/40 p-4 transition-colors hover:bg-muted">
    <div class="font-medium text-foreground">5-Minute Quickstart →</div>
    <div class="text-xs text-muted-foreground mt-1">Analyze your first codebase and explore the graph.</div>
  </a>
</div>
