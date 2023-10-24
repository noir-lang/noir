use crate::{
    graph::CrateId,
    node_interner::{FuncId, TraitId, TraitMethodId},
    Generics, Ident, NoirFunction, Type, TypeVariable, TypeVariableId,
};
use noirc_errors::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitFunction {
    pub name: Ident,
    pub generics: Generics,
    pub arguments: Vec<Type>,
    pub return_type: Type,
    pub span: Span,
    pub default_impl: Option<Box<NoirFunction>>,
    pub default_impl_file_id: fm::FileId,
    pub default_impl_module_id: crate::hir::def_map::LocalModuleId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitConstant {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitType {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

/// Represents a trait in the type system. Each instance of this struct
/// will be shared across all Type::Trait variants that represent
/// the same trait.
#[derive(Debug, Eq, Clone)]
pub struct Trait {
    /// A unique id representing this trait type. Used to check if two
    /// struct traits are equal.
    pub id: TraitId,

    pub crate_id: CrateId,

    pub methods: Vec<TraitFunction>,
    pub constants: Vec<TraitConstant>,
    pub types: Vec<TraitType>,

    pub name: Ident,
    pub generics: Generics,
    pub span: Span,

    /// When resolving the types of Trait elements, all references to `Self` resolve
    /// to this TypeVariable. Then when we check if the types of trait impl elements
    /// match the definition in the trait, we bind this TypeVariable to whatever
    /// the correct Self type is for that particular impl block.
    pub self_type_typevar_id: TypeVariableId,
    pub self_type_typevar: TypeVariable,
}

pub struct TraitImpl {
    pub ident: Ident,
    pub typ: Type,
    pub trait_id: TraitId,
    pub methods: Vec<FuncId>, // methods[i] is the implementation of trait.methods[i] for Type typ
}

#[derive(Debug, Clone)]
pub struct TraitConstraint {
    pub typ: Type,
    pub trait_id: TraitId,
    // pub trait_generics: Generics, TODO
}

impl std::hash::Hash for Trait {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Trait {
    pub fn new(
        id: TraitId,
        name: Ident,
        crate_id: CrateId,
        span: Span,
        generics: Generics,
        self_type_typevar_id: TypeVariableId,
        self_type_typevar: TypeVariable,
    ) -> Trait {
        Trait {
            id,
            name,
            crate_id,
            span,
            methods: Vec::new(),
            constants: Vec::new(),
            types: Vec::new(),
            generics,
            self_type_typevar_id,
            self_type_typevar,
        }
    }

    pub fn set_methods(&mut self, methods: Vec<TraitFunction>) {
        self.methods = methods;
    }

    pub fn find_method(&self, name: Ident) -> Option<TraitMethodId> {
        for (idx, method) in self.methods.iter().enumerate() {
            if method.name == name {
                return Some(TraitMethodId { trait_id: self.id, method_index: idx });
            }
        }
        None
    }
}

impl std::fmt::Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TraitFunction {
    pub fn get_type(&self) -> Type {
        Type::Function(
            self.arguments.clone(),
            Box::new(self.return_type.clone()),
            Box::new(Type::Unit),
        )
    }
}
