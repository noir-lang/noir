use std::fmt::Display;

use crate::{Ident, NoirFunction, Path, UnresolvedType};
use noirc_errors::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirStruct {
    pub name: Ident,
    pub generics: Vec<Ident>,
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

#[derive(Clone, Debug)]
pub struct NoirImpl {
    pub type_path: Path,
    pub generics: Vec<Ident>,
    pub methods: Vec<NoirFunction>,
}

impl Display for NoirStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;

        for (name, typ) in self.fields.iter() {
            writeln!(f, "    {name}: {typ},")?;
        }

        write!(f, "}}")
    }
}

impl Display for NoirImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "impl {} {{", self.type_path)?;

        for method in self.methods.iter() {
            let method = method.to_string();
            for line in method.lines() {
                writeln!(f, "    {line}")?;
            }
        }

        write!(f, "}}")
    }
}
