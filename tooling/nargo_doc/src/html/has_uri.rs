use crate::items::{
    Crate, Function, Global, Module, PrimitiveType, PrimitiveTypeKind, Struct, Trait, TypeAlias,
};

/// A trait that associates an HTML uri with an item.
/// The uri is relative to the page where the item is typically shown
/// (for example, a crate is shown in the workspace, a struct is shown in its module, etc.).
pub(super) trait HasUri {
    fn uri(&self) -> String;
}

impl HasUri for Crate {
    fn uri(&self) -> String {
        format!("{}/index.html", self.name)
    }
}

impl HasUri for Module {
    fn uri(&self) -> String {
        format!("{}/index.html", self.name)
    }
}

impl HasUri for Struct {
    fn uri(&self) -> String {
        format!("struct.{}.html", self.name)
    }
}

impl HasUri for Trait {
    fn uri(&self) -> String {
        format!("trait.{}.html", self.name)
    }
}

impl HasUri for TypeAlias {
    fn uri(&self) -> String {
        format!("type.{}.html", self.name)
    }
}

impl HasUri for Global {
    fn uri(&self) -> String {
        format!("global.{}.html", self.name)
    }
}

impl HasUri for Function {
    fn uri(&self) -> String {
        format!("fn.{}.html", self.name)
    }
}

impl HasUri for PrimitiveType {
    fn uri(&self) -> String {
        self.kind.uri()
    }
}

impl HasUri for PrimitiveTypeKind {
    fn uri(&self) -> String {
        format!("primitive.{self}.html")
    }
}
