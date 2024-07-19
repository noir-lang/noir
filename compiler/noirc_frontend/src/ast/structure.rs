use std::fmt::Display;

use crate::ast::{Ident, UnresolvedGenerics, UnresolvedType};
use crate::token::SecondaryAttribute;

use iter_extended::vecmap;
use noirc_errors::Span;

/// Ast node for a struct
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirStruct {
    pub name: Ident,
    pub attributes: Vec<SecondaryAttribute>,
    pub generics: UnresolvedGenerics,
    pub fields: Vec<(Ident, UnresolvedType)>,
    pub span: Span,
    pub is_comptime: bool,
}

impl Display for NoirStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        writeln!(f, "struct {}{} {{", self.name, generics)?;

        for (name, typ) in self.fields.iter() {
            writeln!(f, "    {name}: {typ},")?;
        }

        write!(f, "}}")
    }
}
