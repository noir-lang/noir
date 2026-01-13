use crate::tests::{assert_no_errors, check_errors};

#[test]
fn turbofish_numeric_generic_nested_function_call() {
    // Check for turbofish numeric generics used with function calls
    let src = r#"
    fn foo<let N: u32>() -> [u8; N] {
        [0; N]
    }

    fn bar<let N: u32>() -> [u8; N] {
        foo::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_numeric_generic_nested_method_call() {
    // Check for turbofish numeric generics used with method calls
    let src = r#"
    struct Foo<T> {
        a: T
    }

    impl<T> Foo<T> {
        pub fn static_method<let N: u32>() -> [u8; N] {
            [0; N]
        }

        pub fn impl_method<let N: u32>(self) -> [T; N] {
            [self.a; N]
        }
    }

    fn bar<let N: u32>() -> [u8; N] {
        let _ = Foo::<u8>::static_method::<N>();
        let x: Foo<u8> = Foo { a: 0 };
        x.impl_method::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_constructor_generics_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let _ = Foo::<i32, i64> { x: 1 };
                   ^^^^^^^^^^^^ struct Foo expects 1 generic but 2 were given
    }
    "#;
    check_errors(src);
}

#[test]
fn turbofish_in_constructor() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let x: Field = 0;
        let _ = Foo::<i32> { x: x };
                                ^ Expected type i32, found type Field
    }
    "#;
    check_errors(src);
}

#[test]
fn turbofish_in_struct_pattern() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<Field> { x } = Foo { x: value };
        let _ = x;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_struct_pattern_errors_if_type_mismatch() {
    // TODO: maybe the error should be on the expression
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<i32> { x } = Foo { x: value };
            ^^^^^^^^^^^^^^^^ Cannot assign an expression of type Foo<i32> to a value of type Foo<Field>
        let _ = x;
    }
    "#;
    check_errors(src);
}

#[test]
fn turbofish_in_struct_pattern_generic_count_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value = 0;
        let Foo::<i32, i64> { x } = Foo { x: value };
               ^^^^^^^^^^^^ struct Foo expects 1 generic but 2 were given
        let _ = x;
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_turbofish() {
    let src = r#"
    struct Reader<let N: u32> {
    }

    impl<let N: u32> Reader<N> {
        fn read<let C: u32>(_self: Self) {}
    }

    fn main() {
        let reader: Reader<1234> = Reader {};
        let _ = reader.read::<1234>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_turbofish_after_module() {
    let src = r#"
    mod moo {
        pub fn foo() {}
    }

    fn main() {
        moo::<i32>::foo();
           ^^^^^^^ turbofish (`::<_>`) not allowed on module `moo`
    }
    "#;
    check_errors(src);
}

#[test]
fn turbofish_in_type_before_call_does_not_error() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    impl <T> Foo<T> {
        fn new(x: T) -> Self {
            Foo { x }
        }
    }

    fn main() {
        let _ = Foo::<i32>::new(1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_type_before_call_errors() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    impl <T> Foo<T> {
        fn new(x: T) -> Self {
            Foo { x }
        }
    }

    fn main() {
        let _ = Foo::<i32>::new(true);
                                ^^^^ Expected type i32, found type bool
    }
    "#;
    check_errors(src);
}

#[test]
fn use_generic_type_alias_with_turbofish_in_method_call_does_not_error() {
    let src = r#"
        pub struct Foo<T> {
        }

        impl<T> Foo<T> {
            fn new() -> Self {
                Foo {}
            }
        }

        type Bar<T> = Foo<T>;

        fn foo() -> Foo<i32> {
            Bar::<i32>::new()
        }

        fn main() {
            let _ = foo();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_generic_type_alias_with_turbofish_in_method_call_errors() {
    let src = r#"
        pub struct Foo<T> {
            x: T,
        }

        impl<T> Foo<T> {
            fn new(x: T) -> Self {
                Foo { x }
            }
        }

        type Bar<T> = Foo<T>;

        fn main() {
            let _ = Bar::<i32>::new(true);
                                    ^^^^ Expected type i32, found type bool
        }
    "#;
    check_errors(src);
}

#[test]
fn use_generic_type_alias_with_partial_generics_with_turbofish_in_method_call_does_not_error() {
    let src = r#"
        pub struct Foo<T, U> {
            x: T,
            y: U,
        }

        impl<T, U> Foo<T, U> {
            fn new(x: T, y: U) -> Self {
                Foo { x, y }
            }
        }

        type Bar<T> = Foo<T, i32>;

        fn main() {
            let _ = Bar::<bool>::new(true, 1);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_generic_type_alias_with_partial_generics_with_turbofish_in_method_call_errors_first_type() {
    let src = r#"
        pub struct Foo<T, U> {
            x: T,
            y: U,
        }

        impl<T, U> Foo<T, U> {
            fn new(x: T, y: U) -> Self {
                Foo { x, y }
            }
        }

        type Bar<T> = Foo<T, i32>;

        fn main() {
            let _ = Bar::<bool>::new(1, 1);
                                     ^ Expected type bool, found type Field
        }
    "#;
    check_errors(src);
}

#[test]
fn use_generic_type_alias_with_partial_generics_with_turbofish_in_method_call_errors_second_type() {
    let src = r#"
        pub struct Foo<T, U> {
            x: T,
            y: U,
        }

        impl<T, U> Foo<T, U> {
            fn new(x: T, y: U) -> Self {
                Foo { x, y }
            }
        }

        type Bar<T> = Foo<T, i32>;

        fn main() {
            let _ = Bar::<bool>::new(true, true);
                                           ^^^^ Expected type i32, found type bool
        }
    "#;
    check_errors(src);
}

#[test]
fn trait_function_with_turbofish_on_trait_gives_error() {
    let src = r#"
    trait Foo<T> {
        fn foo(_x: T) -> Self;
    }

    impl<T> Foo<T> for i32 {
        fn foo(_x: T) -> Self {
            1
        }
    }

    fn main() {
        let _: i32 = Foo::<bool>::foo(1);
                                      ^ Expected type bool, found type Field
    }
    "#;
    check_errors(src);
}

#[test]
fn turbofish_named_numeric() {
    let src = r#"
    trait Bar {
        let N: u32;
    }

    impl Bar for Field {
        let N: u32 = 1;
    }

    impl Bar for i32 {
        let N: u32 = 2;
    }

    fn foo<B>()
    where
        B: Bar<N = 1>,
    {}

    fn main() {
        foo::<Field>();
        foo::<i32>();
        ^^^^^^^^^^ No matching impl found for `i32: Bar<N = 1>`
        ~~~~~~~~~~ No impl for `i32: Bar<N = 1>`
    }
    "#;
    check_errors(src);
}

#[test]
fn specify_function_types_with_turbofish() {
    let src = r#"
        trait Default2 {
            fn default2() -> Self;
        }

        impl Default2 for Field {
            fn default2() -> Self { 0 }
        }

        impl Default2 for u64 {
            fn default2() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        fn generic_func<T, U>() -> (T, U) where T: Default2, U: Default2 {
            (T::default2(), U::default2())
        }

        fn main() {
            let _ = generic_func::<u64, Field>();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn specify_method_types_with_turbofish() {
    let src = r#"
        trait Default2 {
            fn default2() -> Self;
        }

        impl Default2 for Field {
            fn default2() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        struct Foo<T> {
            inner: T
        }

        impl<T> Foo<T> {
            fn generic_method<U>(_self: Self) -> U where U: Default2 {
                U::default2()
            }
        }

        fn main() {
            let foo: Foo<Field> = Foo { inner: 1 };
            let _ = foo.generic_method::<Field>();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn incorrect_turbofish_count_function_call() {
    let src = r#"
        trait Default2 {
            fn default() -> Self;
        }

        impl Default2 for Field {
            fn default() -> Self { 0 }
        }

        impl Default2 for u64 {
            fn default() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        fn generic_func<T, U>() -> (T, U) where T: Default2, U: Default2 {
            (T::default(), U::default())
        }

        fn main() {
            let _ = generic_func::<u64, Field, Field>();
                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected 2 generics from this function, but 3 were provided
        }
    "#;
    check_errors(src);
}

#[test]
fn incorrect_turbofish_count_method_call() {
    let src = r#"
        trait Default2 {
            fn default() -> Self;
        }

        impl Default2 for Field {
            fn default() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        struct Foo<T> {
            inner: T
        }

        impl<T> Foo<T> {
            fn generic_method<U>(_self: Self) -> U where U: Default2 {
                U::default()
            }
        }

        fn main() {
            let foo: Foo<Field> = Foo { inner: 1 };
            let _ = foo.generic_method::<Field, u32>();
                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected 1 generic from this function, but 2 were provided
        }
    "#;
    check_errors(src);
}

#[test]
fn cannot_determine_type_of_generic_argument_in_function_call_with_regular_generic() {
    let src = r#"
    fn foo<T>() {}

    fn main()
    {
        foo();
        ^^^ Type annotation needed
        ~~~ Could not determine the type of the generic argument `T` declared on the function `foo`
    }

    "#;
    check_errors(src);
}

#[test]
fn cannot_determine_type_of_generic_argument_in_function_call_when_it_is_a_numeric_generic() {
    let src = r#"
    struct Foo<let N: u32> {
        array: [Field; N],
    }

    impl<let N: u32> Foo<N> {
        fn new() -> Self {
            Self { array: [0; N] }
        }
    }

    fn foo<let N: u32>() -> Foo<N> {
        Foo::new()
    }

    fn main() {
        let _ = foo();
                ^^^ Type annotation needed
                ~~~ Could not determine the value of the generic argument `N` declared on the function `foo`
    }
    "#;
    check_errors(src);
}

#[test]
fn static_method_with_generics_on_type_and_method() {
    let src = r#"
    struct Foo<T> {}

    impl<T> Foo<T> {
        fn static_method<U>() {}
    }

    fn main() {
        Foo::<u8>::static_method::<Field>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_on_incorrect_turbofish_on_struct() {
    let src = r#"
        trait From2<T> {
        fn from2(input: T) -> Self;
    }
    struct U60Repr {}
    impl From2<[Field; 3]> for U60Repr {
        fn from2(_: [Field; 3]) -> Self {
            U60Repr {}
        }
    }
    impl From2<Field> for U60Repr {
        fn from2(_: Field) -> Self {
            U60Repr {}
        }
    }

    fn main() {
        let _ = U60Repr::<Field>::from2([1, 2, 3]);
                       ^^^^^^^^^ struct U60Repr expects 0 generics but 1 was given
    }
    "#;
    check_errors(src);
}

#[test]
fn incorrect_turbofish_count_on_primitive_u8() {
    let src = r#"
        trait From<T> {
            fn from(x: T) -> Self;
        }

        impl From<Field> for u8 {
            fn from(x: Field) -> Self {
                x as u8
            }
        }

        fn main() {
            let _ = u8::<u32, i64>::from(5);
                      ^^^^^^^^^^^^ u8 expects 0 generics but 2 were given
        }
    "#;
    check_errors(src);
}

#[test]
fn incorrect_turbofish_count_on_primitive_str() {
    let src = r#"
        trait MyTrait {
            fn foo();
        }

        impl<let N: u32> MyTrait for str<N> {
            fn foo() { }
        }

        fn main() {
            let _ = str::<5, u32>::foo();
                       ^^^^^^^^^^ primitive type str expects 1 generic but 2 were given
        }
    "#;
    check_errors(src);
}

#[test]
fn incorrect_turbofish_count_on_primitive_fmtstr() {
    let src = r#"
        trait MyTrait {
            fn foo();
        }

        impl<let N: u32, T> MyTrait for fmtstr<N, T> {
            fn foo() { }
        }

        fn main() {
            let _ = fmtstr::<5>::foo();
                          ^^^^^ primitive type fmtstr expects 2 generics but 1 was given
        }
    "#;
    check_errors(src);
}

#[test]
fn turbofish_on_primitive_fmtstr() {
    let src = r#"
        trait MyTrait {
            fn foo();
        }

        impl<let N: u32, T> MyTrait for fmtstr<N, T> {
            fn foo() { }
        }

        fn main() {
            let _ = fmtstr::<5, Field>::foo();
        }
    "#;
    check_errors(src);
}

// TODO: WIP
#[test]
fn regression_10363() {
    let src = r#"
    // TODO: WIP
    struct Bar {}

    pub trait Trait {
        fn foo();
    }

    pub fn foo<T: Trait>() {
        let _ = T::<i32, i32>::foo();
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}
