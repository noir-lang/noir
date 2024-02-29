use noirc_frontend::{hir::resolution::errors::Span, token::Token, Expression};

use crate::{
    items::Item,
    utils::FindToken,
    visitor::{expr::NewlineMode, FmtVisitor},
};

pub(crate) fn rewrite(mut visitor: FmtVisitor, array: Vec<Expression>, array_span: Span) -> String {
    let pattern: &[_] = &[' ', '\t'];

    visitor.indent.block_indent(visitor.config);
    let nested_indent = visitor.shape();

    let indent_str = nested_indent.indent.to_string();

    let mut last_position = array_span.start() + 1;
    let end_position = array_span.end() - 1;

    let mut items = array.into_iter().peekable();

    let mut result = Vec::new();
    while let Some(item) = items.next() {
        let item_span = item.span;

        let start: u32 = last_position;
        let end = item_span.start();

        let leading = visitor.slice(start..end).trim_matches(pattern);
        let item = super::sub_expr(&visitor, visitor.shape(), item);
        let next_start = items.peek().map_or(end_position, |expr| expr.span.start());
        let trailing = visitor.slice(item_span.end()..next_start);
        let offset = trailing
            .find_token(Token::Comma)
            .map(|span| span.end() as usize)
            .unwrap_or(trailing.len());
        let trailing = trailing[..offset].trim_end_matches(',').trim_matches(pattern);
        last_position = item_span.end() + offset as u32;

        let (leading, _) = visitor.format_comment_in_block(leading);
        let (trailing, _) = visitor.format_comment_in_block(trailing);

        result.push(Item { leading, value: item, trailing, different_line: false });
    }

    let slice = visitor.slice(last_position..end_position);
    let (comment, _) = visitor.format_comment_in_block(slice);
    result.push(Item {
        leading: "".into(),
        value: "".into(),
        trailing: comment,
        different_line: false,
    });

    visitor.indent.block_unindent(visitor.config);

    let mut items_str = String::new();
    let mut items = result.into_iter().peekable();
    while let Some(next) = items.next() {
        items_str.push_str(&next.leading);
        if next.leading.contains('\n') && !next.value.is_empty() {
            items_str.push_str(&indent_str);
        }
        items_str.push_str(&next.value);
        items_str.push_str(&next.trailing);

        if let Some(item) = items.peek() {
            if !item.value.is_empty() {
                items_str.push(',');
            }

            if !item.leading.contains('\n') && !next.value.is_empty() {
                items_str.push(' ');
            }
        }
    }

    crate::visitor::expr::wrap_exprs(
        "[",
        "]",
        items_str.trim().into(),
        nested_indent,
        visitor.shape(),
        NewlineMode::IfContainsNewLineAndWidth,
    )
}
