use std::future::{self, Future};

use async_lsp::ResponseError;
use async_lsp::lsp_types::{Hover, HoverParams};
use from_reference::hover_from_reference;
use from_visitor::hover_from_visitor;

use crate::LspState;

use super::process_request;

mod from_reference;
mod from_visitor;

pub(crate) fn on_hover_request(
    state: &mut LspState,
    params: HoverParams,
) -> impl Future<Output = Result<Option<Hover>, ResponseError>> + use<> {
    let position = params.text_document_position_params.position;
    let result = process_request(state, params.text_document_position_params, |args| {
        let file_id = args.location.file;
        hover_from_reference(file_id, position, &args)
            .or_else(|| hover_from_visitor(file_id, position, &args))
    });

    future::ready(result)
}

#[cfg(test)]
mod hover_tests {
    use crate::test_utils;

    use super::*;
    use async_lsp::lsp_types::{
        HoverContents, TextDocumentIdentifier, TextDocumentPositionParams, WorkDoneProgressParams,
    };
    use tokio::test;

    /// Source is inline Noir with a `>|<` cursor marker. The fixture workspace
    /// (`test_programs/workspace`) supplies the `one` and `std` dependency crates;
    /// `two/src/lib.nr` itself is replaced by `src` for the duration of the test.
    async fn assert_hover(src: &str, expected_text: &str) {
        let hover_text = get_hover_text(src).await;
        assert_eq!(hover_text, expected_text);
    }

    async fn get_hover_text(src: &str) -> String {
        let (mut state, file_uri, position, _src) =
            test_utils::init_lsp_server_with_inline_source_and_cursor(
                "workspace",
                "two/src/lib.nr",
                src,
            )
            .await;

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
        // cSpell:disable
        assert_hover(
            "use one::>|<subone;",
            r#"    one
    mod subone"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_struct() {
        // cSpell:disable
        assert_hover(
            r#"use one::subone;
fn use_struct() {
    let _ = subone::>|<SubOneStruct { some_field: 0, some_other_field: 123 };
}"#,
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_generic_struct() {
        // cSpell:disable
        assert_hover(
            "use one::subone::>|<GenericStruct;",
            r#"    one::subone
    struct GenericStruct<A, B> {
    }"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_struct_member() {
        // cSpell:disable
        assert_hover(
            r#"use one::subone;
fn use_struct() {
    let _ = subone::SubOneStruct { >|<some_field: 0, some_other_field: 123 };
}"#,
            r#"    one::subone::SubOneStruct
    some_field: i32"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_trait() {
        // cSpell:disable
        assert_hover(
            "use one::subone::>|<SomeTrait;",
            r#"    one::subone
    trait SomeTrait"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_invalid_global() {
        // cSpell:disable
        assert_hover(
            r#"fn use_invalid_global() {
    let _ = one::subone::>|<invalid_global;
}"#,
            r#"    one::subone
    global invalid_global: Field = 0x02"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_valid_global() {
        // cSpell:disable
        assert_hover(
            r#"fn use_valid_global() {
    let _ = one::subone::>|<valid_global;
}"#,
            r#"    one::subone
    global valid_global: Field = 0x02"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_global_array() {
        assert_hover(
            "global a>|<rray: [Field; 3] = [1, 2 + 3, 4];",
            r#"    two
    global array: [Field; 3] = [0x01, 0x05, 0x04]"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_function() {
        assert_hover(
            r#"use one::function_one;
pub fn caller() {
    >|<function_one()
}"#,
            r#"    one
    pub fn function_one<A, B>()"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_local_function() {
        assert_hover(
            "pub fn >|<function_two() {}",
            r#"    two
    pub fn function_two()"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct_method() {
        // cSpell:disable
        assert_hover(
            r#"use one::subone;
fn use_struct_method() {
    let s = subone::SubOneStruct { some_field: 0, some_other_field: 2 };
    s.>|<foo(0, 1);
}"#,
            r#"    one::subone::SubOneStruct
    impl SubOneStruct
    fn foo(self, x: i32, y: i32) -> Field"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_local_var() {
        assert_hover(
            r#"fn use_local_var() {
    let regular_var = 0;
    let _ = >|<regular_var;
}"#,
            "    let regular_var: Field",
        )
        .await;
    }

    #[test]
    async fn hover_on_local_mut_var() {
        assert_hover(
            r#"fn use_local_var() {
    let mut mutable_var = 0;
    >|<mutable_var = 1;
}"#,
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

        let hover_text = get_hover_text(
            r#"use one::subone;
use std::collections::bounded_vec::BoundedVec;

fn instantiate_generic() {
    let >|<x: BoundedVec<subone::SubOneStruct, 3> = BoundedVec::new();
}"#,
        )
        .await;
        assert!(hover_text.contains("    let x: BoundedVec<SubOneStruct, 3>"));
        assert!(hover_text.contains("Go to [BoundedVec](noir-std:"));
        assert!(
            hover_text.contains(&format!(
                "[SubOneStruct](file://{workspace_on_src_lib_path}#L4,12-4,24)"
            ))
        );
    }

    #[test]
    async fn hover_on_parameter() {
        assert_hover(
            r#"fn use_parameter(some_param: i32) {
    let _ = >|<some_param;
}"#,
            "    some_param: i32",
        )
        .await;
    }

    #[test]
    async fn hover_on_alias() {
        // cSpell:disable
        assert_hover(
            "use one::subone::>|<SomeAlias;",
            r#"    one::subone
    type SomeAlias = i32"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_trait_on_call() {
        assert_hover(
            r#"use std::default::Default;
fn use_impl_method() {
    let _: i32 = >|<Default::default();
}"#,
            "    std::default\n    trait Default\n\n---\n\nReturn an implementation-defined default value for the given type.\nThis is most often a zeroed value or an empty container, but there\nare no actual restrictions on what an implementation could return.\n",
        )
        .await;
    }

    #[test]
    async fn hover_on_std_module_in_use() {
        assert_hover(
            "use std::>|<default::Default;",
            r#"    std
    mod default"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_crate_module_in_call() {
        // cSpell:disable
        assert_hover(
            r#"fn use_invalid_global() {
    let _ = one::>|<subone::invalid_global;
}"#,
            r#"    one
    mod subone"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_module_without_crate_or_std_prefix() {
        assert_hover(
            "mod >|<other;",
            r#"    two
    mod other"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_with_crate_prefix() {
        assert_hover(
            r#"mod other;
use crate::>|<other::other_function;"#,
            r#"    two
    mod other"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_on_struct_constructor() {
        // cSpell:disable
        assert_hover(
            r#"use one::subone;
fn use_struct_method() {
    let _ = >|<subone::SubOneStruct { some_field: 0, some_other_field: 2 };
}"#,
            r#"    one
    mod subone"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_type_inside_generic_arguments() {
        // cSpell:disable
        assert_hover(
            r#"use one::subone;
use std::collections::bounded_vec::BoundedVec;

fn instantiate_generic() {
    let x: BoundedVec<subone::>|<SubOneStruct, 3> = BoundedVec::new();
}"#,
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_crate_segment() {
        assert_hover("use o>|<ne::function_one;", "    crate one").await;
    }

    #[test]
    async fn hover_on_attribute_function() {
        assert_hover(
            r#"comptime fn attr(_: FunctionDefinition) -> Quoted {
    quote { pub fn hello() {} }
}

#[>|<attr]
pub fn foo() {}"#,
            "    two\n    comptime fn attr(_: FunctionDefinition) -> Quoted",
        )
        .await;
    }

    #[test]
    async fn hover_on_generic_struct_function() {
        let hover_text = get_hover_text(
            r#"struct Foo<T> {}

impl<U> Foo<U> {
    fn new() -> Self {
        Foo {}
    }
}

fn new_foo() -> Foo<i32> {
    Foo::n>|<ew()
}"#,
        )
        .await;
        assert!(hover_text.starts_with(
            "    two::Foo
    impl<U> Foo<U>
    fn new() -> Foo<U>"
        ));
    }

    #[test]
    async fn hover_on_trait_impl_function_call() {
        let hover_text = get_hover_text(
            r#"struct Foo<T> {}

impl<U> Foo<U> {
    fn new() -> Self { Foo {} }
}

trait Bar<T, U> {
    fn bar_stuff(self);
}

impl<A> Bar<A, i32> for Foo<A> {
    fn bar_stuff(self) {}
}

fn use_bar_stuff() {
    let foo = Foo::new();
    foo.bar_stuf>|<f();
}"#,
        )
        .await;
        assert!(hover_text.starts_with(
            "    two
    impl<A> Bar<A, i32> for Foo<A>
    fn bar_stuff(self)"
        ));
    }

    #[test]
    async fn hover_on_trait_impl_function_self_generics() {
        assert_hover(
            r#"trait NoGenericsTrait {
    fn quux(self);
}

impl<A, B> NoGenericsTrait for (A, B) {
    fn quux(self) {}
}

fn call_quux() {
    let t: (i32, i32) = (1, 2);
    t.>|<quux();
}"#,
            "    two
    impl<A, B> NoGenericsTrait for (A, B)
    fn quux(self)",
        )
        .await;
    }

    #[test]
    async fn hover_on_trait_impl_method_uses_docs_from_trait_method() {
        let hover_text = get_hover_text(
            r#"trait TraitWithDocs {
    /// Some docs
    fn foo();
}

impl TraitWithDocs for Field {
    fn >|<foo() {}
}"#,
        )
        .await;
        assert!(hover_text.contains("Some docs"));
    }

    #[test]
    async fn hover_on_function_with_mut_self() {
        let hover_text = get_hover_text(
            r#"struct Foo<T> {}

impl<U> Foo<U> {
    fn mut>|<_self(&mut self) {}
}"#,
        )
        .await;
        assert!(hover_text.contains("fn mut_self(&mut self)"));
    }

    #[test]
    async fn hover_on_empty_enum_type() {
        let hover_text = get_hover_text(
            r#"/// Red, blue, etc.
enum Empty>|<Color {}"#,
        )
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
        let hover_text = get_hover_text(
            r#"/// Red, blue, etc.
enum Co>|<lor {
    /// Like a tomato
    Red(Field),
}"#,
        )
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
        let hover_text = get_hover_text(
            r#"enum Color {
    /// Like a tomato
    Re>|<d(Field),
}"#,
        )
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
        let hover_text = get_hover_text(
            r#"enum Color {
    /// Like a tomato
    Red(Field),
}

fn test_enum() -> Color {
    Color::R>|<ed(1)
}"#,
        )
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
        // cSpell:disable
        let hover_text = get_hover_text(
            r#"use one::subone;
fn use_struct() {
    let _ = subone::SubOneStruct { some_field: 0, some_other_field: 1>|<23 };
}"#,
        )
        .await;
        // cSpell:enable
        assert_eq!(&hover_text, "    Field\n---\nvalue of literal: `123 (0x7b)`");
    }

    #[test]
    async fn hover_on_negative_integer_literal() {
        let hover_text = get_hover_text(
            r#"fn negative_integer() -> i32 {
    ->|<8
}"#,
        )
        .await;
        assert_eq!(&hover_text, "    i32\n---\nvalue of literal: `-8 (-0x08)`");
    }

    #[test]
    async fn hover_on_i32() {
        let hover_text = get_hover_text(
            r#"fn use_parameter(some_param: i3>|<2) {
    let _ = some_param;
}"#,
        )
        .await;
        assert_eq!(&hover_text, "    i32\n---\nThe 32-bit signed integer type.\n");
    }

    #[test]
    async fn hover_on_doc_comment_reference() {
        // cSpell:disable
        assert_hover(
            r#"/// See [o>|<ne::subone::SubOneStruct]
fn doc_comments_test() {}"#,
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
        // cSpell:enable
    }

    #[test]
    async fn hover_on_numeric_generic() {
        let hover_text = get_hover_text(
            r#"fn hover_on_numeric_generic<let N: u32>() {
    println(>|<N);
}"#,
        )
        .await;
        assert_eq!(&hover_text, "    let N: u32");
    }
}
