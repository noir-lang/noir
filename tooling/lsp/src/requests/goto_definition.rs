use std::future::{self, Future};

use crate::attribute_reference_finder::AttributeReferenceFinder;
use crate::utils;
use crate::{types::GotoDefinitionResult, LspState};
use async_lsp::ResponseError;

use fm::PathString;
use lsp_types::request::GotoTypeDefinitionParams;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use super::{process_request, to_lsp_location};

pub(crate) fn on_goto_definition_request(
    state: &mut LspState,
    params: GotoDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params, false);
    future::ready(result)
}

pub(crate) fn on_goto_type_definition_request(
    state: &mut LspState,
    params: GotoTypeDefinitionParams,
) -> impl Future<Output = Result<GotoDefinitionResult, ResponseError>> {
    let result = on_goto_definition_inner(state, params, true);
    future::ready(result)
}

fn on_goto_definition_inner(
    state: &mut LspState,
    params: GotoDefinitionParams,
    return_type_location_instead: bool,
) -> Result<GotoDefinitionResult, ResponseError> {
    let uri = params.text_document_position_params.text_document.uri.clone();
    let position = params.text_document_position_params.position;
    process_request(state, params.text_document_position_params, |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        let reference_id = args.files.get_file_id(&path).and_then(|file_id| {
            utils::position_to_byte_index(args.files, file_id, &position).and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = AttributeReferenceFinder::new(
                    file_id,
                    byte_index,
                    args.crate_id,
                    args.def_maps,
                );
                finder.find(&parsed_module)
            })
        });
        let location = if let Some(reference_id) = reference_id {
            Some(args.interner.reference_location(reference_id))
        } else {
            args.interner
                .get_definition_location_from(args.location, return_type_location_instead)
                .or_else(|| {
                    args.interner
                        .reference_at_location(args.location)
                        .map(|reference| args.interner.reference_location(reference))
                })
        };
        location.and_then(|found_location| {
            let file_id = found_location.file;
            let definition_position = to_lsp_location(args.files, file_id, found_location.span)?;
            let response = GotoDefinitionResponse::from(definition_position).to_owned();
            Some(response)
        })
    })
}

#[cfg(test)]
mod goto_definition_tests {
    use std::panic;

    use crate::test_utils::{self, search_in_file};
    use lsp_types::{Position, Range};
    use tokio::test;

    use super::*;

    async fn expect_goto_for_all_references(directory: &str, name: &str, definition_index: usize) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

        let ranges = search_in_file(noir_text_document.path(), name);
        let expected_range = ranges[definition_index];

        for (index, range) in ranges.iter().enumerate() {
            // Ideally "go to" at the definition should return the same location, but this isn't currently
            // working. But it's also not that important, so we'll keep it for later.
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
            };
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
        };
    }

    #[test]
    async fn goto_from_function_location_to_declaration() {
        expect_goto_for_all_references("go_to_definition", "another_function", 0).await;
    }

    #[test]
    async fn goto_from_use_as() {
        expect_goto(
            "go_to_definition",
            Position { line: 7, character: 29 }, // The word after `as`,
            "src/main.nr",
            Range {
                start: Position { line: 1, character: 11 },
                end: Position { line: 1, character: 27 },
            },
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
        expect_goto(
            "go_to_definition",
            Position { line: 6, character: 4 }, // "foo" in "use foo::another_function;"
            "src/main.nr",
            Range {
                start: Position { line: 0, character: 4 },
                end: Position { line: 0, character: 7 },
            },
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
        expect_goto_for_all_references("local_variable", "some_var", 0).await;
    }

    #[test]
    async fn goto_at_struct_definition_finds_same_struct() {
        expect_goto(
            "go_to_definition",
            Position { line: 21, character: 7 }, // "Foo" in "struct Foo"
            "src/main.nr",
            Range {
                start: Position { line: 21, character: 7 },
                end: Position { line: 21, character: 10 },
            },
        )
        .await;
    }

    #[test]
    async fn goto_at_trait_definition_finds_same_trait() {
        expect_goto(
            "go_to_definition",
            Position { line: 25, character: 6 }, // "Trait" in "trait Trait"
            "src/main.nr",
            Range {
                start: Position { line: 25, character: 6 },
                end: Position { line: 25, character: 11 },
            },
        )
        .await;
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
        expect_goto(
            "go_to_definition",
            Position { line: 31, character: 3 }, // "attr"
            "src/main.nr",
            Range {
                start: Position { line: 34, character: 12 },
                end: Position { line: 34, character: 16 },
            },
        )
        .await;
    }
}
