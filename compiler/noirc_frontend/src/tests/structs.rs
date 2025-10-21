//! Tests for struct definitions and their method implementations.

use crate::tests::{assert_no_errors, check_errors, check_monomorphization_error};

#[test]
fn duplicate_struct_field() {
    let src = r#"
    pub struct Foo {
        x: i32,
        ~ First struct field found here
        x: i32,
        ^ Duplicate definitions of struct field with name x found
        ~ Second struct field found here
    }
    "#;
    check_errors(src);
}

#[test]
fn object_type_must_be_known_in_method_call() {
    let src = r#"
    pub fn foo<let N: u32>() -> [Field; N] {
        let array = [];
        let mut bar = array[0];
        let _ = bar.len();
                ^^^ Object type is unknown in method call
                ~~~ Type must be known by this point to know which method to call
        bar
    }
    "#;
    check_errors(src);
}

#[test]
fn incorrect_generic_count_on_struct_impl() {
    let src = r#"
    struct Foo {}
    impl <T> Foo<T> {}
             ^^^ Foo expects 0 generics but 1 was given
    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn uses_self_type_for_struct_function_call() {
    let src = r#"
    struct S { }

    impl S {
        fn one() -> Field {
            1
        }

        fn two() -> Field {
            Self::one() + Self::one()
        }
    }

    fn main() {
        let _ = S {}; // silence S never constructed warning
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_with_better_message_when_trying_to_invoke_struct_field_that_is_a_function() {
    let src = r#"
        pub struct Foo {
            wrapped: fn(Field) -> bool,
        }

        impl Foo {
            fn call(self) -> bool {
                self.wrapped(1)
                ^^^^^^^^^^^^^^^ Cannot invoke function field 'wrapped' on type 'Foo' as a method
                ~~~~~~~~~~~~~~~ to call the function stored in 'wrapped', surround the field access with parentheses: '(', ')'
            }
        }
    "#;
    check_errors(src);
}

#[test]
fn check_impl_duplicate_method_without_self() {
    let src = "
    pub struct Foo {}

    impl Foo {
        fn foo() {}
           ~~~ first definition found here
        fn foo() {}
           ^^^ duplicate definitions of foo found
           ~~~ second definition found here
    }
    ";
    check_errors(src);
}

#[test]
fn unconstrained_type_parameter_in_impl() {
    let src = r#"
        pub struct Foo<T> {}

        impl<T, U> Foo<T> {}
                ^ The type parameter `U` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `U` type parameter

        fn main() {
            let _ = Foo::<i32> {};
        }
        "#;
    check_errors(src);
}

#[test]
fn unconstrained_numeric_generic_in_impl() {
    let src = r#"
        pub struct Foo {}

        impl<let N: u32> Foo {}
                 ^ The type parameter `N` is not constrained by the impl trait, self type, or predicates
                 ~ Hint: remove the `N` type parameter

        fn main() {
            let _ = Foo {};
        }
        "#;
    check_errors(src);
}

#[test]
fn cannot_determine_type_of_generic_argument_in_function_call_for_generic_impl() {
    let src = r#"
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {}
    }

    fn main() {
        Foo::one();
             ^^^ Type annotation needed
             ~~~ Could not determine the type of the generic argument `T` declared on the struct `Foo`
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_determine_type_of_generic_argument_in_struct_constructor() {
    let src = r#"
    struct Foo<T> {}

    fn main()
    {
        let _ = Foo {};
                ^^^ Type annotation needed
                ~~~ Could not determine the type of the generic argument `T` declared on the struct `Foo`
    }

    "#;
    check_errors(src);
}

#[test]
fn resolves_generic_type_argument_via_self() {
    let src = "
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {
            Self::two();
        }

        fn two() {}
    }

    fn main() {
        Foo::<i32>::one();
    }
    ";
    check_monomorphization_error(src);
}

#[test]
fn mutable_self_call() {
    let src = r#"
    fn main() {
        let mut bar = Bar {};
        let _ = bar.bar();
    }

    struct Bar {}

    impl Bar {
        fn bar(&mut self) {
            let _ = self;
        }
    }
    "#;
    assert_no_errors(src);
}
