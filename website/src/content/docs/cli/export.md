---
title: omnideps export-cyto
description: Convert internal DependencyGraph JSON files into Cytoscape.js format.
order: 6
---

# `omnideps export-cyto`

The `export-cyto` command converts a previously generated canonical `DependencyGraph` JSON file into Cytoscape.js compatible elements format.

```bash
omnideps export-cyto <INPUT_FILE> <OUTPUT_FILE>
```

---

## When to Use

When running `omnideps analyze ./project -o graph.json --raw`, Omnideps outputs:
1. `graph.json`: Cytoscape format.
2. `raw_graph.json`: Canonical `DependencyGraph` containing full AST component objects and edge kinds.

If you later want to re-export a modified or filtered `raw_graph.json` back into Cytoscape format without re-running the entire source analysis, use `export-cyto`:

```bash
omnideps export-cyto raw_graph.json web_cytoscape.json
```

---

## Output Schema

The resulting file adheres to standard Cytoscape elements structure:

```json
{
  "elements": {
    "nodes": [
      {
        "data": {
          "id": "com.omnideps.core.Engine",
          "label": "Engine",
          "type": "class",
          "parent": "com.omnideps.core"
        }
      }
    ],
    "edges": [
      {
        "data": {
          "id": "e_01",
          "source": "com.omnideps.core.Engine",
          "target": "com.omnideps.core.StorageService",
          "kind": "UsesFieldType"
        }
      }
    ]
  }
}
```
