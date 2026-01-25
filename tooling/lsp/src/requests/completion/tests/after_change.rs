#[cfg(test)]
mod tests {
    use crate::{
        notifications::on_did_open_text_document, on_did_change_text_document,
        requests::on_completion_request, test_utils, utils::get_cursor_line_and_column,
    };

    use async_lsp::lsp_types::{
        CompletionItem, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
        DidOpenTextDocumentParams, PartialResultParams, Position, TextDocumentContentChangeEvent,
        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
        VersionedTextDocumentIdentifier, WorkDoneProgressParams,
    };
    use tokio::test;

    async fn get_completions_after_change(before: &str, after: &str) -> Vec<CompletionItem> {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let _ = on_did_open_text_document(
            &mut state,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: noir_text_document.clone(),
                    language_id: "noir".to_string(),
                    version: 0,
                    text: before.to_string(),
                },
            },
        );

        let (line, column, after) = get_cursor_line_and_column(after);

        let _ = on_did_change_text_document(
            &mut state,
            DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: noir_text_document.clone(),
                    version: 0,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: after,
                }],
            },
        );

        let response = on_completion_request(
            &mut state,
            CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: noir_text_document },
                    position: Position { line: line as u32, character: column as u32 },
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
                context: None,
            },
        )
        .await
        .expect("Could not execute on_completion_request");

        if let Some(CompletionResponse::Array(items)) = response { items } else { vec![] }
    }

    #[test]
    async fn completes_top_level_function_after_change() {
        let before = r#"
        fn hello() {}
        "#;

        let after = r#"
        fn hello_world() {}

        fn main() { hel>|< }
        "#;

        let items = get_completions_after_change(before, after).await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "hello_world()");
    }

    #[test]
    async fn completes_impl_function_after_change() {
        let before = r#"
        struct Foo {}

        impl Foo {
            fn hello() {}
        }
        "#;

        let after = r#"
        struct Foo {}

        impl Foo {
            fn hello_world() {}
        }

        fn main() { Foo::hel>|< }
        "#;

        let items = get_completions_after_change(before, after).await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "hello_world()");
    }

    #[test]
    async fn completes_impl_method_after_change() {
        let before = r#"
        struct Foo {}

        impl Foo {
            fn hello(self) {}
        }
        "#;

        let after = r#"
        struct Foo {}

        impl Foo {
            fn hello_world(self) {}
        }

        fn main() { 
            let foo = Foo {};
            foo.hel>|<
        }
        "#;

        let items = get_completions_after_change(before, after).await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "hello_world()");
    }

    #[test]
    async fn completes_nested_top_level_function_after_change() {
        let before = r#"
        mod moo {
            fn hello() {}
        }
        "#;

        let after = r#"
        mod moo {
            fn hello_world() {}
        }

        fn main() { moo::hel>|< }
        "#;

        let items = get_completions_after_change(before, after).await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "hello_world()");
    }
}
