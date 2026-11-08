---
title: Custom Rules & Tuning
description: How to customize Omnideps analysis and fine-tune resolution behavior.
order: 1
---

# Custom Rules & Tuning

Omnideps can be tuned for specialized architectures, unconventional monorepos, and custom symbol resolution using an `omnideps.json` configuration file.

---

## 1. Initializing Configuration

Generate a base configuration file:

```bash
omnideps config init
```

This outputs `omnideps.json` with the default strategies.

---

## 2. Practical Configuration Recipes

### Example: Enabling Transitive Imports for Java Monorepos

If your Java codebase relies on common re-export facades:

```json
{
  "languages": {
    "java": {
      "transitive_imports": true
    }
  }
}
```

### Example: Custom Receiver Keywords

If analyzing a custom dialect or preprocessed code where the instance receiver is named differently:

```json
{
  "languages": {
    "cpp": {
      "self_keyword": "self"
    }
  }
}
```

---

## 3. Applying the Configuration

Pass the configuration file with the `--config` flag to any analysis, IR dump, or benchmark command:

```bash
# Analyze with custom rules
omnideps analyze ./my-repo --config ./omnideps.json -o ./output.json

# Dump IR with custom rules
omnideps dump-ir ./my-repo --config ./omnideps.json -o ./debug_ir.json

# Benchmark with custom rules
omnideps benchmark run tests/benchmarks/benchmark-java --config ./omnideps.json
```
