---
title: Supported Languages
description: Language support matrix, semantics, and Language Product Line configurations.
order: 2
---

# Supported Languages

Omnideps implements a **Language Product Line** model. Rather than writing divergent analyzers for each language, a unified processing engine is parameterized by orthogonal semantic feature tuples (`LanguageConfig`).

---

## Language Support Status

| Language | Status | Parser | Key Characteristics |
| :--- | :--- | :--- | :--- |
| **Java** | <span class="text-emerald-500 font-semibold">Stable (Production)</span> | `tree-sitter-java` | Strict package declarations, full class/interface inheritance, wildcard imports, generics, type resolution. |
| **Rust** | <span class="text-amber-500 font-semibold">Experimental</span> | `tree-sitter-rust` | File & directory modules, inline `mod`, `impl` blocks, `trait` dispatch, generic bounds, deref coercion. |
| **Python** | <span class="text-amber-500 font-semibold">Experimental</span> | `tree-sitter-python` | File modules, `__init__.py` packages, transitive imports, dynamic field assignment, first-param `self`. |
| **C++** | <span class="text-amber-500 font-semibold">Experimental</span> | `tree-sitter-cpp` | Explicit `namespace` blocks, `#include` tracking, class methods, scope resolution (`::`). |
| **C** | <span class="text-amber-500 font-semibold">Experimental</span> | `tree-sitter-c` | Header file dependency graphs, struct definitions, function signatures, typedefs. |

---

## Semantic Configuration Comparison

Omnideps tunes analysis per language using the following formal parameters:

### 1. Module Organization (`ModuleConfig`)

| Feature | Java | Rust | Python | C++ | C |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`package_decl_based`** | **True** | False | False | False | False |
| **`file_based`** | False | **True** | **True** | False | False |
| **`directory_based`** | False | **True** | **True** | False | False |
| **`namespace_based`** | False | False | False | **True** | False |
| **`inline_mod_based`** | False | **True** | False | False | False |

### 2. Receiver & Type Resolution

| Feature | Java | Rust | Python | C++ | C |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`self_keyword`** | `this` | `self` | *None* | `this` | *None* |
| **`self_type_keyword`** | *None* | `Self` | *None* | *None* | *None* |
| **`implicit_first_param_as_self`** | False | False | **True** | False | False |
| **`transitive_imports`** | False | **True** | **True** | False | False |
| **`extract_dynamic_fields`** | False | False | **True** | False | False |
| **`assignment_type_aliases`** | False | False | **True** | False | False |
| **`callable_types`** | False | False | **True** | **True** | False |

---

## Language Profiles

### Java (Stable)
Java represents the primary reference implementation in Omnideps:
* **Package Authority**: The `package com.foo.bar;` statement is the single source of truth for the fully qualified namespace.
* **Inheritance Mapping**: Full resolution of `extends` and `implements` hierarchies.
* **No Implicit Self**: Method receivers are implicit in bytecode and not listed as user parameters.

### Rust (Experimental)
* **Separation of Structure and Implementation**: Decouples `struct`/`enum` declarations from multiple `impl` blocks that can span different files.
* **Deref Coercion**: Models `Target` projection for smart pointers (`Box`, `Arc`, `Rc`).

### Python (Experimental)
* **Transitive Re-exports**: Correctly resolves symbols re-exported inside `__init__.py` files (`from .submodule import Service`).
* **Dynamic Attributes**: Captures fields assigned dynamically onto `self` inside `__init__`.

### C & C++ (Experimental)
* **Header & Scope Linking**: Traces `#include` directives and resolves namespaces using `::` operators.
