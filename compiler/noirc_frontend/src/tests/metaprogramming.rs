use noirc_errors::Spanned;

use crate::{
    ast::Ident,
    hir::{
        def_collector::{
            dc_crate::CompilationError,
            errors::{DefCollectorErrorKind, DuplicateType},
        },
        resolution::errors::ResolverError,
        type_check::TypeCheckError,
    },
    parser::ParserErrorReason,
};

use super::{assert_no_errors, get_program_errors};

// Regression for #5388
#[test]
fn comptime_let() {
    let src = r#"fn main() {
        comptime let my_var = 2;
        assert_eq(my_var, 2);
    }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 0);
}

#[test]
fn comptime_type_in_runtime_code() {
    let source = "pub fn foo(_f: FunctionDefinition) {}";
    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::ComptimeTypeInRuntimeCode { .. })
    ));
}

#[test]
fn macro_result_type_mismatch() {
    let src = r#"
        fn main() {
            comptime {
                let x = unquote!(quote { "test" });
                let _: Field = x;
            }
        }

        comptime fn unquote(q: Quoted) -> Quoted {
            q
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::TypeMismatch { .. })
    ));
}

#[test]
fn unquoted_integer_as_integer_token() {
    let src = r#"
    trait Serialize<let N: u32> {
        fn serialize() {}
    }

    #[attr]
    pub fn foobar() {}

    comptime fn attr(_f: FunctionDefinition) -> Quoted {
        let serialized_len = 1;
        // We are testing that when we unquote $serialized_len, it's unquoted
        // as the token `1` and not as something else that later won't be parsed correctly
        // in the context of a generic argument.
        quote {
            impl Serialize<$serialized_len> for Field {
                fn serialize() { }
            }
        }
    }

    fn main() {}
    "#;

    assert_no_errors(src);
}

#[test]
fn allows_references_to_structs_generated_by_macros() {
    let src = r#"
    comptime fn make_new_struct(_s: StructDefinition) -> Quoted {
        quote { struct Bar {} }
    }

    #[make_new_struct]
    struct Foo {}

    fn main() {
        let _ = Foo {};
        let _ = Bar {};
    }
    "#;

    assert_no_errors(src);
}

#[test]
fn errors_if_macros_inject_functions_with_name_collisions() {
    let src = r#"
    comptime fn make_colliding_functions(_s: StructDefinition) -> Quoted {
        quote { 
            fn foo() {}
        }
    }

    #[make_colliding_functions]
    struct Foo {}

    #[make_colliding_functions]
    struct Bar {}

    fn main() {
        let _ = Foo {};
        let _ = Bar {};
        foo();
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].0,
        CompilationError::DefinitionError(
            DefCollectorErrorKind::Duplicate {
                typ: DuplicateType::Function,
                first_def: Ident(Spanned { contents, .. }),
                ..
            },
        ) if contents == "foo"
    ));
}

#[test]
fn uses_correct_type_for_attribute_arguments() {
    let src = r#"
    #[foo(32)]
    comptime fn foo(_f: FunctionDefinition, i: u32) {
        let y: u32 = 1;
        let _ = y == i;
    }

    #[bar([0; 2])]
    comptime fn bar(_f: FunctionDefinition, i: [u32; 2]) {
        let y: u32 = 1;
        let _ = y == i[0];
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_fail_to_parse_macro_on_parser_warning() {
    let src = r#"
    #[make_bar]
    pub unconstrained fn foo() {}

    comptime fn make_bar(_: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar() {
                unsafe { 
                    foo();
                }
            }
        }
    }

    fn main() {
        bar()
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ParseError(parser_error) = &errors[0].0 else {
        panic!("Expected a ParseError, got {:?}", errors[0].0);
    };

    assert!(matches!(parser_error.reason(), Some(ParserErrorReason::MissingSafetyComment)));
}
