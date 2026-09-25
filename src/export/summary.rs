use crate::model::*;

use std::collections::HashSet;

// ==================== BENCHMARK ====================
/// Aggregates basic statistics about the extracted components across all provided modules.
///
/// Modules are deduplicated by their fully qualified path (FQN) to avoid inflating
/// the count across per-file compilation units.
pub fn build_analysis_summary(modules: &[Module]) -> AnalysisSummary {
    let mut s = AnalysisSummary::default();
    let mut unique_modules = HashSet::new();

    for m in modules {
        collect_module_summary(m, &[], &mut unique_modules, &mut s);
    }

    s.total_modules = if unique_modules.is_empty() && !modules.is_empty() {
        1
    } else {
        unique_modules.len()
    };

    s
}

fn collect_module_summary(
    m: &Module,
    prefix: &[String],
    unique_modules: &mut HashSet<Vec<String>>,
    s: &mut AnalysisSummary,
) {
    let mut current_path = prefix.to_vec();
    let name_parts: Vec<String> = m
        .name
        .iter()
        .filter(|part| *part != "root")
        .cloned()
        .collect();
    current_path.extend(name_parts);

    if !current_path.is_empty() {
        unique_modules.insert(current_path.clone());
    }

    s.total_structured_types += m.structured_types.len();
    for st in &m.structured_types {
        s.total_structured_types += count_nested_types(st);
        count_refs_in_st(st, &mut s.resolved_refs, &mut s.failed_refs);
    }
    s.total_free_functions += m.free_functions.len();
    for ff in &m.free_functions {
        count_refs_in_func(ff, &mut s.resolved_refs, &mut s.failed_refs);
    }

    for sub in &m.sub_modules {
        collect_module_summary(sub, &current_path, unique_modules, s);
    }
}

fn count_nested_types(st: &StructuredType) -> usize {
    let mut count = st.nested_types.len();
    for nested in &st.nested_types {
        count += count_nested_types(nested);
    }
    count
}

fn count_refs_in_st(st: &StructuredType, resolved: &mut usize, failed: &mut usize) {
    for tp in &st.type_parameters {
        for b in &tp.bounds {
            tally_ref(b, resolved, failed);
        }
    }
    for sup in &st.super_types {
        tally_ref(sup, resolved, failed);
    }
    for f in &st.fields {
        tally_ref(&f.ty, resolved, failed);
    }
    for m in &st.methods {
        count_refs_in_func(m, resolved, failed);
    }
    for nested in &st.nested_types {
        count_refs_in_st(nested, resolved, failed);
    }
}

fn count_refs_in_func(f: &Function, resolved: &mut usize, failed: &mut usize) {
    for tp in &f.type_parameters {
        for b in &tp.bounds {
            tally_ref(b, resolved, failed);
        }
    }
    for p in &f.signature.parameters {
        tally_ref(&p.ty, resolved, failed);
    }
    tally_ref(&f.signature.return_type, resolved, failed);

    if let Some(body) = &f.body {
        count_refs_in_block(body, resolved, failed);
    }
}

fn count_refs_in_block(block: &Block, resolved: &mut usize, failed: &mut usize) {
    for decl in &block.declarations {
        tally_ref(&decl.ty, resolved, failed);
    }
    for call in &block.calls {
        tally_ref(call, resolved, failed);
    }
    for inst in &block.instantiates {
        tally_ref(inst, resolved, failed);
    }
    for sub in &block.sub_blocks {
        count_refs_in_block(sub, resolved, failed);
    }
}

fn tally_ref(tr: &TypeRef, resolved: &mut usize, failed: &mut usize) {
    match tr {
        TypeRef::Resolved(_) | TypeRef::External(_) => *resolved += 1,
        TypeRef::Failed(_) | TypeRef::Unresolved(_) => *failed += 1,
        TypeRef::Union(variants) => {
            for v in variants {
                tally_ref(v, resolved, failed);
            }
        }
        TypeRef::Generic { base, args } => {
            tally_ref(base, resolved, failed);
            for a in args {
                tally_ref(a, resolved, failed);
            }
        }
        TypeRef::TypeVar { bounds, .. } => {
            for b in bounds {
                tally_ref(b, resolved, failed);
            }
        }
        _ => {}
    }
}
