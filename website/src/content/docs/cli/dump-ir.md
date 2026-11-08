---
title: omnideps dump-ir
description: Serialize the uncut, complete Intermediate Representation (IR) for debugging.
order: 2
---

# `omnideps dump-ir`

The `dump-ir` command serializes the complete, unpruned Intermediate Representation (IR) of the codebase directly to JSON.

```bash
omnideps dump-ir <PATH> [OPTIONS]
```

---

## Purpose

During standard analysis (`omnideps analyze`), the extractor prunes internal syntax details and synthesizes high-level graph edges. 

The `dump-ir` command preserves the full hierarchical structure of modules, AST scope trees, nested declarations, raw type signatures, and query definitions before graph synthesis. This is essential for:
* Debugging custom language configurations and scope resolvers.
* Validating that Tree-sitter AST nodes map correctly to formal IR entities.
* Auditing exact qualified names and visibility rules.

---

## Arguments & Options

| Flag | Long Flag | Default | Description |
| :--- | :--- | :--- | :--- |
| `<PATH>` | | *required* | File or directory to analyze. |
| `-o` | `--output <FILE>` | `ir_dump.json` | Destination path for the pretty-printed JSON file. |
| | `--config <FILE>` | *None* | Path to a custom `omnideps.json` configuration file. |
| `-h` | `--help` | | Print help information. |

---

## Example

```bash
omnideps dump-ir ./src -o full_ir.json
```

Outputs a structured JSON hierarchy containing all modules, classes, structs, traits, functions, and query references:

```json
[
  {
    "name": ["com", "omnideps", "core"],
    "submodules": [],
    "structured_types": [
      {
        "name": ["com", "omnideps", "core", "Engine"],
        "kind": "Class",
        "fields": [],
        "methods": []
      }
    ]
  }
]
```
