use std::future::{self, Future};

use crate::utils;
use crate::visitor_reference_finder::VisitorReferenceFinder;
use crate::{LspState, types::GotoDefinitionResult};
use async_lsp::ResponseError;

use async_lsp::lsp_types::{self, LocationLink};
use lsp_types::request::GotoTypeDefinitionParams;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use super::{process_request, to_lsp_location};

pub(crate) fn on_goto_definition_request(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> + use<> {
    let result = on_goto_definition_inner(state, params, false);
    future::ready(result)
}

pub(crate) fn on_goto_type_definition_request(
    state: &mut LspState,
    params: GotoTypeDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> + use<> {
    let result = on_goto_definition_inner(state, params, true);
    future::ready(result)
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDefinitionParams,
    return_type_location_instead: bool,
) -> Result<GotoDefinitionResult, ResponseError> {
    let position = params.text_document_position_params.position;
    process_request(state, params.text_document_position_params, |args| {
        let file_id = args.location.file;
        let result =
            utils::position_to_byte_index(args.files, file_id, &position).and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source, file_id);

                let mut finder = VisitorReferenceFinder::new(file_id, source, byte_index, &args);
                finder.find(&parsed_module)
            });
        let location = if let Some((reference_id, link_lsp_location)) = result {
            let location = args.interner.reference_location(reference_id);
            Some((location, link_lsp_location))
        } else {
            let location = args
                .interner
                .get_definition_location_from(args.location, return_type_location_instead)
                .or_else(|| {
                    args.interner
                        .reference_at_location(args.location)
                        .map(|reference| args.interner.reference_location(reference))
                });
            location.map(|location| (location, None))
        };
        let (location, link_lsp_location) = location?;
        let location = to_lsp_location(args.files, location.file, location.span)?;
        let response = match link_lsp_location {
            Some(lsp_location) => {
                // In case of doc comment references we want the underline to cover the entire
                // range of the reference, not just the word that's being hovered.
                let location_link = LocationLink {
                    origin_selection_range: Some(lsp_location.range),
                    target_uri: location.uri,
                    target_range: location.range,
                    target_selection_range: location.range,
                };
                GotoDefinitionResponse::Link(vec![location_link])
            }
            None => GotoDefinitionResponse::from(location),
        };
        Some(response)
    })
}

#[cfg(test)]
mod goto_definition_tests {
    use std::panic;

    use crate::test_utils::{self, search_in_text};
    use async_lsp::lsp_types::{Position, Range};
    use tokio::test;

    use super::*;

    /// Run goto-definition from every occurrence of `name` in `src` and assert each lands at
    /// the `definition_index`-th occurrence. The definition position itself is skipped because
    /// goto on a definition does not currently return itself.
    async fn expect_goto_for_all_references(src: &str, name: &str, definition_index: usize) {
        let ranges = search_in_text(src, name);
        let expected_range = ranges[definition_index];

        let (mut state, noir_text_document) =
            test_utils::init_lsp_server_with_inline_source("document_symbol", "src/main.nr", src)
                .await;

        for (index, range) in ranges.iter().enumerate() {
            if index == definition_index {
                continue;
            }

            let params = GotoDefinitionParams {
                text_document_position_params: lsp_types::TextDocumentPositionParams {
                    text_document: lsp_types::TextDocumentIdentifier {
                        uri: noir_text_document.clone(),
                    },
                    position: range.start,
                },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            };

            let response = on_goto_definition_request(&mut state, params)
                .await
                .expect("Could execute on_goto_definition_request")
                .unwrap_or_else(|| {
                    panic!("Didn't get a goto definition response for index {index}")
                });

            if let GotoDefinitionResponse::Scalar(location) = response {
                assert_eq!(location.range, expected_range);
            } else {
                panic!("Expected a scalar response");
            }
        }
    }

    /// Run goto-definition at the `>|<` cursor in `src` and assert the response targets the
    /// `[[...]]` range, also embedded in `src`. Both markers are stripped before the source
    /// is sent to the LSP, so the test reads as "click here, expect to land there" without
    /// any line/character arithmetic in the assertion.
    async fn expect_goto_inline(src: &str) {
        let (cleaned, cursor, expected_target) = test_utils::parse_cursor_and_target_marker(src);
        let (mut state, noir_text_document) = test_utils::init_lsp_server_with_inline_source(
            "document_symbol",
            "src/main.nr",
            &cleaned,
        )
        .await;

        let params = GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: noir_text_document.clone(),
                },
                position: cursor,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response = on_goto_definition_request(&mut state, params)
            .await
            .expect("Could execute on_goto_definition_request")
            .unwrap_or_else(|| panic!("Didn't get a goto definition response"));

        if let GotoDefinitionResponse::Scalar(location) = response {
            assert_eq!(location.uri, noir_text_document);
            assert_eq!(location.range, expected_target);
        } else {
            panic!("Expected a scalar response");
        }
    }

    async fn expect_goto(
        directory: &str,
        position: Position,
        expected_file: &str,
        expected_range: Range,
    ) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

        let params = GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: noir_text_document.clone(),
                },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response = on_goto_definition_request(&mut state, params)
            .await
            .expect("Could execute on_goto_definition_request")
            .unwrap_or_else(|| panic!("Didn't get a goto definition response"));

        if let GotoDefinitionResponse::Scalar(location) = response {
            assert!(location.uri.to_string().ends_with(expected_file));
            assert_eq!(location.range, expected_range);
        } else {
            panic!("Expected a scalar response");
        }
    }

    #[test]
    async fn goto_from_function_location_to_declaration() {
        expect_goto_for_all_references(
            r#"fn another_function() -> Field {
    1
}

fn main() {
    another_function();
    another_function();
}
"#,
            "another_function",
            0,
        )
        .await;
    }

    #[test]
    async fn goto_from_use_as() {
        // Clicking on the `aliased_function` introduced by `use ... as` jumps to the
        // underlying function declaration (marked by `[[...]]`).
        expect_goto_inline(
            r#"mod foo {
    pub fn [[another_function]]() -> Field { 1 }
}

use foo::another_function as >|<aliased_function;

fn main() {
    let _ = aliased_function();
}
"#,
        )
        .await;
    }

    #[test]
    async fn goto_module_from_call_path() {
        expect_goto(
            "go_to_definition",
            Position { line: 17, character: 4 }, // "bar" in "bar::baz()"
            "src/bar.nr",
            Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        )
        .await;
    }

    #[test]
    async fn goto_inline_module_from_call_path() {
        expect_goto(
            "go_to_definition",
            Position { line: 18, character: 9 }, // "inline" in "bar::inline::qux()"
            "src/bar.nr",
            Range {
                start: Position { line: 2, character: 4 },
                end: Position { line: 2, character: 10 },
            },
        )
        .await;
    }

    #[test]
    async fn goto_module_from_use_path() {
        expect_goto_inline(
            r#"mod [[foo]] {
    pub fn another_function() -> Field { 1 }
}

use >|<foo::another_function;
"#,
        )
        .await;
    }

    #[test]
    async fn goto_module_from_mod() {
        expect_goto(
            "go_to_definition",
            Position { line: 9, character: 4 }, // "bar" in "mod bar;"
            "src/bar.nr",
            Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        )
        .await;
    }

    #[test]
    async fn goto_for_local_variable() {
        expect_goto_for_all_references(
            r#"fn main() {
    let some_var = 1;
    let _ = some_var + some_var;
}
"#,
            "some_var",
            0,
        )
        .await;
    }

    #[test]
    async fn goto_at_struct_definition_finds_same_struct() {
        expect_goto_inline("struct [[>|<Foo]] {}\n").await;
    }

    #[test]
    async fn goto_at_trait_definition_finds_same_trait() {
        expect_goto_inline("trait [[>|<Trait]] {}\n").await;
    }

    #[test]
    async fn goto_crate() {
        expect_goto(
            "go_to_definition",
            Position { line: 29, character: 6 }, // "dependency" in "use dependency::something"
            "dependency/src/lib.nr",
            Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        )
        .await;
    }

    #[test]
    async fn goto_attribute_function() {
        expect_goto_inline(
            r#"#[>|<attr]
pub fn foo() {}

comptime fn [[attr]](_: FunctionDefinition) -> Quoted {
    quote { pub fn hello() {} }
}
"#,
        )
        .await;
    }

    #[test]
    async fn goto_reference_in_doc_comment() {
        let src = r#"struct Foo {}

/// See [F>|<oo].
fn test_doc_comment() {}
"#;
        let (mut state, noir_text_document, position, src) =
            test_utils::init_lsp_server_with_inline_source_and_cursor(
                "document_symbol",
                "src/main.nr",
                src,
            )
            .await;

        let params = GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: noir_text_document.clone(),
                },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response = on_goto_definition_request(&mut state, params)
            .await
            .expect("Could execute on_goto_definition_request")
            .unwrap_or_else(|| panic!("Didn't get a goto definition response"));
        let GotoDefinitionResponse::Link(links) = response else {
            panic!("Expected a link response");
        };
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(link.target_uri, noir_text_document);

        // Origin = the `[Foo]` clicked in the doc comment.
        let origin = link.origin_selection_range.expect("Expected an origin_selection_range");
        assert_eq!(test_utils::text_at(&src, origin), "[Foo]");

        // Target = `Foo` in `struct Foo {}`.
        assert_eq!(test_utils::text_at(&src, link.target_range), "Foo");
        assert_eq!(link.target_selection_range, link.target_range);
    }
}
