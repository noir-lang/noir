mod comptime_for;
mod skip_interpreter_on_fail;

use crate::{
    elaborator::UnstableFeature,
    hir::{
        comptime::ComptimeError,
        def_collector::{
            dc_crate::CompilationError,
            errors::{DefCollectorErrorKind, DuplicateType},
        },
    },
    test_utils::{get_monomorphized_with_stdlib, stdlib_src},
    tests::{check_errors_using_features, check_errors_with_stdlib},
};

use noirc_errors::CustomDiagnostic;

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
                   ^^^^^^^^^^^^^^^^^^ Comptime-only type `FunctionDefinition` cannot be used in non-comptime function
    ";
    check_errors(source);
}

#[test]
fn comptime_type_in_constructor_in_runtime_code() {
    let source = r#"
    comptime struct MetaOnly {
        value: Field,
    }

    struct RuntimeStruct {
        value: Field,
    }

    comptime type MetaAlias = RuntimeStruct;

    fn id<T>(x: T) -> T {
        x
    }

    fn main() -> pub Field {
        let direct = id(MetaOnly { value: 10 });
                        ^^^^^^^^ Comptime-only type `MetaOnly` cannot be used in non-comptime function
        let alias = MetaAlias { value: 32 };
                    ^^^^^^^^^ Comptime-only type `MetaAlias` cannot be used in non-comptime function
        direct.value + alias.value
    }
    "#;
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
fn method_macro_call_elaborates_quoted_argument_in_comptime_context() {
    let src = r#"
    pub struct Foo {}

    impl Foo {
        pub comptime fn second(_self: Self, q: Quoted) -> Quoted {
            q
        }
    }

    fn main() {
        let _ = Foo {}.second!(quote { 1 + 2 });
    }
    "#;
    assert_no_errors(src);
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
        let serialized_len = 1_u32;
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
        let serialized_len: u32 = 1_u32;
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

// Regression for #11880: comptime attributes on impl methods used to be silently ignored.
#[test]
fn generate_function_with_macros_on_impl_method() {
    let src = "
    pub struct Spam {}

    impl Spam {
        #[foo]
        pub fn struct_method() {}
    }

    pub comptime fn foo(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar(x: i32) -> i32 {
                x + 1
            }
        }
    }
    ";

    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    pub struct Spam {
    }

    impl Spam {
        pub fn struct_method() {
        }
    }

    impl Spam {
        pub fn bar(x: i32) -> i32 {
            x + 1_i32
        }
    }

    pub comptime fn foo(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar(x: i32) -> i32 {
                x + 1
            }
        }
    }
    ");
}

// Regression for asterite's review on #12649: when an attribute on an impl method
// generates a new function, the impl's `where_clause` must be carried into the
// synthetic impl so the generated function can use the bounded generics.
#[test]
fn generate_function_with_macros_on_impl_method_carries_where_clause() {
    let src = "
    pub trait MyDefault {
        fn my_default() -> Self;
    }

    pub struct Foo<T> {}

    impl<T> Foo<T> where T: MyDefault {
        #[generate_bar]
        pub fn foo() {}
    }

    pub comptime fn generate_bar(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar() -> T {
                T::my_default()
            }
        }
    }

    fn main() {}
    ";

    assert_no_errors(src);
}

#[test]
fn generate_function_with_macros_on_trait() {
    let src = "
    #[foo]
    trait MyTrait {}

    impl MyTrait for () {}

    comptime fn foo(_f: TraitDefinition) -> Quoted {
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
    trait MyTrait {

    }

    impl MyTrait for () {

    }

    comptime fn foo(_f: TraitDefinition) -> Quoted {
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
fn do_not_generate_function_with_macros_on_trait_impl() {
    let src = "
    trait MyTrait {}

    struct Foo {}

    #[foo]
    impl MyTrait for Foo {}

    comptime fn foo(_f: TraitImpl) -> Quoted {
                ^^^ unused function foo
                ~~~ unused function
        quote {
            pub fn bar() { }
        }
    }

    fn main() {
        let _ = Foo {};
    }
    ";
    check_errors(src);
}

/// Enum attributes are not run at compile-time.
#[test]
fn do_not_generate_function_with_macros_on_enum() {
    let src = "
    #[foo]
    enum MyEnum {
        Foo(u32),
    }

    comptime fn foo(_f: TypeDefinition) -> Quoted {
                ^^^ unused function foo
                ~~~ unused function
        quote {
            pub fn bar() { }
        }
    }

    fn main() {
        let _ = MyEnum::Foo;
    }
    ";
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
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
    // TODO(https://github.com/noir-lang/noir/issues/10601): have the error
    // also point to `concat!` as a secondary
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

#[test]
fn cannot_generate_module_declarations() {
    let src = r#"
        #[bad_attr]
        ~~~~~~~~~~~ While running this function attribute
        fn main() {}

        comptime fn bad_attr(_: FunctionDefinition) -> Quoted {
            quote { mod new_module; }
                    ^^^^^^^^^^^^^^^ Unsupported statement type to unquote
                    ~~~~~~~~~~~~~~~ Only functions, structs, globals, and impls can be unquoted here
        }
    "#;
    check_errors(src);
}

#[test]
fn cannot_generate_imports() {
    let src = r#"
        #[bad_attr]
        ~~~~~~~~~~~ While running this function attribute
        fn main() {}

        comptime fn bad_attr(_: FunctionDefinition) -> Quoted {
            quote { use std::hash; }
                    ^^^^^^^^^^^^^^ Unsupported statement type to unquote
                    ~~~~~~~~~~~~~~ Only functions, structs, globals, and impls can be unquoted here
        }
    "#;
    check_errors(src);
}

#[test]
fn cannot_generate_traits() {
    let src = r#"
        #[bad_attr]
        ~~~~~~~~~~~ While running this function attribute
        fn main() {}

        comptime fn bad_attr(_: FunctionDefinition) -> Quoted {
            quote { trait MyTrait {} }
                    ^^^^^^^^^^^^^^^^ Unsupported statement type to unquote
                    ~~~~~~~~~~~~~~~~ Only functions, structs, globals, and impls can be unquoted here
        }
    "#;
    check_errors(src);
}

#[test]
fn cannot_generate_type_aliases() {
    let src = r#"
        #[bad_attr]
        ~~~~~~~~~~~ While running this function attribute
        fn main() {}

        comptime fn bad_attr(_: FunctionDefinition) -> Quoted {
            quote { type MyType = Field; }
                    ^^^^^^^^^^^^^^^^^^^^ Unsupported statement type to unquote
                    ~~~~~~~~~~~~~~~~~~~~ Only functions, structs, globals, and impls can be unquoted here
        }
    "#;
    check_errors(src);
}

#[test]
fn cannot_generate_submodules() {
    let src = r#"
        #[bad_attr]
        ~~~~~~~~~~~ While running this function attribute
        fn main() {}

        comptime fn bad_attr(_: FunctionDefinition) -> Quoted {
            quote { mod inner { fn foo() {} } }
                    ^^^^^^^^^^^^^^^^^^^^^^^^^ Unsupported statement type to unquote
                    ~~~~~~~~~~~~~~~~~~~~~~~~~ Only functions, structs, globals, and impls can be unquoted here
        }
    "#;
    check_errors(src);
}

#[test]
fn cannot_generate_inner_attributes() {
    let src = r#"
        #[bad_attr]
        ~~~~~~~~~~~ While running this function attribute
        fn main() {}

        comptime fn bad_attr(_: FunctionDefinition) -> Quoted {
            quote { #![inner_attr] }
                    ^^^^^^^^^^^^^^ Unsupported statement type to unquote
                    ~~~~~~~~~~~~~~ Only functions, structs, globals, and impls can be unquoted here
        }
    "#;
    check_errors(src);
}

#[test]
fn attributes_run_in_textual_order_within_module() {
    let src = r#"
        comptime mut global counter: Field = 0;

        #[assert_source_order(0)]
        fn first() {}

        #[assert_source_order(1)]
        fn second() {}

        #[assert_source_order(2)]
        fn third() {}

        comptime fn assert_source_order(_: FunctionDefinition, expected: Field) {
            assert(counter == expected);
            counter += 1;
        }

        fn main() {
            let _ = first();
            let _ = second();
            let _ = third();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn impl_method_and_free_function_attributes_run_in_source_order() {
    let src = r#"
        comptime mut global counter: Field = 0;

        #[assert_source_order(0)]
        fn first_free() {}

        pub struct S {}

        impl S {
            #[assert_source_order(1)]
            fn m1() {}

            #[assert_source_order(2)]
            fn m2() {}
        }

        #[assert_source_order(3)]
        fn middle_free() {}

        impl S {
            #[assert_source_order(4)]
            fn m3() {}
        }

        #[assert_source_order(5)]
        fn last_free() {}

        comptime fn assert_source_order(_: FunctionDefinition, expected: Field) {
            assert(counter == expected);
            counter += 1;
        }

        fn main() {
            let _ = first_free();
            let _ = S::m1();
            let _ = S::m2();
            let _ = middle_free();
            let _ = S::m3();
            let _ = last_free();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn sibling_modules_run_in_textual_order() {
    let src = r#"
          comptime mut global counter: Field = 0;

          mod first_child {
              #[crate::assert_source_order(0)]
              pub fn first() {}
          }

          mod second_child {
              #[crate::assert_source_order(1)]
              pub fn second() {}
          }

          #[assert_source_order(2)]
          fn parent() {}

          comptime fn assert_source_order(_: FunctionDefinition, expected: Field) {
              assert(counter == expected);
              counter += 1;
          }

          fn main() {
              let _ = first_child::first();
              let _ = second_child::second();
              let _ = parent();
          }
      "#;
    assert_no_errors(src);
}

/// Attributes on methods of macro-generated impl blocks all share the source location
/// of the quoted fragment they were generated from, so the `(module, span)` run-order
/// sort cannot order them. Their run order must then follow generation order — which
/// requires the collection that holds the impls to iterate in insertion order.
#[test]
fn macro_generated_impl_attributes_run_in_generation_order() {
    let src = r#"
        comptime mut global counter: Field = 0;

        pub struct A {}
        pub struct B {}
        pub struct C {}
        pub struct D {}
        pub struct E {}
        pub struct F {}
        pub struct G {}
        pub struct H {}

        #[gen_impls]
        fn dummy() {}

        comptime fn gen_impls(_: FunctionDefinition) -> Quoted {
            let types: [(Quoted, Quoted); 8] = [
                (quote { A }, quote { 0 }),
                (quote { B }, quote { 1 }),
                (quote { C }, quote { 2 }),
                (quote { D }, quote { 3 }),
                (quote { E }, quote { 4 }),
                (quote { F }, quote { 5 }),
                (quote { G }, quote { 6 }),
                (quote { H }, quote { 7 }),
            ];
            let mut result = quote {};
            for index in 0..8 {
                let (typ, i) = types[index];
                result = quote { $result
                    impl $typ { #[assert_gen_order($i)] fn method(_self: Self) {} }
                };
            }
            result
        }

        comptime fn assert_gen_order(_: FunctionDefinition, expected: Field) {
            assert(counter == expected);
            counter += 1;
        }

        fn main() {
            dummy();
            let _ = A {}.method();
            let _ = B {}.method();
            let _ = C {}.method();
            let _ = D {}.method();
            let _ = E {}.method();
            let _ = F {}.method();
            let _ = G {}.method();
            let _ = H {}.method();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn child_module_attributes_run_before_parent() {
    let src = r#"
        comptime mut global counter: Field = 0;

        mod child {
            #[crate::assert_source_order(0)]
            pub fn child_fn() {}
        }

        #[assert_source_order(1)]
        fn parent_fn() {}

        comptime fn assert_source_order(_: FunctionDefinition, expected: Field) {
            assert(counter == expected);
            counter += 1;
        }

        fn main() {
            let _ = child::child_fn();
            let _ = parent_fn();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_child_modules_run_innermost_first() {
    let src = r#"
        comptime mut global counter: Field = 0;

        mod parent {
            pub mod child {
                #[crate::assert_source_order(0)]
                pub fn innermost() {}
            }

            #[crate::assert_source_order(1)]
            pub fn middle() {}
        }

        #[assert_source_order(2)]
        fn outermost() {}

        comptime fn assert_source_order(_: FunctionDefinition, expected: Field) {
            assert(counter == expected);
            counter += 1;
        }

        fn main() {
            let _ = parent::child::innermost();
            let _ = parent::middle();
            let _ = outermost();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_blocks_all_local_variables() {
    let src = r#"
        fn main() {
            comptime {
                let x = comptime { 5 };
                assert_eq(x, 5);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_let_used_in_separate_comptime_block() {
    let src = r#"
        fn main() {
            comptime let x = 5;
            comptime {
                let y = x + 1;
                assert_eq(y, 6);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_let_mut_in_separate_comptime_block() {
    let src = r#"
        fn main() {
            comptime let mut x = 0;
            comptime {
                x = 5;
            }
            assert_eq(x, 5);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn multiple_comptime_blocks_share_scope() {
    let src = r#"
        fn main() {
            comptime let x = 10;
            comptime { assert_eq(x, 10); }
            comptime { assert_eq(x + 5, 15); }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_statement_accesses_outer_comptime_variable() {
    let src = r#"
        fn main() {
            comptime {
                let x = 5;
                comptime {
                    let y = x + 1;
                    assert_eq(y, 6);
                }
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_expression_accesses_outer_comptime_variable() {
    let src = r#"
        fn main() {
            comptime {
                let x = 5;
                let y = comptime { x + 1 } ;
                assert_eq(y, 6);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_accesses_outer_comptime_func_variable() {
    let src = r#"
    comptime fn main() {
        let x = 0;
        comptime {
            let y = x + 1;
            assert_eq(y, 6);
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_with_mut_variable() {
    let src = r#"
        fn main() {
            comptime {
                let mut x = 0;
                comptime {
                    x = 5;
                }
                assert_eq(x, 5);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_mut_outer_comptime_func_variable() {
    let src = r#"
    comptime fn main() {
        let mut x = 0;
        comptime {
            x = 5;
        }
        assert_eq(x, 5);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_function_with_comptime_block_called_from_comptime() {
    let src = r#"
        comptime fn helper(x: Field) -> Field {
            comptime {
                assert_eq(x, 5);
            }
            x + 1
        }

        fn main() {
            comptime {
                let x = 5;
                let result = helper(x);
                assert_eq(result, 6);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn runtime_function_with_comptime_block_called_from_comptime() {
    let src = r#"
        fn helper(x: Field) -> Field {
            comptime {
                assert_eq(x, 5);
                          ^ Non-comptime variable `x` referenced in comptime code
                          ~ Non-comptime variables can't be used in comptime code
            }
            x + 1
        }

        fn main() {
            comptime {
                let x = 5;
                let result = helper(x);
                assert_eq(result, 6);
            }
        }
    "#;
    check_errors(src);
}

#[test]
fn nested_comptime_with_trait_method_calls() {
    let src = r#"
        trait MyTrait {
            fn foo() -> Field;
        }

        impl MyTrait for Field {
            fn foo() -> Field { 42 }
        }

        fn main() {
            comptime {
                let x = comptime {
                    Field::foo()
                };
                assert_eq(x, 42);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_comptime_with_generics() {
    let src = r#"
        trait MyTrait { }

        struct Foo { }

        impl MyTrait for Foo { }

        fn generic_fn<T>() -> Field where T: MyTrait {
            5
        }

        fn main() {
            comptime {
                comptime {
                    let x = generic_fn::<Foo>();
                    assert_eq(x, 5);
                }
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_if_expression() {
    let src = r#"
        fn main() {
            comptime {
                let x = if true { 5 } else { 10 };
                assert_eq(x, 5);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_loop_with_break() {
    let src = r#"
        fn main() {
            comptime {
                let mut i = 0;
                loop {
                    if i == 5 { break; }
                    i += 1;
                }
                assert_eq(i, 5);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_shadows_runtime_variable() {
    let src = r#"
        fn main() {
            let x = 10;
            comptime let x = 5;
            assert_eq(x, 5);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_shadows_comptime_variable() {
    let src = r#"
        fn main() {
            comptime let x = 5;
            comptime let x = 10;
            assert_eq(x, 10);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_block_explicit_type_mismatch() {
    let src = r#"
        fn main() {
            let _x: Field = comptime { true };
                            ^^^^^^^^^^^^^^^^^ Expected type Field, found type bool
        }
    "#;
    check_errors(src);
}

#[test]
fn empty_comptime_block() {
    let src = r#"
        fn main() {
            let _: () = comptime { };
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_uhashmap_of_vectors() {
    let src = r#"
    pub struct Option<T> {
        _is_some: bool,
        _value: T,
    }

    pub struct Slot<K, V> {
        _key_value: Option<(K, V)>,
        _is_deleted: bool,
    }

    pub struct UHashMap<K, V> {
        _table: [Slot<K, V>],
        _len: u32,
    }

    pub fn example_umap<T>() -> UHashMap<u32, T> {
        let _table = @[];
        let _len = 0;
        UHashMap { _table, _len }
    }

    fn main() {
        comptime let _ = {
            let _ = example_umap::<[u32]>();
        };
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_uhashmap_of_vectors_attribute() {
    let src = r#"
    pub struct Option<T> {
        _is_some: bool,
        _value: T,
    }

    impl<T> Option<T> {
        pub fn none(zeroed_value: T) -> Self {
            Self { _is_some: false, _value: zeroed_value }
        }
    }

    pub struct Slot<K, V> {
        _key_value: Option<(K, V)>,
        _is_deleted: bool,
    }

    impl<K, V> Slot<K, V> {
        fn default_slot(zeroed_value: (K, V)) -> Slot<K, V> {
            Slot { _key_value: Option::none(zeroed_value), _is_deleted: false }
        }
    }

    pub struct UHashMap<K, V> {
        _table: [Slot<K, V>],
        _len: u32,
    }

    impl<K, V> UHashMap<K, V> {
        fn default_umap(zeroed_value: (K, V)) -> UHashMap<K, V>
        {
            let _table = @[Slot::default_slot(zeroed_value)];
            let _len = 0;
            UHashMap { _table, _len }
        }
    }

    comptime fn empty_function_definition_vector() -> [FunctionDefinition] {
        @[]
    }

    comptime mut global REGISTRY: UHashMap<bool, [FunctionDefinition]> =
        UHashMap::default_umap((false, empty_function_definition_vector()));

    comptime fn add_to_registry(
        _registry: &mut UHashMap<bool, [FunctionDefinition]>,
        _f: FunctionDefinition,
    ) { }

    #[attr]
    pub fn foo() {}

    comptime fn attr(function: FunctionDefinition) {
        add_to_registry(&mut REGISTRY, function);
    }

    fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_11016() {
    let src = "
    fn main() {
        let _s1 = comptime {
            foo
            ^^^ cannot find `foo` in this scope
            ~~~ not found in this scope
        };
        call!(quote {});
    }

    comptime fn call(x: Quoted) -> Quoted {
        quote { $x() }
    }
    ";
    check_errors(src);
}

#[test]
fn varargs_on_non_comptime_function() {
    let src = "
    #[varargs]
    ^^^^^^^^^^ #[varargs] can only be applied to comptime functions
    fn main() {
    }
    ";
    check_errors(src);
}

#[test]
fn varargs_on_function_without_arguments() {
    let src = "
    #[varargs]
    ^^^^^^^^^^ #[varargs] requires its function to have at least one parameter
    pub comptime fn foo() {}

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn varargs_on_function_without_last_vector_parameter() {
    let src = "
    #[foo(1, 2, 3, 4)] // Make sure no error is triggered here because of the varargs error
    #[varargs]
    pub comptime fn foo(_: FunctionDefinition, _x: Field, _y: Field) {}
                                                          ^^ The last parameter of a #[varargs] function must be a vector

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn unify_comptime_block_expression_with_target_type() {
    let src = r#"
    fn main() {
        let _: u8 = comptime { 1 };
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn unify_comptime_block_statement_with_target_type() {
    let src = r#"
    fn main() {
    }

    pub fn foo() -> u8 {
        comptime { 1 }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn error_on_self_on_trait_impl_for_comptime_type_on_non_comptime_function_with_explicit_self() {
    let src = r#"
    trait Trait {
        fn foo(self) -> Self;
    }

    impl Trait for Quoted {
        fn foo(self: Self) -> Self {
                              ^^^^ Comptime-only type `Quoted` cannot be used in non-comptime function
                     ^^^^ Comptime-only type `Quoted` cannot be used in non-comptime function
            self
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn no_duplicate_comptime_type_error_for_self_type_variable() {
    let src = r#"
    trait Trait {
        fn foo(self) -> Self;
    }

    impl Trait for Quoted {
        fn foo(self: Self) -> Self {
            self
        }
    }

    fn main() {}
    "#;
    let errors = get_program_errors(src);
    // 2 uses of Quoted in non-comptime positions (the Self types) (with duplicates we were getting 4 diagnostics)
    assert_eq!(errors.len(), 2);
}

#[test]
fn error_on_self_on_trait_impl_for_comptime_type_on_non_comptime_function_with_implicit_self() {
    let src = r#"
    trait Trait {
        fn foo(self);
    }

    impl Trait for Quoted {
        fn foo(self) {
               ^^^^ Comptime-only type `Quoted` cannot be used in non-comptime function
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn zeroed_comptime_type() {
    let module_hash_str = "
        #[builtin(module_hash)]
        comptime fn module_hash(_module: Module) -> Field {}
    ";
    let src = r#"
    fn main() {
        comptime {
            let m: Module = zeroed();
            let _ = module_hash(m);
                                ^ Expected a concrete `Module` but a zeroed value was given
                                ~ A zeroed value of `Module` may be created to satisfy the type system, but it's not expected to be used
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::ZEROED, module_hash_str]);
}

#[test]
fn zeroed_array_of_references_does_not_alias_in_comptime() {
    // Each slot of a zeroed `[&mut Field; N]` must own its own allocation in the comptime
    // interpreter. If the slots aliased, writing through one would be observable through the
    // others and the comptime asserts below would fail during elaboration.
    let src = r#"
    fn main() {
        comptime {
            let arr: [&mut Field; 3] = zeroed();
            *arr[1] = 7;
            assert_eq(*arr[0], 0);
            assert_eq(*arr[1], 7);
            assert_eq(*arr[2], 0);
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::ZEROED]);
}

#[test]
fn recursive_attribute_causes_expansion_limit_error() {
    use crate::elaborator::MAX_MACRO_EXPANSION_DEPTH;
    use crate::hir::comptime::InterpreterError;

    let src = r#"
    #[foo]
    comptime fn foo(_: FunctionDefinition) -> Quoted {
        quote {
            #[foo]
            fn bar() {}
        }
    }

    fn main() {}
    "#;
    // Fetch the errors directly as we will get many repeated errors up until the recursion limit is hit
    let errors = get_program_errors(src);
    // Ignore any unused function warnings
    let errors = errors.into_iter().filter(|err| err.is_error()).collect::<Vec<_>>();
    assert!(errors.len() <= MAX_MACRO_EXPANSION_DEPTH);

    // Helper to check for the recursion limit error, which may be wrapped in ComptimeError::ErrorRunningAttribute
    fn is_recursion_limit_error(error: &CompilationError) -> bool {
        match error {
            CompilationError::InterpreterError(
                InterpreterError::AttributeRecursionLimitExceeded { .. },
            ) => true,
            CompilationError::ComptimeError(ComptimeError::ErrorRunningAttribute {
                error, ..
            }) => is_recursion_limit_error(error),
            _ => false,
        }
    }

    // The test should produce the recursion limit error
    let has_recursion_limit_error = errors.iter().any(is_recursion_limit_error);
    assert!(has_recursion_limit_error, "Expected AttributeRecursionLimitExceeded error");
}

/// Verifies that mutually recursive attributes are caught by the global macro expansion depth limit.
/// Three mutually recursive attributes: foo -> bar -> baz -> foo -> ...
/// With a global counter, this correctly errors at [`crate::elaborator::MAX_MACRO_EXPANSION_DEPTH`] total expansions.
#[test]
fn mutually_recursive_attributes_cause_expansion_limit_error() {
    use crate::elaborator::MAX_MACRO_EXPANSION_DEPTH;
    use crate::hir::comptime::InterpreterError;

    let src = r#"
    #[foo]
    comptime fn foo(_: FunctionDefinition) -> Quoted {
        quote {
            #[bar]
            fn generated_by_foo() {}
        }
    }

    #[bar]
    comptime fn bar(_: FunctionDefinition) -> Quoted {
        quote {
            #[baz]
            fn generated_by_bar() {}
        }
    }

    #[baz]
    comptime fn baz(_: FunctionDefinition) -> Quoted {
        quote {
            #[foo]
            fn generated_by_baz() {}
        }
    }

    fn main() {}
    "#;

    let errors = get_program_errors(src);
    // Ignore any unused function warnings
    let errors = errors.into_iter().filter(|err| err.is_error()).collect::<Vec<_>>();
    // With a global depth counter, mutual recursion is detected at the same depth as single-function
    // recursion. If tracking were per-function, 3 mutually recursive functions could generate up to
    // 3 × MAX_MACRO_EXPANSION_DEPTH errors before any single counter hit the limit.
    assert!(errors.len() <= MAX_MACRO_EXPANSION_DEPTH);

    fn is_recursion_limit_error(error: &CompilationError) -> bool {
        match error {
            CompilationError::InterpreterError(
                InterpreterError::AttributeRecursionLimitExceeded { .. },
            ) => true,
            CompilationError::ComptimeError(ComptimeError::ErrorRunningAttribute {
                error, ..
            }) => is_recursion_limit_error(error),
            _ => false,
        }
    }

    let has_recursion_limit_error = errors.iter().any(is_recursion_limit_error);
    assert!(
        has_recursion_limit_error,
        "Expected AttributeRecursionLimitExceeded error for mutually recursive attributes"
    );
}

#[test]
fn many_non_recursive_attributes_do_not_trigger_macro_expansion_limit() {
    use std::fmt::Write;

    // Verifies that the recursion limit tracks depth, not total calls.
    // A program with many sequential (non-nested) uses of the same attribute should work
    // because each attribute completes before the next starts, keeping depth at 1.
    let count = 50;
    let functions: String = (1..=count).fold(String::new(), |mut output, i| {
        let _ = writeln!(output, "    #[attr] fn f{i}() {{}}");
        output
    });
    let calls: String = (1..=count).fold(String::new(), |mut output, i| {
        let _ = write!(output, "f{i}(); ");
        output
    });
    let src = format!(
        r#"
    comptime fn attr(_: FunctionDefinition) {{}}

{functions}
    fn main() {{
        {calls}
    }}
    "#
    );
    assert_no_errors(&src);
}

#[test]
fn unquote_in_nested_quote() {
    let src = r#"
    #[foo]
    pub comptime fn foo(_: FunctionDefinition) -> Quoted {
        let x = 0;
        quote {
            pub comptime fn bar() -> Quoted {
                quote { $x }
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn substitute_unquoted_in_nested_quote() {
    let src = r#"
    fn main() {
        do_func!(
            |i: u32| {
                quote {
                    $do_func!(|_| {
                        quote {
                            let _ = $i;
                        }
                    });
            }
            },
        );
    }

    pub comptime fn do_func(body: fn(u32) -> Quoted) -> Quoted {
        body(123)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn invalid_quote_escape() {
    let src = r#"
        fn main() {
            comptime {
                let _ = quote { \1 };
                                 ^ `1` cannot be escaped in quoted expressions
                                 ~ Only `$` may be escaped in `quote` expressions
            }
        }
    "#;
    check_errors(src);
}

#[test]
fn escape_nested_unquote() {
    let src = r#"
        // unroll_loop has been modified to remove stdlib fns so it no longer conceptually unrolls loops
        pub comptime fn unroll_loop(start: u32, end: u32, body: fn(u32) -> Quoted) -> Quoted {
            let mut iterations = quote[];
            for i in start..end {
                iterations = body(i);
            }
            iterations
        }

        pub fn u64s_to_bytes(row: [u64; 4]) -> [u8; 32] {
            let mut result: [u8; 32] = [0; 32];
            unroll_loop!(
                0_u32,
                4_u32,
                |i| {
                    quote {
                    $unroll_loop!(0_u32, 8_u32, |j| {
                        let i = $i;
                        let byte_idx = i * 8 + j;
                        let shift = (j * 8) as u64;
                        quote {
                            result[\$byte_idx] = (((row[$i] >> \$shift) << 56) >> 56) as u8;
                        }
                    });
                }
                },
            );

            result
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn unifies_macro_call_type_with_variable_type_in_comptime_block() {
    let src = r#"
    comptime fn unquote(code: Quoted) -> Quoted {
        code
    }

    struct Foo<let N: u32> {}

    impl<let N: u32> Foo<N> {
        fn len(_self: Self) -> u32 {
            N
        }
    }

    fn main() -> pub u32 {
        comptime {
            let foo: Foo<_> = unquote!(quote { Foo::<10> {} });
            foo.len()
        }
    }
    "#;
    assert_no_errors(src);
}

// Regression test for https://github.com/noir-lang/noir/issues/11575
#[test]
fn path_inside_module_attribute() {
    let src = r#"
    pub mod one {
        pub comptime fn attr(_: Module, _: Config) {}

        pub struct Config {}

        impl Config {
            pub fn new() -> Self {
                Self {}
            }
        }
    }

    use one::{attr, Config};

    #[attr(Config::new())]
    mod coco {}

    pub fn main() {
        let _ = Config::new();
    }
    "#;
    assert_no_errors(src);
}

// Regression: macro-call validation was bypassed inside comptime blocks
#[test]
fn non_comptime_macro_call_in_comptime_block() {
    let src = r#"
    fn not_comptime() -> Field {
        7
    }

    fn main() {
        let _x: Field = comptime {
            not_comptime!()
            ^^^^^^^^^^^^^^^ This macro call is to a non-comptime function
            ~~~~~~~~~~~~~~~ Macro calls must be to comptime functions
            ^^^^^^^^^^^^^^^ Expected macro call to return a `Quoted` but found a(n) `Field`
            ~~~~~~~~~~~~~~~ Macro calls must return quoted values, otherwise there is no code to insert.
            ~~~~~~~~~~~~~~~ Hint: remove the `!` from the end of the function name.
        };
    }
    "#;
    check_errors(src);
}

// Regression: comptime fn returning non-Quoted accepted as macro in comptime block
#[test]
fn comptime_fn_returning_non_quoted_macro_call_in_comptime_block() {
    let src = r#"
    comptime fn bad_macro() -> Field {
        42
    }

    fn main() {
        let _x: Field = comptime {
            bad_macro!()
            ^^^^^^^^^^^^ Expected macro call to return a `Quoted` but found a(n) `Field`
            ~~~~~~~~~~~~ Macro calls must return quoted values, otherwise there is no code to insert.
            ~~~~~~~~~~~~ Hint: remove the `!` from the end of the function name.
        };
    }
    "#;
    check_errors(src);
}

// Regression: function-value macro call accepted in comptime block
#[test]
fn function_value_macro_call_in_comptime_block() {
    let src = r#"
    fn not_comptime() -> Field {
        7
    }

    fn main() {
        let _x: Field = comptime {
            let f = not_comptime;
            f!()
            ^^^^ Invalid syntax in macro call
            ~~~~ Macro calls must call a comptime function directly, they cannot use higher-order functions
            ^^^^ Expected macro call to return a `Quoted` but found a(n) `Field`
            ~~~~ Macro calls must return quoted values, otherwise there is no code to insert.
            ~~~~ Hint: remove the `!` from the end of the function name.
        };
    }
    "#;
    check_errors(src);
}

#[test]
fn match_in_comptime_errors_instead_of_panicking() {
    let src = r#"
    enum Foo { Bar }

    fn main() {
        comptime {
            let foo = Foo::Bar;
            match foo { Foo::Bar => {} }
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Match expressions in comptime code is currently unimplemented
        };
    }
    "#;
    check_errors(src);
}

#[test]
fn runtime_variable_in_macro_gives_specific_error() {
    let src = r#"
    comptime fn ident(val: Quoted) -> Quoted {
        val
    }

    comptime fn wrap_with_add(x: Field) -> Quoted {
        quote { $x + 41 }
    }

    fn main() {
        let x = 1;
            ^ unused variable x
            ~ unused variable
        let _y: Field = wrap_with_add!(ident!(quote { x }));
                                                      ^ variable `x` is a runtime variable and cannot be used in comptime code
                                                      ~ this variable is not available in comptime
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_allow_constructing_struct_with_private_fields_with_macro_call() {
    let src = r#"
    mod victim_crate {
        pub struct Account {
            balance: Field,
        }
    }

    use victim_crate::Account;

    comptime fn unquote(code: Quoted) -> Quoted {
        code
    }

    fn main() {
        let _: Account = unquote!(quote { Account { balance: 0 }});
                                                    ^^^^^^^ balance is private and not visible from the current module
                                                    ~~~~~~~ balance is private
    }
    "#;
    check_errors(src);
}

#[test]
fn function_generated_with_constraint_does_not_drop_constraint() {
    let src = r#"
    pub trait Constraint {}
    pub trait Target {
        fn run(self);
    }
    pub struct Wrapper<T> {
        inner: T,
    }
    impl Constraint for i32 {}

    pub struct Marker {}

    #[generate_function]
    comptime fn generate_function(_: FunctionDefinition) -> Quoted {
        quote {
            fn run<T: Constraint>(_: T) {}
        }
    }

    fn main() {
        run(true);
        ^^^ No matching impl found for `bool: Constraint`
        ~~~ No impl for `bool: Constraint`
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_generated_with_constraint_does_not_drop_constraint() {
    let src = r#"
    pub trait Constraint {}
    pub trait Target {
        fn run(self);
    }
    pub struct Wrapper<T> {
        inner: T,
    }
    impl Constraint for i32 {}

    #[generate_impl]
    pub struct Marker {}

    comptime fn generate_impl(_s: TypeDefinition) -> Quoted {
        quote {
            impl<T: Constraint> Wrapper<T> {
                fn run(_self: Self) { 
                }
            }
        }
    }

    fn main() {
        let w = Wrapper { inner: false };
        w.run();
        ^^^^^ No matching impl found for `bool: Constraint`
        ~~~~~ No impl for `bool: Constraint`
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_impl_generated_with_constraint_does_not_drop_constraint() {
    let src = r#"
    pub trait Constraint {}
    pub trait Target {
        fn run(self);
    }
    pub struct Wrapper<T> {
        inner: T,
    }
    impl Constraint for i32 {}

    #[generate_impl]
    pub struct Marker {}

    comptime fn generate_impl(_s: TypeDefinition) -> Quoted {
        quote {
            impl<T: Constraint> Target for Wrapper<T> {
                fn run(_self: Self) { }
            }
        }
    }

    fn main() {
        let w = Wrapper { inner: false };
        w.run();
        ^^^^^ No matching impl found for `bool: Constraint`
        ~~~~~ No impl for `bool: Constraint`
    }
    "#;
    check_errors(src);
}

#[test]
fn inline_bound_on_quoted_generic_is_preserved() {
    let stdlib = r#"
        pub enum Option<T> { None, Some(T) }
        impl TypeDefinition {
            #[builtin(type_def_generics)]
            pub comptime fn generics(self) -> [(Type, Option<Type>)] {}
        }
    "#;
    let src = r#"
    pub trait Constraint {}
    pub trait Target { fn run(self); }
    pub struct Wrapper<T> { inner: T }
    impl Constraint for i32 {}

    #[generate_impl]
    pub struct Marker<T> {}

    comptime fn generate_impl(s: TypeDefinition) -> Quoted {
        let t = s.generics()[0].0;
        quote {
            impl<$t: Constraint> Target for Wrapper<$t> {
                fn run(_self: Self) { }
            }
        }
    }

    fn main() {
        let w = Wrapper { inner: false };
        w.run();
        ^^^^^ No matching impl found for `bool: Constraint`
        ~~~~~ No impl for `bool: Constraint`
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

#[test]
fn reference_generated_struct_in_function_signature() {
    let src = r#"
    #[make_struct]
    pub fn foo() {}

    comptime fn make_struct(_f: FunctionDefinition) -> Quoted {
        quote {
            pub struct MyStruct {}
        }
    }

    pub fn bar(_: MyStruct) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_two_generated_structs_should_work() {
    let src = r#"
    #[gen_struct(quote { Foo })]
    mod Foo {
        #[super::gen_struct(quote { Bar })]
        pub mod Bar {}
    }

    comptime fn gen_struct(_: Module, name: Quoted) -> Quoted {
        quote {
            pub struct $name {
            }
        }
    }

    pub fn use_struct(_: Foo::Foo) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_generated_struct_in_impl() {
    let src = r#"
    #[make_struct]
    pub fn foo() {}

    comptime fn make_struct(_f: FunctionDefinition) -> Quoted {
        quote {
            pub struct MyStruct {}
        }
    }

    pub struct Bar {}

    impl Bar {
        pub fn bar(_: MyStruct) {}
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_generated_struct_in_trait_impl() {
    let src = r#"
    #[make_struct]
    pub fn foo() {}

    comptime fn make_struct(_f: FunctionDefinition) -> Quoted {
        quote {
            pub struct MyStruct {}
        }
    }

    pub struct Bar {}
    pub trait Trait {
        fn bar(_: MyStruct);
    }

    impl Trait for Bar {
        fn bar(_: MyStruct) {}
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_generated_struct_in_another_struct_field() {
    let src = r#"
    #[make_struct]
    pub fn foo() {}

    comptime fn make_struct(_f: FunctionDefinition) -> Quoted {
        quote {
            pub struct MyStruct {}
        }
    }

    pub struct Bar {
        s: MyStruct,
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_generated_struct_in_a_global() {
    let src = r#"
    #[make_struct]
    pub fn foo() {}

    comptime fn make_struct(_f: FunctionDefinition) -> Quoted {
        quote {
            pub struct MyStruct {}
        }
    }

    pub global s: MyStruct = MyStruct {};
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_generated_struct_in_an_enum_variant() {
    let src = r#"
    #[make_struct]
    pub fn foo() {}

    comptime fn make_struct(_f: FunctionDefinition) -> Quoted {
        quote {
            pub struct MyStruct {}
        }
    }

    pub enum Bar {
        Variant(MyStruct),
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    crate::tests::assert_no_errors_using_features(src, &features);
}

// Regression for https://github.com/noir-lang/noir-claude/issues/1047:
// `as_witness` is declared `fn(Field) -> ()`, so calling it as the final expression
// of an inferred `comptime` block must produce a unit value, not the `Field` argument.
#[test]
fn comptime_as_witness_returns_unit() {
    let stdlib = r#"
        #[builtin(as_witness)]
        pub fn as_witness(_x: Field) {}
    "#;
    let src = r#"
    fn main() -> pub Field {
                     ^^^^^ expected type Field, found type ()
                     ~~~~~ expected Field because of return type
        let x = comptime {
            as_witness(1)
        };
        x
        ~ () returned here
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

#[test]
fn varargs_through_vector_alias_rejects_vector_argument() {
    // When the varargs parameter type is an alias to a vector, the element type used to check
    // each extra argument must be the vector's element type (`Field`), not the alias itself.
    let src = "
    type Args = [Field];

    #[attr([1, 2])]
           ^^^^^^ Expected type Field, found type [Field; 2]
    pub fn target() {}

    #[varargs]
    comptime fn attr(_f: FunctionDefinition, _xs: Args) {}

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn quote_at_runtime() {
    let src = r#"
    fn main() {
        foo(quote { test })
            ^^^^^^^^^^^^^^ `quote` cannot be used in runtime code
            ~~~~~~~~~~~~~~ Wrap this in a `comptime` block or function to use it
    }

    fn foo(q: Quoted) {
              ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime function
        let _ = q;
    }
    "#;
    check_errors(src);
}

#[test]
fn varargs_through_vector_alias_accepts_scalar_arguments() {
    // The valid spelling `#[attr(1, 2)]` must type-check: each extra argument unifies with the
    // alias's element type `Field`, not with the alias `Args` itself.
    let src = "
    type Args = [Field];

    #[attr(1, 2)]
    pub fn target() {}

    #[varargs]
    comptime fn attr(_f: FunctionDefinition, _xs: Args) {}

    fn main() {}
    ";
    assert_no_errors(src);
}

#[test]
fn quoted_in_non_comptime_global() {
    let src = r#"
    global foo: Quoted = quote { 1 };
                ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime global

    fn main() {
        let _ = foo;
    }
    "#;
    check_errors(src);
}

#[test]
fn macro_inserts_non_generic_type_into_generics_list() {
    let stdlib = r#"
        impl Quoted {
            #[builtin(quoted_as_type)]
            pub comptime fn as_type(self) -> Type {}
        }
    "#;
    let src = r#"
    #[foo]
    ~~~~~~ While running this function attribute
    comptime fn foo(_: FunctionDefinition) -> Quoted {
        let t = quote { [i32; 3] }.as_type();
        quote {
            struct Foo<$t> {
                        ^ Type `[i32; 3]` was inserted into a generics list from a macro, but it is not a generic
                        ~ Type `[i32; 3]` is not a generic
                x: $t,
            }
        }
    }

    fn main() {
        let _ = Foo::<i32> { x: [1, 2, 3] };
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

#[test]
fn type_definition_attribute_assertion_failure() {
    let src = r#"
    #[fail_assert]
    pub struct Foo { x: Field }

    comptime fn fail_assert(_typ: TypeDefinition) {
        assert(false);
               ^^^^^ Assertion failed
    }

    fn main() {
        let _ = Foo { x: 1 };
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_attribute_fails_to_parse_token_stream_into_item() {
    let stdlib = r#"
        impl Quoted {
            #[builtin(quoted_as_type)]
            pub comptime fn as_type(self) -> Type {}
        }
    "#;
    let src = r#"
    #[foo]
    ~~~~~~ Failed to parse macro's token stream into top-level item
    fn main() {}

    comptime fn foo(_f: FunctionDefinition) -> Quoted {
        let t = quote { Field }.as_type();
        quote { use $t; }
                     ^ Expected an identifier, `crate`, `dep` or `super` but found '(type)'
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

#[test]
fn comptime_pattern_match_struct_params() {
    let src = r#"
    pub struct Foo {
        msg: str<5>,
        value: u32,
    }
    pub struct Bar {
        msg: str<5>,
        value: u32,
    }
    fn main() {
        comptime {
            print_foo(Bar { msg: "Hello", value: 123 })
                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected type Foo, found type Bar
        }
    }

    fn print_foo(_foo: Foo) {}
    "#;
    check_errors(src);
}

#[test]
fn comptime_signed_division_by_minus_one_overflow() {
    let src = r#"
    fn main() {
        let _ = comptime { -128_i8 / -1 };
                           ^^^^^^^^^^^^ Attempt to divide with overflow
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_signed_modulo_by_minus_one_overflow() {
    let src = r#"
    fn main() {
        let _ = comptime { -128_i8 % -1 };
                           ^^^^^^^^^^^^ Attempt to calculate the remainder with overflow
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_right_shift_overflow() {
    let src = r#"
    fn main(x: Field) -> pub Field {
        let y = comptime { 1 >> 32 };
                           ^^^^^^^ Attempt to bit-shift with overflow
        x + y as Field
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_bitshift_failure() {
    let src = r#"
    unconstrained fn main() {
        comptime {
            func_1(false)
        }
    }

    unconstrained fn func_1(terminate: bool) {
        if !terminate {
            let _ = -8_i8 >> -8_i8;
                    ^^^^^^^^^^^^^^ Attempt to bit-shift with overflow
            func_1(true)
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_negate_with_overflow() {
    let src = r#"
    fn main() {
        comptime {
            let i: i8 = -128;
            let _ = -i;
                    ^^ Attempt to negate with overflow
        }

        comptime {
            let i: u8 = 1;
            let _ = -i;
                    ^^ Cannot apply unary operator `-` to type `u8`
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_constants_do_not_accept_turbofish() {
    let src = r#"
    pub trait Foo {
        let N: u32;

        fn foo() -> [Field; Self::N::<i32>] {
                                   ^^^^^^^ Generic Associated Types (GATs) are currently unsupported in Noir
                                   ~~~~~~~ Cannot apply generics to an associated type
            [0; Self::N::<i32>]
                       ^^^^^^^ Generic Associated Types (GATs) are currently unsupported in Noir
                       ~~~~~~~ Cannot apply generics to an associated type
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn comptime_static_assert_failure() {
    let stdlib = r#"
        #[builtin(static_assert)]
        pub fn static_assert<T>(_predicate: bool, _message: T) {}
    "#;
    let src = r#"
    comptime fn foo(x: Field) -> bool {
        static_assert(x == 4, "x != 4");
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ static_assert failed: x != 4
        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Assertion failed
        x == 4
    }

    fn main() {
        comptime {
            static_assert(foo(3), "expected message");
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

// This program should fail to compile, but it must not panic the compiler.
#[test]
fn regression_10924_comptime_closure_arity_mismatch() {
    let src = r#"
    fn main() -> pub u32 {
        comptime {
            let a = 10;
            foo();
            a
        }
    }

    fn foo() -> u32 {
        comptime {
            let f = |a: u32, b: u32| a + b;
            let _ = f(1);
        }
    }
    "#;
    let errors = get_program_errors(src);
    let messages: Vec<_> = errors.iter().map(|e| e.to_string()).collect();
    assert!(
        messages.iter().any(|m| m.contains("Function expects 2 parameters but 1 were given")),
        "expected arity mismatch error, got {messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains(r#"Expected type "u32" is not the same as "()""#)),
        "expected type mismatch error, got {messages:?}"
    );
}

/// Stub stdlib for tests that drive the comptime meta API: a minimal `Option`,
/// the `Quoted`/`Module`/`FunctionDefinition` builtins, and `quoted_eq` for name
/// comparison. Enough to enumerate a module's functions and call `as_typed_expr`.
const META_API_STDLIB: &str = r#"
    pub struct Option<T> {
        _is_some: bool,
        _value: T,
    }

    impl<T> Option<T> {
        pub comptime fn none() -> Self {
            Option { _is_some: false, _value: zeroed() }
        }
        pub fn some(value: T) -> Self {
            Option { _is_some: true, _value: value }
        }
        pub fn unwrap(self) -> T {
            assert(self._is_some);
            self._value
        }
    }

    #[builtin(zeroed)]
    pub comptime fn zeroed<T>() -> T {}

    impl<T> [T] {
        #[builtin(array_len)]
        pub fn len(self) -> u32 {}
    }

    impl Quoted {
        #[builtin(quoted_as_module)]
        pub comptime fn as_module(self) -> Option<Module> {}

        #[builtin(quoted_as_expr)]
        pub comptime fn as_expr(self) -> Option<Expr> {}
    }

    impl Expr {
        #[builtin(expr_resolve)]
        pub comptime fn resolve(self, _in_function: Option<FunctionDefinition>) -> TypedExpr {}
    }

    #[builtin(quoted_eq)]
    pub comptime fn quoted_eq(_first: Quoted, _second: Quoted) -> bool {}

    impl Module {
        #[builtin(module_functions)]
        pub comptime fn functions(self) -> [FunctionDefinition] {}

        #[builtin(module_child_modules)]
        pub comptime fn child_modules(self) -> [Module] {}

        #[builtin(module_named_attribute_args)]
        pub comptime fn named_attribute_args<let N: u32>(self, _name: str<N>) -> [[Quoted]] {}
    }

    impl FunctionDefinition {
        #[builtin(function_def_name)]
        pub comptime fn name(self) -> Quoted {}

        #[builtin(function_def_as_typed_expr)]
        pub comptime fn as_typed_expr(self) -> TypedExpr {}

        #[builtin(function_def_parameters)]
        pub comptime fn parameters(self) -> [(Quoted, Type)] {}

        #[builtin(function_def_return_type)]
        pub comptime fn return_type(self) -> Type {}

        #[builtin(function_def_named_attribute_args)]
        pub comptime fn named_attribute_args<let N: u32>(self, _name: str<N>) -> [[Quoted]] {}
    }

    impl TypedExpr {
        #[builtin(typed_expr_as_function_definition)]
        pub comptime fn as_function_definition(self) -> Option<FunctionDefinition> {}
    }

    impl TypeDefinition {
        #[builtin(type_def_named_attribute_args)]
        pub comptime fn named_attribute_args<let N: u32>(self, _name: str<N>) -> [[Quoted]] {}

        #[builtin(type_def_fields_as_written)]
        pub comptime fn fields_as_written(self) -> [(Quoted, Type, Quoted)] {}
    }
"#;

#[test]
fn comptime_as_typed_expr_visibility() {
    let src = r#"
    mod victim {
        pub struct Vault {
            secret: Field,
        }

        pub fn new() -> Vault {
            Vault { secret: 1 }
        }

        pub fn read(v: Vault) -> Field {
            v.secret
        }

        fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
           ^^^^^^^^^^ unused function set_secret
           ~~~~~~~~~~ unused function
            v.secret = new_secret;
            v
        }
    }

    fn main() {
        let original_vault = victim::new();
            ^^^^^^^^^^^^^^ unused variable original_vault
            ~~~~~~~~~~~~~~ unused variable

        let hijacked_vault = comptime {
            let victim_module = quote { victim }.as_module().unwrap();
            let mut found = Option::none();
            for f in victim_module.functions() {
                if quoted_eq(f.name(), quote { set_secret }) {
                    found = Option::some(f);
                }
            }
            let set_secret = found.unwrap();
            let set_secret_expr = set_secret.as_typed_expr();
                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^ Function `set_secret` is private
                                  ~~~~~~~~~~~~~~~~~~~~~~~~~~ `set_secret` is declared in `victim`
            quote { $set_secret_expr(original_vault, 31337) }
        };

        assert(victim::read(hijacked_vault) == 31337);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

/// A `pub fn` inside a private module is not reachable from outside that module through a
/// normal path (the enclosing module is private), so it must not be reachable through
/// `as_typed_expr` either. Checking only the function's own visibility misses the private
/// enclosing module.
#[test]
fn comptime_as_typed_expr_visibility_through_private_module() {
    let src = r#"
    mod victim {
        pub struct Vault {
            secret: Field,
        }

        pub fn new() -> Vault {
            Vault { secret: 1 }
        }

        pub fn read(v: Vault) -> Field {
            v.secret
        }

        mod priv_mod {
            use super::Vault;

            pub fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
                v.secret = new_secret;
                v
            }
        }
    }

    fn main() {
        let original_vault = victim::new();
            ^^^^^^^^^^^^^^ unused variable original_vault
            ~~~~~~~~~~~~~~ unused variable

        let hijacked_vault = comptime {
            let victim_module = quote { victim }.as_module().unwrap();
            let mut found = Option::none();
            for m in victim_module.child_modules() {
                for f in m.functions() {
                    if quoted_eq(f.name(), quote { set_secret }) {
                        found = Option::some(f);
                    }
                }
            }
            let set_secret = found.unwrap();
            let set_secret_expr = set_secret.as_typed_expr();
                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^ Function `set_secret` is private
                                  ~~~~~~~~~~~~~~~~~~~~~~~~~~ `set_secret` is declared in `victim::priv_mod`
            quote { $set_secret_expr(original_vault, 31337) }
        };

        assert(victim::read(hijacked_vault) == 31337);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

/// The private-module check must not over-restrict: a `pub fn` inside a `pub mod` is reachable
/// through a normal path, so it must remain reachable through `as_typed_expr` as well.
#[test]
fn comptime_as_typed_expr_visibility_through_public_module() {
    let src = r#"
    mod victim {
        pub struct Vault {
            secret: Field,
        }

        pub fn new() -> Vault {
            Vault { secret: 1 }
        }

        pub fn read(v: Vault) -> Field {
            v.secret
        }

        pub mod pub_mod {
            use super::Vault;

            pub fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
                v.secret = new_secret;
                v
            }
        }
    }

    fn main() {
        let original_vault = victim::new();

        let updated_vault = comptime {
            let victim_module = quote { victim }.as_module().unwrap();
            let mut found = Option::none();
            for m in victim_module.child_modules() {
                for f in m.functions() {
                    if quoted_eq(f.name(), quote { set_secret }) {
                        found = Option::some(f);
                    }
                }
            }
            let set_secret = found.unwrap();
            let set_secret_expr = set_secret.as_typed_expr();
            quote { $set_secret_expr(original_vault, 31337) }
        };

        assert(victim::read(updated_vault) == 31337);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

// Mirrors `type_def_fields_kind_mismatch`: calling `TypeDefinition::fields` with a
// `Normal`-kinded type in a slot whose generic is `Numeric(u32)` errors at comptime.
#[test]
fn type_def_fields_kind_mismatch() {
    let stdlib = r#"
        pub struct Option<T> {
            _is_some: bool,
            _value: T,
        }
        impl<T> Option<T> {
            #[builtin(zeroed)]
            pub comptime fn zeroed() -> Self {}
            pub fn unwrap(self) -> T {
                assert(self._is_some);
                self._value
            }
        }

        impl Quoted {
            #[builtin(quoted_as_type)]
            pub comptime fn as_type(self) -> Type {}
        }

        impl Type {
            #[builtin(type_as_data_type)]
            pub comptime fn as_data_type(self) -> Option<(TypeDefinition, [Type])> {}
        }

        impl TypeDefinition {
            #[builtin(type_def_fields)]
            pub comptime fn fields(self, _generic_args: [Type]) -> [(Quoted, Type, Quoted)] {}
        }
    "#;
    let src = r#"
    pub struct Foo<let N: u32, T> {
        a: [T; N],
    }

    fn main() {
        comptime {
            let (foo_def, _) =
                quote { Foo<3, Field> }.as_type().as_data_type().unwrap();

            let bool_t = quote { bool }.as_type();
            let field_t = quote { Field }.as_type();

            let _ = foo_def.fields(@[bool_t, field_t]);
                                   ^^^^^^^^^^^^^^^^^^ `TypeDefinition::fields` expected generic argument 0 of `Foo` to have kind `u32` but found kind `normal`
                                   ~~~~~~~~~~~~~~~~~~ Assertion failed
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

// Mirrors `comptime_user_error`: a comptime attribute that calls `std::meta::error`
// to issue a user diagnostic with a primary and secondary message.
#[test]
fn comptime_user_error() {
    let stdlib = r#"
        pub struct Option<T> {
            _is_some: bool,
            _value: T,
        }
        impl<T> Option<T> {
            pub fn some(value: T) -> Self {
                Option { _is_some: true, _value: value }
            }
        }

        impl FunctionDefinition {
            #[builtin(function_def_location)]
            pub comptime fn location(self) -> Location {}
        }

        #[builtin(issue_error)]
        pub comptime fn error<let N: u32, T, let N2: u32, T2>(
            _msg: fmtstr<N, T>,
            _secondary: Option<fmtstr<N2, T2>>,
            _location: Location,
        ) {}
    "#;
    let src = r#"
    #[reject]
    fn forbidden() {}
       ^^^^^^^^^ I am the error on `forbidden`
       ~~~~~~~~~ I am the secondary on `forbidden`

    comptime fn reject(f: FunctionDefinition) {
        error(
            f"I am the error on `{f}`",
            Option::some(f"I am the secondary on `{f}`"),
            f.location(),
        );
    }

    fn main() {
        forbidden();
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
}

// Mirrors `macro_result_type`: a method call in a comptime block whose providing
// trait is implemented but not imported into scope.
#[test]
fn comptime_method_call_with_trait_not_in_scope() {
    let src = r#"
    mod m {
        pub trait AsCtString {
            fn as_ctstring(self) -> Field;
        }

        impl AsCtString for Field {
            fn as_ctstring(self) -> Field {
                self
            }
        }
    }

    fn main() {
        comptime {
            let _ = (1 as Field).as_ctstring();
                    ^^^^^^^^^^^^^^^^^^^^^^^^^^ trait `m::AsCtString` which provides `as_ctstring` is implemented but not in scope, please import it
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_resolve_visibility() {
    let src = r#"
    mod victim {
        pub struct Vault {
            secret: Field,
        }

        pub fn new() -> Vault {
            Vault { secret: 1 }
        }

        fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
            v.secret = new_secret;
            v
        }
    }

    fn main() {
        let _hijacked = comptime {
            let victim_module = quote { victim }.as_module().unwrap();
            let secret_expr = quote { set_secret }.as_expr().unwrap().resolve(Option::some(victim_module.functions()[0]));
                              ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ While evaluating `Expr::resolve`
                                      ^^^^^^^^^^ set_secret is private and not visible from the current module
                                      ~~~~~~~~~~ set_secret is private
            quote { $secret_expr(victim::new(), 31337) }
        };
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn comptime_division_overflow() {
    let src = r#"
    fn main() -> pub i8 {
        comptime {
            func_1(-128_i8)
        }
    }

    unconstrained fn func_1(a: i8) -> i8 {
        (a / -1)
         ^^^^^^ Attempt to divide with overflow
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_bit_shift_overflow() {
    let src = r#"
    fn main() -> pub i64 {
        comptime {
            func_1()
        }
    }
    unconstrained fn func_1() -> i64 {
        (-1525866727742442465_i64 / -7342926532602101880_i64) << func_2()
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Attempt to bit-shift with overflow
    }
    unconstrained fn func_2() -> i64 {
        -877061792390071735_i64
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_checked_transmute_failure() {
    let src = r#"
    fn main() {
        comptime {
            let x: Field = 1;
            let _: u32 = checked_transmute(x);
                                           ^ Checked transmute failed: `Field` != `u32`
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::CHECKED_TRANSMUTE]);
}

#[test]
fn checked_transmute_array_length_mismatch() {
    let src = r#"
    fn main() {
        let _: [Field; 2] = transmute_fail([1]);
    }

    pub fn transmute_fail<let N: u32>(x: [Field; N]) -> [Field; N + 1] {
        checked_transmute(x)
    }
    "#;
    let error = get_monomorphized_with_stdlib(src, &[stdlib_src::CHECKED_TRANSMUTE])
        .expect_err("Expected a monomorphization error");
    let diagnostic = CustomDiagnostic::from(error);
    assert_eq!(
        diagnostic.message,
        "checked_transmute failed: expected `[Field; 2]` but found `[Field; 1]`"
    );
}

#[test]
fn comptime_closure_callstack_reports_assertion_failure() {
    let src = r#"
    fn main() {
        comptime {
            let z = || assert(false);
                              ^^^^^ Assertion failed
            let y = || z();
            let x = || y();
            x()
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn comptime_invalid_struct_constructor() {
    let src = r#"
    struct Foo {
        a: u32,
        b: bool,
    }
    fn main() {
        comptime {
            let x = Foo { a: 1, a: "hello", c: true };
                    ^^^ missing field b in struct Foo
                                ^ duplicate field a
                                            ^ no such field c defined in struct Foo
            println(x);
            println(x.c);
                      ^ Type Foo has no member named c
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::PRINT]);
}

#[test]
fn comptime_error_on_macro_expansion() {
    // Type errors inside code generated by a function attribute (both directly in the
    // macro's quoted block and via an unquoted value from another module) must be reported.
    let src = r#"
    mod other {
        pub comptime fn expr() -> Quoted {
            quote { 1 + "a" }
        }
    }
    use other::expr;

    #[foo]
    comptime fn foo(_: FunctionDefinition) -> Quoted {
        quote {
            pub fn generated_by_foo() {
                1 + "a";
            }
        }
    }

    #[bar]
    comptime fn bar(_: FunctionDefinition) -> Quoted {
        let expr = expr();
        quote {
            pub fn generated_by_bar() {
                $expr;
            }
        }
    }

    fn main() {}
    "#;
    let errors = get_program_errors(src);
    let messages: Vec<String> = errors.iter().map(|e| CustomDiagnostic::from(e).message).collect();
    let type_errors = messages
        .iter()
        .filter(|m| {
            m.as_str() == "Types in a binary operation should match, but found Field and str<1>"
        })
        .count();
    assert_eq!(
        type_errors, 2,
        "Expected a binary operation type error from each generated function, got: {messages:?}"
    );
>>>>>>> master
}

#[test]
fn self_in_macro_inside_comptime_block_inside_impl_method() {
    let src = r#"
    struct S { x: Field }

    impl S {
        fn foo(_self: Self) {
            comptime {
                let _: Quoted = quote { Self { x: 0 } };
                let _ = make_self!();
            };
        }
    }

    comptime fn make_self() -> Quoted {
        quote { Self { x: 0 } }
    }

    fn main() {
        S { x: 0 }.foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn assoc_type_in_macro_inside_comptime_block_inside_impl_method() {
    let src = r#"
    struct S {}

    trait HasAssoc {
        type Assoc;
        fn foo(self);
    }

    impl HasAssoc for S {
        type Assoc = Field;
        fn foo(_self: Self) {
            comptime {
                let _ = make!();
            };
        }
    }

    comptime fn make() -> Quoted {
        quote { let _: Self::Assoc = 0; }
    }

    fn main() {
        S {}.foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn qualified_self_assoc_in_macro_inside_comptime_block_inside_impl_method() {
    let src = r#"
    struct S {}

    trait HasAssoc {
        type Assoc;
        fn foo(self);
    }

    impl HasAssoc for S {
        type Assoc = Field;
        fn foo(_self: Self) {
            comptime {
                let _ = make!();
            };
        }
    }

    comptime fn make() -> Quoted {
        quote { let _: <Self as HasAssoc>::Assoc = 0; }
    }

    fn main() {
        S {}.foo();
    }
    "#;
    assert_no_errors(src);
}

// `Expr::resolve` elaborates a quoted expression eagerly and stores the resulting `ExprId`
// inside a `TypedExpr`. When that `TypedExpr` is later unquoted into a different context the
// elaborator must revalidate it against the splice site rather than trusting the context it
// was originally resolved in. The following tests pin that revalidation.

#[test]
fn resolve_does_not_let_comptime_local_escape_into_runtime() {
    let src = r#"
    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let secret = 42;
        let _ = secret;
        let typed = quote { secret }.as_expr().unwrap().resolve(Option::none());
                            ^^^^^^ Comptime variable `secret` cannot be used in runtime code
                            ~~~~~~ `secret` was resolved in a comptime scope that is no longer in scope here
        quote {
            fn generated() -> Field {
                $typed
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() -> pub Field {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn resolve_revalidates_unconstrained_call_spliced_into_constrained() {
    let src = r#"
    unconstrained fn helper() -> Field {
        0
    }

    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let call = quote { helper() }.as_expr().unwrap().resolve(Option::none());
                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
        quote {
            fn generated() -> Field {
                $call
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() -> pub Field {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn resolve_revalidates_verify_proof_spliced_into_unconstrained() {
    let stdlib = r#"
    pub fn verify_proof_with_type<let N: u32, let M: u32, let K: u32>(
        _verification_key: [Field; N],
        _proof: [Field; M],
        _public_inputs: [Field; K],
        _key_hash: Field,
        _proof_type: u32,
    ) {}
    "#;
    let src = r#"
    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let scope = quote { safe_scope }
            .as_expr()
            .unwrap()
            .resolve(Option::none())
            .as_function_definition()
            .unwrap();

        let call = quote {
            crate::verify_proof_with_type([0; 114], [0; 94], [0], 0, 0)
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Cannot call `std::verify_proof_with_type` in unconstrained context
        }
            .as_expr()
            .unwrap()
            .resolve(Option::some(scope));

        quote {
            unconstrained fn generated() {
                $call
            }
        }
    }

    fn safe_scope() {}

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() {
        safe_scope();
        // Safety: test
        unsafe { generated() };
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB, stdlib]);
}

#[test]
fn resolve_revalidates_while_loop_spliced_into_constrained() {
    let src = r#"
    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let body = quote { { while true {} } }.as_expr().unwrap().resolve(Option::none());
                             ^^^^^^^^^^^^^ `while` is only allowed in unconstrained functions
                             ~~~~~~~~~~~~~ Constrained code must always have a known number of loop iterations
        quote {
            fn generated() {
                $body
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn resolve_revalidates_loop_spliced_into_constrained() {
    let src = r#"
    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let body = quote { { loop { break; } } }.as_expr().unwrap().resolve(Option::none());
                             ^^^^^^^^^^^^^^^ `loop` is only allowed in unconstrained functions
                             ~~~~~~~~~~~~~~~ Constrained code must always have a known number of loop iterations
                                    ^^^^^^ break is only allowed in unconstrained functions
                                    ~~~~~~ Constrained code must always have a known number of loop iterations
        quote {
            fn generated() {
                $body
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn resolve_revalidates_break_spliced_into_constrained() {
    let src = r#"
    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let body = quote { { for _ in 0..3 { break; } } }.as_expr().unwrap().resolve(Option::none());
                                             ^^^^^^ break is only allowed in unconstrained functions
                                             ~~~~~~ Constrained code must always have a known number of loop iterations
        quote {
            fn generated() {
                $body
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

// The revalidation walk recurses into the structure of a resolved expression, so a
// boundary-crossing call nested inside a block is caught the same as a top-level one.
#[test]
fn resolve_revalidates_unconstrained_call_nested_in_block() {
    let src = r#"
    unconstrained fn helper() -> Field {
        0
    }

    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let body = quote { { helper() } }.as_expr().unwrap().resolve(Option::none());
                             ^^^^^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
        quote {
            fn generated() -> Field {
                $body
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() -> pub Field {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

// Revalidation must not reject a resolved unconstrained call that is spliced into an
// `unsafe` block: the boundary is crossed legally there, exactly as for hand-written code.
#[test]
fn resolve_allows_unconstrained_call_spliced_into_unsafe_block() {
    let src = r#"
    unconstrained fn helper() -> Field {
        0
    }

    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let call = quote { helper() }.as_expr().unwrap().resolve(Option::none());
        quote {
            fn generated() -> Field {
                // Safety: test
                unsafe { $call }
            }
        }
    }

    #[emit]
    fn main() -> pub Field {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

// An assignment target referencing a comptime local must be revalidated just like a read: the
// `Assign` lvalue is walked so the comptime local cannot escape into runtime code (where it would
// otherwise reach monomorphization with no value).
#[test]
fn resolve_does_not_let_comptime_local_escape_via_assignment_lvalue() {
    let src = r#"
    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let mut secret = 42;
        secret = 0;
        let _ = secret;
        let typed = quote { { secret = 1; } }.as_expr().unwrap().resolve(Option::none());
                              ^^^^^^ Comptime variable `secret` cannot be used in runtime code
                              ~~~~~~ `secret` was resolved in a comptime scope that is no longer in scope here
        quote {
            fn generated() {
                $typed
            }
        }
    }

    #[emit]
    ~~~~~~~ While running this function attribute
    fn main() {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

// Revalidation must preserve the unsafe-block context of a resolved expression: an unconstrained
// call wrapped in an `unsafe` block *inside* the resolved expression crosses the boundary legally,
// so it must not be reported as needing an `unsafe` block at the splice site.
#[test]
fn resolve_allows_unconstrained_call_in_resolved_unsafe_block() {
    let src = r#"
    unconstrained fn helper() -> Field {
        0
    }

    comptime fn emit(_f: FunctionDefinition) -> Quoted {
        let scope = quote { constrained_scope }
            .as_expr()
            .unwrap()
            .resolve(Option::none())
            .as_function_definition()
            .unwrap();

        let call = quote {
            // Safety: test
            unsafe { helper() }
        }
            .as_expr()
            .unwrap()
            .resolve(Option::some(scope));

        quote {
            fn generated() -> Field {
                $call
            }
        }
    }

    fn constrained_scope() {}

    #[emit]
    fn main() -> pub Field {
        generated()
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn meta_attribute_coerces_function_path_to_function_definition() {
    // https://github.com/noir-lang/noir/issues/13186
    // A function path passed as a meta-attribute argument should coerce to a
    // `FunctionDefinition` parameter, mirroring how a trait path coerces to `TraitDefinition`.
    let src = r#"
    #[validate(check)]
    pub fn target() {}

    pub fn check() {}

    comptime fn validate(_f: FunctionDefinition, _method: FunctionDefinition) {}

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn meta_attribute_coerces_inherent_method_path_to_function_definition() {
    // https://github.com/noir-lang/noir/issues/13186
    // An inherent method path (`Type::method`) is not resolvable as a plain value path the way a
    // free function is, but it must still coerce to a `FunctionDefinition` argument — mirroring how
    // the expression `Type::method` resolves to that method.
    let src = r#"
    pub struct Foo {}

    impl Foo {
        pub fn check(_self: &Self) {}
    }

    #[validate(Foo::check)]
    pub fn target() {}

    comptime fn validate(_f: FunctionDefinition, _method: FunctionDefinition) {}

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn meta_attribute_function_definition_argument_can_be_inspected() {
    // The coerced `FunctionDefinition` is a real definition whose signature can be inspected,
    // which is the point of accepting it as `FunctionDefinition` rather than `Quoted`.
    let src = r#"
    #[validate(check)]
    pub fn target() {}

    pub fn check(_x: Field) {}

    comptime fn validate(_f: FunctionDefinition, method: FunctionDefinition) {
        assert(method.parameters().len() == 1);
        let _ = method.return_type();
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn meta_attribute_function_definition_argument_must_be_a_path() {
    // A non-path argument (here an integer literal) is rejected.
    let src = r#"
    #[validate(1)]
    pub fn target() {}

    comptime fn validate(_f: FunctionDefinition, _method: FunctionDefinition) {}

    fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert!(
        errors.iter().any(|error| format!("{error:?}").contains("FunctionDefinitionMustBeAPath")),
        "expected FunctionDefinitionMustBeAPath, got: {errors:?}"
    );
}

#[test]
fn meta_attribute_function_definition_argument_must_be_a_function() {
    // A path that resolves to a non-function value (here a global) is rejected.
    let src = r#"
    global NOT_A_FN: Field = 0;

    #[validate(NOT_A_FN)]
    pub fn target() {}

    comptime fn validate(_f: FunctionDefinition, _method: FunctionDefinition) {}

    fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert!(
        errors
            .iter()
            .any(|error| format!("{error:?}").contains("FailedToResolveFunctionDefinition")),
        "expected FailedToResolveFunctionDefinition, got: {errors:?}"
    );
}

#[test]
fn type_def_named_attribute_args_returns_attribute_arguments() {
    // https://github.com/noir-lang/noir/issues/13187
    // `named_attribute_args` captures *every* occurrence of an attribute and *all* of each
    // occurrence's argument expressions, as token streams that can be spliced into generated code.
    let src = r#"
    #[generate]
    #[value(1, 2)]
    #[value(3, 4, 5)]
    pub struct Foo {}

    #[varargs]
    comptime fn value(_s: TypeDefinition, _v: [Field]) {}

    comptime fn generate(s: TypeDefinition) -> Quoted {
        let occurrences = s.named_attribute_args("value");

        // Both `#[value(..)]` occurrences are captured, in source order, with all their arguments.
        assert(occurrences.len() == 2);
        assert(occurrences[0].len() == 2);
        assert(occurrences[1].len() == 3);

        // Each argument comes back as a `Quoted` token stream; splice all five into a sum.
        let a = occurrences[0][0];
        let b = occurrences[0][1];
        let c = occurrences[1][0];
        let d = occurrences[1][1];
        let e = occurrences[1][2];
        quote {
            pub global TOTAL: Field = $a + $b + $c + $d + $e;
        }
    }

    fn main() {
        assert(TOTAL == 15);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn type_def_named_attribute_args_is_empty_when_attribute_absent() {
    // An absent attribute yields no occurrences, so `named_attribute_args` subsumes
    // `has_named_attribute` (`!args.is_empty()`).
    let src = r#"
    #[check]
    pub struct Foo {}

    comptime fn check(s: TypeDefinition) {
        assert(s.named_attribute_args("nonexistent").len() == 0);
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn type_def_named_attribute_args_captures_zero_arg_occurrence() {
    // An attribute used without arguments still counts as an occurrence, with an empty argument list.
    let src = r#"
    #[check]
    #[mark]
    pub struct Foo {}

    comptime fn mark(_s: TypeDefinition) {}

    comptime fn check(s: TypeDefinition) {
        let occurrences = s.named_attribute_args("mark");
        assert(occurrences.len() == 1);
        assert(occurrences[0].len() == 0);
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn function_def_named_attribute_args_returns_attribute_arguments() {
    // The same accessor exists on `FunctionDefinition`, reading the function's own attributes.
    let src = r#"
    #[generate]
    #[value(7)]
    pub fn target() {}

    comptime fn value(_f: FunctionDefinition, _v: Field) {}

    comptime fn generate(f: FunctionDefinition) {
        let occurrences = f.named_attribute_args("value");
        assert(occurrences.len() == 1);
        assert(occurrences[0].len() == 1);
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn module_named_attribute_args_returns_attribute_arguments() {
    // The same accessor exists on `Module`, reading the module's own attributes.
    let src = r#"
    #[generate]
    #[value(9)]
    mod my_mod {}

    comptime fn value(_m: Module, _v: Field) {}

    comptime fn generate(m: Module) {
        let occurrences = m.named_attribute_args("value");
        assert(occurrences.len() == 1);
        assert(occurrences[0].len() == 1);
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn spliced_field_type_generic_rebinds_to_generated_impl_generic() {
    // Regression test for https://github.com/noir-lang/noir/issues/10747
    //
    // A field type obtained via `fields_as_written()` carries the *struct's* generic
    // (`PublicImmutable<Context>` where `Context` is `Storage`'s generic). When it is spliced
    // via `$typ` into a generated `impl<Context> Storage<Context>`, the spliced type's `Context`
    // must resolve to the *impl's* `Context`, exactly like the textually-written `Context` does.
    let src = r#"
    pub trait StateVariable<Context> {
        fn new(context: Context) -> Self;
    }

    pub struct PublicImmutable<Context> {
        context: Context,
    }

    impl<Context> StateVariable<Context> for PublicImmutable<Context> {
        fn new(context: Context) -> Self {
            PublicImmutable { context }
        }
    }

    #[storage]
    struct Storage<Context> {
        symbol: PublicImmutable<Context>,
    }

    pub comptime fn storage(s: TypeDefinition) -> Quoted {
        let (name, typ, _) = s.fields_as_written()[0];
        quote {
            impl<Context> Storage<Context> {
                fn init(context: Context) -> Self {
                    Self {
                        $name: <$typ as StateVariable<Context>>::new(context),
                    }
                }
            }
        }
    }

    struct PrivateContext {}

    fn main() {
        let context: PrivateContext = PrivateContext {};
        let _ = Storage::init(context);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

#[test]
fn spliced_bare_generic_field_type_rebinds_in_generated_impl() {
    // Companion to the regression above: the field type is the bare generic `Context` itself.
    // The spliced `$typ` (the struct's `Context`) must rebind to the generated impl's `Context`.
    let src = r#"
    #[identity_impl]
    struct Wrapper<Context> {
        inner: Context,
    }

    pub comptime fn identity_impl(s: TypeDefinition) -> Quoted {
        let (_name, typ, _) = s.fields_as_written()[0];
        quote {
            impl<Context> Wrapper<Context> {
                fn identity(value: $typ) -> Context {
                    value
                }
            }
        }
    }

    fn main() {
        let _ = Wrapper::identity(1);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}

/// `Expr::resolve` with a foreign function scope must judge visibility from the caller's module,
/// including the first path segment. A `pub` item inside a private module of the foreign scope is
/// reachable there as a plain first segment, but it must not become reachable from the caller.
#[test]
fn comptime_resolve_first_segment_visibility() {
    let src = r#"
    mod victim {
        pub fn foreign_scope() {}

        mod secret_mod {
            pub struct Leaked { pub x: Field }
        }
    }

    fn main() {
        let leaked = comptime {
            let victim_module = quote { victim }.as_module().unwrap();
            let expr = quote { secret_mod::Leaked { x: 5 } }.as_expr().unwrap().resolve(Option::some(victim_module.functions()[0]));
                       ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ While evaluating `Expr::resolve`
                               ^^^^^^^^^^ secret_mod is private and not visible from the current module
                               ~~~~~~~~~~ secret_mod is private
            quote { $expr.x }
        };
        assert(leaked == 5);
    }
    "#;
    check_errors_with_stdlib(src, [META_API_STDLIB]);
}
