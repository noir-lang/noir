use crate::{Ident, UnresolvedType};
use noirc_errors::Span;
use std::fmt::Display;

/// Ast node for type aliases
#[derive(Clone, Debug)]
pub struct NoirTyAlias {
    pub name: Ident,
    pub ty: UnresolvedType,
    pub span: Span,
    // TODO: should probabaly allow generics
    // eg. type foo = Vec<T>;
    // pub generics: UnresolvedGenerics,
}

impl NoirTyAlias {
    pub fn new(name: Ident, ty: UnresolvedType, span: Span) -> NoirTyAlias {
        NoirTyAlias { name, ty, span }
    }
}

impl Display for NoirTyAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type {} = {}", self.name, self.ty)
    }
}
