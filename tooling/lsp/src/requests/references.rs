use std::future::{self, Future};

use async_lsp::ResponseError;
use lsp_types::{Location, ReferenceParams};

use crate::LspState;

use super::{process_request, to_lsp_location};

pub(crate) fn on_references_request(
    state: &mut LspState,
    params: ReferenceParams,
) -> impl Future<Output = Result<Option<Vec<Location>>, ResponseError>> {
    let result =
        process_request(state, params.text_document_position, |location, interner, files| {
            interner.find_all_references(location, params.context.include_declaration, true).map(
                |locations| {
                    locations
                        .iter()
                        .filter_map(|location| to_lsp_location(files, location.file, location.span))
                        .collect()
                },
            )
        });
    future::ready(result)
}

#[cfg(test)]
mod references_tests {
    use super::*;
    use crate::test_utils::{self, search_in_file};
    use lsp_types::{
        PartialResultParams, ReferenceContext, TextDocumentPositionParams, WorkDoneProgressParams,
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
}
