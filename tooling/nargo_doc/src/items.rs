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
    pub generics: Vec<String>,
    pub fields: Vec<StructField>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub r#type: Type,
    pub comments: Option<String>,
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
    pub generics: Vec<String>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Trait {
    pub id: usize,
    pub name: String,
    pub generics: Vec<String>,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TypeAlias {
    pub id: usize,
    pub name: String,
    pub generics: Vec<String>,
    pub r#type: Type,
    pub comments: Option<String>,
}

// TODO: use an enum
#[derive(Serialize, Deserialize)]
pub struct Type {
    pub name: String,
}
