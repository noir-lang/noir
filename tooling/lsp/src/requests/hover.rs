use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::PathString;
use from_reference::hover_from_reference;
use from_visitor::hover_from_visitor;
use lsp_types::{Hover, HoverParams};

use crate::LspState;

use super::process_request;

mod from_reference;
mod from_visitor;

pub(crate) fn on_hover_request(
    state: &mut LspState,
    params: HoverParams,
) -> impl Future<Output = Result<Option<Hover>, ResponseError>> {
    let uri = params.text_document_position_params.text_document.uri.clone();
    let position = params.text_document_position_params.position;
    let result = process_request(state, params.text_document_position_params, |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        let file_id = args.files.get_file_id(&path);
        hover_from_reference(file_id, position, &args)
            .or_else(|| hover_from_visitor(file_id, position, &args))
    });

    future::ready(result)
}

#[cfg(test)]
mod hover_tests {
    use crate::test_utils;

    use super::*;
    use lsp_types::{
        HoverContents, Position, TextDocumentIdentifier, TextDocumentPositionParams, Url,
        WorkDoneProgressParams,
    };
    use tokio::test;

    async fn assert_hover(directory: &str, file: &str, position: Position, expected_text: &str) {
        let hover_text = get_hover_text(directory, file, position).await;
        assert_eq!(hover_text, expected_text);
    }

    async fn get_hover_text(directory: &str, file: &str, position: Position) -> String {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

        // noir_text_document is always `src/main.nr` in the workspace directory, so let's go to the workspace dir
        let noir_text_document = noir_text_document.to_file_path().unwrap();
        let workspace_dir = noir_text_document.parent().unwrap().parent().unwrap();

        let file_uri = Url::from_file_path(workspace_dir.join(file)).unwrap();

        let hover = on_hover_request(
            &mut state,
            HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: file_uri },
                    position,
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            },
        )
        .await
        .expect("Could not execute hover")
        .unwrap();

        let HoverContents::Markup(markup) = hover.contents else {
            panic!("Expected hover contents to be Markup");
        };

        markup.value
    }

    #[test]
    async fn hover_on_module() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 6, character: 9 },
            r#"    one
    mod subone"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 9, character: 20 },
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_generic_struct() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 46, character: 17 },
            r#"    one::subone
    struct GenericStruct<A, B> {
    }"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct_member() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 9, character: 35 },
            r#"    one::subone::SubOneStruct
    some_field: i32"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_trait() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 12, character: 17 },
            r#"    one::subone
    trait SomeTrait"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_global() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 15, character: 25 },
            r#"    one::subone
    global some_global: Field = 2"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_function() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 3, character: 4 },
            r#"    one
    pub fn function_one<A, B>()"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_local_function() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 2, character: 7 },
            r#"    two
    pub fn function_two()"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct_method() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 20, character: 6 },
            r#"    one::subone::SubOneStruct
    impl SubOneStruct
    fn foo(self, x: i32, y: i32) -> Field"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_local_var() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 25, character: 12 },
            "    let regular_var: Field",
        )
        .await;
    }

    #[test]
    async fn hover_on_local_mut_var() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 27, character: 4 },
            "    let mut mutable_var: Field",
        )
        .await;
    }

    #[test]
    async fn hover_on_local_var_whose_type_you_can_navigate_to() {
        let workspace_on_src_lib_path = std::env::current_dir()
            .unwrap()
            .join("test_programs/workspace/one/src/lib.nr")
            .canonicalize()
            .expect("Could not resolve root path");
        let workspace_on_src_lib_path = workspace_on_src_lib_path.to_string_lossy();

        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 51, character: 8 },
            &format!("    let x: BoundedVec<SubOneStruct, 3>\n\nGo to [SubOneStruct](file://{}#L4,12-4,24)", workspace_on_src_lib_path),
        )
        .await;
    }

    #[test]
    async fn hover_on_parameter() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 31, character: 12 },
            "    some_param: i32",
        )
        .await;
    }

    #[test]
    async fn hover_on_alias() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 34, character: 17 },
            r#"    one::subone
    type SomeAlias = i32"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_trait_on_call() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 39, character: 17 },
            r#"    std::default
    trait Default"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_std_module_in_use() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 36, character: 9 },
            r#"    std
    mod default"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_crate_module_in_call() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 15, character: 17 },
            r#"    one
    mod subone"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_without_crate_or_std_prefix() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 42, character: 4 },
            r#"    two
    mod other"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_with_crate_prefix() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 43, character: 11 },
            r#"    two
    mod other"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_on_struct_constructor() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 19, character: 12 },
            r#"    one
    mod subone"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_type_inside_generic_arguments() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 51, character: 30 },
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_crate_segment() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 0, character: 5 },
            "    crate one",
        )
        .await;
    }

    #[test]
    async fn hover_on_attribute_function() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 54, character: 2 },
            "    two\n    comptime fn attr(_: FunctionDefinition) -> Quoted",
        )
        .await;
    }

    #[test]
    async fn hover_on_generic_struct_function() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 70, character: 11 })
                .await;
        assert!(hover_text.starts_with(
            "    two::Foo
    impl<U> Foo<U>
    fn new() -> Foo<U>"
        ));
    }

    #[test]
    async fn hover_on_trait_impl_function_call() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 83, character: 16 })
                .await;
        assert!(hover_text.starts_with(
            "    two
    impl<A> Bar<A, i32> for Foo<A>
    fn bar_stuff(self)"
        ));
    }

    #[test]
    async fn hover_on_trait_impl_method_uses_docs_from_trait_method() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 92, character: 8 })
                .await;
        assert!(hover_text.contains("Some docs"));
    }

    #[test]
    async fn hover_on_function_with_mut_self() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 96, character: 10 })
                .await;
        assert!(hover_text.contains("fn mut_self(&mut self)"));
    }

    #[test]
    async fn hover_on_empty_enum_type() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 100, character: 8 })
                .await;
        assert!(hover_text.contains(
            "    two
    enum EmptyColor {
    }

---

 Red, blue, etc."
        ));
    }

    #[test]
    async fn hover_on_non_empty_enum_type() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 103, character: 8 })
                .await;
        assert!(hover_text.contains(
            "    two
    enum Color {
        Red(Field),
    }

---

 Red, blue, etc."
        ));
    }

    #[test]
    async fn hover_on_enum_variant() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 105, character: 6 })
                .await;
        assert!(hover_text.contains(
            "    two::Color
    Red(Field)

---

 Like a tomato"
        ));
    }

    #[test]
    async fn hover_on_enum_variant_in_call() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 109, character: 12 })
                .await;
        assert!(hover_text.contains(
            "    two::Color
    Red(Field)

---

 Like a tomato"
        ));
    }

    #[test]
    async fn hover_on_integer_literal() {
        let hover_text =
            get_hover_text("workspace", "two/src/lib.nr", Position { line: 9, character: 69 })
                .await;
        assert_eq!(&hover_text, "    Field\n---\nvalue of literal: `123 (0x7b)`");
    }
}
