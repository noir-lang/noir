//! Handles LSP semantic tokens requests.
//!
//! Semantic tokens in Noir are provided in two contexts:
//! - links inside doc comments. For example,a doc comment that has `[println]` in it will
//!   be colorized as a function reference (only if such function actually exists).
//! - code blocks inside doc comments. If these are Noir or Rust code blocks, a Lexer
//!   will be used to colorize keywords and such.
use std::{collections::HashMap, future};

use async_lsp::{
    ResponseError,
    lsp_types::{
        self, Position, SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensParams,
        SemanticTokensResult, TextDocumentPositionParams,
    },
};
use fm::FileId;
use nargo_doc::links::{LinkFinder, LinkTarget};
use noirc_errors::Span;
use noirc_frontend::{
    ast::{
        LetStatement, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait, TraitItem, TypeAlias,
        Visitor,
    },
    elaborator::PrimitiveType,
    hir::def_map::{LocalModuleId, ModuleDefId, ModuleId},
    lexer::Lexer,
    node_interner::ReferenceId,
    parser::ParsedSubModule,
    token::{Keyword, LocatedToken, Token},
};

use crate::{
    LspState,
    doc_comments::current_module_and_type,
    requests::{
        ProcessRequestCallbackArgs, process_request, semantic_token_types_map, to_lsp_location,
    },
};

pub(crate) fn on_semantic_tokens_full_request(
    state: &mut LspState,
    params: SemanticTokensParams,
) -> impl Future<Output = Result<Option<SemanticTokensResult>, ResponseError>> + use<> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    let result = process_request(state, text_document_position_params, |args| {
        let file_id = args.location.file;
        let file = args.files.get_file(file_id).unwrap();
        let source = file.source();
        let (parsed_module, _errors) = noirc_frontend::parse_program(source, file_id);

        let mut collector = SemanticTokenCollector::new(source, file_id, &args);
        let tokens = collector.collect(&parsed_module);
        Some(SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data: tokens }))
    });
    future::ready(result)
}

struct SemanticTokenCollector<'args> {
    source: &'args str,
    args: &'args ProcessRequestCallbackArgs<'args>,
    file_id: FileId,
    link_finder: LinkFinder,
    tokens: Vec<SemanticToken>,
    previous_line: u32,
    previous_char: u32,
    token_types: HashMap<SemanticTokenType, usize>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CodeBlock {
    None,
    Noir,
    Other,
}

impl<'args> SemanticTokenCollector<'args> {
    fn new(
        source: &'args str,
        file_id: FileId,
        args: &'args ProcessRequestCallbackArgs<'args>,
    ) -> Self {
        let link_finder = LinkFinder::default();
        let tokens = Vec::new();
        let token_types = semantic_token_types_map();
        SemanticTokenCollector {
            source,
            args,
            file_id,
            link_finder,
            tokens,
            previous_line: 0,
            previous_char: 0,
            token_types,
        }
    }

    fn collect(&mut self, parsed_module: &noirc_frontend::ParsedModule) -> Vec<SemanticToken> {
        // Find the module the current file belongs to
        let krate = self.args.crate_id;
        let def_map = &self.args.def_maps[&krate];
        let local_id = if let Some((module_index, _)) = def_map
            .modules()
            .iter()
            .find(|(_, module_data)| module_data.location.file == self.file_id)
        {
            LocalModuleId::new(module_index)
        } else {
            def_map.root()
        };
        let module_id = ModuleId { krate, local_id };

        // Process doc comments on the module itself
        self.process_reference_id(ReferenceId::Module(module_id));

        parsed_module.accept(self);
        std::mem::take(&mut self.tokens)
    }

    /// Checks doc comments on the given ReferenceId. Semantic tokens are produced for any links found,
    /// so that they can be colorized in the editor.
    fn process_reference_id(&mut self, id: ReferenceId) {
        let Some(doc_comments) = self.args.interner.doc_comments(id) else {
            return;
        };
        let Some((current_module_id, current_type)) = current_module_and_type(id, self.args) else {
            return;
        };

        let mut code_block = CodeBlock::None;

        self.link_finder.reset();
        for located_comment in doc_comments {
            let location = located_comment.location();
            if location.file != self.file_id {
                // A module's comments might happen inline in the same file or in a different file.
                // We should not process comments that are not in the current file.
                continue;
            }

            let contents = located_comment.contents.trim();
            let mut fence = false;

            match code_block {
                CodeBlock::None => {
                    if contents == "```" || contents == "```noir" || contents == "```rust" {
                        code_block = CodeBlock::Noir;
                        fence = true;
                    } else if contents.starts_with("```") {
                        code_block = CodeBlock::Other;
                        fence = true;
                    }
                }
                CodeBlock::Noir | CodeBlock::Other => {
                    if contents == "```" {
                        code_block = CodeBlock::None;
                        fence = true;
                    }
                }
            }

            let location = located_comment.location();
            let Some(lsp_location) = to_lsp_location(self.args.files, location.file, location.span)
            else {
                continue;
            };
            let start_line = lsp_location.range.start.line;
            let start_char = lsp_location.range.start.character;

            // Read comments from source based on location: the comments in `located_comment` might
            // have been slightly adjusted.
            let comments =
                &self.source[location.span.start() as usize..location.span.end() as usize];

            if code_block == CodeBlock::Noir && !fence {
                self.colorize_code_block_line(lsp_location, comments);
            }

            let links = self.link_finder.find_links(
                comments,
                current_module_id,
                current_type,
                self.args.interner,
                self.args.def_maps,
                self.args.crate_graph,
            );
            for link in links {
                let Some(target) = link.target else {
                    continue;
                };

                let Some(token_type) = self.link_target_token_type(target) else {
                    continue;
                };
                let token_type = self.token_types[&token_type] as u32;
                let delta_line = start_line + link.line as u32;
                let delta_start =
                    if link.line == 0 { start_char + link.start as u32 } else { link.start as u32 };
                let length = (link.end - link.start) as u32;
                let token = SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset: 0,
                };
                self.push_token(token);
            }
        }
    }

    fn colorize_code_block_line(&mut self, lsp_location: lsp_types::Location, line: &str) {
        // The code block line will start with either "///", "//!", or optionally "*".
        // We remove it, but then we'll need to add an offset when calculating the character position.
        let (line, offset) = if let Some(line) = line.strip_prefix("///") {
            (line, 3)
        } else if let Some(line) = line.strip_prefix("//!") {
            (line, 3)
        } else if let Some(line) = line.strip_prefix("*") {
            (line, 1)
        } else {
            (line, 0)
        };
        let lexer = Lexer::new_with_dummy_file(line);
        let mut tokens = Vec::new();

        for token in lexer {
            let Ok(token) = token else {
                // If lexing fails, give up
                return;
            };
            if matches!(token.token(), Token::EOF) {
                break;
            }

            tokens.push(token);
        }

        for (index, token) in tokens.iter().enumerate() {
            let previous_token = if index == 0 { None } else { Some(&tokens[index - 1]) };
            let next_token = tokens.get(index + 1);
            let next_next_token = tokens.get(index + 2);
            self.colorize_token(
                token,
                previous_token,
                next_token,
                next_next_token,
                &lsp_location,
                offset,
            );
        }
    }

    fn colorize_token(
        &mut self,
        token: &LocatedToken,
        previous_token: Option<&LocatedToken>,
        next_token: Option<&LocatedToken>,
        next_next_token: Option<&LocatedToken>,
        location: &lsp_types::Location,
        offset: usize,
    ) {
        let span = token.span();
        let token = token.token();
        let semantic_token_type = match token {
            Token::Int(..) => SemanticTokenType::NUMBER,
            Token::Bool(_) => SemanticTokenType::KEYWORD,
            Token::Str(_) | Token::RawStr(_, _) | Token::FmtStr(..) => SemanticTokenType::STRING,
            Token::Keyword(_) => SemanticTokenType::KEYWORD,
            Token::LineComment(..) | Token::BlockComment(..) => SemanticTokenType::COMMENT,
            Token::Quote(tokens) => {
                // Colorize "quote"
                let semantic_token_type = self.token_types[&SemanticTokenType::KEYWORD] as u32;
                let sematic_token = SemanticToken {
                    delta_line: location.range.start.line,
                    delta_start: location.range.start.character + span.start() + offset as u32,
                    length: 5,
                    token_type: semantic_token_type,
                    token_modifiers_bitset: 0,
                };
                self.push_token(sematic_token);

                for (index, token) in tokens.0.iter().enumerate() {
                    let previous_token = if index == 0 { None } else { Some(&tokens.0[index - 1]) };
                    let next_token = tokens.0.get(index + 1);
                    let next_next_token = tokens.0.get(index + 2);
                    self.colorize_token(
                        token,
                        previous_token,
                        next_token,
                        next_next_token,
                        location,
                        offset,
                    );
                }
                return;
            }
            Token::Ident(name) => {
                if name == "self" {
                    SemanticTokenType::KEYWORD
                } else if name.chars().next().is_some_and(|char| char.is_ascii_uppercase())
                    || PrimitiveType::lookup_by_name(name).is_some()
                {
                    // Heuristic: if the name starts with an uppercase letter, or it denotes a primitive type,
                    // colorize it as a struct
                    SemanticTokenType::STRUCT
                } else if next_token.is_some_and(|token| matches!(token.token(), Token::LeftParen))
                    || previous_token
                        .is_some_and(|token| matches!(token.token(), Token::Keyword(Keyword::Fn)))
                {
                    // Heuristic colorize "foo" in "foo(" and "fn foo" as a function
                    SemanticTokenType::FUNCTION
                } else if next_token
                    .is_some_and(|token| matches!(token.token(), Token::DoubleColon))
                {
                    // Heuristic: colorize "foo" in "foo::" as a module.
                    // However, if it's "foo::<", colorize it as either a struct or a function depending
                    // on whether it starts with uppercase or not.
                    if next_next_token.is_some_and(|token| matches!(token.token(), Token::Less)) {
                        if name.chars().next().is_some_and(|char| char.is_ascii_uppercase()) {
                            SemanticTokenType::STRUCT
                        } else {
                            SemanticTokenType::FUNCTION
                        }
                    } else {
                        SemanticTokenType::NAMESPACE
                    }
                } else if previous_token.is_some_and(|token| {
                    matches!(
                        token.token(),
                        Token::Keyword(Keyword::Struct)
                            | Token::Keyword(Keyword::Enum)
                            | Token::Keyword(Keyword::Impl)
                            | Token::Keyword(Keyword::Trait)
                            | Token::Keyword(Keyword::Type)
                    )
                }) {
                    // Heuristic: colorize "foo" in "struct foo", "enum foo", etc., as a struct
                    SemanticTokenType::STRUCT
                } else {
                    SemanticTokenType::VARIABLE
                }
            }
            Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Equal
            | Token::NotEqual
            | Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Ampersand
            | Token::Caret
            | Token::ShiftLeft
            | Token::ShiftRight
            | Token::LeftParen
            | Token::RightParen
            | Token::LeftBrace
            | Token::RightBrace
            | Token::LeftBracket
            | Token::RightBracket
            | Token::Pipe
            | Token::Assign
            | Token::Arrow
            | Token::FatArrow
            | Token::LogicalAnd
            | Token::Comma
            | Token::AttributeStart { .. }
            | Token::Semicolon => SemanticTokenType::OPERATOR,
            Token::QuotedType(_) => SemanticTokenType::STRUCT,
            Token::InternedExpr(..)
            | Token::InternedStatement(..)
            | Token::InternedLValue(..)
            | Token::InternedUnresolvedTypeData(..)
            | Token::InternedPattern(..)
            | Token::InternedCrate(..)
            | Token::Dot
            | Token::DoubleDot
            | Token::DoubleDotEqual
            | Token::Pound
            | Token::Colon
            | Token::DoubleColon
            | Token::Backslash
            | Token::Bang
            | Token::DollarSign
            | Token::At
            | Token::DeprecatedVectorStart
            | Token::EOF
            | Token::Whitespace(_)
            | Token::UnquoteMarker(_)
            | Token::Invalid(_) => return,
        };
        let semantic_token_type = self.token_types[&semantic_token_type] as u32;
        let sematic_token = SemanticToken {
            delta_line: location.range.start.line,
            delta_start: location.range.start.character + span.start() + offset as u32,
            length: span.end() - span.start(),
            token_type: semantic_token_type,
            token_modifiers_bitset: 0,
        };
        self.push_token(sematic_token);
    }

    fn link_target_token_type(&self, target: LinkTarget) -> Option<SemanticTokenType> {
        let token_type = match target {
            LinkTarget::TopLevelItem(module_def_id) => match module_def_id {
                ModuleDefId::ModuleId(_) => SemanticTokenType::NAMESPACE,
                ModuleDefId::FunctionId(func_id) => {
                    let func_meta = self.args.interner.function_meta(&func_id);
                    if func_meta.self_type.is_some() {
                        SemanticTokenType::METHOD
                    } else {
                        SemanticTokenType::FUNCTION
                    }
                }
                ModuleDefId::TypeId(_) => SemanticTokenType::STRUCT,
                ModuleDefId::TypeAliasId(_) => SemanticTokenType::STRUCT,
                ModuleDefId::TraitId(_) => SemanticTokenType::INTERFACE,
                ModuleDefId::TraitAssociatedTypeId(_) => return None,
                ModuleDefId::GlobalId(_) => SemanticTokenType::VARIABLE,
            },
            LinkTarget::Method(..) => SemanticTokenType::METHOD,
            LinkTarget::StructMember(..) => SemanticTokenType::PROPERTY,
            LinkTarget::PrimitiveType(_) => return None,
            LinkTarget::PrimitiveTypeFunction(..) => SemanticTokenType::FUNCTION,
        };
        Some(token_type)
    }

    fn push_token(&mut self, mut token: SemanticToken) {
        let previous_line = std::mem::replace(&mut self.previous_line, token.delta_line);
        let previous_char = std::mem::replace(&mut self.previous_char, token.delta_start);

        // See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_semanticTokens
        // for an explanation of delta_line and delta_start.
        let same_line = token.delta_line == previous_line;
        token.delta_line -= previous_line;
        if same_line {
            token.delta_start -= previous_char;
        }

        self.tokens.push(token);
    }
}

// Visit every AST node that can have doc comments on it. Links are then searched in these.
impl Visitor for SemanticTokenCollector<'_> {
    fn visit_parsed_submodule(&mut self, module: &ParsedSubModule, _: Span) -> bool {
        let name_location = module.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };

        true
    }

    fn visit_noir_function(&mut self, function: &NoirFunction, _span: Span) -> bool {
        let name_location = function.name_ident().location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };

        false
    }

    fn visit_noir_struct(&mut self, noir_struct: &NoirStruct, _: Span) -> bool {
        let name_location = noir_struct.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };

        for field in noir_struct.fields.iter() {
            let field_name_location = field.item.name.location();
            if let Some(reference) = self.args.interner.reference_at_location(field_name_location) {
                self.process_reference_id(reference);
            };
        }

        false
    }

    fn visit_noir_enum(&mut self, noir_enum: &NoirEnumeration, _: Span) -> bool {
        let name_location = noir_enum.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };

        for variant in noir_enum.variants.iter() {
            let variant_name_location = variant.item.name.location();
            if let Some(reference) = self.args.interner.reference_at_location(variant_name_location)
            {
                self.process_reference_id(reference);
            };
        }

        false
    }

    fn visit_noir_trait(&mut self, noir_trait: &NoirTrait, _: Span) -> bool {
        let name_location = noir_trait.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };

        for item in noir_trait.items.iter() {
            if let TraitItem::Function { name, .. } = &item.item {
                let func_name_location = name.location();
                if let Some(reference) =
                    self.args.interner.reference_at_location(func_name_location)
                {
                    self.process_reference_id(reference);
                };
            }
        }

        true
    }

    fn visit_global(&mut self, let_statement: &LetStatement, _: Span) -> bool {
        let name_location = let_statement.pattern.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };
        false
    }

    fn visit_noir_type_alias(&mut self, type_alias: &TypeAlias, _: Span) -> bool {
        let name_location = type_alias.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.process_reference_id(reference);
        };
        false
    }
}

#[cfg(test)]
mod tests {
    use async_lsp::lsp_types::{
        DidOpenTextDocumentParams, PartialResultParams, SemanticToken, SemanticTokensParams,
        SemanticTokensResult, TextDocumentIdentifier, TextDocumentItem, WorkDoneProgressParams,
    };
    use insta::assert_snapshot;
    use tokio::test;

    use crate::{
        notifications::on_did_open_text_document, requests::on_semantic_tokens_full_request,
        test_utils,
    };

    async fn get_semantic_tokens(src: &str) -> Vec<SemanticToken> {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let _ = on_did_open_text_document(
            &mut state,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: noir_text_document.clone(),
                    language_id: "noir".to_string(),
                    version: 0,
                    text: src.to_string(),
                },
            },
        );

        let response = on_semantic_tokens_full_request(
            &mut state,
            SemanticTokensParams {
                text_document: TextDocumentIdentifier { uri: noir_text_document },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
            },
        )
        .await
        .expect("Could not execute on_semantic_tokens_full_request");

        let SemanticTokensResult::Tokens(tokens) = response.unwrap() else {
            panic!("Expected SemanticTokensResult::Tokens");
        };
        tokens.data
    }

    #[test]
    async fn test_doc_comments() {
        // This is mainly a regression test. You can check the snapshot to match
        // highlighted tokens with their positions in the source code.
        let src = "
        /// See also [Bar] and [Bar].
        /// 
        /// ```text
        /// This is not a link: [Bar].
        /// ```
        /// 
        /// ```noir
        /// fn foo() {
        ///     let x: i32 = 1;
        ///     let y: Foo = foo::foo();
        /// }
        /// ```
        /// 
        /// And also [Bar].
        struct Foo {}

        struct Bar {}

        trait Baz {
            /// See [Foo]
            fn baz();
        }
        ";

        let tokens = get_semantic_tokens(src).await;
        let tokens = format!("{tokens:#?}");
        assert_snapshot!(tokens, @r"
        [
            SemanticToken {
                delta_line: 1,
                delta_start: 21,
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 10,
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 7,
                delta_start: 12,
                length: 2,
                token_type: 9,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 3,
                length: 3,
                token_type: 4,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 3,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 1,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 2,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 1,
                delta_start: 16,
                length: 3,
                token_type: 9,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 4,
                length: 1,
                token_type: 6,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 3,
                length: 3,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 4,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 2,
                length: 1,
                token_type: 8,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 1,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 1,
                delta_start: 16,
                length: 3,
                token_type: 9,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 4,
                length: 1,
                token_type: 6,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 3,
                length: 3,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 4,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 2,
                length: 3,
                token_type: 0,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 5,
                length: 3,
                token_type: 4,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 3,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 1,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,
                delta_start: 1,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 1,
                delta_start: 12,
                length: 1,
                token_type: 11,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 3,
                delta_start: 21,
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 6,
                delta_start: 20,
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
        ]
        ");
    }
}
