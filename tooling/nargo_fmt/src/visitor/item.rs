use noirc_frontend::{
    parser::{Item, ItemKind},
    ParsedModule,
};

impl super::FmtVisitor<'_> {
    pub(crate) fn visit_module(&mut self, module: ParsedModule) {
        for Item { kind, span } in module.items {
            match kind {
                ItemKind::Function(func) => {
                    let (fn_before_block, force_brace_newline) =
                        self.format_fn_before_block(func.clone(), span.start());

                    self.format_missing_indent(span.start(), false);

                    self.push_str(&fn_before_block);
                    self.push_str(if force_brace_newline { "\n" } else { " " });

                    self.visit_block(func.def.body, func.def.span, false);
                }
                _ => self.format_missing(span.end()),
            }
        }

        self.format_missing_indent(self.source.len() as u32, false);
    }
}
