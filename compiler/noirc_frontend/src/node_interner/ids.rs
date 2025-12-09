use std::fmt;

use noirc_arena::Index;
use noirc_errors::Location;

use crate::{
    graph::CrateId,
    hir::def_map::{DefMaps, LocalModuleId, ModuleId},
    node_interner::globals::GlobalId,
};

/// A reference to a module, struct, trait, etc., mainly used by the LSP code
/// to keep track of how symbols reference each other.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ReferenceId {
    Module(ModuleId),
    Type(TypeId),
    StructMember(TypeId, usize),
    EnumVariant(TypeId, usize),
    Trait(TraitId),
    TraitAssociatedType(TraitAssociatedTypeId),
    Global(GlobalId),
    Function(FuncId),
    Alias(TypeAliasId),
    Local(DefinitionId),
    Reference(Location, bool /* is Self */),
}

impl ReferenceId {
    pub fn is_self_type_name(&self) -> bool {
        matches!(self, Self::Reference(_, true))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DefinitionId(pub(super) usize);

impl DefinitionId {
    //dummy id for error reporting
    pub fn dummy_id() -> DefinitionId {
        DefinitionId(usize::MAX)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct StmtId(pub(super) Index);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct ExprId(pub(super) Index);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct FuncId(pub(super) Index);

impl fmt::Display for FuncId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A [TypeId] wraps a [ModuleId], because types are represented by an anonymous module.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct TypeId(pub(super) ModuleId);

impl TypeId {
    pub fn module_id(self) -> ModuleId {
        self.0
    }

    pub fn krate(self) -> CrateId {
        self.0.krate
    }

    pub fn local_module_id(self) -> LocalModuleId {
        self.0.local_id
    }

    /// Returns the module where this struct is defined.
    pub fn parent_module_id(self, def_maps: &DefMaps) -> ModuleId {
        self.module_id().parent(def_maps).expect("Expected struct module parent to exist")
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct TypeAliasId(pub usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitId(pub ModuleId);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct TraitAssociatedTypeId(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TraitImplId(pub usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TraitItemId {
    pub trait_id: TraitId,
    /// This is the definition id of the method or associated constant in the trait, not an impl
    pub item_id: DefinitionId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuotedTypeId(pub(super) Index);

/// The ID of an [crate::ast::ExpressionKind] that's been pushed into the [NodeInterner][crate::node_interner::NodeInterner].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedExpressionKind(pub(super) Index);

/// The ID of a [crate::ast::StatementKind] that's been pushed into the [NodeInterner][crate::node_interner::NodeInterner].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedStatementKind(pub(super) Index);

/// The ID of a [crate::ast::UnresolvedTypeData] that's been pushed into the [NodeInterner][crate::node_interner::NodeInterner].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedUnresolvedTypeData(pub(super) Index);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedPattern(pub(super) Index);

macro_rules! into_index {
    ($id_type:ty) => {
        impl From<$id_type> for Index {
            fn from(t: $id_type) -> Self {
                t.0
            }
        }

        impl From<&$id_type> for Index {
            fn from(t: &$id_type) -> Self {
                t.0
            }
        }
    };
}

into_index!(ExprId);
into_index!(StmtId);
