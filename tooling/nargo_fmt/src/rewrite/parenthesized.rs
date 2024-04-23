use noirc_frontend::ast::{Expression, ExpressionKind};
use noirc_frontend::hir::resolution::errors::Span;

use crate::visitor::{FmtVisitor, Shape};

pub(crate) fn rewrite(
    visitor: &FmtVisitor<'_>,
    shape: Shape,
    mut span: Span,
    mut sub_expr: Expression,
) -> String {
    let remove_nested_parens = visitor.config.remove_nested_parens;

    let mut leading;
    let mut trailing;

    loop {
        let leading_span = span.start() + 1..sub_expr.span.start();
        let trailing_span = sub_expr.span.end()..span.end() - 1;

        leading = visitor.format_comment(leading_span.into());
        trailing = visitor.format_comment(trailing_span.into());

        if let ExpressionKind::Parenthesized(ref sub_sub_expr) = sub_expr.kind {
            if remove_nested_parens && leading.is_empty() && trailing.is_empty() {
                span = sub_expr.span;
                sub_expr = *sub_sub_expr.clone();
                continue;
            }
        }

        break;
    }

    if !leading.contains("//") && !trailing.contains("//") {
        let sub_expr = super::sub_expr(visitor, shape, sub_expr);
        format!("({leading}{sub_expr}{trailing})")
    } else {
        let mut visitor = visitor.fork();

        let indent = visitor.indent.to_string_with_newline();
        visitor.indent.block_indent(visitor.config);
        let nested_indent = visitor.indent.to_string_with_newline();

        let sub_expr = super::sub_expr(&visitor, shape, sub_expr);

        let mut result = String::new();
        result.push('(');

        if !leading.is_empty() {
            result.push_str(&nested_indent);
            result.push_str(&leading);
        }

        result.push_str(&nested_indent);
        result.push_str(&sub_expr);

        if !trailing.is_empty() {
            result.push_str(&nested_indent);
            result.push_str(&trailing);
        }

        result.push_str(&indent);
        result.push(')');

        result
    }
}
