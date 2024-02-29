use std::iter::zip;

use noirc_frontend::{Expression, ExpressionKind};

use crate::{
    rewrite,
    utils::{first_line_width, is_single_line},
    visitor::{FmtVisitor, Shape},
};

pub(crate) fn rewrite(visitor: FmtVisitor, expr: Expression, shape: Shape) -> String {
    match flatten(visitor.fork(), &expr) {
        Some((exprs, separators)) => rewrite_single_line(shape, &exprs, &separators)
            .unwrap_or_else(|| rewrite_multiline(visitor, &exprs, &separators)),
        None => {
            let ExpressionKind::Infix(infix) = expr.kind else { unreachable!() };

            format!(
                "{} {} {}",
                rewrite::sub_expr(&visitor, shape, infix.lhs),
                infix.operator.contents.as_string(),
                rewrite::sub_expr(&visitor, shape, infix.rhs)
            )
        }
    }
}

fn rewrite_single_line(shape: Shape, exprs: &[String], separators: &[String]) -> Option<String> {
    let mut result = String::new();

    for (rewrite, separator) in zip(exprs, separators) {
        if !is_single_line(rewrite) || result.len() > shape.width {
            return None;
        }

        result.push_str(rewrite);
        result.push(' ');
        result.push_str(separator);
        result.push(' ');
    }

    let last = exprs.last().unwrap();
    result.push_str(last);

    if first_line_width(&result) > shape.width {
        return None;
    }

    result.into()
}

fn rewrite_multiline(visitor: FmtVisitor, exprs: &[String], separators: &[String]) -> String {
    let mut visitor = visitor.fork();
    visitor.indent.block_indent(visitor.config);
    let indent_str = visitor.indent.to_string_with_newline();

    let mut result = exprs[0].clone();

    for (rewrite, separator) in exprs[1..].iter().zip(separators.iter()) {
        result.push_str(&indent_str);
        result.push_str(separator);
        result.push(' ');
        result.push_str(rewrite);
    }

    result
}

pub(crate) fn flatten(
    mut visitor: FmtVisitor,
    mut node: &Expression,
) -> Option<(Vec<String>, Vec<String>)> {
    let top_operator = match node.kind {
        ExpressionKind::Infix(ref infix) => infix.operator.contents,
        _ => return None,
    };

    let mut result = Vec::new();

    let mut stack: Vec<&Expression> = Vec::new();
    let mut separators = Vec::new();

    loop {
        match &node.kind {
            ExpressionKind::Infix(infix) if top_operator == infix.operator.contents => {
                stack.push(node);
                node = &infix.lhs;
            }
            _ => {
                let rewrite = if result.is_empty() {
                    rewrite::sub_expr(&visitor, visitor.shape(), node.clone())
                } else {
                    visitor.indent.block_indent(visitor.config);
                    rewrite::sub_expr(&visitor, visitor.shape(), node.clone())
                };

                result.push(rewrite);

                let Some(pop) = stack.pop() else {
                    break;
                };

                match &pop.kind {
                    ExpressionKind::Infix(infix) => {
                        separators.push(infix.operator.contents.to_string());
                        node = &infix.rhs;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    (result, separators).into()
}
