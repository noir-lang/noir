use std::fmt::Display;

use crate::ast::{Ident, UnresolvedGenerics, UnresolvedType};
use crate::token::SecondaryAttribute;

use iter_extended::vecmap;
use noirc_errors::Span;

use super::Documented;

/// Ast node for a struct
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirStruct {
    pub name: Ident,
    pub attributes: Vec<SecondaryAttribute>,
    pub generics: UnresolvedGenerics,
    pub fields: Vec<Documented<StructField>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructField {
    pub name: Ident,
    pub typ: UnresolvedType,
}

impl Display for NoirStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        writeln!(f, "struct {}{} {{", self.name, generics)?;

        for field in self.fields.iter() {
            writeln!(f, "    {}: {},", field.item.name, field.item.typ)?;
        }

        write!(f, "}}")
    }
}
