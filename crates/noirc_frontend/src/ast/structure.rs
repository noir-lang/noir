use std::fmt::Display;

use crate::{Ident, UnresolvedGenerics, UnresolvedType};
use iter_extended::vecmap;
use noirc_errors::Span;

/// Ast node for a struct
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirStruct {
    pub name: Ident,
    pub generics: UnresolvedGenerics,
    pub fields: Vec<(Ident, UnresolvedType)>,
    pub span: Span,
}

impl NoirStruct {
    pub fn new(
        name: Ident,
        generics: Vec<Ident>,
        fields: Vec<(Ident, UnresolvedType)>,
        span: Span,
    ) -> NoirStruct {
        NoirStruct { name, generics, fields, span }
    }
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
