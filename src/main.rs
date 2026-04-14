use language_agnostic_analyzer::extractors;
use language_agnostic_analyzer::ir::Component;

fn main() {
    let source = r#"
        struct Point { x: i32, y: i32 }
        fn add(a: i32, b: i32) -> i32 { a + b }
    "#;

    let (modules, graph, summary) =
        extractors::full_analysis(extractors::languages::rust(), source).unwrap();

    println!("=== SUMMARY ===");
    println!("Moduli: {}", summary.total_modules);
    println!("Structured types: {}", summary.total_structured_types);
    println!("Free functions: {}", summary.total_free_functions);

    println!("\n=== GRAPH NODES ===");
    for node in &graph.nodes {
        match node {
            Component::StructuredType(st) => println!("Structured: {:?}", st.name),
            Component::FreeFunction(ff) => println!("Function: {}", ff.name),
            _ => {}
        }
    }
}
