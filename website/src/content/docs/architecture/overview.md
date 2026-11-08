---
title: Pipeline Architecture
description: The three-phase formal model for polyglot extraction, resolution, and graph construction.
order: 1
---

# Pipeline Architecture

Omnideps processes software architectures through a formal three-phase pipeline based on language-independent Intermediate Representation (IR).

---

## The 3-Phase Pipeline

```
  Source Files (Java, Rust, Python, C++, C)
                     │
                     ▼  [ Phase 1: Parallel AST Extraction (α) ]
            Concrete Syntax Trees (Tree-sitter)
                     │
                     ▼
             Scope Hierarchy & Entity Tree
                     │
                     ▼  [ Phase 2: Name & Type Resolution (β) ]
             Resolved Intermediate Representation (D_res)
                     │
                     ▼  [ Phase 3: Graph Synthesis (γ) ]
            Directed Multi-Graph G = ⟨V, E⟩
                     │
      ┌──────────────┴──────────────┐
      ▼                             ▼
Cytoscape.js Format           CSV Metrics & Raw IR
```

---

## Phase 1: Parallel AST Extraction (α)

1. **Language Discovery**: Scans target folders and resolves file extensions to [`SupportedLanguage`](https://github.com/FerrarioChristian/omnideps).
2. **Rayon Concurrency**: Distributes parsing tasks across CPU worker pools. Each file is parsed into a Concrete Syntax Tree (CST) using its official Tree-sitter grammar.
3. **Component Extraction**:
   * **Modules**: Package hierarchies, namespaces, directories, and files.
   * **Structured Types**: Classes, structs, interfaces, traits, and enums.
   * **Callables**: Methods, free functions, constructors, and lambdas.
   * **Variables & Fields**: Member variables, function parameters, and return types.

---

## Phase 2: Name & Scope Resolution (β)

Resolves syntactical query references into canonical fully-qualified names:

1. **Local Scope Lookup**: Resolves identifiers within blocks and function bodies.
2. **Receiver Evaluation**: Tracks `this` / `self` and instantiated receiver types.
3. **Cross-File Symbol Linking**: Evaluates package namespaces, includes, and `import` hierarchies.
4. **Inheritance & Generics**: Walks superclass and interface implementations, propagating type bindings for parameterized types (e.g. `List<T>`, `Result<T, E>`).

---

## Phase 3: Graph Synthesis (γ: D_res → G)

Synthesizes a directed dependency multi-graph `G = ⟨V, E⟩` containing canonical nodes and typed edges.

### The 16 Formal Edge Kinds (`DependencyEdgeKind`)

Omnideps categorizes dependencies into 16 formal relationship types:

| Edge Kind | Category | Description |
| :--- | :--- | :--- |
| **`IsA`** | Structural | Class inheritance (`extends`). |
| **`Implements`** | Structural | Interface or trait implementation (`implements`, `impl Trait for Struct`). |
| **`UsesFieldType`** | Structural | Field declaration typed as another component. |
| **`UsesLocalType`** | Structural | Local variable typed as another component. |
| **`UsesParamType`** | Structural | Method/function parameter typed as another component. |
| **`UsesReturnType`**| Structural | Function or method return type. |
| **`NestedIn`** | Containment | Inner classes, structs, or nested types. |
| **`ModuleContainment`** | Containment | Structural placement of components within modules. |
| **`Calls`** | Behavioral | Direct function or method invocation. |
| **`Instantiates`** | Behavioral | Object or struct creation (`new Foo()`, `Foo { ... }`). |
| **`AccessesField`**| Behavioral | Direct member field read or write. |
| **`Imports`** | Dependency | Explicit module or symbol import. |
| **`CastsTo`** | Behavioral | Explicit type conversion or cast. |
| **`Aliases`** | Structural | Type alias definition (`type Foo = Bar`). |
| **`AnnotatedWith`**| Metadata | Applied decorators or annotations (`@Override`, `#[derive(...)]`). |
| **`UsesType`** | General | General type references (e.g., generic type parameter bounds). |
