use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Crates {
    pub name: String,
    pub crates: Vec<Crate>,
}

#[derive(Serialize, Deserialize)]
pub struct Crate {
    pub name: String,
    pub root_module: Module,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Item {
    Module(Module),
    Struct(Struct),
    Trait(Trait),
    TypeAlias(TypeAlias),
    Function(Function),
    Global(Global),
}

#[derive(Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Struct {
    pub id: usize,
    pub name: String,
    pub generics: Vec<Generic>,
    /// All `pub` fields of the struct.
    pub fields: Vec<StructField>,
    /// `true` if the struct has any private fields, besides the public ones listed in `fields`.
    pub has_private_fields: bool,
    pub impls: Vec<Impl>,
    pub trait_impls: Vec<TraitImpl>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub r#type: Type,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Impl {
    pub generics: Vec<Generic>,
    pub r#type: Type,
    pub methods: Vec<Function>,
}

#[derive(Serialize, Deserialize)]
pub struct TraitImpl {
    pub generics: Vec<Generic>,
    pub trait_id: usize,
    pub trait_name: String,
    pub trait_generics: Vec<Type>,
    pub r#type: Type,
    pub where_clause: Vec<TraitConstraint>,
    pub methods: Vec<Function>,
}

#[derive(Serialize, Deserialize)]
pub struct Global {
    pub name: String,
    pub comptime: bool,
    pub mutable: bool,
    pub r#type: Type,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Function {
    pub unconstrained: bool,
    pub comptime: bool,
    pub name: String,
    pub generics: Vec<Generic>,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub where_clause: Vec<TraitConstraint>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub r#type: Type,
}

#[derive(Serialize, Deserialize)]
pub struct Trait {
    pub id: usize,
    pub name: String,
    pub generics: Vec<Generic>,
    pub bounds: Vec<TraitBound>,
    pub where_clause: Vec<TraitConstraint>,
    pub methods: Vec<Function>,
    pub trait_impls: Vec<TraitImpl>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TypeAlias {
    pub id: usize,
    pub name: String,
    pub generics: Vec<Generic>,
    pub r#type: Type,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Generic {
    pub name: String,
    pub numeric: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TraitConstraint {
    pub r#type: Type,
    pub bound: TraitBound,
}

#[derive(Serialize, Deserialize)]
pub struct TraitBound {
    pub trait_id: usize,
    pub trait_name: String,
    pub ordered_generics: Vec<Type>,
    pub named_generics: BTreeMap<String, Type>,
}

#[derive(Serialize, Deserialize)]
pub enum Type {
    Unit,
    Primitive(String),
    Array {
        length: Box<Type>,
        element: Box<Type>,
    },
    Slice {
        element: Box<Type>,
    },
    String {
        length: Box<Type>,
    },
    FmtString {
        length: Box<Type>,
        element: Box<Type>,
    },
    Tuple(Vec<Type>),
    Reference {
        r#type: Box<Type>,
        mutable: bool,
    },
    Struct {
        id: usize,
        name: String,
        generics: Vec<Type>,
    },
    TypeAlias {
        id: usize,
        name: String,
        generics: Vec<Type>,
    },
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        env: Box<Type>,
        unconstrained: bool,
    },
    Constant(String),
    Generic(String),
    InfixExpr {
        lhs: Box<Type>,
        operator: String,
        rhs: Box<Type>,
    },
    TraitAsType {
        trait_id: usize,
        trait_name: String,
        ordered_generics: Vec<Type>,
        named_generics: BTreeMap<String, Type>,
    },
}
