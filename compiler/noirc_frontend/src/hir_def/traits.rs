use std::collections::HashMap;

use crate::{
    graph::CrateId,
    node_interner::{FuncId, TraitId, TraitMethodId},
    Generics, Ident, NoirFunction, Type, TypeBindings, TypeVariable, TypeVariableId,
};
use fm::FileId;
use noirc_errors::{Location, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitFunction {
    pub name: Ident,
    pub typ: Type,
    pub location: Location,
    pub default_impl: Option<Box<NoirFunction>>,
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
#[derive(Debug, Eq)]
pub struct Trait {
    /// A unique id representing this trait type. Used to check if two
    /// struct traits are equal.
    pub id: TraitId,

    pub crate_id: CrateId,

    pub methods: Vec<TraitFunction>,

    /// Maps method_name -> method id.
    /// This map is separate from methods since TraitFunction ids
    /// are created during collection where we don't yet have all
    /// the information needed to create the full TraitFunction.
    pub method_ids: HashMap<String, FuncId>,

    pub constants: Vec<TraitConstant>,
    pub types: Vec<TraitType>,

    pub name: Ident,
    pub generics: Generics,
    pub location: Location,

    /// When resolving the types of Trait elements, all references to `Self` resolve
    /// to this TypeVariable. Then when we check if the types of trait impl elements
    /// match the definition in the trait, we bind this TypeVariable to whatever
    /// the correct Self type is for that particular impl block.
    pub self_type_typevar_id: TypeVariableId,
    pub self_type_typevar: TypeVariable,
}

#[derive(Debug)]
pub struct TraitImpl {
    pub ident: Ident,
    pub typ: Type,
    pub trait_id: TraitId,
    pub trait_generics: Vec<Type>,
    pub file: FileId,
    pub methods: Vec<FuncId>, // methods[i] is the implementation of trait.methods[i] for Type typ

    /// The where clause, if present, contains each trait requirement which must
    /// be satisfied for this impl to be selected. E.g. in `impl Eq for [T] where T: Eq`,
    /// `where_clause` would contain the one `T: Eq` constraint. If there is no where clause,
    /// this Vec is empty.
    pub where_clause: Vec<TraitConstraint>,
}

#[derive(Debug, Clone)]
pub struct TraitConstraint {
    pub typ: Type,
    pub trait_id: TraitId,
    pub trait_generics: Vec<Type>,
}

impl TraitConstraint {
    pub fn new(typ: Type, trait_id: TraitId, trait_generics: Vec<Type>) -> Self {
        Self { typ, trait_id, trait_generics }
    }

    pub fn apply_bindings(&mut self, type_bindings: &TypeBindings) {
        self.typ = self.typ.substitute(type_bindings);

        for typ in &mut self.trait_generics {
            *typ = typ.substitute(type_bindings);
        }
    }
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
    pub fn set_methods(&mut self, methods: Vec<TraitFunction>) {
        self.methods = methods;
    }

    pub fn find_method(&self, name: &str) -> Option<TraitMethodId> {
        for (idx, method) in self.methods.iter().enumerate() {
            if &method.name == name {
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
    pub fn arguments(&self) -> &[Type] {
        match &self.typ {
            Type::Function(args, _, _) => args,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(args, _, _) => args,
                _ => unreachable!("Trait function does not have a function type"),
            },
            _ => unreachable!("Trait function does not have a function type"),
        }
    }

    pub fn generics(&self) -> &[TypeVariable] {
        match &self.typ {
            Type::Function(..) => &[],
            Type::Forall(generics, _) => generics,
            _ => unreachable!("Trait function does not have a function type"),
        }
    }

    pub fn return_type(&self) -> &Type {
        match &self.typ {
            Type::Function(_, return_type, _) => return_type,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(_, return_type, _) => return_type,
                _ => unreachable!("Trait function does not have a function type"),
            },
            _ => unreachable!("Trait function does not have a function type"),
        }
    }
}
