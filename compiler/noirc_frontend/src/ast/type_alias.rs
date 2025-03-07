use super::{Ident, ItemVisibility, UnresolvedGenerics, UnresolvedType};
use iter_extended::vecmap;
use noirc_errors::Location;
use std::fmt::Display;

/// Ast node for type aliases
/// A Noir Type Alias can be an alias to a normal type, or to a numeric generic type.
#[derive(Clone, Debug)]
pub enum NoirTypeAlias {
    NormalTypeAlias(NormalTypeAlias),
    NumericTypeAlias(NumericTypeAlias),
}

impl Display for NoirTypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoirTypeAlias::NormalTypeAlias(alias) => write!(f, "{}", alias),
            NoirTypeAlias::NumericTypeAlias(alias) => write!(f, "{}", alias),
        }
    }
}
#[derive(Clone, Debug)]
pub struct NormalTypeAlias {
    pub name: Ident,
    pub generics: UnresolvedGenerics,
    pub typ: UnresolvedType,
    pub visibility: ItemVisibility,
    pub location: Location,
}

impl Display for NormalTypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        write!(f, "type {}<{}> = {}", self.name, generics.join(", "), self.typ)
    }
}

#[derive(Clone, Debug)]
pub struct NumericTypeAlias {
    pub type_alias: NormalTypeAlias,
    pub numeric_type: UnresolvedType,
}

impl Display for NumericTypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.type_alias.generics, |generic| generic.to_string());
        write!(
            f,
            "numeric ({}) type {}<{}> = {}",
            self.numeric_type,
            self.type_alias.name,
            generics.join(", "),
            self.type_alias.typ
        )
    }
}
impl NoirTypeAlias {
    pub fn name(&self) -> Ident {
        match self {
            NoirTypeAlias::NormalTypeAlias(alias) => alias.name.clone(),
            NoirTypeAlias::NumericTypeAlias(alias) => alias.type_alias.name.clone(),
        }
    }

    pub fn visibility(&self) -> ItemVisibility {
        match self {
            NoirTypeAlias::NormalTypeAlias(alias) => alias.visibility,
            NoirTypeAlias::NumericTypeAlias(alias) => alias.type_alias.visibility,
        }
    }

    pub fn generics(&self) -> &UnresolvedGenerics {
        match self {
            NoirTypeAlias::NormalTypeAlias(alias) => &alias.generics,
            NoirTypeAlias::NumericTypeAlias(alias) => &alias.type_alias.generics,
        }
    }

    pub fn location(&self) -> Location {
        match self {
            NoirTypeAlias::NormalTypeAlias(alias) => alias.location,
            NoirTypeAlias::NumericTypeAlias(alias) => alias.type_alias.location,
        }
    }

    pub fn type_alias(&self) -> &NormalTypeAlias {
        match self {
            NoirTypeAlias::NormalTypeAlias(alias) => alias,
            NoirTypeAlias::NumericTypeAlias(alias) => &alias.type_alias,
        }
    }
}
