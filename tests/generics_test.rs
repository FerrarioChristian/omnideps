use omnideps::analyzer::analyze_code_snippet;
use omnideps::config::AnalyzerConfig;
use omnideps::language::SupportedLanguage;
use omnideps::model::*;

fn analyze_snippet(lang: SupportedLanguage, source: &str, filename: &str) -> DependencyGraph {
    let config = AnalyzerConfig::default();
    let (_resolved_modules, graph) = analyze_code_snippet(lang, source, filename, &config)
        .expect("Failed to parse source snippet");
    graph
}

fn ends_with(full: &[String], suffix: &[&str]) -> bool {
    if full.len() < suffix.len() {
        return false;
    }
    let offset = full.len() - suffix.len();
    full[offset..]
        .iter()
        .zip(suffix.iter())
        .all(|(a, b)| a == *b)
}

fn has_edge(
    graph: &DependencyGraph,
    from_suffix: &[&str],
    to_suffix: &[&str],
    kind: DependencyEdgeKind,
) -> bool {
    graph
        .edges
        .iter()
        .any(|e| ends_with(&e.from, from_suffix) && ends_with(&e.to, to_suffix) && e.kind == kind)
}

// =========================================================================
// RUST TESTS
// =========================================================================

#[test]
fn test_rust_generic_struct_and_field_dependencies() {
    let code = r#"
        pub struct Item {
            pub id: u32,
        }

        pub struct Tag {
            pub label: String,
        }

        pub struct Container<T> {
            pub value: T,
        }

        pub struct Pair<K, V> {
            pub key: K,
            pub val: V,
        }

        pub struct Warehouse {
            pub single: Container<Item>,
            pub entry: Pair<Item, Tag>,
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Rust, code, "src/lib.rs");

    // Warehouse.single uses Container and Item
    assert!(
        has_edge(
            &graph,
            &["Warehouse", "single"],
            &["Container"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Warehouse.single -> Container (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Warehouse", "single"],
            &["Item"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Warehouse.single -> Item (UsesFieldType)"
    );

    // Warehouse.entry uses Pair, Item, and Tag
    assert!(
        has_edge(
            &graph,
            &["Warehouse", "entry"],
            &["Pair"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Warehouse.entry -> Pair (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Warehouse", "entry"],
            &["Item"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Warehouse.entry -> Item (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Warehouse", "entry"],
            &["Tag"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Warehouse.entry -> Tag (UsesFieldType)"
    );
}

#[test]
fn test_rust_generic_trait_bound_dispatch() {
    let code = r#"
        pub trait Worker {
            fn do_work(&self);
        }

        pub fn execute_inline_bound<T: Worker>(w: T) {
            w.do_work();
        }

        pub fn execute_where_clause<T>(w: T) where T: Worker {
            w.do_work();
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Rust, code, "src/lib.rs");

    assert!(
        has_edge(
            &graph,
            &["execute_inline_bound"],
            &["Worker", "do_work"],
            DependencyEdgeKind::Calls
        ),
        "Expected execute_inline_bound -> Worker.do_work (Calls)"
    );

    assert!(
        has_edge(
            &graph,
            &["execute_where_clause"],
            &["Worker", "do_work"],
            DependencyEdgeKind::Calls
        ),
        "Expected execute_where_clause -> Worker.do_work (Calls)"
    );
}

#[test]
fn test_rust_generic_multiple_trait_bounds() {
    let code = r#"
        pub trait Printable {
            fn print(&self);
        }

        pub trait Loggable {
            fn log(&self);
        }

        pub fn execute_both_bounds<T: Printable + Loggable>(item: T) {
            item.print();
            item.log();
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Rust, code, "src/lib.rs");

    assert!(
        has_edge(
            &graph,
            &["execute_both_bounds"],
            &["Printable", "print"],
            DependencyEdgeKind::Calls
        ),
        "Expected execute_both_bounds -> Printable.print (Calls)"
    );
    assert!(
        has_edge(
            &graph,
            &["execute_both_bounds"],
            &["Loggable", "log"],
            DependencyEdgeKind::Calls
        ),
        "Expected execute_both_bounds -> Loggable.log (Calls)"
    );
}

// =========================================================================
// C++ TESTS
// =========================================================================

#[test]
fn test_cpp_template_class_and_concrete_usage() {
    let code = r#"
        class Car {
        public:
            int speed;
        };

        template <typename T>
        class Box {
        public:
            T item;
        };

        template <typename K, typename V>
        class KeyValue {
        public:
            K key;
            V val;
        };

        class Garage {
        public:
            Box<Car> car_box;
            KeyValue<int, Car> mapped_car;
        };
    "#;

    let graph = analyze_snippet(SupportedLanguage::Cpp, code, "src/garage.cpp");

    assert!(
        has_edge(
            &graph,
            &["Garage", "car_box"],
            &["Box"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Garage.car_box -> Box (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Garage", "car_box"],
            &["Car"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Garage.car_box -> Car (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Garage", "mapped_car"],
            &["KeyValue"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Garage.mapped_car -> KeyValue (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Garage", "mapped_car"],
            &["Car"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Garage.mapped_car -> Car (UsesFieldType)"
    );
}

#[test]
fn test_cpp_template_member_chain_call() {
    let code = r#"
        class Vehicle {
        public:
            void honk();
        };

        template <typename T>
        class Holder {
        public:
            T value;
            T get_value() { return value; }
        };

        void operate_holder(Holder<Vehicle>& h) {
            h.get_value().honk();
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Cpp, code, "src/holder.cpp");

    assert!(
        has_edge(
            &graph,
            &["operate_holder"],
            &["Holder"],
            DependencyEdgeKind::UsesParamType
        ),
        "Expected operate_holder -> Holder (UsesParamType)"
    );
    assert!(
        has_edge(
            &graph,
            &["operate_holder"],
            &["Vehicle"],
            DependencyEdgeKind::UsesParamType
        ),
        "Expected operate_holder -> Vehicle (UsesParamType)"
    );
    assert!(
        has_edge(
            &graph,
            &["operate_holder"],
            &["Vehicle", "honk"],
            DependencyEdgeKind::Calls
        ),
        "Expected operate_holder -> Vehicle.honk (Calls)"
    );
}

#[test]
fn test_cpp_nested_template_vector_usage() {
    let code = r#"
        namespace std {
            template <typename T>
            class vector {};
        }

        class Engine {};

        template <typename T>
        class Container {
        public:
            T element;
        };

        class Fleet {
        public:
            std::vector<Container<Engine>> engines;
        };
    "#;

    let graph = analyze_snippet(SupportedLanguage::Cpp, code, "src/fleet.cpp");

    assert!(
        has_edge(
            &graph,
            &["Fleet", "engines"],
            &["Container"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Fleet.engines -> Container (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Fleet", "engines"],
            &["Engine"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Fleet.engines -> Engine (UsesFieldType)"
    );
}

// =========================================================================
// JAVA TESTS
// =========================================================================

#[test]
fn test_java_generic_bound_method_call() {
    let code = r#"
        public interface Service {
            void execute();
        }

        public class Runner {
            public <T extends Service> void runService(T s) {
                s.execute();
            }
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Java, code, "src/Runner.java");

    assert!(
        has_edge(
            &graph,
            &["Runner", "runService"],
            &["Service", "execute"],
            DependencyEdgeKind::Calls
        ),
        "Expected Runner.runService -> Service.execute (Calls)"
    );
}

#[test]
fn test_java_generic_container_field_dependencies() {
    let code = r#"
        public class Entity {}

        public class Repository<T> {
            private T data;
        }

        public class Manager {
            private Repository<Entity> repo;
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Java, code, "src/Manager.java");

    assert!(
        has_edge(
            &graph,
            &["Manager", "repo"],
            &["Repository"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Manager.repo -> Repository (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Manager", "repo"],
            &["Entity"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Manager.repo -> Entity (UsesFieldType)"
    );
}

#[test]
fn test_java_multiple_type_parameters() {
    let code = r#"
        public class KeyType {}
        public class ValType {}

        public class Mapping<K, V> {
            private K key;
            private V value;
        }

        public class Cache {
            private Mapping<KeyType, ValType> entry;
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Java, code, "src/Cache.java");

    assert!(
        has_edge(
            &graph,
            &["Cache", "entry"],
            &["Mapping"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Cache.entry -> Mapping (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Cache", "entry"],
            &["KeyType"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Cache.entry -> KeyType (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Cache", "entry"],
            &["ValType"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Cache.entry -> ValType (UsesFieldType)"
    );
}

// =========================================================================
// PYTHON TESTS
// =========================================================================

#[test]
fn test_python_typevar_bound_method_call() {
    let code = r#"
        from typing import TypeVar, Generic

        class Model:
            def save(self):
                pass

        M = TypeVar('M', bound=Model)

        def persist_entity(item: M):
            item.save()
    "#;

    let graph = analyze_snippet(SupportedLanguage::Python, code, "src/app.py");

    assert!(
        has_edge(
            &graph,
            &["persist_entity"],
            &["Model", "save"],
            DependencyEdgeKind::Calls
        ),
        "Expected persist_entity -> Model.save (Calls)"
    );
}

#[test]
fn test_python_generic_class_and_nested_subscripts() {
    let code = r#"
        from typing import TypeVar, Generic

        T = TypeVar('T')

        class User:
            pass

        class Box(Generic[T]):
            pass

        class Storage:
            def __init__(self, u: User) -> None:
                self.single: Box[User] = Box()
                self.nested: list[list[User]] = [[u]]
                self.mapped: dict[str, User] = {"k": u}
    "#;

    let graph = analyze_snippet(SupportedLanguage::Python, code, "src/storage.py");

    assert!(
        has_edge(
            &graph,
            &["Storage", "single"],
            &["Box"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Storage.single -> Box (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Storage", "single"],
            &["User"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Storage.single -> User (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Storage", "nested"],
            &["User"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Storage.nested -> User (UsesFieldType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Storage", "mapped"],
            &["User"],
            DependencyEdgeKind::UsesFieldType
        ),
        "Expected Storage.mapped -> User (UsesFieldType)"
    );
}

#[test]
fn test_python_assignment_type_alias() {
    let code = r#"
        class Account:
            pass

        AccountList = list[Account]
    "#;

    let graph = analyze_snippet(SupportedLanguage::Python, code, "src/alias.py");

    assert!(
        has_edge(
            &graph,
            &["AccountList"],
            &["Account"],
            DependencyEdgeKind::Aliases
        ),
        "Expected AccountList -> Account (Aliases)"
    );
}

#[test]
fn test_python_empty_file() {
    let code = "# Just a comment\n";
    let _graph = analyze_snippet(SupportedLanguage::Python, code, "src/empty.py");
}

#[test]
fn test_python_self_referential_assignment() {
    let code = r#"
        mergedParents = mergedParents[k]
    "#;

    let _graph = analyze_snippet(SupportedLanguage::Python, code, "src/cyclic.py");
}

#[test]
fn test_python_wildcard_import_cycle() {
    let code = r#"
        from antlr4 import *

        class TestLexer(Lexer):
            pass
    "#;

    let _graph = analyze_snippet(
        SupportedLanguage::Python,
        code,
        "Python3/tests/TestLexer.py",
    );
}

#[test]
fn test_java_multiple_interfaces_resolution() {
    let code = r#"
        interface A { void a(); }
        interface B { void b(); }
        interface C { void c(); }
        interface D { void d(); }

        class MultiImpl implements A, B, C, D {
            public void a() {}
            public void b() {}
            public void c() {}
            public void d() {}
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Java, code, "MultiImpl.java");
    assert!(
        has_edge(&graph, &["MultiImpl"], &["A"], DependencyEdgeKind::IsA),
        "Expected MultiImpl -> A (IsA)"
    );
    assert!(
        has_edge(&graph, &["MultiImpl"], &["D"], DependencyEdgeKind::IsA),
        "Expected MultiImpl -> D (IsA)"
    );
}

#[test]
fn test_constructor_and_destructor_dependencies() {
    let code = r#"
        class Config {};
        class Logger {
        public:
            void log();
        };

        class Server {
        private:
            Logger logger;
        public:
            Server(Config c) {
                logger.log();
            }
            ~Server() {
                logger.log();
            }
        };
    "#;

    let graph = analyze_snippet(SupportedLanguage::Cpp, code, "Server.cpp");
    assert!(
        has_edge(
            &graph,
            &["Server"],
            &["Server", "Server"],
            DependencyEdgeKind::NestedIn
        ),
        "Expected Server -> Server.Server (NestedIn)"
    );
    assert!(
        has_edge(
            &graph,
            &["Server", "Server"],
            &["Config"],
            DependencyEdgeKind::UsesParamType
        ),
        "Expected Server.Server -> Config (UsesParamType)"
    );
    assert!(
        has_edge(
            &graph,
            &["Server", "Server"],
            &["Logger", "log"],
            DependencyEdgeKind::Calls
        ),
        "Expected Server.Server -> Logger.log (Calls)"
    );
    assert!(
        has_edge(
            &graph,
            &["Server"],
            &["Server", "~Server"],
            DependencyEdgeKind::NestedIn
        ),
        "Expected Server -> Server.~Server (NestedIn)"
    );
    assert!(
        has_edge(
            &graph,
            &["Server", "~Server"],
            &["Logger", "log"],
            DependencyEdgeKind::Calls
        ),
        "Expected Server.~Server -> Logger.log (Calls)"
    );
}

#[test]
fn test_c_parenthesized_cast_and_call_disambiguation() {
    let code = r#"
        typedef int CustomInt;

        int helper_func(int x) {
            return x + 1;
        }

        CustomInt do_cast(double val) {
            return (CustomInt)(val);
        }

        int do_call(int a) {
            return (helper_func)(a);
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::C, code, "src/math.c");

    assert!(
        has_edge(
            &graph,
            &["do_cast"],
            &["CustomInt"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_cast -> CustomInt (CastsTo)"
    );
    assert!(
        has_edge(
            &graph,
            &["do_call"],
            &["helper_func"],
            DependencyEdgeKind::Calls
        ),
        "Expected do_call -> helper_func (Calls)"
    );
}

#[test]
fn test_c_advanced_cast_and_call_scenarios() {
    let code = r#"
        typedef int CustomInt;

        int helper_func(int x) {
            return x + 1;
        }

        CustomInt do_nested_cast(double val) {
            return ((CustomInt))(val);
        }

        int do_nested_call(int a) {
            return ((helper_func))(a);
        }

        int do_primitive_cast(double val) {
            return (int)(val);
        }

        int do_expr_cast(double val) {
            return 10 + (CustomInt)(val);
        }

        CustomInt do_cast_of_call(int a) {
            return (CustomInt)(helper_func(a));
        }

        int do_call_with_cast(double val) {
            return (helper_func)((CustomInt)(val));
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::C, code, "src/math.c");

    // 1. Nested cast
    assert!(
        has_edge(
            &graph,
            &["do_nested_cast"],
            &["CustomInt"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_nested_cast -> CustomInt (CastsTo)"
    );

    // 2. Nested call
    assert!(
        has_edge(
            &graph,
            &["do_nested_call"],
            &["helper_func"],
            DependencyEdgeKind::Calls
        ),
        "Expected do_nested_call -> helper_func (Calls)"
    );

    // 3. Primitive cast
    assert!(
        has_edge(
            &graph,
            &["do_primitive_cast"],
            &["int"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_primitive_cast -> int (CastsTo)"
    );

    // 4. Cast in expression
    assert!(
        has_edge(
            &graph,
            &["do_expr_cast"],
            &["CustomInt"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_expr_cast -> CustomInt (CastsTo)"
    );

    // 5. Cast of call result (should have CastsTo CustomInt AND Calls helper_func)
    assert!(
        has_edge(
            &graph,
            &["do_cast_of_call"],
            &["CustomInt"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_cast_of_call -> CustomInt (CastsTo)"
    );
    assert!(
        has_edge(
            &graph,
            &["do_cast_of_call"],
            &["helper_func"],
            DependencyEdgeKind::Calls
        ),
        "Expected do_cast_of_call -> helper_func (Calls)"
    );

    // 6. Call with cast argument (should have Calls helper_func AND CastsTo CustomInt)
    assert!(
        has_edge(
            &graph,
            &["do_call_with_cast"],
            &["helper_func"],
            DependencyEdgeKind::Calls
        ),
        "Expected do_call_with_cast -> helper_func (Calls)"
    );
    assert!(
        has_edge(
            &graph,
            &["do_call_with_cast"],
            &["CustomInt"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_call_with_cast -> CustomInt (CastsTo)"
    );
}

#[test]
fn test_c_pointer_cast_scenarios() {
    let code = r#"
        typedef struct Point {
            int x;
            int y;
        } Point;

        typedef int CustomInt;

        void* do_ptr_cast(void* ptr) {
            CustomInt* p1 = (CustomInt*)(ptr);
            Point* p2 = (Point*)(ptr);
            return (void*)(p1);
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::C, code, "src/points.c");

    assert!(
        has_edge(
            &graph,
            &["do_ptr_cast"],
            &["CustomInt"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_ptr_cast -> CustomInt (CastsTo)"
    );
    assert!(
        has_edge(
            &graph,
            &["do_ptr_cast"],
            &["Point"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected do_ptr_cast -> Point (CastsTo)"
    );
}

#[test]
fn test_c_function_pointer_cast_and_invocation() {
    let code = r#"
        typedef void (*Callback)(int);

        void my_handler(int x) {}

        void test_fp() {
            Callback cb = (Callback)(my_handler);
            (cb)(42);
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::C, code, "src/fp.c");

    assert!(
        has_edge(
            &graph,
            &["test_fp"],
            &["Callback"],
            DependencyEdgeKind::CastsTo
        ),
        "Expected test_fp -> Callback (CastsTo)"
    );
}

#[test]
fn test_c_local_struct_ast() {
    let code = r#"
void factory() {
    struct LocalProduct {
        int id;
    };
    struct LocalProduct p = {1};
}
"#;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    println!("C AST: {}", tree.root_node().to_sexp());
}

#[test]
fn test_c_macro_ast() {
    let code = r#"
#define MAX_BUFFER 1024
#define SQUARE(x) ((x) * (x))
#define LOG_NODE(n) printf("Node ID: %d", n->id)
"#;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    println!("C AST: {}", tree.root_node().to_sexp());
}

#[test]

fn test_rust_closure() {
    let code = r#"
        struct TargetStruct {
            x: i32,
        }
        impl TargetStruct {
            fn target_method() {}
        }
        fn test_func() {
            let f = |x| x + 1;
            let g = |y: i32| {
                TargetStruct::target_method();
                y + 2
            };
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Rust, code, "src/closure.rs");

    assert!(
        has_edge(
            &graph,
            &["test_func"],
            &["TargetStruct", "target_method"],
            DependencyEdgeKind::Calls
        ),
        "Expected test_func -> TargetStruct::target_method (Calls)"
    );
    assert!(
        !graph.edges.iter().any(|e| ends_with(&e.from, &["test_func"]) && e.to == &["x"]),
        "Closure parameter 'x' should not be emitted as an edge from test_func"
    );
    assert!(
        !graph.edges.iter().any(|e| ends_with(&e.from, &["test_func"]) && e.to == &["y"]),
        "Closure parameter 'y' should not be emitted as an edge from test_func"
    );
    assert!(
        !graph.nodes.iter().any(|n| match n {
            Component::External(qn) => qn == &["x"] || qn == &["y"],
            _ => false,
        }),
        "External nodes for closure parameters 'x' or 'y' should not exist"
    );
}

#[test]
fn test_python_lambda() {
    let code = r#"
class TargetClass:
    @staticmethod
    def static_method(val):
        pass

def test_func():
    f = lambda x: TargetClass.static_method(x)
"#;

    let graph = analyze_snippet(SupportedLanguage::Python, code, "src/closure.py");
    assert!(
        has_edge(
            &graph,
            &["test_func"],
            &["TargetClass", "static_method"],
            DependencyEdgeKind::Calls
        ),
        "Expected test_func -> TargetClass::static_method (Calls)"
    );
    assert!(
        !graph.edges.iter().any(|e| ends_with(&e.to, &["x"])),
        "Lambda parameter 'x' should not be emitted as an edge"
    );
    assert!(
        !graph.nodes.iter().any(|n| match n {
            Component::External(qn) => ends_with(qn, &["x"]),
            _ => false,
        }),
        "External node for lambda parameter 'x' should not exist"
    );
}

#[test]
fn test_java_lambda() {
    let code = r#"
class TargetClass {
    public static void staticMethod() {}
}

class Main {
    void testFunc() {
        Runnable r = () -> {
            TargetClass.staticMethod();
        };
    }
}
"#;

    let graph = analyze_snippet(SupportedLanguage::Java, code, "src/Main.java");
    assert!(
        has_edge(
            &graph,
            &["testFunc"],
            &["TargetClass", "staticMethod"],
            DependencyEdgeKind::Calls
        ),
        "Expected testFunc -> TargetClass::staticMethod (Calls)"
    );
}

#[test]
fn test_cpp_lambda() {
    let code = r#"
class TargetClass {
public:
    static void static_method() {}
};

void test_func() {
    auto f = [](int x) {
        TargetClass::static_method();
        return x + 1;
    };
}
"#;

    let graph = analyze_snippet(SupportedLanguage::Cpp, code, "src/main.cpp");
    assert!(
        has_edge(
            &graph,
            &["test_func"],
            &["TargetClass", "static_method"],
            DependencyEdgeKind::Calls
        ),
        "Expected test_func -> TargetClass::static_method (Calls)"
    );
}

#[test]
fn test_rust_local_variable_constructor_type_inference() {
    let code = r#"
struct Point {
    x: i32,
}
impl Point {
    fn new() -> Self {
        Point { x: 0 }
    }
    fn display(&self) {}
}
fn test_func() {
    let p = Point::new();
    p.display();
}
"#;

    let graph = analyze_snippet(SupportedLanguage::Rust, code, "src/main.rs");
    println!("Graph edges: {:?}", graph.edges);
    assert!(
        has_edge(
            &graph,
            &["test_func"],
            &["Point", "new"],
            DependencyEdgeKind::Calls
        ),
        "Expected test_func -> Point::new (Calls)"
    );
    assert!(
        has_edge(
            &graph,
            &["test_func"],
            &["Point", "display"],
            DependencyEdgeKind::Calls
        ),
        "Expected test_func -> Point::display (Calls)"
    );
}

#[test]
fn test_java_chained_method_call_and_constructor() {
    let code = r#"
        class Engine {
            public void start() {}
        }

        class Car {
            private Engine engine = new Engine();
            public Engine getEngine() { return engine; }
        }

        class Garage {
            private Car car = new Car();
            public void testChain() {
                car.getEngine().start();
                new Car().getEngine().start();
            }
        }
    "#;

    let graph = analyze_snippet(SupportedLanguage::Java, code, "src/Garage.java");

    assert!(
        has_edge(
            &graph,
            &["Garage", "testChain"],
            &["Car", "getEngine"],
            DependencyEdgeKind::Calls
        ),
        "Expected Garage.testChain -> Car.getEngine (Calls)"
    );
    assert!(
        has_edge(
            &graph,
            &["Garage", "testChain"],
            &["Engine", "start"],
            DependencyEdgeKind::Calls
        ),
        "Expected Garage.testChain -> Engine.start (Calls)"
    );
}





