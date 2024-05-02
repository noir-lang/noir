use rustc_hash::FxHashMap as HashMap;

use crate::{macros_api::StructId, node_interner::{TypeAliasId, DefinitionId}};
use crate::hir::comptime::Value;

#[derive(Default)]
pub(super) struct Scope {
    types: HashMap<String, TypeId>,
    values: HashMap<String, DefinitionId>,
    comptime_values: HashMap<String, Value>,
}

pub(super) enum TypeId {
    Struct(StructId),
    Alias(TypeAliasId),
}
