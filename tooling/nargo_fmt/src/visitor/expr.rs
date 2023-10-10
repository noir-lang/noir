use noirc_frontend::{
    hir::resolution::errors::Span, ArrayLiteral, BlockExpression, Expression, ExpressionKind,
    Literal, Statement,
};

use super::FmtVisitor;

impl FmtVisitor<'_> {
    pub(crate) fn visit_expr(&mut self, expr: Expression) {
        let span = expr.span;

        let rewrite = self.format_expr(expr);
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
