use crate::tests::{assert_no_errors, check_errors, check_monomorphization_error};

#[test]
fn resolve_empty_function() {
    let src = "
        fn main() {

        }
    ";
    assert_no_errors(src);
}

#[test]
fn resolve_basic_function() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(y == x);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_call_expr() {
    let src = r#"
        fn main(x : Field) {
            let _z = foo(x);
        }

        fn foo(x : Field) -> Field {
            x
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn unconditional_recursion_fail() {
    // These examples are self recursive top level functions, which would actually
    // not be inlined in the SSA (there is nothing to inline into but self), so it
    // wouldn't panic due to infinite recursion, but the errors asserted here
    // come from the compilation checks, which does static analysis to catch the
    // problem before it even has a chance to cause a panic.
    let sources = vec![
        r#"
        fn main() {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            main()
        }
        "#,
        r#"
        fn main() -> pub bool {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            if main() { true } else { false }
        }
        "#,
        r#"
        fn main() -> pub bool {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            if true { main() } else { main() }
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            main() + main()
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            1 + main()
        }
        "#,
        r#"
        fn main() -> pub bool {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            let _ = main();
            true
        }
        "#,
        r#"
        fn main(a: u64, b: u64) -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            main(a + b, main(a, b))
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            foo(1, main())
        }
        fn foo(a: u64, b: u64) -> u64 {
            a + b
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            let (a, b) = (main(), main());
            a + b
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            let mut sum = 0;
            for i in 0 .. main() {
                sum += i;
            }
            sum
        }
        "#,
    ];

    for src in sources {
        check_errors(src);
    }
}

#[test]
fn unconditional_recursion_pass() {
    let sources = vec![
        r#"
        fn main() {
            if false { main(); }
        }
        "#,
        r#"
        fn main(i: u64) -> pub u64 {
            if i == 0 { 0 } else { i + main(i-1) }
        }
        "#,
        // Only immediate self-recursion is detected.
        r#"
        fn main() {
            foo();
        }
        fn foo() {
            bar();
        }
        fn bar() {
            foo();
        }
        "#,
        // For loop bodies are not checked.
        r#"
        fn main() -> pub u64 {
            let mut sum = 0;
            for _ in 0 .. 10 {
                sum += main();
            }
            sum
        }
        "#,
        // Lambda bodies are not checked.
        r#"
        fn main() {
            let foo = || main();
            foo();
        }
        "#,
    ];

    for src in sources {
        assert_no_errors(src);
    }
}

#[test]
fn allows_multiple_underscore_parameters() {
    let src = r#"
        pub fn foo(_: i32, _: i64) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_return_vector_from_main() {
    let src = r#"
    fn main() -> pub [Field] {
                     ^^^^^^^ Invalid type found in the entry point to a program
                     ~~~~~~~ Vector is not a valid entry point type. Found: [Field]
        @[1,2]

    }
        "#;
    check_errors(src);
}

#[test]
fn builtin_function_with_body() {
    let src = r#"
    #[builtin(foo)]
    ^^^^^^^^^^^^^^^ Definition of low-level function outside of standard library
    ~~~~~~~~~~~~~~~ Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library
    pub fn foo() {
           ^^^ Builtin and low-level function declarations cannot have a body
           ~~~ This function body should be removed
        let x = 1;
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_duplicate_parameter_name() {
    let src = r#"
    fn main(x: i32, x: i32) {
                    ^ duplicate definitions of x found
                    ~ second definition found here
            ~ first definition found here
        let _ = x;
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_clashing_const_generic_and_parameter_name() {
    let src = r#"
    pub fn foo<let N: u32>(N: u32) -> u32 { N }
                           ^ duplicate definitions of N found
                           ~ second definition found here
                   ~ first definition found here

    "#;
    check_errors(src);
}

#[test]
fn non_entry_point_main() {
    let src = r#"
    mod moo {
        #[allow(dead_code)]
        pub fn main() -> i32 {
            1
        }
    }

    pub struct Foo {}
    impl Foo {
        pub fn main() -> i32 {
            1
        }
    }

    pub trait Trait {
        fn main() -> i32;
    }
    impl Trait for Foo {
        fn main() -> i32 {
            1
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn call_type_variable_of_kind_any() {
    // Regression for https://github.com/noir-lang/noir/issues/10719
    let src = "
        trait Foo {
            type Bar;

            fn bar(self) -> Self::Bar;
        }

        impl Foo for () {
            type Bar = fn() -> fn() -> ();

            fn bar(self) -> Self::Bar {
                || {
                    || {
                        ()
                    }
                }
            }
        }

        struct Baz<T> {
            inner: T,
        }

        impl<T> Foo for Baz<T>
        where
            T: Foo,
        {
            type Bar = <T as Foo>::Bar;

            fn bar(self) -> Self::Bar {
                self.inner.bar()
            }
        }

        fn main() {
            let _: () = (Baz { inner: () }.bar()())();
        }
    ";
    assert_no_errors(src);
}

#[test]
fn error_on_returning_empty_unit_array() {
    let src = r#"
    fn main() -> pub [(); 0] {
                     ^^^^^^^ Invalid type found in the entry point to a program
                     ~~~~~~~ Unit is not a valid entry point type
        [(); 0]
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_returning_non_empty_unit_array() {
    let src = r#"
    fn main() -> pub [(); 1] {
                     ^^^^^^^ Invalid type found in the entry point to a program
                     ~~~~~~~ Unit is not a valid entry point type
        [()]
    }
    "#;
    check_errors(src);
}

#[test]
fn no_error_on_returning_empty_array() {
    let src = r#"
    fn main() -> pub [u32; 0] {
        []
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_error_on_returning_empty_array_with_empty_nested_array() {
    let src = r#"
    fn main() -> pub [[u32; 0]; 0] {
        []
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_error_on_returning_non_empty_array_with_empty_nested_array() {
    let src = r#"
    fn main() -> pub [[u32; 0]; 1] {
        [[]]
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_error_on_returning_empty_array_with_tuple_of_empty_arrays() {
    let src = r#"
    fn main() -> pub [([u32; 0], [u32; 0]); 0] {
        []
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn error_on_taking_string_with_zero_length() {
    let src = r#"
    fn main(_s: str<0>) {
                ^^^^^^ Invalid type found in the entry point to a program
                ~~~~~~ Empty string is not a valid entry point type. Found: str<0>
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_taking_string_with_non_eval_length() {
    let src = r#"
    fn main(_s: str<-1>) {
                ^^^^^^^ Invalid type found in the entry point to a program
                ~~~~~~~ Empty string is not a valid entry point type. Found: str<error>
                    ^^ Cannot apply unary operator `-` to type `u32`
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_returning_string_with_non_eval_length() {
    let src = r#"
    unconstrained fn main() -> pub str<-1> {
                                   ^^^^^^^ Invalid type found in the entry point to a program
                                   ~~~~~~~ Empty string is not a valid entry point type. Found: str<error>
                                       ^^ Cannot apply unary operator `-` to type `u32`
        negative_str()
    }

    #[oracle(negative_str)]
    unconstrained fn negative_str() -> str<-1> {}
                                           ^^ Cannot apply unary operator `-` to type `u32`
    "#;
    check_errors(src);
}

#[test]
fn invalid_generic_fold_entry_point_input_type() {
    let src = r#"
    fn main() {
        foo(@[1]);
    }

    #[fold]
    fn foo<T>(_: T) {}
              ^ Invalid type found in the entry point to a program
              ~ Vector is not a valid entry point type. Found: [Field]
    "#;
    check_monomorphization_error(src);
}

#[test]
fn errors_if_using_comptime_type_alias_in_fn() {
    let src = r#"
    pub comptime type Alias = Quoted;

    pub fn foo(_: Alias) {}
                  ^^^^^ Comptime-only type `Alias` cannot be used in non-comptime function
    "#;
    check_errors(src);
}

#[test]
fn errors_if_using_comptime_struct_in_fn() {
    let src = r#"
    pub comptime struct Foo {}

    pub fn foo(_: Foo) {}
                  ^^^ Comptime-only type `Foo` cannot be used in non-comptime function
    "#;
    check_errors(src);
}

#[test]
fn errors_if_using_comptime_enum_in_fn() {
    let src = r#"
    pub comptime enum Foo {}

    pub fn foo(_: Foo) {}
                  ^^^ Comptime-only type `Foo` cannot be used in non-comptime function
    "#;
    check_errors(src);
}

#[test]
fn can_use_trait_associated_constant_in_main_signature() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;
    }

    impl Deserialize for Field {
        let N: u32 = 1;
    }

    unconstrained fn main(fields: [Field; <Field as Deserialize>::N]) -> pub Field {
        fields[0]
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn rejects_tuple_pattern_in_main_param() {
    let src = r#"
    fn main((a, b): pub (Field, Field)) -> pub Field {
            ^^^^^^ Entry point parameter must use a simple identifier pattern
            ~~~~~~ Destructuring patterns are not allowed here; bind to a name and destructure inside the body
        a + b
    }
    "#;
    check_errors(src);
}

#[test]
fn rejects_struct_pattern_in_main_param() {
    let src = r#"
    pub struct P { a: Field, b: Field }

    fn main(P { a, b }: pub P) -> pub Field {
            ^^^^^^^^^^ Entry point parameter must use a simple identifier pattern
            ~~~~~~~~~~ Destructuring patterns are not allowed here; bind to a name and destructure inside the body
        a + b
    }
    "#;
    check_errors(src);
}

#[test]
fn rejects_mutable_tuple_pattern_in_main_param() {
    let src = r#"
    fn main(mut (a, b): pub (Field, Field)) -> pub Field {
            ^^^^^^^^^^ Entry point parameter must use a simple identifier pattern
            ~~~~~~~~~~ Destructuring patterns are not allowed here; bind to a name and destructure inside the body
                 ^ variable does not need to be mutable
                    ^ variable does not need to be mutable
        a + b
    }
    "#;
    check_errors(src);
}

#[test]
fn accepts_mutable_identifier_pattern_in_main_param() {
    let src = r#"
    fn main(mut x: pub Field) -> pub Field {
        x = x + 1;
        x
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn can_use_trait_associated_constant_via_global_in_main_signature() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;
    }

    impl Deserialize for Field {
        let N: u32 = 1;
    }

    global FIELD_N: u32 = <Field as Deserialize>::N;

    unconstrained fn main(fields: [Field; FIELD_N]) -> pub Field {
        fields[0]
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_on_duplicate_function_declaration() {
    let src = r#"
    fn hello(x: Field) -> Field {
       ~~~~~ First definition found here
        x
    }

    fn hello(x: Field) -> Field {
       ^^^^^ Duplicate definitions of function with name hello found
       ~~~~~ Second definition found here
        x
    }

    fn main() {
        let _ = hello(1);
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_duplicate_generic_names() {
    let src = r#"
    fn bar<A, A>(_x: A, _y: A) {}
              ^ duplicate definitions of A found
           ~ first definition found here
              ~ second definition found here

    fn foo<let N: u32, let N: u32>() {}
                           ^^^^^^ duplicate definitions of N found
               ~~~~~~ first definition found here
                           ~~~~~~ second definition found here

    fn main() {
        bar::<u32, u32>(0, 1);
        foo::<1, 2>();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_main_with_generics() {
    let src = r#"
    fn main<let F: u32>(x: [Field; F]) {
                ^^^^^^ `main` entry-point function is not allowed to have generic parameters
                           ^^^^^^^^^^ Invalid type found in the entry point to a program
                           ~~~~~~~~~~ Invalid entry point type: [Field; F]
        assert(x[0] != x[1]);
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unknown_integer_bit_size() {
    let src = r#"
    fn main() -> pub u63 {
                     ^^^ Could not resolve 'u63' in path
        5
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_unit_in_main() {
    let src = r#"
    fn main(_: ()) {}
               ^^ Invalid type found in the entry point to a program
               ~~ Unit is not a valid entry point type
    "#;
    check_errors(src);
}

#[test]
fn error_on_struct_with_vector_field_in_main() {
    let src = r#"
    struct Foo {
           ~~~ Struct Foo has an invalid entry point type
        bar: Bar,
        ~~~ Field bar has an invalid entry point type
    }

    struct Bar {
           ~~~ Struct Bar has an invalid entry point type
        baz: [Field],
        ~~~ Field baz has an invalid entry point type
        ~~~ Vector is not a valid entry point type. Found: [Field]
    }

    type SomeAlias = Foo;
         ~~~~~~~~~ Alias SomeAlias has an invalid entry point type

    fn main(_: SomeAlias) {}
               ^^^^^^^^^ Invalid type found in the entry point to a program
               ~~~~~~~~~ This type has an invalid entry point type inside it
    "#;
    check_errors(src);
}

#[test]
fn error_on_fold_returning_array_of_references() {
    let src = r#"
    fn main() {
        let _ = foo::<[&mut Field; 0]>();
    }

    #[fold]
    fn foo<T>() -> [T; 0] {
                   ^^^^^^ Invalid type found in the entry point to a program
                   ~~~~~~ Reference is not a valid entry point type. Found: &mut Field
        []
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn error_on_reference_in_test_function() {
    let src = r#"
    fn main() {}

    #[test]
    fn test(_arg: &mut i32) {}
                  ^^^^^^^^ Invalid type found in the entry point to a program
                  ~~~~~~~~ Reference is not a valid entry point type. Found: &mut i32
    "#;
    check_errors(src);
}

#[test]
fn error_on_empty_string_in_call_data_param() {
    let src = r#"
    fn main(
        _a: (i8, u32, i8),
        _b: call_data(0) [(i8, i8, bool, bool, str<0>); 2],
                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Invalid type found in the entry point to a program
                         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Empty string is not a valid entry point type. Found: str<0>
    ) -> pub [(bool, str<3>, str<0>, u32); 0] {
        []
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_empty_array_param_with_call_data() {
    let src = r#"
    fn main(_empty: [u32; 0], value_1: u32, value_2: call_data(0) u32) {
                    ^^^^^^^^ Invalid type found in the entry point to a program
                    ~~~~~~~~ Invalid entry point type: [u32; 0]
        assert_eq(value_1 + 1, value_2);
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_empty_nested_array_param() {
    // Regression for https://github.com/noir-lang/noir/issues/7952
    let src = r#"
    fn main(a: [[u32; 0]; 1], b: bool) -> pub [u32; 0] {
               ^^^^^^^^^^^^^ Invalid type found in the entry point to a program
               ~~~~~~~~~~~~~ Invalid entry point type: [u32; 0]
        if (b) {
            a[0]
        } else {
            a[0]
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn error_on_empty_composite_array_param_and_out_of_bounds_index() {
    let src = r#"
    fn main(empty_input: [(Field, Field); 0]) {
                         ^^^^^^^^^^^^^^^^^^^ Invalid type found in the entry point to a program
                         ~~~~~~~~~~~~~~~~~~~ Invalid entry point type: [(Field, Field); 0]
        let empty_array: [(Field, Field); 0] = [];
        let _ = empty_input[0];
                            ^ Index 0 is out of bounds for this array of length 0
        let _ = empty_array[0];
                            ^ Index 0 is out of bounds for this array of length 0
    }
    "#;
    check_errors(src);
}

#[test]
fn warns_on_unnecessary_mut_function_parameter() {
    let src = r#"
    fn foo(mut x: Field) -> Field {
               ^ variable does not need to be mutable
        x
    }

    fn main() {
        assert(foo(1) == 1);
    }
    "#;
    check_errors(src);
}

#[test]
fn warns_on_unnecessary_mut_self_parameter() {
    let src = r#"
    struct Counter {
        count: Field,
    }

    impl Counter {
        fn count(mut self) -> Field {
                     ^^^^ variable does not need to be mutable
            self.count
        }
    }

    fn main() {
        let counter = Counter { count: 1 };
        assert(counter.count() == 1);
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_warn_on_mutated_mut_function_parameter() {
    let src = r#"
    fn foo(mut x: Field) -> Field {
        x = x + 1;
        x
    }

    fn main() {
        assert(foo(1) == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unnecessary_mut_parameter_with_underscore_name() {
    let src = r#"
    fn foo(mut _x: Field) -> Field {
        1
    }

    fn main() {
        assert(foo(1) == 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unmutated_mut_reference_self_parameter() {
    let src = r#"
    struct Counter {
        count: Field,
    }

    impl Counter {
        fn count(&mut self) -> Field {
            self.count
        }
    }

    fn main() {
        let mut counter = Counter { count: 1 };
        assert(counter.count() == 1);
    }
    "#;
    assert_no_errors(src);
}
