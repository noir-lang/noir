use noirc_errors::Span;

use crate::{Ident, Type};


#[derive(Clone, Debug, PartialEq)]
pub struct NoirStruct {
    pub name: Ident,
    pub fields: Vec<(Ident, Type)>,
    pub span: Span,
}

impl NoirStruct {
    pub fn new(name: Ident, fields: Vec<(Ident, Type)>, span: Span) -> NoirStruct {
        NoirStruct { name, fields, span }
    }
}
