use std::fmt::Display;

use crate::ast::{Ident, UnresolvedGenerics, UnresolvedType};
use crate::token::SecondaryAttribute;

use iter_extended::vecmap;
use noirc_errors::Span;

use super::{Documented, ItemVisibility};

/// Ast node for an enum
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirEnumeration {
    pub name: Ident,
    pub attributes: Vec<SecondaryAttribute>,
    pub visibility: ItemVisibility,
    pub generics: UnresolvedGenerics,
    pub variants: Vec<Documented<EnumVariant>>,
    pub span: Span,
}

impl NoirEnumeration {
    pub fn is_abi(&self) -> bool {
        self.attributes.iter().any(|attr| attr.is_abi())
    }
}

/// We only support  variants of the form `Name(A, B, ...)` currently.
/// Enum variants like `Name { a: A, b: B, .. }` will be implemented later
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: Ident,
    pub parameters: Vec<UnresolvedType>,
}

impl Display for NoirEnumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        writeln!(f, "enum {}{} {{", self.name, generics)?;

        for variant in self.variants.iter() {
            let parameters = vecmap(&variant.item.parameters, ToString::to_string).join(", ");
            writeln!(f, "    {}({}),", variant.item.name, parameters)?;
        }

        write!(f, "}}")
    }
}
