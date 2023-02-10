use std::fmt::Display;

use crate::{Ident, NoirFunction, UnresolvedGenerics, UnresolvedType};
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

/// Ast node for an impl
#[derive(Clone, Debug)]
pub struct NoirImpl {
    pub object_type: UnresolvedType,
    pub type_span: Span,
    pub generics: UnresolvedGenerics,
    pub methods: Vec<NoirFunction>,
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

impl Display for NoirImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        writeln!(f, "impl{} {} {{", generics, self.object_type)?;

        for method in self.methods.iter() {
            let method = method.to_string();
            for line in method.lines() {
                writeln!(f, "    {line}")?;
            }
        }

        write!(f, "}}")
    }
}
