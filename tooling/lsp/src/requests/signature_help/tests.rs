#[cfg(test)]
mod signature_help_tests {
    use crate::{
        notifications::on_did_open_text_document, requests::on_signature_help_request, test_utils,
    };

    use lsp_types::{
        DidOpenTextDocumentParams, ParameterLabel, Position, SignatureHelp, SignatureHelpParams,
        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
        WorkDoneProgressParams,
    };
    use tokio::test;

    async fn get_signature_help(src: &str) -> SignatureHelp {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let (line, column) = src
            .lines()
            .enumerate()
            .find_map(|(line_index, line)| {
                line.find(">|<").map(|char_index| (line_index, char_index))
            })
            .expect("Expected to find one >|< in the source code");

        let src = src.replace(">|<", "");

        on_did_open_text_document(
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

        on_signature_help_request(
            &mut state,
            SignatureHelpParams {
                context: None,
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: noir_text_document },
                    position: Position { line: line as u32, character: column as u32 },
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            },
        )
        .await
        .expect("Could not execute on_signature_help_request")
        .unwrap()
    }

    fn check_label(signature_label: &str, parameter_label: &ParameterLabel, expected_string: &str) {
        let ParameterLabel::LabelOffsets(offsets) = parameter_label else {
            panic!("Expected label to be LabelOffsets, got {:?}", parameter_label);
        };

        assert_eq!(&signature_label[offsets[0] as usize..offsets[1] as usize], expected_string);
    }

    #[test]
    async fn test_signature_help_for_call_at_first_argument() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }
            fn wrapper(x: u32) {}

            fn bar() {
                wrapper(foo(>|<1, 2));
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "fn foo(x: i32, y: Field) -> u32");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "x: i32");
        check_label(&signature.label, &params[1].label, "y: Field");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_call_between_arguments() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                foo(1,>|< 2);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.active_parameter, Some(1));
    }

    #[test]
    async fn test_signature_help_for_call_at_second_argument() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                foo(1, >|<2);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.active_parameter, Some(1));
    }

    #[test]
    async fn test_signature_help_for_call_past_last_argument() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                foo(1, 2, >|<);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.active_parameter, Some(2));
    }

    #[test]
    async fn test_signature_help_for_method_call() {
        let src = r#"
            struct Foo {}

            impl Foo {
              fn foo(self, x: i32, y: Field) -> u32 { 0 }
            }

            fn wrapper(x: u32) {}

            fn bar(f: Foo) {
                wrapper(f.foo(>|<1, 2));
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "fn foo(self, x: i32, y: Field) -> u32");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "x: i32");
        check_label(&signature.label, &params[1].label, "y: Field");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_fn_call() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                let f = foo;
                f(>|<1, 2);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "fn(i32, Field) -> u32");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "i32");
        check_label(&signature.label, &params[1].label, "Field");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_assert() {
        let src = r#"
            fn bar() {
                assert(>|<1, "hello");
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "assert(predicate: bool, [failure_message: str<N>])");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "predicate: bool");
        check_label(&signature.label, &params[1].label, "[failure_message: str<N>]");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_assert_eq() {
        let src = r#"
            fn bar() {
                assert_eq(>|<true, false, "oops");
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "assert_eq(lhs: T, rhs: T, [failure_message: str<N>])");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 3);

        check_label(&signature.label, &params[0].label, "lhs: T");
        check_label(&signature.label, &params[1].label, "rhs: T");
        check_label(&signature.label, &params[2].label, "[failure_message: str<N>]");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_enum_variant() {
        let src = r#"
            enum Enum {
                Variant(Field, i32)
            }

            fn bar() {
                Enum::Variant(>|<(), ());
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "enum Enum::Variant(Field, i32)");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "Field");
        check_label(&signature.label, &params[1].label, "i32");

        assert_eq!(signature.active_parameter, Some(0));
    }
}
