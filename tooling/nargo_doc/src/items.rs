use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Crates {
    pub crates: Vec<Crate>,
}

#[derive(Serialize, Deserialize)]
pub struct Crate {
    pub name: String,
    pub items: Vec<Item>,
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
    pub name: Option<String>,
    pub items: Vec<Item>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Struct {
    pub id: usize,
    pub name: String,
    pub generics: Vec<Generic>,
    pub fields: Vec<StructField>,
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
    pub r#type: Type,
    pub generics: Vec<Generic>,
    pub methods: Vec<Function>,
    pub trait_id: usize,
    pub trait_name: String,
    pub trait_generics: Vec<Type>,
    pub where_clause: Vec<TraitConstraint>,
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
    pub trait_id: usize,
    pub trait_name: String,
    pub ordered_generics: Vec<Type>,
    pub named_generics: BTreeMap<String, Type>,
}

// TODO: use an enum
#[derive(Serialize, Deserialize)]
pub struct Type {
    pub name: String,
}
