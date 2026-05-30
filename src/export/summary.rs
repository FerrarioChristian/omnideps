use crate::ir::*;

// ==================== BENCHMARK ====================
/// Aggregates basic statistics about the extracted components across all provided modules.
pub fn build_analysis_summary(modules: &[Module]) -> AnalysisSummary {
    let mut s = AnalysisSummary {
        total_modules: modules.len(),
        ..Default::default()
    };
    for m in modules {
        s.total_structured_types += m.structured_types.len();
        for st in &m.structured_types {
            s.total_structured_types += count_nested_types(st);
            count_refs_in_st(st, &mut s.resolved_refs, &mut s.failed_refs);
        }
        s.total_free_functions += m.free_functions.len();
        for ff in &m.free_functions {
            count_refs_in_func(ff, &mut s.resolved_refs, &mut s.failed_refs);
        }

        let sub_s = build_analysis_summary(&m.sub_modules);
        s.total_modules += sub_s.total_modules;
        s.total_structured_types += sub_s.total_structured_types;
        s.total_free_functions += sub_s.total_free_functions;
        s.resolved_refs += sub_s.resolved_refs;
        s.failed_refs += sub_s.failed_refs;
    }
    s
}

fn count_nested_types(st: &StructuredType) -> usize {
    let mut count = st.nested_types.len();
    for nested in &st.nested_types {
        count += count_nested_types(nested);
    }
    count
}

fn count_refs_in_st(st: &StructuredType, resolved: &mut usize, failed: &mut usize) {
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
        _ => {}
    }
}
