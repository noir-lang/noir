use noirc_frontend::{
    hir::resolution::errors::Span, lexer::Lexer, token::Token, ArrayLiteral, BlockExpression,
    Expression, ExpressionKind, Literal, Statement,
};

use super::FmtVisitor;

impl FmtVisitor<'_> {
    pub(crate) fn visit_expr(&mut self, expr: Expression) {
        let span = expr.span;

        let rewrite = self.format_expr(expr);
        let rewrite = recover_comment_removed(slice!(self, span.start(), span.end()), rewrite);
        self.push_rewrite(rewrite, span);

        self.last_position = span.end();
    }

    fn format_expr(&self, Expression { kind, span }: Expression) -> String {
        match kind {
            ExpressionKind::Block(block) => {
                let mut visitor = FmtVisitor::new(self.source, self.config);

                visitor.block_indent = self.block_indent;
                visitor.visit_block(block, span, true);

                visitor.buffer
            }
            ExpressionKind::Prefix(prefix) => {
                format!("{}{}", prefix.operator, self.format_expr(prefix.rhs))
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
            ExpressionKind::Tuple(elements) => {
                let mut visitor = FmtVisitor::new(self.source, self.config);
                visitor.block_indent = self.block_indent;
                visitor.block_indent.block_indent(self.config);

                let mut elements =
                    TupleElements::new(&mut visitor, span, elements).collect::<Vec<_>>();

                let elements = if elements.len() == 1 {
                    format!("{},", elements.pop().unwrap())
                } else {
                    elements.join(", ")
                };

                format!("({elements})")
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
            ExpressionKind::Parenthesized(subexpr) => format!("({})", self.format_expr(*subexpr)),
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

fn recover_comment_removed(original: &str, new: String) -> String {
    if changed_comment_content(original, &new) {
        original.to_string()
    } else {
        new
    }
}

fn changed_comment_content(original: &str, new: &str) -> bool {
    comments(original).ne(comments(new))
}

fn comments(source: &str) -> impl Iterator<Item = String> + '_ {
    Lexer::new(source).skip_comments(false).flatten().filter_map(|spanned| {
        if let Token::LineComment(content) | Token::BlockComment(content) = spanned.into_token() {
            Some(content)
        } else {
            None
        }
    })
}

struct TupleElements<'me> {
    visitor: &'me mut FmtVisitor<'me>,
    elements: std::iter::Peekable<std::vec::IntoIter<Expression>>,
    last_position: u32,
    end_position: u32,
    emit_newline: bool,
}

impl<'me> TupleElements<'me> {
    fn new(visitor: &'me mut FmtVisitor<'me>, span: Span, elements: Vec<Expression>) -> Self {
        Self {
            visitor,
            last_position: span.start() + 1, /*(*/
            end_position: span.end() - 1,    /*)*/
            elements: elements.into_iter().peekable(),
            emit_newline: false,
        }
    }
}

impl Iterator for TupleElements<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.elements.next()?;
        let element_span = element.span;

        let start = self.last_position;
        let end = element_span.start();

        let next_start = self.elements.peek().map_or(self.end_position, |expr| expr.span.start());

        let leading = {
            let mut emit_newline = false;

            let leading = slice!(self.visitor, start, end);
            let leading_trimmed = slice!(self.visitor, start, end).trim();

            let starts_with_block_comment = leading_trimmed.starts_with("/*");
            let ends_with_block_comment = leading_trimmed.ends_with("*/");
            let starts_with_single_line_comment = leading_trimmed.starts_with("//");

            if ends_with_block_comment {
                let comment_end = leading_trimmed.rfind(|c| c == '/').unwrap();

                if leading[comment_end..].contains('\n') {
                    emit_newline = true;
                }
            } else if starts_with_single_line_comment || starts_with_block_comment {
                emit_newline = true;
            };

            if emit_newline {
                self.emit_newline = true;
            }

            let newline = if emit_newline { "\n" } else { "" };
            let indent =
                if emit_newline { self.visitor.block_indent.to_string() } else { String::new() };
            format!("{newline}{indent}{leading_trimmed}{newline}{indent}")
        };

        let trailing = slice!(self.visitor, element_span.end(), next_start);
        let element_str = self.visitor.format_expr(element);

        let end = trailing.find_token(Token::Comma).unwrap_or(trailing.len() as u32);

        let trailing = trailing[..end as usize].trim_matches(',').trim();
        self.last_position = element_span.end() + end;

        let newline = if self.end_position == next_start && self.emit_newline {
            self.visitor.block_indent.block_unindent(self.visitor.config);
            format!("\n{}", self.visitor.block_indent.to_string())
        } else {
            String::new()
        };

        dbg!(format!("{leading}{element_str}{trailing}{newline}").into())
    }
}

trait FindToken {
    fn find_token(&self, token: Token) -> Option<u32>;
}

impl FindToken for str {
    fn find_token(&self, token: Token) -> Option<u32> {
        Lexer::new(self).flatten().find_map(|it| (it.token() == &token).then(|| it.to_span().end()))
    }
}
