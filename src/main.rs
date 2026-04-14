use language_agnostic_analyzer::extractor::{full_analysis, languages};

fn main() {
    let sources = vec![
        (
            "Rust",
            languages::rust(),
            r#"
            struct Point { x: i32 }
            fn add(a: i32, b: i32) -> i32 { a + b }
            impl Point { fn new() -> Self { Point { x: 0 } } }
        "#,
        ),
        (
            "Java",
            languages::java(),
            r#"
            class MyClass { int field; void method() {} }
        "#,
        ),
        (
            "Python",
            languages::python(),
            r#"
            class MyClass:
                def method(self): pass
            def free_func(): pass
        "#,
        ),
        (
            "C",
            languages::c(),
            r#"
            struct MyStruct { int x; };
            int free_func(int a) { return a; }
        "#,
        ),
        (
            "C++",
            languages::cpp(),
            r#"
            class MyClass { void method(); };
        "#,
        ),
    ];

    for (lang_name, lang, code) in sources {
        println!("\n=== {} ===", lang_name);
        let (modules, graph, summary) = full_analysis(lang, code).unwrap();
        println!("Structured: {}", summary.total_structured_types);
        println!("Free funcs: {}", summary.total_free_functions);
        println!("Edges generati: {}", graph.edges.len());
    }
}
