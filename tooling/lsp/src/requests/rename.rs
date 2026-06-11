use std::{
    collections::HashMap,
    future::{self, Future},
};

use async_lsp::ResponseError;
use async_lsp::lsp_types;
use lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use noirc_frontend::node_interner::ReferenceId;

use crate::LspState;

use super::{find_all_references_in_workspace, process_request};

pub(crate) fn on_prepare_rename_request(
    state: &mut LspState,
    params: TextDocumentPositionParams,
) -> impl Future<Output = Result<Option<PrepareRenameResponse>, ResponseError>> + use<> {
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
) -> impl Future<Output = Result<Option<WorkspaceEdit>, ResponseError>> + use<> {
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
            locations.iter().fold(
                HashMap::new(),
                |mut acc: HashMap<Url, Vec<TextEdit>>, location| {
                    let edit =
                        TextEdit { range: location.range, new_text: params.new_name.clone() };
                    acc.entry(location.uri.clone()).or_default().push(edit);
                    acc
                },
            )
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
    use crate::test_utils::{self, search_in_text};
    use async_lsp::lsp_types::{Range, WorkDoneProgressParams};
    use tokio::test;

    /// Rename every occurrence of `name` in `src` and assert the LSP returns rename edits at
    /// the exact same set of ranges — once per occurrence, since the rename should be
    /// triggerable from any of them.
    async fn check_rename_succeeds(src: &str, name: &str) {
        let ranges = search_in_text(src, name);
        let (mut state, noir_text_document) =
            test_utils::init_lsp_server_with_inline_source("document_symbol", "src/main.nr", src)
                .await;

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
                panic!(
                    "Rename locations did not match.\nThese renames were not found: {extra_in_ranges:?}\nThese renames should not have been found: {extra_in_changes:?}"
                );
            }
            assert_eq!(changes, ranges);
        }
    }

    async fn check_prepare_rename_is_not_applicable(src: &str) {
        let (mut state, noir_text_document, position, _src) =
            test_utils::init_lsp_server_with_inline_source_and_cursor(
                "document_symbol",
                "src/main.nr",
                src,
            )
            .await;

        let params = TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: noir_text_document },
            position,
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
    async fn test_on_prepare_rename_request_cannot_be_applied_if_there_are_no_matches() {
        check_prepare_rename_is_not_applicable(">|<\nfn another_function() {}\n").await;
    }

    #[test]
    async fn test_on_prepare_rename_request_cannot_be_applied_on_self_type_name() {
        check_prepare_rename_is_not_applicable(
            r#"struct Foo {}

impl Foo {
    fn new() -> Self {
        >|<Self {}
    }
}
"#,
        )
        .await;
    }

    #[test]
    async fn test_rename_function() {
        check_rename_succeeds(
            r#"fn another_function() -> Field {
    1
}

fn main() {
    another_function();
    another_function();
}
"#,
            "another_function",
        )
        .await;
    }

    #[test]
    async fn test_rename_qualified_function() {
        check_rename_succeeds(
            r#"mod foo {
    pub fn bar() {}
}

fn main() {
    foo::bar();
    foo::bar();
}
"#,
            "bar",
        )
        .await;
    }

    #[test]
    async fn test_rename_function_in_use_statement() {
        check_rename_succeeds(
            r#"mod foo {
    pub fn some_function() {}
}

use foo::some_function;

fn main() {
    some_function();
}
"#,
            "some_function",
        )
        .await;
    }

    #[test]
    async fn test_rename_method() {
        check_rename_succeeds(
            r#"struct Foo {}

impl Foo {
    fn some_method(self) {}
}

fn main() {
    let foo = Foo {};
    foo.some_method();
    foo.some_method();
}
"#,
            "some_method",
        )
        .await;
    }

    #[test]
    async fn test_rename_struct() {
        check_rename_succeeds(
            r#"struct Foo {}

impl Foo {
    fn new() -> Self {
        Foo {}
    }
}

fn make_foo() -> Foo {
    Foo::new()
}
"#,
            "Foo",
        )
        .await;
    }

    #[test]
    async fn test_rename_trait() {
        check_rename_succeeds(
            r#"trait Foo {
    fn foo(self);
}

impl Foo for Field {
    fn foo(self) {}
}
"#,
            "Foo",
        )
        .await;
    }

    #[test]
    async fn test_rename_type_alias() {
        check_rename_succeeds(
            r#"type Bar = Field;

fn make() -> Bar {
    1
}
"#,
            "Bar",
        )
        .await;
    }

    #[test]
    async fn test_rename_global() {
        check_rename_succeeds(
            r#"global FOO: Field = 1;

fn main() -> Field {
    FOO + FOO
}
"#,
            "FOO",
        )
        .await;
    }

    #[test]
    async fn test_rename_local_variable() {
        check_rename_succeeds(
            r#"fn main() {
    let some_var = 1;
    let _ = some_var + some_var;
}
"#,
            "some_var",
        )
        .await;
    }

    #[test]
    async fn test_rename_struct_member() {
        check_rename_succeeds(
            r#"struct Foo {
    some_member: Field,
}

fn main() {
    let foo = Foo { some_member: 1 };
    let _ = foo.some_member;
}
"#,
            "some_member",
        )
        .await;
    }
}
