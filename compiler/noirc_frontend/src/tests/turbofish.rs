use crate::check_errors;

use crate::assert_no_errors;

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
        let _ = Foo::static_method::<N>();
        let x: Foo<u8> = Foo { a: 0 };
        x.impl_method::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}
