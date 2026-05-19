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
        }
        s.total_free_functions += m.free_functions.len();

        let sub_s = build_analysis_summary(&m.sub_modules);
        s.total_modules += sub_s.total_modules;
        s.total_structured_types += sub_s.total_structured_types;
        s.total_free_functions += sub_s.total_free_functions;
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
