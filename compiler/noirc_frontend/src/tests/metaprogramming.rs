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
    test_utils::stdlib_src,
    tests::{check_errors_using_features, check_errors_with_stdlib},
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
fn comptime_for_loop() {
    let src = r#"
        fn main() {
            comptime {
                let mut sum = 0;
                for i in 0..3 {
                    sum += i;
                }
                assert_eq(sum, 3);
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
                              ^^^^ Comptime-only type `Quoted` cannot be used in runtime code
                              ~~~~ Comptime-only type used here
                     ^^^^ Comptime-only type `Quoted` cannot be used in runtime code
                     ~~~~ Comptime-only type used here
            self
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_self_on_trait_impl_for_comptime_type_on_non_comptime_function_with_implicit_self() {
    let src = r#"
    trait Trait {
        fn foo(self);
    }

    impl Trait for Quoted {
        fn foo(self) {
               ^^^^ Comptime-only type `Quoted` cannot be used in runtime code
               ~~~~ Comptime-only type used here
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn zeroed_comptime_type() {
    let mut stdlib = stdlib_src::ZEROED.to_string();
    stdlib.push_str(
        "
        #[builtin(module_hash)]
        comptime fn module_hash(_module: Module) -> Field {}
    ",
    );
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
    check_errors_with_stdlib(src, &stdlib);
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
/// With a global counter, this correctly errors at [crate::elaborator::MAX_MACRO_EXPANSION_DEPTH] total expansions.
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
