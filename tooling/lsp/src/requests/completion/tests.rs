#[cfg(test)]
mod completion_tests {
    use crate::{
        notifications::on_did_open_text_document,
        requests::{
            completion::{
                completion_items::{
                    completion_item_with_sort_text,
                    completion_item_with_trigger_parameter_hints_command, crate_completion_item,
                    field_completion_item, module_completion_item, simple_completion_item,
                    snippet_completion_item,
                },
                sort_text::{auto_import_sort_text, self_mismatch_sort_text},
            },
            on_completion_request,
        },
        test_utils,
    };

    use lsp_types::{
        CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionParams,
        CompletionResponse, DidOpenTextDocumentParams, PartialResultParams, Position, Range,
        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, TextEdit,
        WorkDoneProgressParams,
    };
    use tokio::test;

    async fn get_completions(src: &str) -> Vec<CompletionItem> {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let (line, column) = src
            .lines()
            .enumerate()
            .filter_map(|(line_index, line)| {
                line.find(">|<").map(|char_index| (line_index, char_index))
            })
            .next()
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

        if let Some(CompletionResponse::Array(items)) = response {
            items
        } else {
            vec![]
        }
    }

    fn assert_items_match(mut items: Vec<CompletionItem>, mut expected: Vec<CompletionItem>) {
        items.sort_by_key(|item| item.label.clone());

        expected.sort_by_key(|item| item.label.clone());

        if items != expected {
            println!(
                "Items: {:?}",
                items.iter().map(|item| item.label.clone()).collect::<Vec<_>>()
            );
            println!(
                "Expected: {:?}",
                expected.iter().map(|item| item.label.clone()).collect::<Vec<_>>()
            );
        }

        assert_eq!(items, expected);
    }

    async fn assert_completion(src: &str, expected: Vec<CompletionItem>) {
        let items = get_completions(src).await;
        assert_items_match(items, expected);
    }

    async fn assert_completion_excluding_auto_import(src: &str, expected: Vec<CompletionItem>) {
        let items = get_completions(src).await;
        let items = items.into_iter().filter(|item| item.additional_text_edits.is_none()).collect();
        assert_items_match(items, expected);
    }

    pub(super) fn function_completion_item(
        label: impl Into<String>,
        insert_text: impl Into<String>,
        description: impl Into<String>,
    ) -> CompletionItem {
        completion_item_with_trigger_parameter_hints_command(snippet_completion_item(
            label,
            CompletionItemKind::FUNCTION,
            insert_text,
            Some(description.into()),
        ))
    }

    #[test]
    async fn test_use_first_segment() {
        let src = r#"
            mod foo {}
            mod foobar {}
            use f>|<
        "#;

        assert_completion(
            src,
            vec![module_completion_item("foo"), module_completion_item("foobar")],
        )
        .await;
    }

    #[test]
    async fn test_use_second_segment() {
        let src = r#"
            mod foo {
                mod bar {}
                mod baz {}
            }
            use foo::>|<
        "#;

        assert_completion(src, vec![module_completion_item("bar"), module_completion_item("baz")])
            .await;
    }

    #[test]
    async fn test_use_second_segment_after_typing() {
        let src = r#"
            mod foo {
                mod bar {}
                mod brave {}
            }
            use foo::ba>|<
        "#;

        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_use_struct() {
        let src = r#"
            mod foo {
                struct Foo {}
            }
            use foo::>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "Foo",
                CompletionItemKind::STRUCT,
                Some("Foo".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_use_function() {
        let src = r#"
            mod foo {
                pub fn bar(x: i32) -> u64 { 0 }
                fn bar_is_private(x: i32) -> u64 { 0 }
            }
            use foo::>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn(i32) -> u64".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_use_after_crate_and_letter() {
        // Prove that "std" shows up
        let src = r#"
            use s>|<
        "#;
        assert_completion(src, vec![crate_completion_item("std")]).await;

        // "std" doesn't show up anymore because of the "crate::" prefix
        let src = r#"
            mod something {}
            use crate::s>|<
        "#;
        assert_completion(src, vec![module_completion_item("something")]).await;
    }

    #[test]
    async fn test_use_suggests_hardcoded_crate() {
        let src = r#"
            use c>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item("crate::", CompletionItemKind::KEYWORD, None)],
        )
        .await;
    }

    #[test]
    async fn test_use_in_tree_after_letter() {
        let src = r#"
            mod foo {
                mod bar {}
            }
            use foo::{b>|<}
        "#;

        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_use_in_tree_after_colons() {
        let src = r#"
            mod foo {
                mod bar {
                    mod baz {}
                }
            }
            use foo::{bar::>|<}
        "#;

        assert_completion(src, vec![module_completion_item("baz")]).await;
    }

    #[test]
    async fn test_use_in_tree_after_colons_after_another_segment() {
        let src = r#"
            mod foo {
                mod bar {}
                mod qux {}
            }
            use foo::{bar, q>|<}
        "#;

        assert_completion(src, vec![module_completion_item("qux")]).await;
    }

    #[test]
    async fn test_use_in_nested_module() {
        let src = r#"
            mod foo {
                mod something {}

                use s>|<
            }
        "#;

        assert_completion(
            src,
            vec![
                module_completion_item("something"),
                crate_completion_item("std"),
                simple_completion_item("super::", CompletionItemKind::KEYWORD, None),
            ],
        )
        .await;
    }

    #[test]
    async fn test_use_after_super() {
        let src = r#"
            mod foo {}

            mod bar {
                mod something {}

                use super::f>|<
            }
        "#;

        assert_completion(src, vec![module_completion_item("foo")]).await;
    }

    #[test]
    async fn test_use_after_crate_and_letter_nested_in_module() {
        let src = r#"
            mod something {
                mod something_else {}
                use crate::s>|<
            }
            
        "#;
        assert_completion(src, vec![module_completion_item("something")]).await;
    }

    #[test]
    async fn test_use_after_crate_segment_and_letter_nested_in_module() {
        let src = r#"
            mod something {
                mod something_else {}
                use crate::something::s>|<
            }
            
        "#;
        assert_completion(src, vec![module_completion_item("something_else")]).await;
    }

    #[test]
    async fn test_complete_path_shows_module() {
        let src = r#"
          mod foobar {}

          fn main() {
            fo>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("foobar")]).await;
    }

    #[test]
    async fn test_complete_path_after_colons_shows_submodule() {
        let src = r#"
          mod foo {
            mod bar {}
          }

          fn main() {
            foo::>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_complete_path_after_colons_and_letter_shows_submodule() {
        let src = r#"
          mod foo {
            mod qux {}
          }

          fn main() {
            foo::q>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("qux")]).await;
    }

    #[test]
    async fn test_complete_path_with_local_variable() {
        let src = r#"
          fn main() {
            let local = 1;
            l>|<
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "local",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_shadowed_local_variable() {
        let src = r#"
          fn main() {
            let local = 1;
            let local = true;
            l>|<
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "local",
                CompletionItemKind::VARIABLE,
                Some("bool".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_function_argument() {
        let src = r#"
          fn main(local: Field) {
            l>|<
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "local",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_function_without_arguments() {
        let src = r#"
          fn hello() { }

          fn main() {
            h>|<
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![function_completion_item("hello()", "hello()", "fn()")],
        )
        .await;
    }

    #[test]
    async fn test_complete_function() {
        let src = r#"
          fn hello(x: i32, y: Field) { }

          fn main() {
            h>|<
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![function_completion_item(
                "hello(…)",
                "hello(${1:x}, ${2:y})",
                "fn(i32, Field)".to_string(),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_builtin_functions() {
        let src = r#"
          fn main() {
            a>|<
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![
                snippet_completion_item(
                    "assert(…)",
                    CompletionItemKind::FUNCTION,
                    "assert(${1:predicate})",
                    Some("fn(T)".to_string()),
                ),
                function_completion_item("assert_constant(…)", "assert_constant(${1:x})", "fn(T)"),
                snippet_completion_item(
                    "assert_eq(…)",
                    CompletionItemKind::FUNCTION,
                    "assert_eq(${1:lhs}, ${2:rhs})",
                    Some("fn(T, T)".to_string()),
                ),
            ],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_in_impl() {
        let src = r#"
          struct SomeStruct {}

          impl SomeStruct {
            fn foo() {
                So>|<
            }
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "SomeStruct",
                CompletionItemKind::STRUCT,
                Some("SomeStruct".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_in_trait_impl() {
        let src = r#"
          struct SomeStruct {}
          trait Trait {}

          impl Trait for SomeStruct {
            fn foo() {
                So>|<
            }
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "SomeStruct",
                CompletionItemKind::STRUCT,
                Some("SomeStruct".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_for_argument() {
        let src = r#"
          fn main() {
            for index in 0..10 {
                ind>|<
            }
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "index",
                CompletionItemKind::VARIABLE,
                Some("u32".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_lambda_argument() {
        let src = r#"
          fn lambda(f: fn(i32)) { }

          fn main() {
            lambda(|lambda_var| lambda_v>|<)
          }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "lambda_var",
                CompletionItemKind::VARIABLE,
                Some("_".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_struct_field_type() {
        let src = r#"
          struct Something {}

          fn SomeFunction() {}

          struct Another {
            some: So>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_function_parameter() {
        let src = r#"
          struct Something {}

          fn foo(x: So>|<) {}
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_function_return_type() {
        let src = r#"
          struct Something {}

          fn foo() -> So>|< {}
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_type_alias() {
        let src = r#"
          struct Something {}

          type Foo = So>|<
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_trait_function() {
        let src = r#"
          struct Something {}

          trait Trait {
            fn foo(s: So>|<);
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_trait_function_return_type() {
        let src = r#"
          struct Something {}

          trait Trait {
            fn foo() -> So>|<;
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_let_type() {
        let src = r#"
          struct Something {}

          fn main() {
            let x: So>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_lambda_parameter() {
        let src = r#"
          struct Something {}

          fn main() {
            foo(|s: So>|<| s)
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_builtin_types() {
        let src = r#"
            fn foo(x: i>|<) {}
        "#;

        let items = get_completions(src).await;
        let items = items.into_iter().filter(|item| item.label.starts_with('i')).collect();

        assert_items_match(
            items,
            vec![
                simple_completion_item("i8", CompletionItemKind::STRUCT, Some("i8".to_string())),
                simple_completion_item("i16", CompletionItemKind::STRUCT, Some("i16".to_string())),
                simple_completion_item("i32", CompletionItemKind::STRUCT, Some("i32".to_string())),
                simple_completion_item("i64", CompletionItemKind::STRUCT, Some("i64".to_string())),
            ],
        );
    }

    #[test]
    async fn test_suggest_true() {
        let src = r#"
            fn main() {
                let x = t>|<
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "true",
                CompletionItemKind::KEYWORD,
                Some("bool".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_regarding_if_scope() {
        let src = r#"
            fn main() {
                let good = 1;
                if true {
                    let great = 2;
                    g>|<
                } else {
                    let greater = 3;
                }
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![
                simple_completion_item(
                    "good",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
                simple_completion_item(
                    "great",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
            ],
        )
        .await;

        let src = r#"
            fn main() {
                let good = 1;
                if true {
                    let great = 2;
                } else {
                    let greater = 3;
                    g>|<
                }
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![
                simple_completion_item(
                    "good",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
                simple_completion_item(
                    "greater",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
            ],
        )
        .await;

        let src = r#"
            fn main() {
                let good = 1;
                if true {
                    let great = 2;
                } else {
                    let greater = 3;
                }
                g>|<
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "good",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_regarding_block_scope() {
        let src = r#"
            fn main() {
                let good = 1;
                {
                    let great = 2;
                    g>|<
                }
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![
                simple_completion_item(
                    "good",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
                simple_completion_item(
                    "great",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
            ],
        )
        .await;

        let src = r#"
            fn main() {
                let good = 1;
                {
                    let great = 2;
                }
                g>|<
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "good",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_struct_type_parameter() {
        let src = r#"
            struct Foo<Context> {
                context: Cont>|<
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item("Context", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_impl_type_parameter() {
        let src = r#"
            struct Foo<Context> {}

            impl <TypeParam> Foo<TypeParam> {
                fn foo() {
                    let x: TypeP>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("TypeParam", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_trait_impl_type_parameter() {
        let src = r#"
            struct Foo {}
            trait Trait<Context> {}

            impl <TypeParam> Trait<TypeParam> for Foo {
                fn foo() {
                    let x: TypeP>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("TypeParam", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_trait_function_type_parameter() {
        let src = r#"
            struct Foo {}
            trait Trait {
                fn foo<TypeParam>() {
                    let x: TypeP>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("TypeParam", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_function_type_parameters() {
        let src = r#"
            fn foo<Context>(x: Cont>|<) {}
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item("Context", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_and_letter() {
        let src = r#"
            struct Some {
                property: i32,
            }

            fn foo(s: Some) {
                s.p>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("property", "i32")]).await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_and_letter_for_generic_type() {
        let src = r#"
            struct Some<T> {
                property: T,
            }

            fn foo(s: Some<i32>) {
                s.p>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("property", "i32")]).await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_followed_by_brace() {
        let src = r#"
            struct Some {
                property: i32,
            }

            fn foo(s: Some) {
                s.>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("property", "i32")]).await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_chain() {
        let src = r#"
            struct Some {
                property: Other,
            }

            struct Other {
                bar: i32,
            }

            fn foo(some: Some) {
                some.property.>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("bar", "i32")]).await;
    }

    #[test]
    async fn test_suggests_struct_impl_method() {
        let src = r#"
            struct Some {
            }

            impl Some {
                fn foobar(self, x: i32) {}
                fn foobar2(&mut self, x: i32) {}
                fn foobar3(y: i32) {}
            }

            fn foo(some: Some) {
                some.f>|<
            }
        "#;
        assert_completion(
            src,
            vec![
                function_completion_item("foobar(…)", "foobar(${1:x})", "fn(self, i32)"),
                function_completion_item("foobar2(…)", "foobar2(${1:x})", "fn(&mut self, i32)"),
            ],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_trait_impl_method() {
        let src = r#"
            struct Some {
            }

            trait SomeTrait {
                fn foobar(self, x: i32);
                fn foobar2(y: i32);
            }

            impl SomeTrait for Some {
                fn foobar(self, x: i32) {}
                fn foobar2(y: i32) {}
            }

            fn foo(some: Some) {
                some.f>|<
            }
        "#;
        assert_completion(
            src,
            vec![function_completion_item("foobar(…)", "foobar(${1:x})", "fn(self, i32)")],
        )
        .await;
    }

    #[test]
    async fn test_suggests_primitive_trait_impl_method() {
        let src = r#"
            trait SomeTrait {
                fn foobar(self, x: i32);
                fn foobar2(y: i32);
            }

            impl SomeTrait for Field {
                fn foobar(self, x: i32) {}
                fn foobar2(y: i32) {}
            }

            fn foo(field: Field) {
                field.f>|<
            }
        "#;
        assert_completion(
            src,
            vec![function_completion_item("foobar(…)", "foobar(${1:x})", "fn(self, i32)")],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_methods_after_colons() {
        let src = r#"
            struct Some {
            }

            impl Some {
                fn foobar(self, x: i32) {}
                fn foobar2(&mut self, x: i32) {}
                fn foobar3(y: i32) {}
            }

            fn foo() {
                Some::>|<
            }
        "#;
        assert_completion(
            src,
            vec![
                completion_item_with_sort_text(
                    function_completion_item(
                        "foobar(…)",
                        "foobar(${1:self}, ${2:x})",
                        "fn(self, i32)",
                    ),
                    self_mismatch_sort_text(),
                ),
                completion_item_with_sort_text(
                    function_completion_item(
                        "foobar2(…)",
                        "foobar2(${1:self}, ${2:x})",
                        "fn(&mut self, i32)",
                    ),
                    self_mismatch_sort_text(),
                ),
                function_completion_item("foobar3(…)", "foobar3(${1:y})", "fn(i32)"),
            ],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_behind_alias_methods_after_dot() {
        let src = r#"
            struct Some {
            }

            type Alias = Some;

            impl Some {
                fn foobar(self, x: i32) {}
            }

            fn foo(some: Alias) {
                some.>|<
            }
        "#;
        assert_completion(
            src,
            vec![function_completion_item("foobar(…)", "foobar(${1:x})", "fn(self, i32)")],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_behind_alias_methods_after_colons() {
        let src = r#"
            struct Some {
            }

            type Alias = Some;

            impl Some {
                fn foobar(self, x: i32) {}
                fn foobar2(&mut self, x: i32) {}
                fn foobar3(y: i32) {}
            }

            fn foo() {
                Alias::>|<
            }
        "#;
        assert_completion(
            src,
            vec![
                completion_item_with_sort_text(
                    function_completion_item(
                        "foobar(…)",
                        "foobar(${1:self}, ${2:x})",
                        "fn(self, i32)",
                    ),
                    self_mismatch_sort_text(),
                ),
                completion_item_with_sort_text(
                    function_completion_item(
                        "foobar2(…)",
                        "foobar2(${1:self}, ${2:x})",
                        "fn(&mut self, i32)",
                    ),
                    self_mismatch_sort_text(),
                ),
                function_completion_item("foobar3(…)", "foobar3(${1:y})", "fn(i32)"),
            ],
        )
        .await;
    }

    #[test]
    async fn test_completes_in_broken_if_after_dot() {
        let src = r#"
            struct Some {
                foo: i32,
            }

            fn foo(s: Some) {
                if s.>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("foo", "i32")]).await;
    }

    #[test]
    async fn test_completes_in_nested_expression() {
        let src = r#"
            struct Foo { bar: Bar }
            struct Bar { baz: i32 }

            fn foo(f: Foo) {
                f.bar & f.>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("bar", "Bar")]).await;
    }

    #[test]
    async fn test_completes_in_call_chain() {
        let src = r#"
            struct Foo {}

            impl Foo {
                fn foo(self) -> Foo { self }
            }

            fn foo(f: Foo) {
                f.foo().>|<
            }
        "#;
        assert_completion(src, vec![function_completion_item("foo()", "foo()", "fn(self) -> Foo")])
            .await;
    }

    #[test]
    async fn test_completes_when_assignment_follows() {
        let src = r#"
            struct Foo {
                bar: i32,
            }

            fn foo(f: Foo) {
                let mut x = 1;

                f.>|<

                x = 2;
            }
        "#;
        assert_completion(src, vec![field_completion_item("bar", "i32")]).await;
    }

    #[test]
    async fn test_completes_tuple_fields() {
        let src = r#"
            fn main() {
                let tuple = (1, true);
                tuple.>|<
            }
        "#;

        let items = get_completions(src).await;
        let items = items.into_iter().filter(|item| item.kind == Some(CompletionItemKind::FIELD));
        let items = items.collect();

        assert_items_match(
            items,
            vec![field_completion_item("0", "Field"), field_completion_item("1", "bool")],
        );
    }

    #[test]
    async fn test_completes_constructor_fields() {
        let src = r#"
            mod foobar {
                struct Foo {
                    bb: i32,
                    bbb: Field,
                    bbbb: bool,
                    bbbbb: str<6>,
                }
            }

            fn main() {
                foobar::Foo { bbb: 1, b>|<, bbbbb }
            }
        "#;
        assert_completion(
            src,
            vec![field_completion_item("bb", "i32"), field_completion_item("bbbb", "bool")],
        )
        .await;
    }

    #[test]
    async fn test_completes_trait_methods() {
        let src = r#"
            trait One {
                fn one() -> Self;
            }

            fn main() {
                One::>|<
            }
        "#;
        assert_completion(src, vec![function_completion_item("one()", "one()", "fn() -> Self")])
            .await;
    }

    #[test]
    async fn test_auto_imports() {
        let src = r#"
            mod foo {
                mod bar {
                    pub fn hello_world() {}

                    struct Foo {
                    }

                    impl Foo {
                        // This is here to make sure it's not offered for completion
                        fn hello_world() {}
                    }
                }
            }

            fn main() {
                hel>|<
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.label, "hello_world()");
        assert_eq!(
            item.label_details,
            Some(CompletionItemLabelDetails {
                detail: Some("(use foo::bar::hello_world)".to_string()),
                description: Some("fn()".to_string())
            })
        );

        assert_eq!(
            item.additional_text_edits,
            Some(vec![TextEdit {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
                new_text: "use foo::bar::hello_world;\n".to_string(),
            }])
        );

        assert_eq!(item.sort_text, Some(auto_import_sort_text()));
    }

    #[test]
    async fn test_auto_imports_when_in_nested_module_and_item_is_further_nested() {
        let src = r#"
            mod foo {
                mod bar {
                    pub fn hello_world() {}
                }

                fn foo() {
                    hel>|<
                }
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.label, "hello_world()");
        assert_eq!(
            item.label_details,
            Some(CompletionItemLabelDetails {
                detail: Some("(use bar::hello_world)".to_string()),
                description: Some("fn()".to_string())
            })
        );

        assert_eq!(
            item.additional_text_edits,
            Some(vec![TextEdit {
                range: Range {
                    start: Position { line: 2, character: 4 },
                    end: Position { line: 2, character: 4 },
                },
                new_text: "use bar::hello_world;\n\n    ".to_string(),
            }])
        );
    }

    #[test]
    async fn test_auto_imports_when_in_nested_module_and_item_is_not_further_nested() {
        let src = r#"
            mod foo {
                mod bar {
                    pub fn hello_world() {}
                }

                mod baz {
                    fn foo() {
                        hel>|<
                    }
                }
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.label, "hello_world()");
        assert_eq!(
            item.label_details,
            Some(CompletionItemLabelDetails {
                detail: Some("(use super::bar::hello_world)".to_string()),
                description: Some("fn()".to_string())
            })
        );

        assert_eq!(
            item.additional_text_edits,
            Some(vec![TextEdit {
                range: Range {
                    start: Position { line: 7, character: 8 },
                    end: Position { line: 7, character: 8 },
                },
                new_text: "use super::bar::hello_world;\n\n        ".to_string(),
            }])
        );
    }

    #[test]
    async fn test_auto_import_inserts_after_last_use() {
        let src = r#"
            mod foo {
                mod bar {
                    pub fn hello_world() {}
                }
            }

            use foo::bar;

            fn main() {
                hel>|<
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(
            item.additional_text_edits,
            Some(vec![TextEdit {
                range: Range {
                    start: Position { line: 8, character: 0 },
                    end: Position { line: 8, character: 0 },
                },
                new_text: "use foo::bar::hello_world;\n".to_string(),
            }])
        );
    }

    #[test]
    async fn test_does_not_auto_import_test_functions() {
        let src = r#"
            mod foo {
                mod bar {
                    #[test]
                    pub fn hello_world() {}
                }
            }

            use foo::bar;

            fn main() {
                hel>|<
            }
        "#;
        let items = get_completions(src).await;
        assert!(items.is_empty());
    }

    #[test]
    async fn test_does_not_auto_import_private_functions() {
        let src = r#"
            mod foo {
                mod bar {
                    fn hello_world() {}
                }
            }

            use foo::bar;

            fn main() {
                hel>|<
            }
        "#;
        let items = get_completions(src).await;
        assert!(items.is_empty());
    }

    #[test]
    async fn test_auto_import_suggests_modules_too() {
        let src = r#"
            mod foo {
                mod barbaz {
                    fn hello_world() {}
                }
            }

            fn main() {
                barb>|<
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.label, "barbaz");
        assert_eq!(
            item.label_details,
            Some(CompletionItemLabelDetails {
                detail: Some("(use foo::barbaz)".to_string()),
                description: None
            })
        );
    }

    #[test]
    async fn test_completes_matching_any_part_of_an_identifier_by_underscore() {
        let src = r#"
            struct Foo {
                some_property: i32,
            }

            fn foo(f: Foo) {
                f.prop>|<
            }
        "#;
        assert_completion(src, vec![field_completion_item("some_property", "i32")]).await;
    }

    #[test]
    async fn test_completes_in_impl_type() {
        let src = r#"
            struct FooBar {
            }

            impl FooB>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "FooBar",
                CompletionItemKind::STRUCT,
                Some("FooBar".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_completes_in_impl_for_type() {
        let src = r#"
            struct FooBar {
            }

            impl Default for FooB>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "FooBar",
                CompletionItemKind::STRUCT,
                Some("FooBar".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_auto_import_with_super() {
        let src = r#"
            pub fn bar_baz() {}

            mod tests {
                fn foo() {
                    bar_b>|<
                }
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.label, "bar_baz()");
        assert_eq!(
            item.label_details,
            Some(CompletionItemLabelDetails {
                detail: Some("(use super::bar_baz)".to_string()),
                description: Some("fn()".to_string())
            })
        );
    }

    #[test]
    async fn test_auto_import_from_std() {
        let src = r#"
            fn main() {
                compute_merkle_roo>|<
            }
        "#;
        let items = get_completions(src).await;
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.label, "compute_merkle_root(…)");
        assert_eq!(
            item.label_details.as_ref().unwrap().detail,
            Some("(use std::merkle::compute_merkle_root)".to_string()),
        );
    }

    #[test]
    async fn test_completes_after_first_letter_of_path() {
        let src = r#"
            fn main() {
                h>|<ello();
            }

            fn hello_world() {}
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "hello_world",
                CompletionItemKind::FUNCTION,
                Some("fn()".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_completes_after_colon_in_the_middle_of_an_ident_last_segment() {
        let src = r#"
            mod foo {
                pub fn bar() {}
            }

            fn main() {
                foo::>|<b
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn()".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_completes_after_colon_in_the_middle_of_an_ident_middle_segment() {
        let src = r#"
            mod foo {
                pub fn bar() {}
            }

            fn main() {
                foo::>|<b::baz
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn()".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_completes_at_function_call_name() {
        let src = r#"
            mod foo {
                pub fn bar() {}
            }

            fn main() {
                foo::b>|<x()
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn()".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_completes_at_method_call_name() {
        let src = r#"
            struct Foo {}

            impl Foo {
                pub fn bar(self) {}
            }

            fn x(f: Foo) {
                f.b>|<x()
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn(self)".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_completes_at_method_call_name_after_dot() {
        let src = r#"
            struct Foo {}

            impl Foo {
                pub fn bar(self) {}
            }

            fn x(f: Foo) {
                f.>|<()
            }
        "#;
        assert_completion_excluding_auto_import(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn(self)".to_string()),
            )],
        )
        .await;
    }
}
