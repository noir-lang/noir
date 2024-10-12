use noirc_frontend::ast::{UnresolvedType, UnresolvedTypeData};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_type(&mut self, typ: UnresolvedType) {
        self.skip_comments_and_whitespace();

        match typ.typ {
            UnresolvedTypeData::Integer(..) | UnresolvedTypeData::FieldElement => {
                self.write_current_token();
                self.bump();
            }
            UnresolvedTypeData::Array(unresolved_type_expression, unresolved_type) => todo!(),
            UnresolvedTypeData::Slice(unresolved_type) => todo!(),
            UnresolvedTypeData::Bool => todo!(),
            UnresolvedTypeData::Expression(unresolved_type_expression) => todo!(),
            UnresolvedTypeData::String(unresolved_type_expression) => todo!(),
            UnresolvedTypeData::FormatString(unresolved_type_expression, unresolved_type) => {
                todo!()
            }
            UnresolvedTypeData::Unit => {
                self.write_left_paren();
                self.write_right_paren();
            }
            UnresolvedTypeData::Parenthesized(unresolved_type) => todo!(),
            UnresolvedTypeData::Named(path, generic_type_args, _) => todo!(),
            UnresolvedTypeData::TraitAsType(path, generic_type_args) => todo!(),
            UnresolvedTypeData::MutableReference(unresolved_type) => todo!(),
            UnresolvedTypeData::Tuple(vec) => todo!(),
            UnresolvedTypeData::Function(vec, unresolved_type, unresolved_type1, _) => todo!(),
            UnresolvedTypeData::Quoted(quoted_type) => todo!(),
            UnresolvedTypeData::AsTraitPath(as_trait_path) => todo!(),
            UnresolvedTypeData::Resolved(quoted_type_id) => todo!(),
            UnresolvedTypeData::Interned(interned_unresolved_type_data) => todo!(),
            UnresolvedTypeData::Unspecified => todo!(),
            UnresolvedTypeData::Error => todo!(),
        }
    }
}
