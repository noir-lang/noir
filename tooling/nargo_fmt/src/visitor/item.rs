use noirc_frontend::{
    parser::{Item, ItemKind},
    token::Token,
    NoirFunction, ParsedModule,
};

impl super::FmtVisitor<'_> {
    fn format_fn_before_block(&self, func: NoirFunction, start: u32) -> (String, bool) {
        let slice = self.slice(start..func.span().start());
        let force_brace_newline = slice.contains("//");
        (slice.trim_end().to_string(), force_brace_newline)
    }

    pub(crate) fn visit_file(&mut self, module: ParsedModule) {
        self.visit_module(module);
        self.format_missing_indent(self.source.len() as u32, false);
    }

    fn visit_module(&mut self, module: ParsedModule) {
        for Item { kind, span } in module.items {
            match kind {
                ItemKind::Function(func) => {
                    let (fn_before_block, force_brace_newline) =
                        self.format_fn_before_block(func.clone(), span.start());

                    self.format_missing_indent(span.start(), true);

                    self.push_str(&fn_before_block);
                    self.push_str(if force_brace_newline { "\n" } else { " " });

                    self.visit_block(func.def.body, func.def.span);
                }
                ItemKind::Submodules(module) => {
                    let name = module.name;

                    self.format_missing(span.start());

                    let after_brace = self.span_after(span, Token::LeftBrace).start();
                    self.last_position = after_brace;

                    let keyword = if module.is_contract { "contract" } else { "mod" };

                    let indent = if self.at_start()
                        || self.buffer.ends_with(|ch: char| ch.is_whitespace())
                    {
                        self.indent.to_string()
                    } else {
                        self.indent.to_string_with_newline()
                    };
                    self.push_str(&format!("{indent}{keyword} {name} "));

                    if module.contents.items.is_empty() {
                        self.visit_empty_block((after_brace - 1..span.end()).into());
                    } else {
                        self.push_str("{");
                        let indent = self.with_indent(|this| {
                            this.visit_module(module.contents);

                            let mut indent = this.indent;
                            indent.block_unindent(self.config);
                            indent.to_string_with_newline()
                        });
                        self.push_str(&format!("{indent}}}"));
                    }

                    self.last_position = span.end();
                }
                ItemKind::Import(_)
                | ItemKind::Struct(_)
                | ItemKind::Trait(_)
                | ItemKind::TraitImpl(_)
                | ItemKind::Impl(_)
                | ItemKind::TypeAlias(_)
                | ItemKind::Global(_)
                | ItemKind::ModuleDecl(_) => {
                    self.push_rewrite(self.slice(span).to_string(), span);
                    self.last_position = span.end();
                }
            }
        }
    }
}
