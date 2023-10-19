use std::ops::Range;

use noirc_frontend::{
    hir::resolution::errors::Span, token::Token, ArrayLiteral, BlockExpression, Expression,
    ExpressionKind, Literal, Statement, UnaryOp,
};

use super::{FmtVisitor, Shape};
use crate::{
    utils::{self, Expr, FindToken},
    Config,
};

impl FmtVisitor<'_> {
    pub(crate) fn visit_expr(&mut self, expr: Expression) {
        let span = expr.span;

        let rewrite = self.format_expr(expr);
        let rewrite =
            utils::recover_comment_removed(slice!(self, span.start(), span.end()), rewrite);
        self.push_rewrite(rewrite, span);

        self.last_position = span.end();
    }

    pub(crate) fn format_expr(&self, Expression { kind, mut span }: Expression) -> String {
        match kind {
            ExpressionKind::Block(block) => {
                let mut visitor = self.fork();
                visitor.visit_block(block, span, true);
                visitor.buffer
            }
            ExpressionKind::Prefix(prefix) => {
                let op = match prefix.operator {
                    UnaryOp::Minus => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::MutableReference => "&mut ",
                    UnaryOp::Dereference { implicitly_added } => {
                        if implicitly_added {
                            ""
                        } else {
                            "*"
                        }
                    }
                };

                format!("{op}{}", self.format_expr(prefix.rhs))
            }
            ExpressionKind::Cast(cast) => {
                format!("{} as {}", self.format_expr(cast.lhs), cast.r#type)
            }
            ExpressionKind::Infix(infix) => {
                format!(
                    "{} {} {}",
                    self.format_expr(infix.lhs),
                    infix.operator.contents.as_string(),
                    self.format_expr(infix.rhs)
                )
            }
            ExpressionKind::Call(call_expr) => {
                let span = call_expr.func.span.end()..span.end();
                let span =
                    skip_useless_tokens(Token::LeftParen, slice!(self, span.start, span.end), span);

                let callee = self.format_expr(*call_expr.func);
                let args = format_parens(self.fork(), false, call_expr.arguments, span);

                format!("{callee}{args}")
            }
            ExpressionKind::MethodCall(method_call_expr) => {
                let span = method_call_expr.method_name.span().end()..span.end();
                let span =
                    skip_useless_tokens(Token::LeftParen, slice!(self, span.start, span.end), span);

                let object = self.format_expr(method_call_expr.object);
                let method = method_call_expr.method_name.to_string();
                let args = format_parens(self.fork(), false, method_call_expr.arguments, span);

                format!("{object}.{method}{args}")
            }
            ExpressionKind::MemberAccess(member_access_expr) => {
                let lhs_str = self.format_expr(member_access_expr.lhs);
                format!("{}.{}", lhs_str, member_access_expr.rhs)
            }
            ExpressionKind::Index(index_expr) => {
                let span = index_expr.collection.span.end()..span.end();

                let span = skip_useless_tokens(
                    Token::LeftBracket,
                    slice!(self, span.start, span.end),
                    span,
                );

                let collection = self.format_expr(index_expr.collection);
                let index = format_brackets(self.fork(), false, vec![index_expr.index], span);

                format!("{collection}{index}")
            }
            ExpressionKind::Tuple(exprs) => {
                format_parens(self.fork(), exprs.len() == 1, exprs, span)
            }
            ExpressionKind::Literal(literal) => match literal {
                Literal::Integer(_) | Literal::Bool(_) | Literal::Str(_) | Literal::FmtStr(_) => {
                    slice!(self, span.start(), span.end()).to_string()
                }
                Literal::Array(ArrayLiteral::Repeated { repeated_element, length }) => {
                    let repeated = self.format_expr(*repeated_element);
                    let length = self.format_expr(*length);

                    format!("[{repeated}; {length}]")
                }
                Literal::Array(ArrayLiteral::Standard(exprs)) => {
                    format_brackets(self.fork(), false, exprs, span)
                }
                Literal::Unit => "()".to_string(),
            },
            ExpressionKind::Parenthesized(mut sub_expr) => {
                let remove_nested_parens = self.config.remove_nested_parens;

                let mut leading;
                let mut trailing;

                loop {
                    let leading_span = span.start() + 1..sub_expr.span.start();
                    let trailing_span = sub_expr.span.end()..span.end() - 1;

                    leading = self.format_comment(leading_span.into());
                    trailing = self.format_comment(trailing_span.into());

                    if let ExpressionKind::Parenthesized(ref sub_sub_expr) = sub_expr.kind {
                        if remove_nested_parens && leading.is_empty() && trailing.is_empty() {
                            span = sub_expr.span;
                            sub_expr = sub_sub_expr.clone();
                            continue;
                        }
                    }

                    break;
                }

                if !leading.contains("//") && !trailing.contains("//") {
                    let sub_expr = self.format_expr(*sub_expr);
                    format!("({leading}{sub_expr}{trailing})")
                } else {
                    let mut visitor = self.fork();

                    let indent = visitor.indent.to_string_with_newline();
                    visitor.indent.block_indent(self.config);
                    let nested_indent = visitor.indent.to_string_with_newline();

                    let sub_expr = visitor.format_expr(*sub_expr);

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
            // TODO:
            _expr => slice!(self, span.start(), span.end()).to_string(),
        }
    }

    pub(crate) fn visit_block(
        &mut self,
        block: BlockExpression,
        block_span: Span,
        should_indent: bool,
    ) {
        if block.is_empty() {
            self.visit_empty_block(block_span, should_indent);
            return;
        }

        self.last_position = block_span.start() + 1; // `{`
        self.push_str("{");

        self.trim_spaces_after_opening_brace(&block.0);

        self.with_indent(|this| {
            this.visit_stmts(block.0);
        });

        let slice = slice!(self, self.last_position, block_span.end() - 1).trim_end();
        self.push_str(slice);

        self.last_position = block_span.end();

        self.push_str("\n");
        if should_indent {
            self.push_str(&self.indent.to_string());
        }
        self.push_str("}");
    }

    fn trim_spaces_after_opening_brace(&mut self, block: &[Statement]) {
        if let Some(first_stmt) = block.first() {
            let slice = slice!(self, self.last_position, first_stmt.span.start());
            let len =
                slice.chars().take_while(|ch| ch.is_whitespace()).collect::<String>().rfind('\n');
            self.last_position += len.unwrap_or(0) as u32;
        }
    }

    fn visit_empty_block(&mut self, block_span: Span, should_indent: bool) {
        let slice = slice!(self, block_span.start(), block_span.end());
        let comment_str = slice[1..slice.len() - 1].trim();
        let block_str = if comment_str.is_empty() {
            "{}".to_string()
        } else {
            self.indent.block_indent(self.config);
            let open_indent = self.indent.to_string();
            self.indent.block_unindent(self.config);
            let close_indent = if should_indent { self.indent.to_string() } else { String::new() };

            let ret = format!("{{\n{open_indent}{comment_str}\n{close_indent}}}");
            ret
        };
        self.last_position = block_span.end();
        self.push_str(&block_str);
    }
}

fn format_expr_seq(
    prefix: &str,
    sufix: &str,
    mut visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
    limit: Option<usize>,
) -> String {
    visitor.indent.block_indent(visitor.config);

    let nested_indent = visitor.shape();
    let exprs: Vec<_> = utils::Exprs::new(&visitor, span, exprs).collect();
    let exprs = format_exprs(visitor.config, trailing_comma, exprs, nested_indent, limit);

    visitor.indent.block_unindent(visitor.config);

    wrap_exprs(prefix, sufix, exprs, nested_indent, visitor.shape())
}

fn format_brackets(
    visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
) -> String {
    let array_width = visitor.config.array_width;
    format_expr_seq("[", "]", visitor, trailing_comma, exprs, span, array_width.into())
}

fn format_parens(
    visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
) -> String {
    format_expr_seq("(", ")", visitor, trailing_comma, exprs, span, None)
}

fn format_exprs(
    config: &Config,
    trailing_comma: bool,
    exprs: Vec<Expr>,
    shape: Shape,
    limit: Option<usize>,
) -> String {
    let mut result = String::new();
    let indent_str = shape.indent.to_string();

    let tactic = Tactic::of(&exprs, config.short_array_element_width_threshold, limit);
    let mut exprs = exprs.into_iter().enumerate().peekable();
    let mut line_len = 0;
    let mut prev_expr_trailing_comment = false;

    while let Some((index, expr)) = exprs.next() {
        let is_first = index == 0;
        let separate = exprs.peek().is_some() || trailing_comma;
        let separate_len = usize::from(separate);

        match tactic {
            Tactic::Vertical if !is_first && !expr.value.is_empty() && !result.is_empty() => {
                result.push('\n');
                result.push_str(&indent_str);
            }
            Tactic::Horizontal if !is_first => {
                result.push(' ');
            }
            Tactic::Mixed => {
                let total_width = expr.total_width() + separate_len;

                if line_len > 0 && line_len + 1 + total_width > shape.width
                    || prev_expr_trailing_comment
                {
                    result.push('\n');
                    result.push_str(&indent_str);
                    line_len = 0;
                } else if line_len > 0 {
                    result.push(' ');
                    line_len += 1;
                }

                line_len += total_width;
            }
            _ => {}
        }

        result.push_str(&expr.leading);

        if expr.different_line {
            result.push('\n');
            result.push_str(&indent_str);
            line_len = expr.value.chars().count();
        } else if !expr.leading.is_empty() {
            result.push(' ');
        }

        result.push_str(&expr.value);

        if tactic == Tactic::Horizontal {
            result.push_str(&expr.trailing);
        }

        if separate && expr.trailing.find_token(Token::Comma).is_none() {
            result.push(',');
        }

        if tactic != Tactic::Horizontal {
            prev_expr_trailing_comment = !expr.trailing.is_empty();

            if !expr.different_line && !expr.trailing.is_empty() {
                result.push(' ');
            }

            result.push_str(&expr.trailing);
        }
    }

    result
}

fn wrap_exprs(
    prefix: &str,
    sufix: &str,
    exprs: String,
    nested_shape: Shape,
    shape: Shape,
) -> String {
    let first_line_width = exprs.lines().next().map_or(0, |line| line.chars().count());

    if first_line_width <= shape.width {
        let allow_trailing_newline = exprs
            .lines()
            .last()
            .unwrap_or_default()
            .find_token_with(|token| matches!(token, Token::LineComment(_, _)))
            .is_some();

        let trailing_newline = if allow_trailing_newline {
            shape.indent.to_string_with_newline()
        } else {
            String::new()
        };

        format!("{prefix}{exprs}{trailing_newline}{sufix}")
    } else {
        let nested_indent_str = nested_shape.indent.to_string_with_newline();
        let indent_str = shape.indent.to_string();

        format!("{prefix}{nested_indent_str}{exprs}{indent_str}{sufix}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tactic {
    Vertical,
    Horizontal,
    Mixed,
}

impl Tactic {
    fn of(exprs: &[Expr], max_width: usize, limit: Option<usize>) -> Self {
        let mut tactic = if exprs.iter().any(|item| {
            has_single_line_comment(&item.leading) || has_single_line_comment(&item.trailing)
        }) {
            Tactic::Vertical
        } else {
            Tactic::Horizontal
        };

        if tactic == Tactic::Vertical && no_long_exprs(exprs, max_width) {
            tactic = Tactic::Mixed;
        }

        if let Some(limit) = limit {
            let total_width: usize = exprs.iter().map(|expr| expr.total_width()).sum();
            if total_width <= limit && !exprs.iter().any(|expr| expr.is_multiline()) {
                tactic = Tactic::Horizontal;
            } else {
                tactic = Tactic::Mixed;
            }
        }

        tactic
    }
}

fn has_single_line_comment(slice: &str) -> bool {
    slice.trim_start().starts_with("//")
}

fn skip_useless_tokens(token: Token, slice: &str, mut span: Range<u32>) -> Span {
    let offset = slice.find_token(token).unwrap();
    span.start += offset;
    span.into()
}

fn no_long_exprs(exprs: &[Expr], max_width: usize) -> bool {
    exprs.iter().all(|expr| expr.value.len() <= max_width)
}
