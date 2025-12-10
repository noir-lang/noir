use super::{Ident, ItemVisibility, UnresolvedGenerics, UnresolvedType};
use iter_extended::vecmap;
use noirc_errors::Location;
use std::fmt::Display;

/// Ast node for type aliases
/// Depending on 'numeric_type', a Type Alias can be an alias to a normal type, or to a numeric generic type
#[derive(Clone, Debug)]
pub struct TypeAlias {
    pub name: Ident,
    pub generics: UnresolvedGenerics,
    pub typ: UnresolvedType,
    pub visibility: ItemVisibility,
    pub location: Location,
    pub numeric_type: Option<UnresolvedType>,
    pub numeric_location: Location,
}

impl Display for TypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        write!(f, "type {}<{}> = {}", self.name, generics.join(", "), self.typ)
    }
}
