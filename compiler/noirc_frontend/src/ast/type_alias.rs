use super::{Ident, ItemVisibility, UnresolvedGenerics, UnresolvedType};
use iter_extended::vecmap;
use noirc_errors::Span;
use std::fmt::Display;

/// Ast node for type aliases
#[derive(Clone, Debug)]
pub struct NoirTypeAlias {
    pub name: Ident,
    pub generics: UnresolvedGenerics,
    pub typ: UnresolvedType,
    pub visibility: ItemVisibility,
    pub span: Span,
}

impl NoirTypeAlias {
    pub fn new(
        name: Ident,
        generics: UnresolvedGenerics,
        typ: UnresolvedType,
        visibility: ItemVisibility,
        span: Span,
    ) -> NoirTypeAlias {
        NoirTypeAlias { name, generics, typ, visibility, span }
    }
}

impl Display for NoirTypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        write!(f, "type {}<{}> = {}", self.name, generics.join(", "), self.typ)
    }
}
