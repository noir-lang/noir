use crate::{
    check_errors,
    hir::{
        comptime::ComptimeError,
        def_collector::{
            dc_crate::CompilationError,
            errors::{DefCollectorErrorKind, DuplicateType},
        },
    },
};

use crate::{assert_no_errors, get_program_errors};

// Regression for #5388
#[named]
#[test]
fn comptime_let() {
    let src = r#"fn main() {
        comptime let my_var = 2;
        assert_eq(my_var, 2);
    }"#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn comptime_code_rejects_dynamic_variable() {
    let src = r#"
    fn main(x: Field) {
        comptime let my_var = (x - x) + 2;
                               ^ Non-comptime variable `x` referenced in comptime code
                               ~ Non-comptime variables can't be used in comptime code
        assert_eq(my_var, 2);
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn comptime_type_in_runtime_code() {
    let source = "
    pub fn foo(_f: FunctionDefinition) {}
                   ^^^^^^^^^^^^^^^^^^ Comptime-only type `FunctionDefinition` cannot be used in runtime code
                   ~~~~~~~~~~~~~~~~~~ Comptime-only type used here
    ";
    check_errors!(source);
}

#[named]
#[test]
fn macro_result_type_mismatch() {
    let src = r#"
        fn main() {
            comptime {
                let x = unquote!(quote { "test" });
                        ^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected type Field, found type str<4>
                let _: Field = x;
            }
        }

        comptime fn unquote(q: Quoted) -> Quoted {
            q
        }
    "#;
    check_errors!(src);
}

#[named]
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

    assert_no_errors!(src);
}

#[named]
#[test]
fn allows_references_to_structs_generated_by_macros() {
    let src = r#"
    comptime fn make_new_struct(_s: TypeDefinition) -> Quoted {
        quote { struct Bar {} }
    }

    #[make_new_struct]
    struct Foo {}

    fn main() {
        let _ = Foo {};
        let _ = Bar {};
    }
    "#;

    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_macros_inject_functions_with_name_collisions() {
    // This can't be tested using `check_errors` right now because the two secondary
    // errors land on the same span.
    let src = r#"
    comptime fn make_colliding_functions(_s: TypeDefinition) -> Quoted {
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

    let mut errors = get_program_errors!(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ComptimeError(ComptimeError::ErrorRunningAttribute { error, .. }) =
        errors.remove(0)
    else {
        panic!("Expected a ComptimeError, got {:?}", errors[0]);
    };

    let CompilationError::DefinitionError(DefCollectorErrorKind::Duplicate {
        typ: DuplicateType::Function,
        first_def,
        ..
    }) = *error
    else {
        panic!("Expected a duplicate error");
    };

    assert_eq!(first_def.as_str(), "foo");
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn does_not_fail_to_parse_macro_on_parser_warning() {
    let src = r#"
    #[make_bar]
    pub unconstrained fn foo() {}

    comptime fn make_bar(_: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar() {
                unsafe { 
                ^^^^^^ Unsafe block must have a safety comment above it
                ~~~~~~ The comment must start with the "Safety: " word
                    foo();
                }
            }
        }
    }

    fn main() {
        bar()
    }
    "#;
    check_errors!(src);
}
