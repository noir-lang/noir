use crate::{Ident, UnresolvedGenerics, UnresolvedType};
use iter_extended::vecmap;
use noirc_errors::Span;
use std::fmt::Display;

/// Ast node for type aliases
#[derive(Clone, Debug)]
pub struct NoirTyAlias {
    pub name: Ident,
    pub generics: UnresolvedGenerics,
    pub ty: UnresolvedType,
    pub span: Span,
}

impl NoirTyAlias {
    pub fn new(
        name: Ident,
        generics: UnresolvedGenerics,
        ty: UnresolvedType,
        span: Span,
    ) -> NoirTyAlias {
        NoirTyAlias { name, generics, ty, span }
    }
}

impl Display for NoirTyAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        write!(f, "type {}<{}> = {}", self.name, generics, self.ty)
    }
}
