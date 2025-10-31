use crate::items::{Crate, Function, Global, Module, Struct, Trait, TypeAlias};

pub(super) trait HasPath {
    fn path(&self) -> String;
}

impl HasPath for Crate {
    fn path(&self) -> String {
        format!("{}/index.html", self.name)
    }
}

impl HasPath for Module {
    fn path(&self) -> String {
        format!("{}/index.html", self.name)
    }
}

impl HasPath for Struct {
    fn path(&self) -> String {
        format!("struct.{}.html", self.name)
    }
}

impl HasPath for Trait {
    fn path(&self) -> String {
        format!("trait.{}.html", self.name)
    }
}

impl HasPath for TypeAlias {
    fn path(&self) -> String {
        format!("type.{}.html", self.name)
    }
}

impl HasPath for Global {
    fn path(&self) -> String {
        format!("global.{}.html", self.name)
    }
}

impl HasPath for Function {
    fn path(&self) -> String {
        format!("fn.{}.html", self.name)
    }
}
