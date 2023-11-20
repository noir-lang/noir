use noirc_frontend::{
    parser::{Item, ItemKind},
    token::Token,
    Distinctness, NoirFunction, ParsedModule, Visibility,
};

use crate::{utils::last_line_contains_single_line_comment, visitor::expr::format_seq};

use super::expr::Tactic::LimitedHorizontalVertical;

impl super::FmtVisitor<'_> {
    fn format_fn_before_block(&self, func: NoirFunction, start: u32) -> (String, bool) {
        let tactic = LimitedHorizontalVertical(self.config.max_width);

        let name_span = func.name_ident().span();
        let func_span = func.span();

        let mut result = self.slice(start..name_span.end()).to_owned();

        let params_open =
            self.span_before(name_span.end()..func_span.start(), Token::LeftParen).start();

        let last_span = if func.parameters().is_empty() {
            params_open..func_span.start()
        } else {
            func.parameters().last().unwrap().span.end()..func_span.start()
        };

        let params_end = self.span_after(last_span, Token::RightParen).start();

        let params_span = params_open..params_end;
        let return_type_span = func.return_type().span;
        let parameters = func.def.parameters;

        if !func.def.generics.is_empty() {
            let full_span = name_span.end()..params_open;
            let start = name_span.end();
            let end = self.span_after(full_span, Token::Greater).start();

            let generics = func.def.generics;
            let span = (start..end).into();
            let generics = format_seq("<", ">", self.fork(), false, generics, span, tactic, false);

            result.push_str(&generics);
        }

        let parameters = if parameters.is_empty() {
            self.slice(params_span).into()
        } else {
            format_seq("(", ")", self.fork(), false, parameters, params_span.into(), tactic, true)
        };

        result.push_str(&parameters);

        if let Some(span) = return_type_span {
            result.push_str(" -> ");

            if let Distinctness::Distinct = func.def.return_distinctness {
                result.push_str("distinct ");
            }

            if let Visibility::Public = func.def.return_visibility {
                result.push_str("pub ");
            }

            result.push_str(self.slice(span));

            let slice = self.slice(span.end()..func_span.start());
            if !slice.trim().is_empty() {
                result.push_str(slice);
            }
        } else {
            result.push_str(self.slice(params_end..func_span.start()));
        }

        let maybe_comment = self.slice(params_end..func_span.start());

        (result.trim_end().to_string(), last_line_contains_single_line_comment(maybe_comment))
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
