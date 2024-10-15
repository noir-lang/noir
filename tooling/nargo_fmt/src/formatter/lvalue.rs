use noirc_frontend::ast::LValue;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_lvalue(&mut self, lvalue: LValue) {
        match lvalue {
            LValue::Ident(ident) => self.write_identifier(ident),
            LValue::MemberAccess { object: _, field_name: _, span: _ } => {
                todo!("Format lvalue member access")
            }
            LValue::Index { array: _, index: _, span: _ } => todo!("Format lvalue index"),
            LValue::Dereference(_lvalue, _span) => todo!("Format lvalue dereference"),
            LValue::Interned(..) => {
                unreachable!("Should not be present in the AST")
            }
        }
    }
}
