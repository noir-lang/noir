//! Contains the data structures that represent items to be shown by the documentation generator.

use std::collections::BTreeMap;

use noirc_errors::Location;
use noirc_frontend::{ast::ItemVisibility, hir::def_map::ModuleId};

pub trait ItemProperties {
    fn name(&self) -> String;
    fn comments(&self) -> Option<&Comments>;
    fn is_deprecated(&self) -> bool;
}

/// Uniquely identifies an item.
/// This is done by using a type's name, location in source code and kind.
/// With macros, two types might end up being defined in the same location but they will likely
/// have different names.
/// This is just a temporary solution until we have a better way to uniquely identify items
/// across crates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId {
    pub location: Location,
    pub kind: ItemKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Module,
    Struct,
    Trait,
    TypeAlias,
    Function,
    Global,
}

/// A markdown link that resolves to an item, in one of these forms:
/// - `[name]` (`path` will be the same as `name`)
/// - `[name][path]`
/// - `[name](path)`
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Link {
    pub name: String,
    pub path: String,
    pub target: LinkTarget,
    /// The line number in the comments where this link occurs (0-based).
    pub line: usize,
    /// The start byte in the line where the link occurs.
    pub start: usize,
    /// The end byte in the line where the link occurs.
    pub end: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    TopLevelItem(ItemId),
    Method(ItemId, String),
    StructMember(ItemId, String),
    PrimitiveType(PrimitiveTypeKind),
    PrimitiveTypeFunction(PrimitiveTypeKind, String),
}

impl LinkTarget {
    pub fn id(&self) -> Option<&ItemId> {
        match self {
            Self::TopLevelItem(id) | Self::Method(id, _) | Self::StructMember(id, _) => Some(id),
            Self::PrimitiveType(_) | Self::PrimitiveTypeFunction(..) => None,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::TopLevelItem(_) | Self::PrimitiveType(_) => None,
            Self::Method(_, name)
            | Self::StructMember(_, name)
            | Self::PrimitiveTypeFunction(_, name) => Some(name),
        }
    }

    pub fn primitive_type(&self) -> Option<PrimitiveTypeKind> {
        match self {
            Self::TopLevelItem(..) | Self::Method(..) | Self::StructMember(..) => None,
            Self::PrimitiveType(primitive_type_kind)
            | Self::PrimitiveTypeFunction(primitive_type_kind, _) => Some(*primitive_type_kind),
        }
    }
}

pub type Links = Vec<Link>;

pub type Comments = (String, Links);

#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Crate {
    pub name: String,
    pub version: Option<String>,
    pub root_module: Module,
    pub root_file: String,
}

impl ItemProperties for Crate {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.root_module.comments()
    }

    fn is_deprecated(&self) -> bool {
        self.root_module.is_deprecated()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Module(Module),
    Struct(Struct),
    Trait(Trait),
    TypeAlias(TypeAlias),
    Function(Function),
    Global(Global),
    PrimitiveType(PrimitiveType),
    Reexport(Reexport),
}

impl Item {
    pub fn set_name(&mut self, new_name: String) {
        match self {
            Item::Module(module) => module.name = new_name,
            Item::Struct(struct_) => struct_.name = new_name,
            Item::Trait(trait_) => trait_.name = new_name,
            Item::TypeAlias(type_alias) => type_alias.name = new_name,
            Item::Function(function) => function.name = new_name,
            Item::Global(global) => global.name = new_name,
            Item::PrimitiveType(_) => {}
            Item::Reexport(reexport) => reexport.name = new_name,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Module {
    pub id: ItemId,
    pub module_id: ModuleId,
    pub name: String,
    pub items: Vec<(ItemVisibility, Item)>,
    pub comments: Option<Comments>,
    pub is_contract: bool,
}

impl Module {
    pub fn has_public_items(&self) -> bool {
        self.items.iter().any(|(visibility, _)| *visibility == ItemVisibility::Public)
    }
}

impl ItemProperties for Module {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Struct {
    pub id: ItemId,
    pub name: String,
    pub generics: Vec<Generic>,
    /// All `pub` fields of the struct.
    pub fields: Vec<StructField>,
    /// `true` if the struct has any private fields, besides the public ones listed in `fields`.
    pub has_private_fields: bool,
    pub impls: Vec<Impl>,
    pub trait_impls: Vec<TraitImpl>,
    pub comments: Option<Comments>,
}

impl ItemProperties for Struct {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StructField {
    pub name: String,
    pub r#type: Type,
    pub comments: Option<Comments>,
}

impl ItemProperties for StructField {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Impl {
    pub generics: Vec<Generic>,
    pub r#type: Type,
    pub methods: Vec<Function>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TraitImpl {
    pub generics: Vec<Generic>,
    pub trait_id: ItemId,
    pub trait_name: String,
    pub trait_generics: Vec<Type>,
    pub r#type: Type,
    pub where_clause: Vec<TraitConstraint>,
    pub methods: Vec<Function>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Global {
    pub id: ItemId,
    pub name: String,
    pub comptime: bool,
    pub mutable: bool,
    pub r#type: Type,
    pub comments: Option<Comments>,
}

impl ItemProperties for Global {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub id: ItemId,
    pub unconstrained: bool,
    pub comptime: bool,
    pub name: String,
    pub generics: Vec<Generic>,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub where_clause: Vec<TraitConstraint>,
    pub comments: Option<Comments>,
    pub deprecated: Option<Option<String>>,
}

impl ItemProperties for Function {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        self.deprecated.is_some()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FunctionParam {
    pub name: String,
    pub r#type: Type,
    pub mut_ref: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Trait {
    pub id: ItemId,
    pub name: String,
    pub generics: Vec<Generic>,
    pub bounds: Vec<TraitBound>,
    pub where_clause: Vec<TraitConstraint>,
    pub associated_types: Vec<AssociatedType>,
    pub associated_constants: Vec<AssociatedConstant>,
    pub required_methods: Vec<Function>,
    pub provided_methods: Vec<Function>,
    pub trait_impls: Vec<TraitImpl>,
    pub comments: Option<Comments>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AssociatedType {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AssociatedConstant {
    pub name: String,
    pub r#type: Type,
}

impl ItemProperties for Trait {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypeAlias {
    pub id: ItemId,
    pub name: String,
    pub generics: Vec<Generic>,
    pub r#type: Type,
    pub comments: Option<Comments>,
}

impl ItemProperties for TypeAlias {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Generic {
    pub name: String,
    pub numeric: Option<Type>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TraitConstraint {
    pub r#type: Type,
    pub bound: TraitBound,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TraitBound {
    pub trait_id: ItemId,
    pub trait_name: String,
    pub ordered_generics: Vec<Type>,
    pub named_generics: BTreeMap<String, Type>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
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
        id: ItemId,
        name: String,
        generics: Vec<Type>,
    },
    TypeAlias {
        id: ItemId,
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
        trait_id: ItemId,
        trait_name: String,
        ordered_generics: Vec<Type>,
        named_generics: BTreeMap<String, Type>,
    },
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrimitiveType {
    pub kind: PrimitiveTypeKind,
    pub impls: Vec<Impl>,
    pub trait_impls: Vec<TraitImpl>,
    pub comments: Option<Comments>,
}

impl ItemProperties for PrimitiveType {
    fn name(&self) -> String {
        self.kind.to_string()
    }

    fn comments(&self) -> Option<&Comments> {
        self.comments.as_ref()
    }

    fn is_deprecated(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Reexport {
    pub id: ItemId,
    pub item_name: String,
    pub name: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
    Field,
    Str,
    Fmtstr,
    Array,
    Slice,
    Expr,
    Quoted,
    Type,
    TypedExpr,
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
            PrimitiveTypeKind::Field => "Field",
            PrimitiveTypeKind::Str => "str",
            PrimitiveTypeKind::Fmtstr => "fmtstr",
            PrimitiveTypeKind::Array => "array",
            PrimitiveTypeKind::Slice => "slice",
            PrimitiveTypeKind::Expr => "Expr",
            PrimitiveTypeKind::Quoted => "Quoted",
            PrimitiveTypeKind::Type => "Type",
            PrimitiveTypeKind::TypedExpr => "TypedExpr",
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
