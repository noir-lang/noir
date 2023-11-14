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
                    self.format_missing_indent(span.start(), true);

                    if std::mem::take(&mut self.ignore_next_node) {
                        self.push_str(self.slice(span));
                        self.last_position = span.end();
                        continue;
                    }

                    let (fn_before_block, force_brace_newline) =
                        self.format_fn_before_block(func.clone(), span.start());

                    self.push_str(&fn_before_block);
                    self.push_str(if force_brace_newline { "\n" } else { " " });

                    self.visit_block(func.def.body, func.def.span);
                }
                ItemKind::Submodules(module) => {
                    self.format_missing_indent(span.start(), true);

                    if std::mem::take(&mut self.ignore_next_node) {
                        self.push_str(self.slice(span));
                        self.last_position = span.end();
                        continue;
                    }

                    let name = module.name;
                    let after_brace = self.span_after(span, Token::LeftBrace).start();
                    self.last_position = after_brace;

                    let keyword = if module.is_contract { "contract" } else { "mod" };

                    self.push_str(&format!("{keyword} {name} "));

                    if module.contents.items.is_empty() {
                        self.visit_empty_block((after_brace - 1..span.end()).into());
                        continue;
                    } else {
                        self.push_str("{");
                        self.indent.block_indent(self.config);
                        self.visit_module(module.contents);
                    }

                    self.close_block((self.last_position..span.end() - 1).into());
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
