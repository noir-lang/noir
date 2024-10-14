use noirc_frontend::ast::{Expression, ExpressionKind, Literal};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_expression(&mut self, expression: Expression) {
        self.skip_comments_and_whitespace();

        match expression.kind {
            ExpressionKind::Literal(literal) => self.format_literal(literal),
            ExpressionKind::Block(block_expression) => todo!(),
            ExpressionKind::Prefix(prefix_expression) => todo!(),
            ExpressionKind::Index(index_expression) => todo!(),
            ExpressionKind::Call(call_expression) => todo!(),
            ExpressionKind::MethodCall(method_call_expression) => todo!(),
            ExpressionKind::Constructor(constructor_expression) => todo!(),
            ExpressionKind::MemberAccess(member_access_expression) => todo!(),
            ExpressionKind::Cast(cast_expression) => todo!(),
            ExpressionKind::Infix(infix_expression) => todo!(),
            ExpressionKind::If(if_expression) => todo!(),
            ExpressionKind::Variable(path) => todo!(),
            ExpressionKind::Tuple(vec) => todo!(),
            ExpressionKind::Lambda(lambda) => todo!(),
            ExpressionKind::Parenthesized(expression) => todo!(),
            ExpressionKind::Quote(tokens) => todo!(),
            ExpressionKind::Unquote(expression) => todo!(),
            ExpressionKind::Comptime(block_expression, span) => todo!(),
            ExpressionKind::Unsafe(block_expression, span) => todo!(),
            ExpressionKind::AsTraitPath(as_trait_path) => todo!(),
            ExpressionKind::TypePath(type_path) => todo!(),
            ExpressionKind::Resolved(expr_id) => todo!(),
            ExpressionKind::Interned(interned_expression_kind) => todo!(),
            ExpressionKind::InternedStatement(interned_statement_kind) => todo!(),
            ExpressionKind::Error => todo!(),
        }
    }

    fn format_literal(&mut self, literal: Literal) {
        match literal {
            Literal::Array(array_literal) => todo!(),
            Literal::Slice(array_literal) => todo!(),
            Literal::Bool(_) => todo!(),
            Literal::Integer(..) => {
                self.write_current_token();
                self.bump();
            }
            Literal::Str(_) => todo!(),
            Literal::RawStr(_, _) => todo!(),
            Literal::FmtStr(_) => todo!(),
            Literal::Unit => todo!(),
        }
    }
}
