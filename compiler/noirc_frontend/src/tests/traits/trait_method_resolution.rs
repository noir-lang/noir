//! Tests for trait method resolution and scope rules.
//! Validates that trait methods are correctly resolved based on imports, handles ambiguity, and suggests missing imports.

use crate::tests::{assert_no_errors, check_errors};

#[test]
fn calls_trait_method_if_it_is_in_scope_with_multiple_candidates_but_only_one_decided_by_generics()
{
    let src = r#"
    struct Foo {
        inner: Field,
    }

    trait Converter<N> {
        fn convert(self) -> N;
    }

    impl Converter<Field> for Foo {
        fn convert(self) -> Field {
            self.inner
        }
    }

    impl Converter<u32> for Foo {
        fn convert(self) -> u32 {
            self.inner as u32
        }
    }

    fn main() {
        let foo = Foo { inner: 42 };
        let _: u32 = foo.convert();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_6530() {
    let src = r#"
    pub trait From2<T> {
        fn from2(input: T) -> Self;
    }
    
    pub trait Into2<T> {
        fn into2(self) -> T;
    }
    
    impl<T, U> Into2<T> for U
    where
        T: From2<U>,
    {
        fn into2(self) -> T {
            T::from2(self)
        }
    }
    
    struct Foo {
        inner: Field,
    }
    
    impl Into2<Field> for Foo {
        fn into2(self) -> Field {
            self.inner
        }
    }
    
    fn main() {
        let foo = Foo { inner: 0 };
    
        // This works:
        let _: Field = Into2::<Field>::into2(foo);
    
        // This was failing with 'No matching impl':
        let _: Field = foo.into2();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_9245_small_code() {
    let src = r#"
    pub trait From2<T> {}

    impl<T> From2<T> for T {}

    pub trait Into2<T> {}

    impl From2<u8> for Field {}

    impl<T: From2<U>, U> Into2<T> for U {}

    fn foo<T: Into2<Field>>() {}

    fn main() {
        foo::<u8>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_function_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let _ = Bar::foo();
                     ^^^ trait `private_mod::Foo` which provides `foo` is implemented but not in scope, please import it
    }

    pub struct Bar {
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn calls_trait_function_if_it_is_in_scope() {
    let src = r#"
    use private_mod::Foo;

    fn main() {
        let _ = Bar::foo();
    }

    pub struct Bar {
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn calls_trait_function_if_it_is_only_candidate_in_scope() {
    let src = r#"
    use private_mod::Foo;

    fn main() {
        let _ = Bar::foo();
    }

    pub struct Bar {
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }

        pub trait Foo2 {
            fn foo() -> i32;
        }

        impl Foo2 for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn calls_trait_function_if_it_is_only_candidate_in_scope_in_nested_module_using_super() {
    let src = r#"
    mod moo {
        use super::public_mod::Foo;

        pub fn method() {
            let _ = super::Bar::foo();
        }
    }

    pub struct Bar {}

    pub mod public_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_trait_is_not_in_scope_for_function_call_and_there_are_multiple_candidates() {
    let src = r#"
    fn main() {
        let _ = Bar::foo();
                     ^^^ Could not resolve 'foo' in path
                     ~~~ The following traits which provide `foo` are implemented but not in scope: `private_mod::Foo2`, `private_mod::Foo`
    }

    pub struct Bar {
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }

        pub trait Foo2 {
            fn foo() -> i32;
        }

        impl Foo2 for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_multiple_trait_methods_are_in_scope_for_function_call() {
    let src = r#"
    use private_mod::Foo;
    use private_mod::Foo2;

    fn main() {
        let _ = Bar::foo();
                     ^^^ Multiple applicable items in scope
                     ~~~ All these trait which provide `foo` are implemented and in scope: `private_mod::Foo2`, `private_mod::Foo`
    }

    pub struct Bar {
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }

        pub trait Foo2 {
            fn foo() -> i32;
        }

        impl Foo2 for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_method_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let bar = Bar { x: 42 };
        let _ = bar.foo();
                ^^^^^^^^^ trait `private_mod::Foo` which provides `foo` is implemented but not in scope, please import it
    }

    pub struct Bar {
        x: i32,
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn calls_trait_method_if_it_is_in_scope() {
    let src = r#"
    use private_mod::Foo;

    fn main() {
        let bar = Bar { x: 42 };
        let _ = bar.foo();
    }

    pub struct Bar {
        x: i32,
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_trait_is_not_in_scope_for_method_call_and_there_are_multiple_candidates() {
    let src = r#"
    fn main() {
        let bar = Bar { x: 42 };
        let _ = bar.foo();
                ^^^^^^^^^ Could not resolve 'foo' in path
                ~~~~~~~~~ The following traits which provide `foo` are implemented but not in scope: `private_mod::Foo2`, `private_mod::Foo`
    }

    pub struct Bar {
        x: i32,
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }

        pub trait Foo2 {
            fn foo(self) -> i32;
        }

        impl Foo2 for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_multiple_trait_methods_are_in_scope_for_method_call() {
    let src = r#"
    use private_mod::Foo;
    use private_mod::Foo2;

    fn main() {
        let bar = Bar { x : 42 };
        let _ = bar.foo();
                ^^^^^^^^^ Multiple applicable items in scope
                ~~~~~~~~~ All these trait which provide `foo` are implemented and in scope: `private_mod::Foo2`, `private_mod::Foo`
    }

    pub struct Bar {
        x: i32,
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }

        pub trait Foo2 {
            fn foo(self) -> i32;
        }

        impl Foo2 for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_primitive_function_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let _ = Field::foo();
                       ^^^ trait `private_mod::Foo` which provides `foo` is implemented but not in scope, please import it
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for Field {
            fn foo() -> i32 {
                42
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_primitive_method_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let x: Field = 1;
        let _ = x.foo();
                ^^^^^^^ trait `private_mod::Foo` which provides `foo` is implemented but not in scope, please import it
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for Field {
            fn foo(self) -> i32 {
                self as i32
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_generic_function_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let x: i32 = 1;
        let _ = x.foo();
                ^^^^^^^ trait `private_mod::Foo` which provides `foo` is implemented but not in scope, please import it
    }

    mod private_mod {
        pub trait Foo<T> {
            fn foo(self) -> i32;
        }

        impl<T> Foo<T> for T {
            fn foo(self) -> i32 {
                42
            }
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn calls_trait_method_using_struct_name_when_multiple_impls_exist() {
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
        let _ = U60Repr::from2([1, 2, 3]);
        let _ = U60Repr::from2(1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn suggests_importing_trait_via_reexport() {
    let src = r#"
    mod one {
        mod two {
            pub trait Trait {
                fn method(self);
            }

            impl Trait for bool {
                fn method(self) {}
            }
        }

        pub use two::Trait;
    }

    fn main() {
        true.method()
        ^^^^^^^^^^^^^ trait `one::Trait` which provides `method` is implemented but not in scope, please import it
    }
    "#;
    check_errors(src);
}

#[test]
fn suggests_importing_trait_via_module_reexport() {
    let src = r#"
    mod one {
        mod two {
            pub mod three {
                pub trait Trait {
                    fn method(self);
                }

                impl Trait for bool {
                    fn method(self) {}
                }
            }
        }

        pub use two::three;
    }

    fn main() {
        true.method()
        ^^^^^^^^^^^^^ trait `one::three::Trait` which provides `method` is implemented but not in scope, please import it
    }
    "#;
    check_errors(src);
}

#[test]
fn inherent_impl_shadows_trait_impl_for_qualified_calls() {
    // Inherent impls take precedence over trait impls for qualified method calls.
    let src = r#"
    struct Foo {
        value: Field,
    }

    trait Trait {
        fn method(self) -> Field;
    }

    impl Trait for Foo {
        fn method(self) -> Field {
            100 + self.value
        }
    }

    impl Foo {
        fn method(self) -> Field {
            200 + self.value
        }
    }

    fn main() {
        let foo = Foo { value: 42 };

        // Qualified call should resolve to inherent impl (200, not 100)
        assert(Foo::method(foo) == 242);

        // Instance call should also resolve to inherent impl
        let foo2 = Foo { value: 42 };
        assert(foo2.method() == 242);

        // Trait impl via trait syntax
        // Even when an inherent impl shadows a trait impl for qualified calls,
        // you can still access the trait method using trait syntax.
        assert(Trait::method(foo2) == 142);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn ambiguous_trait_method_multiple_bounds_with_self() {
    let src = r#"
    pub trait One {
        fn method(_self: Self) {}
    }

    pub trait Two {
        fn method(_self: Self) {}
    }

    pub struct Foo {}
    impl One for Foo {}
    impl Two for Foo {}

    fn foo<T: One + Two>(x: T) {
        x.method();
        ^^^^^^^^^^ Multiple applicable items in scope
        ~~~~~~~~~~ All these trait which provide `method` are implemented and in scope: `One`, `Two`
    }

    fn main() {
        foo(Foo {});
    }
    "#;
    check_errors(src);
}

#[test]
fn ambiguous_trait_method_in_parent_child_relationship_with_self() {
    let src = r#"
    trait Parent {
        fn foo(_self: Self);
    }

    trait Child: Parent {
        fn foo(_self: Self);
    }

    pub fn foo<T: Child>(x: T) {
        x.foo();
        ^^^^^^^ Multiple applicable items in scope
        ~~~~~~~ All these trait which provide `foo` are implemented and in scope: `Child`, `Parent`
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn ambiguous_trait_method_in_parent_child_relationship_without_self() {
    let src = r#"
    trait Parent {
        fn foo();
    }

    trait Child: Parent {
        fn foo();
    }

    pub fn foo<T: Child>() {
        T::foo();
        ^^^^^^ Multiple applicable items in scope
        ~~~~~~ All these trait which provide `foo` are implemented and in scope: `Child`, `Parent`
    }

    fn main() {}
    "#;
    check_errors(src);
}
