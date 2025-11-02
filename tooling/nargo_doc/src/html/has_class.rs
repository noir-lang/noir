use crate::items::{Crate, Function, Global, Module, PrimitiveType, Struct, Trait, TypeAlias};

/// A trait that associated a CSS class name with an item.
pub(super) trait HasClass {
    fn class(&self) -> &'static str;
}

impl HasClass for Crate {
    fn class(&self) -> &'static str {
        "crate"
    }
}

impl HasClass for Module {
    fn class(&self) -> &'static str {
        "module"
    }
}

impl HasClass for Struct {
    fn class(&self) -> &'static str {
        "struct"
    }
}

impl HasClass for Trait {
    fn class(&self) -> &'static str {
        "trait"
    }
}

impl HasClass for TypeAlias {
    fn class(&self) -> &'static str {
        "type"
    }
}

impl HasClass for Global {
    fn class(&self) -> &'static str {
        "global"
    }
}

impl HasClass for Function {
    fn class(&self) -> &'static str {
        "fn"
    }
}

impl HasClass for PrimitiveType {
    fn class(&self) -> &'static str {
        "primitive"
    }
}
