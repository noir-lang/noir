//! Contains the data structures that represent items to be shown by the documentation generator.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub trait HasNameAndComments {
    fn name(&self) -> String;
    fn comments(&self) -> Option<&str>;
}

pub type TypeId = usize;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    /// Crates directly defined in this workspace.
    pub crates: Vec<Crate>,
    /// All unique dependencies of `crates`.
    pub dependencies: Vec<Crate>,
}

impl Workspace {
    pub fn all_crates(&self) -> impl Iterator<Item = &Crate> {
        self.crates.iter().chain(self.dependencies.iter())
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Crate {
    pub name: String,
    pub root_module: Module,
    pub root_file: String,
}

impl HasNameAndComments for Crate {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.root_module.comments()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Item {
    Module(Module),
    Struct(Struct),
    Trait(Trait),
    TypeAlias(TypeAlias),
    Function(Function),
    Global(Global),
    PrimitiveType(PrimitiveType),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub comments: Option<String>,
}

impl HasNameAndComments for Module {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Struct {
    pub id: TypeId,
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

impl HasNameAndComments for Struct {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub r#type: Type,
    pub comments: Option<String>,
}

impl HasNameAndComments for StructField {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Impl {
    pub generics: Vec<Generic>,
    pub r#type: Type,
    pub methods: Vec<Function>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitImpl {
    pub generics: Vec<Generic>,
    pub trait_id: TypeId,
    pub trait_name: String,
    pub trait_generics: Vec<Type>,
    pub r#type: Type,
    pub where_clause: Vec<TraitConstraint>,
    pub methods: Vec<Function>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Global {
    pub name: String,
    pub comptime: bool,
    pub mutable: bool,
    pub r#type: Type,
    pub comments: Option<String>,
}

impl HasNameAndComments for Global {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl HasNameAndComments for Function {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub r#type: Type,
    pub mut_ref: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Trait {
    pub id: TypeId,
    pub name: String,
    pub generics: Vec<Generic>,
    pub bounds: Vec<TraitBound>,
    pub where_clause: Vec<TraitConstraint>,
    pub associated_types: Vec<AssociatedType>,
    pub associated_constants: Vec<AssociatedConstant>,
    pub required_methods: Vec<Function>,
    pub provided_methods: Vec<Function>,
    pub trait_impls: Vec<TraitImpl>,
    pub comments: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedType {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedConstant {
    pub name: String,
    pub r#type: Type,
}

impl HasNameAndComments for Trait {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeAlias {
    pub id: TypeId,
    pub name: String,
    pub generics: Vec<Generic>,
    pub r#type: Type,
    pub comments: Option<String>,
}

impl HasNameAndComments for TypeAlias {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&str> {
        self.comments.as_deref()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generic {
    pub name: String,
    pub numeric: Option<Type>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitConstraint {
    pub r#type: Type,
    pub bound: TraitBound,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitBound {
    pub trait_id: TypeId,
    pub trait_name: String,
    pub ordered_generics: Vec<Type>,
    pub named_generics: BTreeMap<String, Type>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Unit,
    Primitive(PrimitiveTypeKind),
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
        id: TypeId,
        name: String,
        generics: Vec<Type>,
    },
    TypeAlias {
        id: TypeId,
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
        trait_id: TypeId,
        trait_name: String,
        ordered_generics: Vec<Type>,
        named_generics: BTreeMap<String, Type>,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimitiveType {
    pub kind: PrimitiveTypeKind,
    pub impls: Vec<Impl>,
    pub trait_impls: Vec<TraitImpl>,
}

impl HasNameAndComments for PrimitiveType {
    fn name(&self) -> String {
        self.kind.to_string()
    }

    fn comments(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveTypeKind {
    Bool,
    U1,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Field,
    Str,
    Fmtstr,
    Array,
    Slice,
    Expr,
    Quoted,
    TopLevelItem,
    Type,
    TypeExpr,
    TypeDefinition,
    TraitConstraint,
    TraitDefinition,
    TraitImpl,
    UnresolvedType,
    FunctionDefinition,
    Module,
    CtString,
}

impl std::fmt::Display for PrimitiveTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PrimitiveTypeKind::Bool => "bool",
            PrimitiveTypeKind::U1 => "u1",
            PrimitiveTypeKind::U8 => "u8",
            PrimitiveTypeKind::U16 => "u16",
            PrimitiveTypeKind::U32 => "u32",
            PrimitiveTypeKind::U64 => "u64",
            PrimitiveTypeKind::U128 => "u128",
            PrimitiveTypeKind::I8 => "i8",
            PrimitiveTypeKind::I16 => "i16",
            PrimitiveTypeKind::I32 => "i32",
            PrimitiveTypeKind::I64 => "i64",
            PrimitiveTypeKind::I128 => "i128",
            PrimitiveTypeKind::Field => "Field",
            PrimitiveTypeKind::Str => "str",
            PrimitiveTypeKind::Fmtstr => "fmtstr",
            PrimitiveTypeKind::Array => "array",
            PrimitiveTypeKind::Slice => "slice",
            PrimitiveTypeKind::Expr => "Expr",
            PrimitiveTypeKind::Quoted => "Quoted",
            PrimitiveTypeKind::TopLevelItem => "TopLevelItem",
            PrimitiveTypeKind::Type => "Type",
            PrimitiveTypeKind::TypeExpr => "TypeExpr",
            PrimitiveTypeKind::TypeDefinition => "TypeDefinition",
            PrimitiveTypeKind::TraitConstraint => "TraitConstraint",
            PrimitiveTypeKind::TraitDefinition => "TraitDefinition",
            PrimitiveTypeKind::TraitImpl => "TraitImpl",
            PrimitiveTypeKind::UnresolvedType => "UnresolvedType",
            PrimitiveTypeKind::FunctionDefinition => "FunctionDefinition",
            PrimitiveTypeKind::Module => "Module",
            PrimitiveTypeKind::CtString => "CtString",
        };
        write!(f, "{name}")
    }
}
