use crate::{
    rewrite::{self, UseTree},
    utils::{
        append_space_if_nonempty, last_line_contains_single_line_comment, last_line_used_width,
        FindToken,
    },
    visitor::expr::{format_seq, NewlineMode},
};
use noirc_frontend::{
    ast::{ItemVisibility, NoirFunction, TraitImplItemKind, UnresolvedTypeData, Visibility},
    lexer::Lexer,
    token::{SecondaryAttribute, TokenKind},
};
use noirc_frontend::{
    hir::resolution::errors::Span,
    parser::{Item, ItemKind},
    token::{Keyword, Token},
    ParsedModule,
};

use super::{
    expr::Tactic::{HorizontalVertical, LimitedHorizontalVertical},
    Shape,
};

impl super::FmtVisitor<'_> {
    fn format_fn_before_block(&self, func: NoirFunction, start: u32) -> (String, bool) {
        let name_span = func.name_ident().span();
        let func_span = func.span();

        let fn_header = self.slice(start..name_span.end());
        let mut result = self.format_fn_header(fn_header, &func);

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
        let return_type = self.format_return_type(return_type_span, &func, func_span, params_end);
        let parameters = func.def.parameters;

        if !func.def.generics.is_empty() {
            let full_span = name_span.end()..params_open;
            let start = self.span_before(full_span.clone(), Token::Less).start();
            let end = self.span_after(full_span, Token::Greater).start();

            let generics = func.def.generics;
            let span = (start..end).into();
            let generics = format_seq(
                self.shape(),
                "<",
                ">",
                self.fork(),
                false,
                generics,
                span,
                HorizontalVertical,
                NewlineMode::IfContainsNewLine,
                false,
            );

            result.push_str(&generics);
        }

        let parameters = if parameters.is_empty() {
            self.slice(params_span).into()
        } else {
            let fn_start = result
                .find_token_with(|token| {
                    matches!(token, Token::Keyword(Keyword::Fn | Keyword::Unconstrained))
                })
                .unwrap()
                .start();

            let slice = self.slice(fn_start..result.len() as u32);
            let indent = self.indent;
            let used_width = last_line_used_width(slice, indent.width());
            let overhead = if return_type.is_empty() { 2 } else { 3 }; // 2 = `()`, 3 = `() `
            let one_line_budget = self.budget(used_width + return_type.len() + overhead);
            let shape = Shape { width: one_line_budget, indent };

            let tactic = LimitedHorizontalVertical(one_line_budget);

            format_seq(
                shape,
                "(",
                ")",
                self.fork(),
                false,
                parameters,
                params_span.into(),
                tactic,
                NewlineMode::IfContainsNewLine,
                false,
            )
        };

        result.push_str(&parameters);
        result.push_str(&return_type);

        let maybe_comment = self.slice(params_end..func_span.start());

        (result.trim_end().to_string(), last_line_contains_single_line_comment(maybe_comment))
    }

    // This formats the function outer doc comments, attributes, modifiers, and `fn name`.
    fn format_fn_header(&self, src: &str, func: &NoirFunction) -> String {
        let mut result = String::new();
        let mut lexer = Lexer::new(src).skip_comments(false).peekable();

        // First there might be outer doc comments
        while let Some(Ok(token)) = lexer.peek() {
            if token.kind() == TokenKind::OuterDocComment {
                result.push_str(&token.to_string());
                result.push('\n');
                result.push_str(&self.indent.to_string());
                lexer.next();

                self.append_comments_if_any(&mut lexer, &mut result);
            } else {
                break;
            }
        }

        // Then, optionally, attributes
        while let Some(Ok(token)) = lexer.peek() {
            if token.kind() == TokenKind::Attribute {
                result.push_str(&token.to_string());
                result.push('\n');
                result.push_str(&self.indent.to_string());
                lexer.next();

                self.append_comments_if_any(&mut lexer, &mut result);
            } else {
                break;
            }
        }

        self.append_comments_if_any(&mut lexer, &mut result);

        // Then, optionally, the `unconstrained` keyword
        // (eventually we'll stop accepting this, but we keep it for backwards compatibility)
        if let Some(Ok(token)) = lexer.peek() {
            if let Token::Keyword(Keyword::Unconstrained) = token.token() {
                lexer.next();
            }
        }

        self.append_comments_if_any(&mut lexer, &mut result);

        // Then the visibility
        let mut has_visibility = false;
        if let Some(Ok(token)) = lexer.peek() {
            if let Token::Keyword(Keyword::Pub) = token.token() {
                has_visibility = true;
                lexer.next();
                if let Some(Ok(token)) = lexer.peek() {
                    if let Token::LeftParen = token.token() {
                        lexer.next(); // Skip '('
                        lexer.next(); // Skip 'crate'
                        lexer.next(); // Skip ')'
                    }
                }
            }
        }

        if has_visibility {
            result.push_str(&func.def.visibility.to_string());
            result.push(' ');
        }

        self.append_comments_if_any(&mut lexer, &mut result);

        // Then, optionally, and again, the `unconstrained` keyword
        if let Some(Ok(token)) = lexer.peek() {
            if let Token::Keyword(Keyword::Unconstrained) = token.token() {
                lexer.next();
            }
        }

        if func.def.is_unconstrained {
            result.push_str("unconstrained ");
        }

        self.append_comments_if_any(&mut lexer, &mut result);

        // Then, optionally, the `comptime` keyword
        if let Some(Ok(token)) = lexer.peek() {
            if let Token::Keyword(Keyword::Comptime) = token.token() {
                lexer.next();
            }
        }

        if func.def.is_comptime {
            result.push_str("comptime ");
        }

        self.append_comments_if_any(&mut lexer, &mut result);

        // Then the `fn` keyword
        lexer.next(); // Skip fn
        result.push_str("fn ");

        self.append_comments_if_any(&mut lexer, &mut result);

        // Then the function name
        result.push_str(&func.def.name.0.contents);

        result
    }

    fn append_comments_if_any(
        &self,
        lexer: &mut std::iter::Peekable<Lexer<'_>>,
        result: &mut String,
    ) {
        while let Some(Ok(token)) = lexer.peek() {
            match token.token() {
                Token::LineComment(..) => {
                    result.push_str(&token.to_string());
                    result.push('\n');
                    result.push_str(&self.indent.to_string());
                    lexer.next();
                }
                Token::BlockComment(..) => {
                    result.push_str(&token.to_string());
                    lexer.next();
                }
                _ => break,
            }
        }
    }

    fn format_return_type(
        &self,
        span: Span,
        func: &NoirFunction,
        func_span: Span,
        params_end: u32,
    ) -> String {
        let mut result = String::new();

        if func.return_type().typ == UnresolvedTypeData::Unit {
            result.push_str(self.slice(params_end..func_span.start()));
        } else {
            result.push_str(" -> ");

            let visibility = match func.def.return_visibility {
                Visibility::Public => "pub",
                Visibility::ReturnData => "return_data",
                Visibility::Private => "",
                Visibility::CallData(_) => {
                    unreachable!("call_data cannot be used for return value")
                }
            };
            result.push_str(&append_space_if_nonempty(visibility.into()));

            let typ = rewrite::typ(self, self.shape(), func.return_type());
            result.push_str(&typ);

            let slice = self.slice(span.end()..func_span.start());
            if !slice.trim().is_empty() {
                result.push_str(slice);
            }
        }

        result
    }

    pub(crate) fn visit_file(&mut self, module: ParsedModule) {
        self.visit_module(module);
        self.format_missing_indent(self.source.len() as u32, false);
    }

    fn visit_module(&mut self, module: ParsedModule) {
        for Item { kind, span, doc_comments } in module.items {
            match kind {
                ItemKind::Function(func) => {
                    self.visit_function(span, func);
                }
                ItemKind::Submodules(module) => {
                    self.format_missing_indent(span.start(), true);

                    if std::mem::take(&mut self.ignore_next_node) {
                        self.push_str(self.slice(span));
                        self.last_position = span.end();
                        continue;
                    }

                    for doc_comment in doc_comments {
                        self.push_str(&format!("///{doc_comment}\n"));
                        self.push_str(&self.indent.to_string());
                    }

                    for attribute in module.outer_attributes {
                        let is_tag = matches!(attribute, SecondaryAttribute::Tag(_));
                        let tag = if is_tag { "'" } else { "" };
                        self.push_str(&format!("#[{tag}{}]\n", attribute.as_ref()));
                        self.push_str(&self.indent.to_string());
                    }

                    let name = module.name;
                    let after_brace = self.span_after(span, Token::LeftBrace).start();
                    self.last_position = after_brace;

                    let visibility = module.visibility;
                    if visibility != ItemVisibility::Private {
                        self.push_str(&format!("{visibility} "));
                    }

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
                ItemKind::Impl(impl_) => {
                    self.format_missing_indent(span.start(), true);

                    if std::mem::take(&mut self.ignore_next_node) {
                        self.push_str(self.slice(span));
                        self.last_position = span.end();
                        continue;
                    }

                    let before_brace = self.span_before(span, Token::LeftBrace).start();
                    let slice = self.slice(self.last_position..before_brace).trim();
                    let after_brace = self.span_after(span, Token::LeftBrace).start();
                    self.last_position = after_brace;

                    self.push_str(&format!("{slice} "));

                    if impl_.methods.is_empty() {
                        self.visit_empty_block((after_brace - 1..span.end()).into());
                        continue;
                    } else {
                        self.push_str("{");
                        self.indent.block_indent(self.config);

                        for (method, span) in impl_.methods {
                            self.visit_function(span, method.item);
                        }

                        self.close_block((self.last_position..span.end() - 1).into());
                        self.last_position = span.end();
                    }
                }
                ItemKind::TraitImpl(noir_trait_impl) => {
                    self.format_missing_indent(span.start(), true);

                    if std::mem::take(&mut self.ignore_next_node) {
                        self.push_str(self.slice(span));
                        self.last_position = span.end();
                        continue;
                    }

                    let before_brace = self.span_before(span, Token::LeftBrace).start();
                    let slice = self.slice(self.last_position..before_brace).trim();
                    let after_brace = self.span_after(span, Token::LeftBrace).start();
                    self.last_position = after_brace;

                    self.push_str(&format!("{slice} "));

                    if noir_trait_impl.items.is_empty() {
                        self.visit_empty_block((after_brace - 1..span.end()).into());
                        continue;
                    } else {
                        self.push_str("{");
                        self.indent.block_indent(self.config);

                        for documented_item in noir_trait_impl.items {
                            let span = documented_item.item.span;
                            match documented_item.item.kind {
                                TraitImplItemKind::Function(method) => {
                                    self.visit_function(span, method);
                                }
                                TraitImplItemKind::Constant(..)
                                | TraitImplItemKind::Type { .. } => {
                                    self.push_rewrite(self.slice(span).to_string(), span);
                                    self.last_position = span.end();
                                }
                            }
                        }

                        self.close_block((self.last_position..span.end() - 1).into());
                        self.last_position = span.end();
                    }
                }
                ItemKind::Import(use_tree, visibility) => {
                    let use_tree = UseTree::from_ast(use_tree);
                    let use_tree = use_tree.rewrite_top_level(self, self.shape(), visibility);
                    self.push_rewrite(use_tree, span);
                    self.last_position = span.end();
                }

                ItemKind::Struct(_)
                | ItemKind::Trait(_)
                | ItemKind::TypeAlias(_)
                | ItemKind::Global(..)
                | ItemKind::ModuleDecl(_)
                | ItemKind::InnerAttribute(_) => {
                    self.push_rewrite(self.slice(span).to_string(), span);
                    self.last_position = span.end();
                }
            }
        }
    }

    fn visit_function(&mut self, span: Span, func: NoirFunction) {
        self.format_missing_indent(span.start(), true);
        if std::mem::take(&mut self.ignore_next_node) {
            self.push_str(self.slice(span));
            self.last_position = span.end();
            return;
        }
        let (fn_before_block, force_brace_newline) =
            self.format_fn_before_block(func.clone(), span.start());
        self.push_str(&fn_before_block);
        self.push_str(if force_brace_newline { "\n" } else { " " });
        self.visit_block(func.def.body, func.def.span);
    }
}
