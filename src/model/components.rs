use super::queries::TypeRef;

pub type Identifier = String;
pub type QualifiedName = Vec<Identifier>;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum StructuredTypeKind {
    Class,
    Struct,
    Interface,
    Trait,
    Enum,
    EnumVariant,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Field {
    pub name: Identifier,
    pub ty: TypeRef,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub annotations: Vec<TypeRef>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Parameter {
    pub name: Option<Identifier>,
    pub ty: TypeRef,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Signature {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TypeAlias {
    pub name: QualifiedName,
    pub target: TypeRef,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Block {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub declarations: Vec<Field>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub calls: Vec<TypeRef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub instantiates: Vec<TypeRef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accesses: Vec<TypeRef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub type_casts: Vec<TypeRef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sub_blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Function {
    pub name: QualifiedName,
    pub signature: Signature,
    pub body: Option<Block>,
    pub is_constructor: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub annotations: Vec<TypeRef>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImplBlock {
    pub name: QualifiedName,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub methods: Vec<Function>,
    pub impl_for: TypeRef,
    pub implements_trait: Option<TypeRef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub nested_types: Vec<StructuredType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub type_aliases: Vec<TypeAlias>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructuredType {
    pub name: QualifiedName,
    pub kind: StructuredTypeKind,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub methods: Vec<Function>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub super_types: Vec<TypeRef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub nested_types: Vec<StructuredType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub annotations: Vec<TypeRef>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Import {
    pub path: QualifiedName,
    pub alias: Option<Identifier>,
    pub is_wildcard: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Module {
    pub name: QualifiedName,
    pub language: Option<String>,
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub imports: Vec<Import>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sub_modules: Vec<Module>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub structured_types: Vec<StructuredType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub type_aliases: Vec<TypeAlias>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub free_functions: Vec<Function>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub impl_blocks: Vec<ImplBlock>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub free_variables: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Component {
    Module(Module),
    StructuredType(StructuredType),
    TypeAlias(TypeAlias),
    Function(Function),
    Field(QualifiedName, TypeRef),
    Primitive(String),
    External(QualifiedName),
}
