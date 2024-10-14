use noirc_frontend::ast::{Expression, ExpressionKind, Literal};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_expression(&mut self, expression: Expression) {
        self.skip_comments_and_whitespace();

        match expression.kind {
            ExpressionKind::Literal(literal) => self.format_literal(literal),
            ExpressionKind::Block(_block_expression) => todo!("Format block"),
            ExpressionKind::Prefix(_prefix_expression) => todo!("Format prefix"),
            ExpressionKind::Index(_index_expression) => todo!("Format index"),
            ExpressionKind::Call(_call_expression) => todo!("Format call"),
            ExpressionKind::MethodCall(_method_call_expression) => todo!("Format method call"),
            ExpressionKind::Constructor(_constructor_expression) => todo!("Format constructor"),
            ExpressionKind::MemberAccess(_member_access_expression) => {
                todo!("Format member access")
            }
            ExpressionKind::Cast(_cast_expression) => todo!("Format cast"),
            ExpressionKind::Infix(_infix_expression) => todo!("Format infix"),
            ExpressionKind::If(_if_expression) => todo!("Format if"),
            ExpressionKind::Variable(_path) => todo!("Format variable"),
            ExpressionKind::Tuple(_vec) => todo!("Format tuple"),
            ExpressionKind::Lambda(_lambda) => todo!("Format lambda"),
            ExpressionKind::Parenthesized(_expression) => todo!("Format parenthesized"),
            ExpressionKind::Quote(_tokens) => todo!("Format quote"),
            ExpressionKind::Unquote(_expression) => todo!("Format unquote"),
            ExpressionKind::Comptime(_block_expression, _span) => todo!("Format comptime"),
            ExpressionKind::Unsafe(_block_expression, _span) => todo!("Format unsafe"),
            ExpressionKind::AsTraitPath(_as_trait_path) => todo!("Format as trait path"),
            ExpressionKind::TypePath(_type_path) => todo!("Format type path"),
            ExpressionKind::Resolved(..)
            | ExpressionKind::Interned(..)
            | ExpressionKind::InternedStatement(..)
            | ExpressionKind::Error => unreachable!("Should not be present in the AST"),
        }
    }

    fn format_literal(&mut self, literal: Literal) {
        match literal {
            Literal::Unit => {
                self.write_left_paren();
                self.write_right_paren();
            }
            Literal::Bool(_)
            | Literal::Integer(..)
            | Literal::Str(_)
            | Literal::FmtStr(_)
            | Literal::RawStr(..) => {
                self.write_current_token();
                self.bump();
            }
            Literal::Array(_array_literal) => todo!("Format array"),
            Literal::Slice(_array_literal) => todo!("Format slice"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_unit() {
        let src = "global x =  ( ) ;";
        let expected = "global x = ();\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_false() {
        let src = "global x =  false ;";
        let expected = "global x = false;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_true() {
        let src = "global x =  true ;";
        let expected = "global x = true;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_integer() {
        let src = "global x =  42 ;";
        let expected = "global x = 42;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_string() {
        let src = "global x =  \"hello\" ;";
        let expected = "global x = \"hello\";\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_fmtstr() {
        let src = "global x =  f\"hello\" ;";
        let expected = "global x = f\"hello\";\n";
        assert_format(src, expected);
    }
}
