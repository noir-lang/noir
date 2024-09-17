use std::future::{self, Future};

use async_lsp::ResponseError;
use lsp_types::{Location, ReferenceParams};

use crate::LspState;

use super::{find_all_references_in_workspace, process_request};

pub(crate) fn on_references_request(
    state: &mut LspState,
    params: ReferenceParams,
) -> impl Future<Output = Result<Option<Vec<Location>>, ResponseError>> {
    let include_declaration = params.context.include_declaration;
    let result = process_request(state, params.text_document_position, |args| {
        find_all_references_in_workspace(
            args.location,
            args.interner,
            args.package_cache,
            args.files,
            include_declaration,
            true,
        )
    });
    future::ready(result)
}

#[cfg(test)]
mod references_tests {
    use super::*;
    use crate::notifications;
    use crate::test_utils::{self, search_in_file};
    use lsp_types::{
        PartialResultParams, Position, Range, ReferenceContext, TextDocumentPositionParams, Url,
        WorkDoneProgressParams,
    };
    use tokio::test;

    async fn check_references_succeeds(
        directory: &str,
        name: &str,
        declaration_index: usize,
        include_declaration: bool,
    ) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

        // First we find out all of the occurrences of `name` in the main.nr file.
        // Note that this only works if that name doesn't show up in other places where we don't
        // expect a rename, but we craft our tests to avoid that.
        let ranges = search_in_file(noir_text_document.path(), name);

        // Test getting references works on any instance of the symbol.
        for target_range in &ranges {
            let target_position = target_range.start;

            let params = ReferenceParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: lsp_types::TextDocumentIdentifier {
                        uri: noir_text_document.clone(),
                    },
                    position: target_position,
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
                context: ReferenceContext { include_declaration },
            };

            let locations = on_references_request(&mut state, params)
                .await
                .expect("Could not execute on_references_request")
                .unwrap();

            let mut references_ranges: Vec<_> =
                locations.iter().map(|location| location.range).collect();
            references_ranges.sort_by_key(|range| range.start.line);

            if include_declaration {
                assert_eq!(ranges, references_ranges);
            } else {
                let mut ranges_without_declaration = ranges.clone();
                ranges_without_declaration.remove(declaration_index);
                assert_eq!(ranges_without_declaration, references_ranges);
            }
        }
    }

    #[test]
    async fn test_on_references_request_including_declaration() {
        check_references_succeeds("rename_function", "another_function", 0, true).await;
    }

    #[test]
    async fn test_on_references_request_without_including_declaration() {
        check_references_succeeds("rename_function", "another_function", 0, false).await;
    }

    // Ignored because making this work slows down everything, so for now things will not work
    // as ideally, but they'll be fast.
    // See https://github.com/noir-lang/noir/issues/5460
    #[ignore]
    #[test]
    async fn test_on_references_request_works_accross_workspace_packages() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("workspace").await;

        // noir_text_document is always `src/main.nr` in the workspace directory, so let's go to the workspace dir
        let noir_text_document = noir_text_document.to_file_path().unwrap();
        let workspace_dir = noir_text_document.parent().unwrap().parent().unwrap();

        // Let's check that we can find references to `function_one` by doing that in the package "one"
        // and getting results in the package "two" too.
        let one_lib = Url::from_file_path(workspace_dir.join("one/src/lib.nr")).unwrap();
        let two_lib = Url::from_file_path(workspace_dir.join("two/src/lib.nr")).unwrap();

        // We call this to open the document, so that the entire workspace is analyzed
        let output_diagnostics = true;

        notifications::process_workspace_for_noir_document(
            &mut state,
            one_lib.clone(),
            output_diagnostics,
        )
        .unwrap();

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: one_lib.clone() },
                position: Position { line: 0, character: 7 },
            },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None },
            context: ReferenceContext { include_declaration: true },
        };

        let mut locations = on_references_request(&mut state, params)
            .await
            .expect("Could not execute on_references_request")
            .unwrap();

        // The definition, a use in "two", and a call in "two"
        assert_eq!(locations.len(), 3);

        locations.sort_by_cached_key(|location| {
            (location.uri.to_file_path().unwrap(), location.range.start.line)
        });

        assert_eq!(locations[0].uri, one_lib);
        assert_eq!(
            locations[0].range,
            Range {
                start: Position { line: 0, character: 7 },
                end: Position { line: 0, character: 19 },
            }
        );

        assert_eq!(locations[1].uri, two_lib);
        assert_eq!(
            locations[1].range,
            Range {
                start: Position { line: 0, character: 9 },
                end: Position { line: 0, character: 21 },
            }
        );

        assert_eq!(locations[2].uri, two_lib);
        assert_eq!(
            locations[2].range,
            Range {
                start: Position { line: 3, character: 4 },
                end: Position { line: 3, character: 16 },
            }
        );
    }
}
