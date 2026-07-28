# PYTHON-1: Missing `AccessesField` edges for class properties within `__init__`

## Description of the Failed Edges

In the `benchmark-python` execution, several edges related to field initialization inside constructors were missing. For example:

- `models.User.__init__ -> models.User.username`
- `models.User.__init__ -> models.User.birth_year`
- `models.Admin.__init__ -> models.Admin.role`

## Cause of the Bug

The analyzer failed to correctly create `AccessesField` edges from the constructor to the class fields due to two related issues in the Extraction and Name Resolution phases:

1. **Incorrect Local Variable Extraction (`src/heuristics/body_extraction.rs`)**:
   During the AST body extraction, Python assignments such as `self.username = username` are parsed as `assignment` nodes. The `extract_declarations` logic indiscriminately treated all `assignment` nodes as local variable declarations. As a result, `username` was wrongly registered as a local variable of `__init__` (`models.User.__init__.username`) instead of triggering a behavioral field access dependency.

2. **Incomplete Scope Path for Classes (`src/resolver/scope.rs`)**:
   Even if the behavioral dependency was correctly extracted as `Unresolved(["self", "username"])`, it would fail to resolve. When `register_structured_type` registers a class in the `ScopeTree`, it assigned the class type for the `self` parameter using `TypeRef::Resolved(st.name.clone())`. However, `st.name` only contains the local name of the class (e.g., `vec!["User"]`). The Name Resolution phase (`executor.rs`) searches for this path starting from the global root. Since `"User"` is not at the root but inside the `"models"` module, `find_scope_for_type` failed to locate the class scope, causing the lexical lookup for `"username"` to fail.

## Solution Implemented

1. **Body Extraction Fix (`src/heuristics/body_extraction.rs`)**:
   Modified `extract_block` to inspect the left-hand side of `assignment` nodes. If the left side is an `attribute` or `field_expression` (like `self.username`), it is excluded from being treated as a local variable declaration. This ensures it falls through to `find_behavioral_deps`, where it is correctly extracted as a field access.

2. **Scope Registration Fix (`src/resolver/scope.rs`)**:
   Updated `register_structured_type` to build the absolute fully-qualified path of the class scope by traversing the `ScopeTree` upwards. The `parent_class_type` passed to `register_function` is now a `TypeRef::Resolved` containing the full path (e.g., `["models", "User"]`), allowing `find_scope_for_type` to correctly locate the class and resolve its fields during Phase 2.

With these changes, the `models.User.__init__ -> models.User.username` edges are now correctly resolved as `AccessesField` dependencies, reducing the missing edges in the Python benchmark from 14 down to 10.

# Python Benchmark Resolution Walkthrough

This document summarizes the steps taken to fix the remaining missing edges in the Python benchmark and improve the `benchmark_all` utility.

## 1. Type Hints as Strings

**Issue:** `create_root(cls, username: str) -> 'SuperAdmin'` had its return type extracted literally with quotes (`'SuperAdmin'`), which prevented it from correctly resolving to the `SuperAdmin` class in the scope tree.
**Fix:** Updated `extract_type_ref` in `src/heuristics/type_extraction.rs` to support `string` nodes directly and strip quotation marks from the extracted string. This correctly identifies the return type as `SuperAdmin`, resolving the missing edge:

- `services.register_admin -> advanced_models.SuperAdmin.grant_permission`

## 2. Nested Field Accesses (`PY-ACC-10`, `PY-ACC-11`)

**Issue:** Expressions like `self.permissions.append(perm)` did not register an access to `self.permissions`, because we skipped recursing into the base object to avoid spurious function pointer accesses.
**Fix:** Updated `extract_call_dependency` in `src/heuristics/body_extraction.rs` to explicitly extract the `object`/`value`/`left` portion of a method invocation as a field access dependency. This captures `self.permissions` without triggering false positives.

- Edge restored: `advanced_models.SuperAdmin.grant_permission -> advanced_models.SuperAdmin.permissions`

## 3. Transitive Imports (`PY-TRANS-IMP-3`)

**Issue:** When `transitive_main` imports `TransitiveClass` from `lib_b` (which actually imports it from `lib_a`), the graph was outputting `transitive_main -> lib_a.TransitiveClass` instead of `lib_b.TransitiveClass`.
**Fix:** Removed the mutation of `imp.path` in `execute_module` (`src/resolver/executor.rs`). `find_global` still dynamically resolves the transitive import when evaluating references, but the export phase now properly retains and emits the exact syntactic import dependency `lib_b.TransitiveClass` as written in the source code.

- Edge restored: `transitive_main -> lib_b.TransitiveClass`

## 4. Java Regression Fix

**Issue:** An accidental deletion of `"field_access"` from the `matches!` block in `type_extraction.rs` during the Python fixes caused the Java benchmark to lose 8 field access edges (`ACC-1` to `ACC-9`).
**Fix:** Restored `"field_access"` to the AST node kind matching block. The Java benchmark has now returned to 35/37 found edges.

## 5. Improved Benchmark Runner

**Issue:** The `benchmark_all` runner printed output across multiple rows, overwriting previous results, making it difficult to track improvements sequentially.
**Fix:** Rewrote `src/bin/benchmark_all.rs` to:

- Open `tests/benchmarks/results_history.csv` in append mode.
- Use a `BTreeMap` to dynamically collect and alphabetically sort all found languages.
- Emit a single row per execution starting with the `Timestamp`, followed by `Language Nodes` and `Language Edges` metrics formatted as `Found/Total` (e.g. `46/46`).
