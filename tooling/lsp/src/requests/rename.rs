use std::{
    collections::HashMap,
    future::{self, Future},
};

use async_lsp::ResponseError;
use lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use noirc_frontend::node_interner::ReferenceId;

use crate::LspState;

use super::{find_all_references_in_workspace, process_request};

pub(crate) fn on_prepare_rename_request(
    state: &mut LspState,
    params: TextDocumentPositionParams,
) -> impl Future<Output = Result<Option<PrepareRenameResponse>, ResponseError>> {
    let result = process_request(state, params, |args| {
        let reference_id = args.interner.reference_at_location(args.location);
        let rename_possible = match reference_id {
            // Rename shouldn't be possible when triggered on top of "Self"
            Some(ReferenceId::Reference(_, true /* is self type name */)) => false,
            Some(_) => true,
            None => false,
        };
        Some(PrepareRenameResponse::DefaultBehavior { default_behavior: rename_possible })
    });
    future::ready(result)
}

pub(crate) fn on_rename_request(
    state: &mut LspState,
    params: RenameParams,
) -> impl Future<Output = Result<Option<WorkspaceEdit>, ResponseError>> {
    let result = process_request(state, params.text_document_position, |args| {
        let rename_changes = find_all_references_in_workspace(
            args.location,
            args.interner,
            args.package_cache,
            args.files,
            true,
            false,
        )
        .map(|locations| {
            let rs = locations.iter().fold(
                HashMap::new(),
                |mut acc: HashMap<Url, Vec<TextEdit>>, location| {
                    let edit =
                        TextEdit { range: location.range, new_text: params.new_name.clone() };
                    acc.entry(location.uri.clone()).or_default().push(edit);
                    acc
                },
            );
            rs
        });

        let response = WorkspaceEdit {
            changes: rename_changes,
            document_changes: None,
            change_annotations: None,
        };

        Some(response)
    });
    future::ready(result)
}

#[cfg(test)]
mod rename_tests {
    use super::*;
    use crate::test_utils::{self, search_in_file};
    use lsp_types::{Range, WorkDoneProgressParams};
    use tokio::test;

    async fn check_rename_succeeds(directory: &str, name: &str) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

        // First we find out all of the occurrences of `name` in the main.nr file.
        // Note that this only works if that name doesn't show up in other places where we don't
        // expect a rename, but we craft our tests to avoid that.
        let ranges = search_in_file(noir_text_document.path(), name);

        // Test renaming works on any instance of the symbol.
        for target_range in &ranges {
            let target_position = target_range.start;

            let params = RenameParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: lsp_types::TextDocumentIdentifier {
                        uri: noir_text_document.clone(),
                    },
                    position: target_position,
                },
                new_name: "renamed_function".to_string(),
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            };

            let response = on_rename_request(&mut state, params)
                .await
                .expect("Could not execute on_prepare_rename_request")
                .unwrap();

            let changes = response.changes.expect("Expected to find rename changes");
            let mut changes: Vec<Range> =
                changes.values().flatten().map(|edit| edit.range).collect();
            changes.sort_by_key(|range| (range.start.line, range.start.character));
            if changes != ranges {
                let extra_in_changes: Vec<_> =
                    changes.iter().filter(|range| !ranges.contains(range)).collect();
                let extra_in_ranges: Vec<_> =
                    ranges.iter().filter(|range| !changes.contains(range)).collect();
                panic!("Rename locations did not match.\nThese renames were not found: {:?}\nThese renames should not have been found: {:?}", extra_in_ranges, extra_in_changes);
            }
            assert_eq!(changes, ranges);
        }
    }

    #[test]
    async fn test_on_prepare_rename_request_cannot_be_applied_if_there_are_no_matches() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("rename_function").await;

        let params = TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
            position: lsp_types::Position { line: 0, character: 0 }, // This is at the "f" of an "fn" keyword
        };

        let response = on_prepare_rename_request(&mut state, params)
            .await
            .expect("Could not execute on_prepare_rename_request");

        assert_eq!(
            response,
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior: false })
        );
    }

    #[test]
    async fn test_on_prepare_rename_request_cannot_be_applied_on_self_type_name() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("rename_struct").await;

        let params = TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
            position: lsp_types::Position { line: 11, character: 24 }, // At "Self"
        };

        let response = on_prepare_rename_request(&mut state, params)
            .await
            .expect("Could not execute on_prepare_rename_request");

        assert_eq!(
            response,
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior: false })
        );
    }

    #[test]
    async fn test_rename_function() {
        check_rename_succeeds("rename_function", "another_function").await;
    }

    #[test]
    async fn test_rename_qualified_function() {
        check_rename_succeeds("rename_qualified_function", "bar").await;
    }

    #[test]
    async fn test_rename_function_in_use_statement() {
        check_rename_succeeds("rename_function_use", "some_function").await;
    }

    #[test]
    async fn test_rename_method() {
        check_rename_succeeds("rename_function", "some_method").await;
    }

    #[test]
    async fn test_rename_struct() {
        check_rename_succeeds("rename_struct", "Foo").await;
    }

    #[test]
    async fn test_rename_trait() {
        check_rename_succeeds("rename_trait", "Foo").await;
    }

    #[test]
    async fn test_rename_type_alias() {
        check_rename_succeeds("rename_type_alias", "Bar").await;
    }

    #[test]
    async fn test_rename_global() {
        check_rename_succeeds("rename_global", "FOO").await;
    }

    #[test]
    async fn test_rename_local_variable() {
        check_rename_succeeds("local_variable", "some_var").await;
    }

    #[test]
    async fn test_rename_struct_member() {
        check_rename_succeeds("struct_member", "some_member").await;
    }
}
