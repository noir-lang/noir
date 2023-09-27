use noirc_frontend::{
    hir::resolution::errors::Span,
    parser::{ItemKind, ParsedModule},
    BlockExpression, Expression, ExpressionKind, NoirFunction, Statement, StatementKind,
};

use crate::config::Config;

/// A macro to create a slice from a given data source, helping to avoid borrow checker errors.
macro_rules! slice {
    ($this:ident, $start:expr, $end:expr) => {
        &$this.source[$start as usize..$end as usize]
    };
}

pub(crate) struct FmtVisitor<'a> {
    config: Config,
    buffer: String,
    source: &'a str,
    block_indent: Indent,
    last_position: u32,
}

impl<'a> FmtVisitor<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            config: Config::default(),
            buffer: String::new(),
            source,
            last_position: 0,
            block_indent: Indent { block_indent: 0 },
        }
    }

    pub(crate) fn finish(self) -> String {
        self.buffer
    }

    fn at_start(&self) -> bool {
        self.buffer.is_empty()
    }

    fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    #[track_caller]
    fn push_rewrite(&mut self, s: String, span: Span) {
        self.format_missing_indent(span.start(), true);
        self.push_str(&s);
    }

    pub(crate) fn visit_module(&mut self, module: ParsedModule) {
        for item in module.items {
            match item.kind {
                ItemKind::Function(func) => {
                    let (fn_before_block, force_brace_newline) =
                        self.rewrite_fn_before_block(func.clone(), item.span.start());

                    self.format_missing_indent(item.span.start(), false);

                    self.push_str(&fn_before_block);
                    self.push_str(if force_brace_newline { "\n" } else { " " });

                    self.visit_block(func.def.body, func.def.span, false);
                }
                _ => {
                    self.format_missing_indent(item.span.end(), false);
                }
            }
        }

        self.format_missing_indent(self.source.len() as u32, false);
    }

    fn visit_block(&mut self, block: BlockExpression, block_span: Span, should_indent: bool) {
        if block.is_empty() {
            let slice = slice!(self, block_span.start(), block_span.end());
            let comment_str = slice[1..slice.len() - 1].trim();

            let block_str = if comment_str.is_empty() {
                "{}".to_string()
            } else {
                let indent = self.block_indent.to_string();
                format!("{{\n{indent}{comment_str}\n{indent}}}",)
            };

            self.last_position = block_span.end();
            self.push_str(&block_str);
            return;
        }

        self.last_position = block_span.start() + 1; // `{`
        self.block_indent.block_indent(&self.config);
        self.push_str("{");

        self.visit_stmts(block.0);

        self.last_position = block_span.end();
        self.block_indent.block_unindent(&self.config);

        self.push_str("\n");
        if should_indent {
            self.push_str(&self.block_indent.to_string());
        }
        self.push_str("}");
    }

    fn visit_stmts(&mut self, stmts: Vec<Statement>) {
        for stmt in stmts {
            match dbg!(stmt.kind) {
                StatementKind::Expression(expr) => self.visit_expr(expr),
                StatementKind::Semi(expr) => {
                    self.visit_expr(expr);
                    self.push_str(";");
                }
                StatementKind::Error => unreachable!(),
                _ => {
                    self.format_missing_inner(stmt.span.end(), |this, slice, _| {
                        this.push_str(slice);
                    });
                }
            }

            self.last_position = stmt.span.end();
        }
    }

    fn visit_expr(&mut self, expr: Expression) {
        let span = expr.span;

        let rewrite = self.format_expr(expr);
        dbg!(&rewrite);
        self.push_rewrite(rewrite, span);

        self.last_position = span.end();
    }

    fn format_expr(&self, Expression { kind, span }: Expression) -> String {
        match kind {
            ExpressionKind::Block(block) => {
                let mut visitor = FmtVisitor::new(self.source);

                visitor.block_indent = self.block_indent;
                visitor.visit_block(block, span, true);

                visitor.buffer
            }
            ExpressionKind::Prefix(prefix) => {
                format!("{}{}", prefix.operator, self.format_expr(prefix.rhs))
            }
            // TODO:
            _expr => slice!(self, span.start(), span.end()).to_string(),
        }
    }

    #[track_caller]
    fn format_missing_indent(&mut self, end: u32, should_indent: bool) {
        self.format_missing_inner(end, |this, last_slice, slice| {
            this.push_str(last_slice.trim_end());

            if last_slice == slice && !this.at_start() {
                this.push_str("\n");
            }

            if should_indent {
                let indent = this.block_indent.to_string();
                this.push_str(&indent);
            }
        });
    }

    #[track_caller]
    fn format_missing_inner(
        &mut self,
        end: u32,
        process_last_slice: impl Fn(&mut Self, &str, &str),
    ) {
        let start = self.last_position;

        if start == end {
            if !self.at_start() {
                process_last_slice(self, "", "");
            }
            return;
        }

        let slice = slice!(self, start, end);
        self.last_position = end;

        if slice.trim().is_empty() && !self.at_start() {
            self.push_str("\n");
            process_last_slice(self, "", slice);
        } else {
            process_last_slice(self, slice, slice);
        }
    }

    fn rewrite_fn_before_block(&self, func: NoirFunction, start: u32) -> (String, bool) {
        let slice = slice!(self, start, func.span().start());
        let force_brace_newline = slice.contains("//");
        (slice.trim_end().to_string(), force_brace_newline)
    }
}

#[derive(Clone, Copy)]
struct Indent {
    block_indent: usize,
}

impl Indent {
    fn block_indent(&mut self, config: &Config) {
        self.block_indent += config.tab_spaces;
    }

    fn block_unindent(&mut self, config: &Config) {
        self.block_indent -= config.tab_spaces;
    }

    #[allow(clippy::inherent_to_string)]
    fn to_string(self) -> String {
        " ".repeat(self.block_indent)
    }
}
