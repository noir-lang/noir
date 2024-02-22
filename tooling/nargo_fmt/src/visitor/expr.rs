use noirc_frontend::{
    hir::resolution::errors::Span, lexer::Lexer, token::Token, BlockExpression,
    ConstructorExpression, Expression, ExpressionKind, IfExpression, Statement, StatementKind,
};

use super::{ExpressionType, FmtVisitor, Shape};
use crate::{
    items::{HasItem, Item, Items},
    rewrite,
    utils::{first_line_width, FindToken},
    Config,
};

impl FmtVisitor<'_> {
    pub(crate) fn visit_expr(&mut self, expr: Expression, expr_type: ExpressionType) {
        let span = expr.span;
        let rewrite = rewrite::expr(self, expr, expr_type, self.shape());
        self.push_rewrite(rewrite, span);
        self.last_position = span.end();
    }

    pub(crate) fn format_if(&self, if_expr: IfExpression) -> String {
        let condition_str = rewrite::sub_expr(self, self.shape(), if_expr.condition);
        let consequence_str = rewrite::sub_expr(self, self.shape(), if_expr.consequence);

        let mut result = format!("if {condition_str} {consequence_str}");

        if let Some(alternative) = if_expr.alternative {
            let alternative = if let Some(ExpressionKind::If(if_expr)) =
                extract_simple_expr(alternative.clone()).map(|expr| expr.kind)
            {
                self.format_if(*if_expr)
            } else {
                rewrite::expr(self, alternative, ExpressionType::Statement, self.shape())
            };

            result.push_str(" else ");
            result.push_str(&alternative);
        };

        result
    }

    pub(crate) fn format_if_single_line(&self, if_expr: IfExpression) -> Option<String> {
        let condition_str = rewrite::sub_expr(self, self.shape(), if_expr.condition);
        let consequence_str =
            rewrite::sub_expr(self, self.shape(), extract_simple_expr(if_expr.consequence)?);

        let if_str = if let Some(alternative) = if_expr.alternative {
            let alternative_str = if let Some(ExpressionKind::If(_)) =
                extract_simple_expr(alternative.clone()).map(|expr| expr.kind)
            {
                return None;
            } else {
                rewrite::expr(
                    self,
                    extract_simple_expr(alternative)?,
                    ExpressionType::Statement,
                    self.shape(),
                )
            };

            format!("if {} {{ {} }} else {{ {} }}", condition_str, consequence_str, alternative_str)
        } else {
            format!("if {{{}}} {{{}}}", condition_str, consequence_str)
        };

        (if_str.len() <= self.config.single_line_if_else_max_width).then_some(if_str)
    }

    pub(crate) fn format_struct_lit(
        &self,
        type_name: &str,
        fields_span: Span,
        constructor: ConstructorExpression,
    ) -> String {
        let fields = {
            let mut visitor = self.fork();
            let is_unit_struct = constructor.fields.is_empty();

            visitor.indent.block_indent(visitor.config);

            let nested_indent = visitor.shape();
            let exprs: Vec<_> =
                Items::new(&visitor, nested_indent, fields_span, constructor.fields).collect();
            let exprs = format_exprs(
                visitor.config,
                Tactic::HorizontalVertical,
                false,
                exprs,
                nested_indent,
                true,
            );

            visitor.indent.block_unindent(visitor.config);

            if exprs.contains('\n') {
                format!(
                    "{}{exprs}{}",
                    nested_indent.indent.to_string_with_newline(),
                    visitor.shape().indent.to_string_with_newline()
                )
            } else if is_unit_struct {
                exprs
            } else {
                format!(" {exprs} ")
            }
        };

        format!("{type_name} {{{fields}}}")
    }

    pub(crate) fn visit_block(&mut self, block: BlockExpression, block_span: Span) {
        if block.is_empty() {
            self.visit_empty_block(block_span);
            return;
        }

        self.last_position = block_span.start() + 1; // `{`
        self.push_str("{");

        self.trim_spaces_after_opening_brace(&block.0);

        self.indent.block_indent(self.config);

        self.visit_stmts(block.0);

        let span = (self.last_position..block_span.end() - 1).into();
        self.close_block(span);
        self.last_position = block_span.end();
    }

    fn trim_spaces_after_opening_brace(&mut self, block: &[Statement]) {
        if let Some(first_stmt) = block.first() {
            let slice = self.slice(self.last_position..first_stmt.span.start());
            let len =
                slice.chars().take_while(|ch| ch.is_whitespace()).collect::<String>().rfind('\n');
            self.last_position += len.unwrap_or(0) as u32;
        }
    }

    pub(crate) fn visit_empty_block(&mut self, block_span: Span) {
        let slice = self.slice(block_span);
        let comment_str = slice[1..slice.len() - 1].trim();

        if comment_str.is_empty() {
            self.push_str("{}");
        } else {
            self.push_str("{");
            self.indent.block_indent(self.config);
            self.close_block(block_span);
        };

        self.last_position = block_span.end();
    }

    pub(crate) fn close_block(&mut self, span: Span) {
        let slice = self.slice(span);

        for spanned in Lexer::new(slice).skip_comments(false).flatten() {
            match spanned.token() {
                Token::LineComment(_, _) | Token::BlockComment(_, _) => {
                    let token_span = spanned.to_span();

                    let offset = token_span.start();
                    let sub_slice = &slice[token_span.start() as usize..token_span.end() as usize];

                    let span_in_between = span.start()..span.start() + offset;
                    let slice_in_between = self.slice(span_in_between);
                    let comment_on_same_line = !slice_in_between.contains('\n');

                    if comment_on_same_line {
                        self.push_str(" ");
                        self.push_str(sub_slice);
                    } else {
                        self.push_str(&self.indent.to_string_with_newline());
                        self.push_str(sub_slice);
                    }
                }
                _ => {}
            }
        }

        self.indent.block_unindent(self.config);
        self.push_str(&self.indent.to_string_with_newline());
        self.push_str("}");
    }
}

// TODO: fixme
#[allow(clippy::too_many_arguments)]
pub(crate) fn format_seq<T: HasItem>(
    shape: Shape,
    prefix: &str,
    suffix: &str,
    visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<T>,
    span: Span,
    tactic: Tactic,
    mode: NewlineMode,
    reduce: bool,
) -> String {
    let mut nested_indent = shape;

    nested_indent.indent.block_indent(visitor.config);

    let exprs: Vec<_> = Items::new(&visitor, nested_indent, span, exprs).collect();
    let exprs = format_exprs(visitor.config, tactic, trailing_comma, exprs, nested_indent, reduce);

    wrap_exprs(prefix, suffix, exprs, nested_indent, shape, mode)
}

pub(crate) fn format_brackets(
    visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
) -> String {
    let array_width = visitor.config.array_width;
    format_seq(
        visitor.shape(),
        "[",
        "]",
        visitor,
        trailing_comma,
        exprs,
        span,
        Tactic::LimitedHorizontalVertical(array_width),
        NewlineMode::Normal,
        false,
    )
}

// TODO: fixme
#[allow(clippy::too_many_arguments)]
pub(crate) fn format_parens(
    max_width: Option<usize>,
    visitor: FmtVisitor,
    shape: Shape,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
    reduce: bool,
    mode: NewlineMode,
) -> String {
    let tactic = max_width.map(Tactic::LimitedHorizontalVertical).unwrap_or(Tactic::Horizontal);
    format_seq(shape, "(", ")", visitor, trailing_comma, exprs, span, tactic, mode, reduce)
}

pub(crate) fn format_exprs(
    config: &Config,
    tactic: Tactic,
    trailing_comma: bool,
    exprs: Vec<Item>,
    shape: Shape,
    reduce: bool,
) -> String {
    let mut result = String::new();
    let indent_str = shape.indent.to_string();

    let tactic = tactic.definitive(&exprs, config.short_array_element_width_threshold, reduce);
    let mut exprs = exprs.into_iter().enumerate().peekable();
    let mut line_len = 0;
    let mut prev_expr_trailing_comment = false;

    while let Some((index, expr)) = exprs.next() {
        let is_first = index == 0;
        let separate = exprs.peek().is_some() || trailing_comma;
        let separate_len = usize::from(separate);

        match tactic {
            DefinitiveTactic::Vertical
                if !is_first && !expr.value.is_empty() && !result.is_empty() =>
            {
                result.push('\n');
                result.push_str(&indent_str);
            }
            DefinitiveTactic::Horizontal if !is_first => {
                result.push(' ');
            }
            DefinitiveTactic::Mixed => {
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

        if tactic == DefinitiveTactic::Horizontal {
            result.push_str(&expr.trailing);
        }

        if separate && expr.trailing.find_token(Token::Comma).is_none() {
            result.push(',');
        }

        if tactic != DefinitiveTactic::Horizontal {
            prev_expr_trailing_comment = !expr.trailing.is_empty();

            if !expr.different_line && !expr.trailing.is_empty() {
                result.push(' ');
            }

            result.push_str(&expr.trailing);
        }
    }

    result
}

#[derive(PartialEq, Eq)]
pub(crate) enum NewlineMode {
    IfContainsNewLine,
    IfContainsNewLineAndWidth,
    Normal,
}

pub(crate) fn wrap_exprs(
    prefix: &str,
    suffix: &str,
    exprs: String,
    nested_shape: Shape,
    shape: Shape,
    newline_mode: NewlineMode,
) -> String {
    let mut force_one_line = if newline_mode == NewlineMode::IfContainsNewLine {
        true
    } else {
        first_line_width(&exprs) <= shape.width
    };

    if matches!(
        newline_mode,
        NewlineMode::IfContainsNewLine | NewlineMode::IfContainsNewLineAndWidth
    ) && force_one_line
    {
        force_one_line = !exprs.contains('\n');
    }

    if force_one_line {
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

        format!("{prefix}{exprs}{trailing_newline}{suffix}")
    } else {
        let nested_indent_str = nested_shape.indent.to_string_with_newline();
        let indent_str = shape.indent.to_string_with_newline();

        format!("{prefix}{nested_indent_str}{exprs}{indent_str}{suffix}")
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum Tactic {
    Horizontal,
    HorizontalVertical,
    LimitedHorizontalVertical(usize),
    Mixed,
}

impl Tactic {
    fn definitive(
        self,
        exprs: &[Item],
        short_width_threshold: usize,
        reduce: bool,
    ) -> DefinitiveTactic {
        let tactic = || {
            let has_single_line_comment = exprs.iter().any(|item| {
                has_single_line_comment(&item.leading) || has_single_line_comment(&item.trailing)
            });

            let limit = match self {
                _ if has_single_line_comment => return DefinitiveTactic::Vertical,

                Tactic::Horizontal => return DefinitiveTactic::Horizontal,
                Tactic::LimitedHorizontalVertical(limit) => limit,
                Tactic::HorizontalVertical | Tactic::Mixed => 100,
            };

            let (sep_count, total_width): (usize, usize) = exprs
                .iter()
                .map(|expr| expr.total_width())
                .fold((0, 0), |(sep_count, total_width), width| {
                    (sep_count + 1, total_width + width)
                });

            let total_sep_len = sep_count.saturating_sub(1);
            let real_total = total_width + total_sep_len;

            if real_total <= limit && !exprs.iter().any(|expr| expr.is_multiline()) {
                DefinitiveTactic::Horizontal
            } else if self == Tactic::Mixed {
                DefinitiveTactic::Mixed
            } else {
                DefinitiveTactic::Vertical
            }
        };

        let definitive_tactic = tactic();
        if reduce {
            definitive_tactic.reduce(exprs, short_width_threshold)
        } else {
            definitive_tactic
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DefinitiveTactic {
    Vertical,
    Horizontal,
    Mixed,
}

impl DefinitiveTactic {
    fn reduce(self, exprs: &[Item], short_array_element_width_threshold: usize) -> Self {
        match self {
            DefinitiveTactic::Vertical
                if no_long_exprs(exprs, short_array_element_width_threshold) =>
            {
                DefinitiveTactic::Mixed
            }
            DefinitiveTactic::Vertical | DefinitiveTactic::Horizontal | DefinitiveTactic::Mixed => {
                self
            }
        }
    }
}

fn has_single_line_comment(slice: &str) -> bool {
    slice.trim_start().starts_with("//")
}

fn no_long_exprs(exprs: &[Item], max_width: usize) -> bool {
    exprs.iter().all(|expr| expr.value.len() <= max_width)
}

fn extract_simple_expr(expr: Expression) -> Option<Expression> {
    if let ExpressionKind::Block(mut block) = expr.kind {
        if block.len() == 1 {
            if let StatementKind::Expression(expr) = block.pop().unwrap() {
                return expr.into();
            }
        }
    }

    None
}
