use noirc_frontend::{
    hir::resolution::errors::Span, lexer::Lexer, token::Token, ArrayLiteral, BlockExpression,
    ConstructorExpression, Expression, ExpressionKind, IfExpression, Literal, Statement,
    StatementKind, UnaryOp,
};

use super::{ExpressionType, FmtVisitor, Indent, Shape};
use crate::{
    rewrite,
    utils::{self, first_line_width, Expr, FindToken, Item},
    Config,
};

impl FmtVisitor<'_> {
    pub(crate) fn visit_expr(&mut self, expr: Expression, expr_type: ExpressionType) {
        let span = expr.span;
        let rewrite = self.format_expr(expr, expr_type);
        self.push_rewrite(rewrite, span);
        self.last_position = span.end();
    }

    pub(crate) fn format_sub_expr(&self, expression: Expression) -> String {
        self.format_expr(expression, ExpressionType::SubExpression)
    }

    pub(crate) fn format_expr(
        &self,
        Expression { kind, mut span }: Expression,
        expr_type: ExpressionType,
    ) -> String {
        match kind {
            ExpressionKind::Block(block) => {
                let mut visitor = self.fork();
                visitor.visit_block(block, span);
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

                format!("{op}{}", self.format_sub_expr(prefix.rhs))
            }
            ExpressionKind::Cast(cast) => {
                format!("{} as {}", self.format_sub_expr(cast.lhs), cast.r#type)
            }
            kind @ ExpressionKind::Infix(_) => {
                let shape = self.shape();
                rewrite::infix(self.fork(), Expression { kind, span }, shape)
            }
            ExpressionKind::Call(call_expr) => {
                let args_span =
                    self.span_before(call_expr.func.span.end()..span.end(), Token::LeftParen);

                let callee = self.format_sub_expr(*call_expr.func);
                let args = format_parens(
                    self.config.fn_call_width.into(),
                    self.fork(),
                    false,
                    call_expr.arguments,
                    args_span,
                    true,
                );

                format!("{callee}{args}")
            }
            ExpressionKind::MethodCall(method_call_expr) => {
                let args_span = self.span_before(
                    method_call_expr.method_name.span().end()..span.end(),
                    Token::LeftParen,
                );

                let object = self.format_sub_expr(method_call_expr.object);
                let method = method_call_expr.method_name.to_string();
                let args = format_parens(
                    self.config.fn_call_width.into(),
                    self.fork(),
                    false,
                    method_call_expr.arguments,
                    args_span,
                    true,
                );

                format!("{object}.{method}{args}")
            }
            ExpressionKind::MemberAccess(member_access_expr) => {
                let lhs_str = self.format_sub_expr(member_access_expr.lhs);
                format!("{}.{}", lhs_str, member_access_expr.rhs)
            }
            ExpressionKind::Index(index_expr) => {
                let index_span = self
                    .span_before(index_expr.collection.span.end()..span.end(), Token::LeftBracket);

                let collection = self.format_sub_expr(index_expr.collection);
                let index = format_brackets(self.fork(), false, vec![index_expr.index], index_span);

                format!("{collection}{index}")
            }
            ExpressionKind::Tuple(exprs) => {
                format_parens(None, self.fork(), exprs.len() == 1, exprs, span, false)
            }
            ExpressionKind::Literal(literal) => match literal {
                Literal::Integer(_) | Literal::Bool(_) | Literal::Str(_) | Literal::FmtStr(_) => {
                    self.slice(span).to_string()
                }
                Literal::Array(ArrayLiteral::Repeated { repeated_element, length }) => {
                    let repeated = self.format_sub_expr(*repeated_element);
                    let length = self.format_sub_expr(*length);

                    format!("[{repeated}; {length}]")
                }
                Literal::Array(ArrayLiteral::Standard(exprs)) => {
                    rewrite::array(self.fork(), exprs, span)
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
                    let sub_expr = self.format_sub_expr(*sub_expr);
                    format!("({leading}{sub_expr}{trailing})")
                } else {
                    let mut visitor = self.fork();

                    let indent = visitor.indent.to_string_with_newline();
                    visitor.indent.block_indent(self.config);
                    let nested_indent = visitor.indent.to_string_with_newline();

                    let sub_expr = visitor.format_sub_expr(*sub_expr);

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
            ExpressionKind::Constructor(constructor) => {
                let type_name = self.slice(span.start()..constructor.type_name.span().end());
                let fields_span = self
                    .span_before(constructor.type_name.span().end()..span.end(), Token::LeftBrace);

                self.format_struct_lit(type_name, fields_span, *constructor)
            }
            ExpressionKind::If(if_expr) => {
                let allow_single_line = expr_type == ExpressionType::SubExpression;

                if allow_single_line {
                    let mut visitor = self.fork();
                    visitor.indent = Indent::default();
                    if let Some(line) = visitor.format_if_single_line(*if_expr.clone()) {
                        return line;
                    }
                }

                self.format_if(*if_expr)
            }
            ExpressionKind::Lambda(_) | ExpressionKind::Variable(_) => self.slice(span).to_string(),
            ExpressionKind::Error => unreachable!(),
        }
    }

    fn format_if(&self, if_expr: IfExpression) -> String {
        let condition_str = self.format_sub_expr(if_expr.condition);
        let consequence_str = self.format_sub_expr(if_expr.consequence);

        let mut result = format!("if {condition_str} {consequence_str}");

        if let Some(alternative) = if_expr.alternative {
            let alternative = if let Some(ExpressionKind::If(if_expr)) =
                extract_simple_expr(alternative.clone()).map(|expr| expr.kind)
            {
                self.format_if(*if_expr)
            } else {
                self.format_expr(alternative, ExpressionType::Statement)
            };

            result.push_str(" else ");
            result.push_str(&alternative);
        };

        result
    }

    fn format_if_single_line(&self, if_expr: IfExpression) -> Option<String> {
        let condition_str = self.format_sub_expr(if_expr.condition);
        let consequence_str = self.format_sub_expr(extract_simple_expr(if_expr.consequence)?);

        let if_str = if let Some(alternative) = if_expr.alternative {
            let alternative_str = if let Some(ExpressionKind::If(_)) =
                extract_simple_expr(alternative.clone()).map(|expr| expr.kind)
            {
                return None;
            } else {
                self.format_expr(extract_simple_expr(alternative)?, ExpressionType::Statement)
            };

            format!("if {} {{ {} }} else {{ {} }}", condition_str, consequence_str, alternative_str)
        } else {
            format!("if {{{}}} {{{}}}", condition_str, consequence_str)
        };

        (if_str.len() <= self.config.single_line_if_else_max_width).then_some(if_str)
    }

    fn format_struct_lit(
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
                utils::Exprs::new(&visitor, fields_span, constructor.fields).collect();
            let exprs = format_exprs(
                visitor.config,
                Tactic::HorizontalVertical,
                false,
                exprs,
                nested_indent,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn format_seq<T: Item>(
    prefix: &str,
    suffix: &str,
    mut visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<T>,
    span: Span,
    tactic: Tactic,
    soft_newline: bool,
) -> String {
    visitor.indent.block_indent(visitor.config);

    let nested_indent = visitor.shape();
    let exprs: Vec<_> = utils::Exprs::new(&visitor, span, exprs).collect();
    let exprs = format_exprs(visitor.config, tactic, trailing_comma, exprs, nested_indent);

    visitor.indent.block_unindent(visitor.config);

    wrap_exprs(prefix, suffix, exprs, nested_indent, visitor.shape(), soft_newline)
}

fn format_brackets(
    visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
) -> String {
    let array_width = visitor.config.array_width;
    format_seq(
        "[",
        "]",
        visitor,
        trailing_comma,
        exprs,
        span,
        Tactic::LimitedHorizontalVertical(array_width),
        false,
    )
}

fn format_parens(
    max_width: Option<usize>,
    visitor: FmtVisitor,
    trailing_comma: bool,
    exprs: Vec<Expression>,
    span: Span,
    soft_newline: bool,
) -> String {
    let tactic = max_width.map(Tactic::LimitedHorizontalVertical).unwrap_or(Tactic::Horizontal);
    format_seq("(", ")", visitor, trailing_comma, exprs, span, tactic, soft_newline)
}

fn format_exprs(
    config: &Config,
    tactic: Tactic,
    trailing_comma: bool,
    exprs: Vec<Expr>,
    shape: Shape,
) -> String {
    let mut result = String::new();
    let indent_str = shape.indent.to_string();

    let tactic = tactic.definitive(&exprs, config.short_array_element_width_threshold);
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

pub(crate) fn wrap_exprs(
    prefix: &str,
    suffix: &str,
    exprs: String,
    nested_shape: Shape,
    shape: Shape,
    soft_newline: bool,
) -> String {
    let first_line_width = first_line_width(&exprs);

    let force_one_line =
        if soft_newline { !exprs.contains('\n') } else { first_line_width <= shape.width };

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
        exprs: &[Expr],
        short_array_element_width_threshold: usize,
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

        tactic().reduce(exprs, short_array_element_width_threshold)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DefinitiveTactic {
    Vertical,
    Horizontal,
    Mixed,
}

impl DefinitiveTactic {
    fn reduce(self, exprs: &[Expr], short_array_element_width_threshold: usize) -> Self {
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

fn no_long_exprs(exprs: &[Expr], max_width: usize) -> bool {
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
