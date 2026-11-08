---
title: omnideps config
description: Generate and tune the omnideps.json language configuration file.
order: 5
---

# `omnideps config`

The `config` command generates and manages the configuration file used to tune language strategies and architectural extraction heuristics.

```bash
omnideps config init [OUTPUT]
```

---

## `omnideps config init`

Generates a default `omnideps.json` file in the current directory (or at the path specified by `[OUTPUT]`).

```bash
# Generate default omnideps.json
omnideps config init

# Or specify custom destination
omnideps config init ./config/my-rules.json
```

---

## The Language Product Line Schema

Omnideps avoids hardcoded `if language == "x"` branches by modeling language behaviors as orthogonal parameters inside `AnalyzerConfig`:

```json
{
  "default_config": {
    "modules": {
      "file_based": false,
      "directory_based": false,
      "package_decl_based": false,
      "namespace_based": false,
      "inline_mod_based": false
    },
    "transitive_imports": false,
    "self_keyword": "this",
    "self_type_keyword": null,
    "implicit_first_param_as_self": false,
    "extract_dynamic_fields": false,
    "assignment_type_aliases": false,
    "callable_types": false
  },
  "languages": {
    "java": {
      "modules": {
        "file_based": false,
        "directory_based": false,
        "package_decl_based": true,
        "namespace_based": false,
        "inline_mod_based": false
      },
      "transitive_imports": false,
      "self_keyword": "this",
      "self_type_keyword": null,
      "implicit_first_param_as_self": false,
      "extract_dynamic_fields": false,
      "assignment_type_aliases": false,
      "callable_types": false
    },
    "rust": {
      "modules": {
        "file_based": true,
        "directory_based": true,
        "package_decl_based": false,
        "namespace_based": false,
        "inline_mod_based": true
      },
      "transitive_imports": true,
      "self_keyword": "self",
      "self_type_keyword": "Self",
      "implicit_first_param_as_self": false,
      "extract_dynamic_fields": false,
      "assignment_type_aliases": false,
      "callable_types": false
    },
    "python": {
      "modules": {
        "file_based": true,
        "directory_based": true,
        "package_decl_based": false,
        "namespace_based": false,
        "inline_mod_based": false
      },
      "transitive_imports": true,
      "self_keyword": null,
      "self_type_keyword": null,
      "implicit_first_param_as_self": true,
      "extract_dynamic_fields": true,
      "assignment_type_aliases": true,
      "callable_types": true
    }
  }
}
```

---

## Parameter Descriptions

### Module Strategies (`modules`)
* **`package_decl_based`**: Modules are dictated by syntactic statements (e.g., Java `package com.foo;`).
* **`file_based`**: Every source file introduces an isolated module scope (e.g., Python, Rust).
* **`directory_based`**: Directory folders define hierarchical module parentage.
* **`inline_mod_based`**: Allows nested inline module blocks within a single file (e.g., Rust `mod foo { ... }`).
* **`namespace_based`**: Modules are organized via explicit `namespace` blocks (e.g., C++).

### Resolution Strategies
* **`transitive_imports`**: When enabled, symbols imported in `__init__.py` or module files are re-exported and accessible through the importing module.
* **`self_keyword`**: Token representing the current instance receiver (`this` in Java/C++, `self` in Rust).
* **`self_type_keyword`**: Token representing the implementing structured type (`Self` in Rust).
* **`implicit_first_param_as_self`**: Treats the first function parameter as the receiver object (Python `def foo(self, x)`).
* **`extract_dynamic_fields`**: Detects dynamically assigned instance fields on receiver types.
* **`assignment_type_aliases`**: Evaluates variable assignments as type aliases (e.g., Python `MyType = int`).
* **`callable_types`**: Recognizes objects that implement callable signatures.
