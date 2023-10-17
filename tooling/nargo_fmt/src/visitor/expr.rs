use noirc_frontend::{
    hir::resolution::errors::Span, lexer::Lexer, token::Token, ArrayLiteral, BlockExpression,
    Expression, ExpressionKind, Literal, Statement, UnaryOp,
};

use super::FmtVisitor;

impl FmtVisitor<'_> {
    pub(crate) fn visit_expr(&mut self, expr: Expression) {
        let span = expr.span;

        let rewrite = self.format_expr(expr);
        let original = slice!(self, span.start(), span.end());
        let changed_comment_content = changed_comment_content(original, &rewrite);

        if changed_comment_content && self.config.error_on_unformatted {
            panic!("{original:?} vs {rewrite:?}");
        }

        self.push_rewrite(
            if changed_comment_content { original.to_string() } else { rewrite },
            span,
        );
        self.last_position = span.end();
    }

    fn format_expr(&self, Expression { kind, mut span }: Expression) -> String {
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
                let formatted_func = self.format_expr(*call_expr.func);
                let formatted_args = call_expr
                    .arguments
                    .into_iter()
                    .map(|arg| self.format_expr(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", formatted_func, formatted_args)
            }
            ExpressionKind::MethodCall(method_call_expr) => {
                let formatted_object = self.format_expr(method_call_expr.object).trim().to_string();
                let formatted_args = method_call_expr
                    .arguments
                    .iter()
                    .map(|arg| {
                        let arg_str = self.format_expr(arg.clone()).trim().to_string();
                        if arg_str.contains('(') {
                            return arg_str
                                .replace(" ,", ",")
                                .replace("( ", "(")
                                .replace(" )", ")");
                        }
                        arg_str
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}.{}({})", formatted_object, method_call_expr.method_name, formatted_args)
            }
            ExpressionKind::MemberAccess(member_access_expr) => {
                let lhs_str = self.format_expr(member_access_expr.lhs);
                format!("{}.{}", lhs_str, member_access_expr.rhs)
            }
            ExpressionKind::Index(index_expr) => {
                let formatted_collection =
                    self.format_expr(index_expr.collection).trim_end().to_string();
                let formatted_index = self.format_expr(index_expr.index);
                format!("{}[{}]", formatted_collection, formatted_index)
            }
            ExpressionKind::Literal(literal) => match literal {
                Literal::Integer(_) => slice!(self, span.start(), span.end()).to_string(),
                Literal::Array(ArrayLiteral::Repeated { repeated_element, length }) => {
                    format!("[{}; {length}]", self.format_expr(*repeated_element))
                }
                // TODO: Handle line breaks when array gets too long.
                Literal::Array(ArrayLiteral::Standard(exprs)) => {
                    let contents: Vec<String> =
                        exprs.into_iter().map(|expr| self.format_expr(expr)).collect();
                    format!("[{}]", contents.join(", "))
                }

                Literal::Bool(_) | Literal::Str(_) | Literal::FmtStr(_) | Literal::Unit => {
                    literal.to_string()
                }
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

                    let indent = visitor.block_indent.to_string_with_newline();
                    visitor.block_indent.block_indent(self.config);
                    let nested_indent = visitor.block_indent.to_string_with_newline();

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
            self.push_str(&self.block_indent.to_string());
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
            self.block_indent.block_indent(self.config);
            let open_indent = self.block_indent.to_string();
            self.block_indent.block_unindent(self.config);
            let close_indent =
                if should_indent { self.block_indent.to_string() } else { String::new() };

            let ret = format!("{{\n{open_indent}{comment_str}\n{close_indent}}}");
            ret
        };
        self.last_position = block_span.end();
        self.push_str(&block_str);
    }
}

fn changed_comment_content(original: &str, new: &str) -> bool {
    comments(original) != comments(new)
}

fn comments(source: &str) -> Vec<String> {
    Lexer::new(source)
        .skip_comments(false)
        .flatten()
        .filter_map(|spanned| {
            if let Token::LineComment(content) | Token::BlockComment(content) = spanned.into_token()
            {
                Some(content)
            } else {
                None
            }
        })
        .collect()
}
