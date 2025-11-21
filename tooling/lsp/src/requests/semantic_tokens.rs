//! Handles LSP semantic tokens requests.
//!
//! Semantic tokens in Noir are provided for links inside doc comments. For example,
//! a doc comment that has `[println]` in it will be colorized as a function reference.

use std::{collections::HashMap, future};

use async_lsp::{
    ResponseError,
    lsp_types::{
        Position, SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensParams,
        SemanticTokensResult, TextDocumentPositionParams,
    },
};
use nargo_doc::links::{LinkFinder, LinkTarget};
use noirc_errors::Span;
use noirc_frontend::{
    ast::{LetStatement, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait, TypeAlias, Visitor},
    hir::def_map::ModuleDefId,
    node_interner::ReferenceId,
    parser::ParsedSubModule,
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

        let mut collector = SemanticTokenCollector::new(source, &args);
        let tokens = collector.collect(&parsed_module);
        Some(SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data: tokens }))
    });
    future::ready(result)
}

struct SemanticTokenCollector<'args> {
    source: &'args str,
    args: &'args ProcessRequestCallbackArgs<'args>,
    link_finder: LinkFinder,
    tokens: Vec<SemanticToken>,
    /// The last token that was added, without delta adjustments.
    last_token: Option<SemanticToken>,
    token_types: HashMap<SemanticTokenType, usize>,
}

impl<'args> SemanticTokenCollector<'args> {
    fn new(source: &'args str, args: &'args ProcessRequestCallbackArgs<'args>) -> Self {
        let link_finder = LinkFinder::default();
        let tokens = Vec::new();
        let token_types = semantic_token_types_map();
        SemanticTokenCollector { source, args, link_finder, tokens, last_token: None, token_types }
    }

    fn collect(&mut self, parsed_module: &noirc_frontend::ParsedModule) -> Vec<SemanticToken> {
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

        self.link_finder.reset();
        for located_comment in doc_comments {
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

            let links = self.link_finder.find_links(
                comments,
                current_module_id,
                current_type,
                self.args.interner,
                self.args.def_maps,
                self.args.crate_graph,
            );
            for link in links {
                let Some(token_type) = self.link_target_token_type(&link.target) else {
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

    fn link_target_token_type(&self, target: &LinkTarget) -> Option<SemanticTokenType> {
        let token_type = match target {
            LinkTarget::TopLevelItem(module_def_id) => match module_def_id {
                ModuleDefId::ModuleId(_) => SemanticTokenType::NAMESPACE,
                ModuleDefId::FunctionId(func_id) => {
                    let func_meta = self.args.interner.function_meta(func_id);
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
        let last_token = self.last_token.replace(token);

        // See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_semanticTokens
        // for an explanation of delta_line and delta_start.
        if let Some(last_token) = last_token {
            let same_line = token.delta_line == last_token.delta_line;
            token.delta_line -= last_token.delta_line;
            if same_line {
                token.delta_start -= last_token.delta_start;
            }
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
        let src = "
        /// See also [Bar] and [Bar].
        /// 
        /// ```
        /// This is not a link: [Bar].
        /// ```
        /// 
        /// And also [Bar].
        struct Foo {}

        struct Bar {}
        ";

        let tokens = get_semantic_tokens(src).await;
        let expected = vec![
            SemanticToken {
                delta_line: 1,
                delta_start: 21,
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 0,   // The second link is on the same line, so no delta
                delta_start: 10, // 10 chars after the previous token start char
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
            SemanticToken {
                delta_line: 6,   // It's on line 8, so six more than before.
                delta_start: 21, // This isn't relative anymore as it's on a new line
                length: 5,
                token_type: 1,
                token_modifiers_bitset: 0,
            },
        ];
        assert_eq!(tokens, expected);
    }
}
