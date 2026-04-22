pub type Identifier = String;
pub type QualifiedName = Vec<Identifier>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PrimitiveType {
    Int,
    Float,
    Bool,
    String,
    Void,
    Other(String),
    // Aggiungere altri tipi primitivi comuni se necessario
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TypeRef {
    Primitive(PrimitiveType),
    UserDefined(QualifiedName),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StructuredTypeKind {
    Class,
    Struct,
    Interface,
    Trait,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: Identifier,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Parameter {
    pub name: Option<Identifier>,
    pub ty: TypeRef,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Method {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FreeFunction {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImplBlock {
    pub name: QualifiedName,
    pub methods: Vec<Method>,
    pub impl_for: TypeRef,
    pub implements_trait: Option<TypeRef>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructuredType {
    pub name: QualifiedName,
    pub kind: StructuredTypeKind,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub super_types: Vec<TypeRef>,
    pub nested_types: Vec<StructuredType>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Module {
    pub name: QualifiedName,
    pub sub_modules: Vec<Module>,
    pub structured_types: Vec<StructuredType>,
    pub free_functions: Vec<FreeFunction>,
    pub impl_blocks: Vec<ImplBlock>,
    // Convertire le tre tipologie di componenti in un'unica lista di Componenti, Vec<Box<Component>>
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Component {
    Module(Module),
    StructuredType(StructuredType),
    FreeFunction(FreeFunction),
    ImplBlock(ImplBlock),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DependencyEdgeKind {
    Inherits,
    Implements,
    UsesFieldType,
    UsesMethodParam,
    UsesMethodReturn,
    UsesFreeFunctionParam,
    UsesFreeFunctionReturn,
    ImplementsFor,
    ImplementsTrait,
    NestedIn,
    ModuleContainment,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    pub from: QualifiedName,
    pub to: QualifiedName,
    pub kind: DependencyEdgeKind,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<Component>,
    pub edges: Vec<Dependency>,
}

// Per i benchmark
#[derive(Debug, Default)]
pub struct AnalysisSummary {
    pub total_modules: usize,
    pub total_structured_types: usize,
    pub total_free_functions: usize,
    pub total_impl_blocks: usize,
    pub resolved_refs: usize,
    pub unknown_refs: usize,
}
