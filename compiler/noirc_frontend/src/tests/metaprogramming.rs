use crate::hir::{
    comptime::ComptimeError,
    def_collector::{
        dc_crate::CompilationError,
        errors::{DefCollectorErrorKind, DuplicateType},
    },
};

use crate::tests::{
    assert_no_errors, assert_no_errors_and_to_string, check_errors, get_program_errors,
};

// Regression for #5388
#[test]
fn comptime_let() {
    let src = r#"fn main() {
        comptime let my_var = 2;
        assert_eq(my_var, 2);
    }"#;
    assert_no_errors(src);
}

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
    check_errors(src);
}

#[test]
fn comptime_type_in_runtime_code() {
    let source = "
    pub fn foo(_f: FunctionDefinition) {}
                   ^^^^^^^^^^^^^^^^^^ Comptime-only type `FunctionDefinition` cannot be used in runtime code
                   ~~~~~~~~~~~~~~~~~~ Comptime-only type used here
    ";
    check_errors(source);
}

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
    check_errors(src);
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
    "#;

    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Serialize<let N: u32> {
        fn serialize() {
        }
    }
    
    impl Serialize<1> for Field {
        fn serialize() {
        }
    }
    
    pub fn foobar() {
    }
    
    comptime fn attr(_f: FunctionDefinition) -> Quoted {
        let serialized_len: Field = 1_Field;
        quote {
            impl Serialize < $serialized_len > for Field {
                fn serialize() {
                    
                }
            }
        }
    }
    ");
}

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

    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    comptime fn make_new_struct(_s: TypeDefinition) -> Quoted {
        quote {
            struct Bar {
                
            }
        }
    }
    
    struct Bar {
    }
    
    struct Foo {
    }
    
    fn main() {
        let _: Foo = Foo { };
        let _: Bar = Bar { };
    }
    ");
}

#[test]
fn generate_function_with_macros() {
    let src = "
    #[foo]
    comptime fn foo(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar(x: i32) -> i32  {  
                let y = x + 1;
                y + 2
            }
        }
    }
    ";

    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    comptime fn foo(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar(x: i32) -> i32 {
                let y = x + 1;
                y + 2
            }
        }
    }
    
    pub fn bar(x: i32) -> i32 {
        let y: i32 = x + 1_i32;
        y + 2_i32
    }
    ");
}

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

    let mut errors = get_program_errors(src);
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
    check_errors(src);
}

#[test]
fn quote_code_fragments() {
    // TODO: have the error also point to `contact!` as a secondary
    // This test ensures we can quote (and unquote/splice) code fragments
    // which by themselves are not valid code. They only need to be valid
    // by the time they are unquoted into the macro's call site.
    let src = r#"
        fn main() {
            comptime {
                concat!(quote { assert( }, quote { false); });
                                                   ^^^^^ Assertion failed
            }
        }

        comptime fn concat(a: Quoted, b: Quoted) -> Quoted {
            quote { $a $b }
        }
    "#;
    check_errors(src);
}

#[test]
fn quote_code_fragments_no_failure() {
    let src = r#"
        fn main() {
            comptime {
                concat!(quote { assert( }, quote { true); });
            }
        }

        comptime fn concat(a: Quoted, b: Quoted) -> Quoted {
            quote { $a $b }
        }
    "#;

    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    fn main() {
        ()
    }
    
    comptime fn concat(a: Quoted, b: Quoted) -> Quoted {
        quote { $a $b }
    }    
    ");
}

#[test]
fn attempt_to_add_with_overflow_at_comptime() {
    let src = r#"
        fn main() -> pub u8 {
            comptime {
                255 as u8 + 1 as u8
                ^^^^^^^^^^^^^^^^^^^ Attempt to add with overflow
            }
        }

        "#;
    check_errors(src);
}

#[test]
fn attempt_to_divide_by_zero_at_comptime() {
    let src = r#"
        fn main() -> pub u8 {
            comptime {
                255 as u8 / 0
                ^^^^^^^^^^^^^ Attempt to divide by zero
            }
        }

        "#;
    check_errors(src);
}

#[test]
fn attempt_to_modulo_by_zero_at_comptime() {
    let src = r#"
        fn main() -> pub u8 {
            comptime {
                255 as u8 % 0
                ^^^^^^^^^^^^^ Attempt to calculate the remainder with a divisor of zero
            }
        }

        "#;
    check_errors(src);
}

#[test]
fn subtract_to_int_min() {
    // This would cause an integer underflow panic before
    let src = r#"
        fn main() {
            let _x: i8 = comptime {
                let y: i8 = -127;
                let z = y - 1;
                z
            };
        }
    "#;

    assert_no_errors(src);
}

#[test]
fn error_if_attribute_not_in_scope() {
    let src = r#"
        #[not_in_scope]
        ^^^^^^^^^^^^^^^ Attribute function `not_in_scope` is not in scope
        fn main() {}
    "#;
    check_errors(src);
}
