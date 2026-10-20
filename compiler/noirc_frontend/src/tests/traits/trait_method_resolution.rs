//! Tests for trait method resolution and scope rules.
//! Validates that trait methods are correctly resolved based on imports, handles ambiguity, and suggests missing imports.

use crate::test_utils::{get_monomorphized, stdlib_src};
use crate::tests::{
    assert_no_errors, check_errors, check_errors_with_stdlib, check_monomorphization_error,
};

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
                     ~~~ Multiple traits which provide `foo` are implemented and in scope: `private_mod::Foo2`, `private_mod::Foo`
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
fn multiple_traits_in_scope_diagnostic_lists_only_in_scope_traits() {
    let src = r#"
    use private_mod::Foo;
    use private_mod::Foo2;

    fn main() {
        let _ = Bar::foo();
                     ^^^ Multiple applicable items in scope
                     ~~~ Multiple traits which provide `foo` are implemented and in scope: `private_mod::Foo2`, `private_mod::Foo`
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

        pub trait Foo3 {
            fn foo() -> i32;
        }

        impl Foo3 for super::Bar {
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
                ~~~~~~~~~ Multiple traits which provide `foo` are implemented and in scope: `private_mod::Foo2`, `private_mod::Foo`
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

/// Regression test for <https://github.com/noir-lang/noir/issues/8963>
///
/// When a path-based trait call like `U60Repr::from2(x)` has multiple candidate
/// impls, only one of which is compatible with the argument type, the trait
/// solver must run before integer-literal defaulting — otherwise the array's
/// element type defaults to `Field` and the only matching impl (`From2<[i32; 3]>`)
/// is no longer reachable.
#[test]
fn calls_trait_method_using_struct_name_picks_impl_before_defaulting_integers() {
    let src = r#"
    trait From2<T> {
        fn from2(input: T) -> Self;
    }

    struct U60Repr {}

    impl From2<[i32; 3]> for U60Repr {
        fn from2(_: [i32; 3]) -> Self {
            U60Repr {}
        }
    }

    impl From2<i32> for U60Repr {
        fn from2(_: i32) -> Self {
            U60Repr {}
        }
    }

    fn main() {
        let x = [1, 2, 3];
        let _ = U60Repr::from2(x);
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
        ~~~~~~~~~~ Multiple traits which provide `method` are implemented and in scope: `One`, `Two`
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
        ~~~~~~~ Multiple traits which provide `foo` are implemented and in scope: `Child`, `Parent`
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
        ~~~~~~ Multiple traits which provide `foo` are implemented and in scope: `Child`, `Parent`
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_10537() {
    let src = r#"
    use cmp::Ord;

    pub fn min<T>(v1: T, v2: T) -> T
    where
        T: Ord,
    {
        if v1 > v2 {
            v2
        } else {
            v1
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::EQ, stdlib_src::ORD]);
}

// Expecting missing `Eq` impl errors to resolve to a `NoMatchingImplFound` error
// with a trait name of "PopulateDummyOperatorTraitsTrait" when the `Eq` trait
// hasn't been explicitly defined, i.e. when it's only been defined by
// `NodeInterner::populate_dummy_operator_traits`
#[test]
fn missing_eq_trait_error() {
    let src = "
    struct Foo {}
    fn main() {
        let x = Foo {};
        let y = Foo {};
        assert(x == y);
               ^^^^^^ No matching impl found for `Foo: PopulateDummyOperatorTraitsTrait`
               ~~~~~~ No impl for `Foo: PopulateDummyOperatorTraitsTrait`
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10219() {
    let src = "
    fn main() {
        let mut x: u32 = 0;
        let mut y: u32 = 0;
        assert(&mut x == &mut y);
               ^^^^^^^^^^^^^^^^ No matching impl found for `&mut u32: Eq`
               ~~~~~~~~~~~~~~~~ No impl for `&mut u32: Eq`
    }
    ";
    check_errors_with_stdlib(src, [stdlib_src::EQ]);
}

#[test]
fn regression_10766() {
    let src = r#"
    trait Foo {
        type Bar;
    }

    pub struct Example { }
    pub struct Baz { }

    impl Foo for Example {
        type Bar = Baz;
    }

    impl Foo::Bar { }
         ^^^^^^^^ Cannot define a trait impl on associated types

    fn main() { }
    "#;
    check_errors(src);
}

#[test]
fn comptime_trait_method_with_generic_struct() {
    let src = r#"
    trait Metadata {
        comptime fn name() -> str<9>;
    }

    struct Typed<T> {
        value: T,
    }

    impl Metadata for Typed<Field> {
        comptime fn name() -> str<9> {
            "TypedFiel"
        }
    }

    fn main() {
        comptime {
            let _ = Typed::<Field>::name();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_trait_method_with_numeric_generic() {
    let src = r#"
    trait Info {
        comptime fn size() -> u32;
    }

    struct FixedArray<let N: u32> {}

    impl<let N: u32> Info for FixedArray<N> {
        comptime fn size() -> u32 {
            N
        }
    }

    fn main() {
        comptime {
            let s = FixedArray::<5>::size();
            assert(s == 5);
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_method_returning_self_with_generic_impl() {
    let src = r#"
    trait Doubled {
        fn doubled(self) -> Self;
    }

    struct Wrapper<T> {
        inner: T,
    }

    impl Doubled for Field {
        fn doubled(self) -> Self {
            self * 2
        }
    }

    impl<T> Doubled for Wrapper<T> where T: Doubled {
        fn doubled(self) -> Self {
            Wrapper { inner: self.inner.doubled() }
        }
    }

    fn double_it<T>(t: T) -> T where T: Doubled {
        t.doubled()
    }

    fn main() {
        let w = Wrapper { inner: 21 as Field };
        let d = double_it(w);
        assert(d.inner == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn two_traits_same_method_name_disambiguated_by_constraint() {
    let src = r#"
    trait TraitA {
        fn do_thing(self) -> Field;
    }

    trait TraitB {
        fn do_thing(self) -> bool;
    }

    struct Foo {
        val: Field,
    }

    impl TraitA for Foo {
        fn do_thing(self) -> Field { self.val }
    }

    impl TraitB for Foo {
        fn do_thing(self) -> bool { self.val != 0 }
    }

    fn use_a<T>(t: T) -> Field where T: TraitA {
        t.do_thing()
    }

    fn use_b<T>(t: T) -> bool where T: TraitB {
        t.do_thing()
    }

    fn main() {
        let f = Foo { val: 42 };
        assert(use_a(f) == 42);
        let f2 = Foo { val: 42 };
        assert(use_b(f2));
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for <https://github.com/noir-lang/noir/issues/11540>
#[test]
fn trait_method_resolved_with_multiple_impls_different_type_params() {
    let src = r#"
    pub struct Foo<let Y: u32, A> {}

    pub trait Bar {
        comptime fn bar();
    }

    impl<let X: u32> Bar for Foo<X, u8> {
        comptime fn bar() {}
    }

    impl<let X: u32> Bar for Foo<X, u16> {
        comptime fn bar() {}
    }

    fn g<let Z: u32>() {
        Foo::<Z, u8>::bar();
    }

    fn main() {
        g::<1>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn type_path_generic_struct_method() {
    let src = r#"
    pub trait Deserialize {
        fn deserialize();
    }

    struct Gen<T> {}

    impl<T> Deserialize for Gen<T> {
        fn deserialize() {}
    }

    fn main() {
        let _ = <Gen<Field>>::deserialize;
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn type_path_generic_array_method_1() {
    let src = r#"
    pub trait Deserialize {
        fn deserialize();
    }

    impl<T, let M: u32> Deserialize for [T; M] {
        fn deserialize() {
            let _: u32 = M;
        }
    }

    fn main() {
        let _ = <[Field; 3]>::deserialize;
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn type_path_generic_array_method_2() {
    let src = r#"
    pub trait Deserialize {
        fn deserialize();
    }

    impl<let M: u32, T> Deserialize for [T; M] {
        fn deserialize() {
            let _: u32 = M;
        }
    }

    fn main() {
        let _ = <[Field; 3]>::deserialize;
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn type_path_generic_array_method_3() {
    let src = r#"
    pub trait Deserialize {
        fn deserialize<U>(x: U) -> U;
    }

    impl<let M: u32, T> Deserialize for [T; M] {
        fn deserialize<U>(x: U) -> U {
            let _: u32 = M;
            x
        }
    }

    fn main() {
        let _: u32 = <[Field; 3]>::deserialize::<u64>(0);
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected type u32, found type u64
    }
    "#;
    check_errors(src);
}

#[test]
fn type_path_generic_tuple_method() {
    let src = r#"
    pub trait Deserialize {
        fn deserialize();
    }

    impl<A, B> Deserialize for (A, B) {
        fn deserialize() {}
    }

    fn main() {
        let _ = <(u32, i32)>::deserialize;
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn trait_method_and_struct_method_with_same_name() {
    let src = r#"
    pub struct MyStruct {}
    impl MyStruct {
        fn foo() -> i32 {
            0
        }
    }

    pub trait MyTrait {
        fn foo();
    }

    impl MyTrait for MyStruct {
        fn foo() {
            // This method should resolve to the impl method, not the trait method,
            // and we verify this by checking that the return type is correct (i32, not ()).
            let _: i32 = MyStruct::foo();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_method_and_struct_method_with_same_name_and_turbofish() {
    let src = r#"
    pub struct MyStruct<T> {}
    impl<T> MyStruct<T> {
        fn foo() -> i32 {
            0
        }
    }

    pub trait MyTrait<T> {
        fn foo();
    }

    impl<T> MyTrait<T> for MyStruct<T> {
        fn foo() {
            // This method should resolve to the impl method, not the trait method,
            // and we verify this by checking that the return type is correct (i32, not ()).
            let _: i32 = MyStruct::<T>::foo();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_eq_impl_for_struct_in_assert_eq() {
    let src = r#"
    struct myStruct {}

    fn main(x: myStruct, y: pub myStruct) {
        assert_eq(x, y);
                  ^^^^ No matching impl found for `myStruct: Eq`
                  ~~~~ No impl for `myStruct: Eq`
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::EQ]);
}

#[test]
fn inherited_static_default_method_on_generic_impl_uses_the_type_arguments() {
    let src = r#"
    pub trait Lim { fn limit() -> u32; fn make() -> u32 { Self::limit() } }
    pub struct G<let N: u32> {}
    impl<let N: u32> Lim for G<N> { fn limit() -> u32 { N } }
    pub struct H<T> {}
    impl<T> Lim for H<T> { fn limit() -> u32 { 5 } }
    fn main() -> pub (u32, u32) { (G::<7>::make(), H::<u8>::make()) }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub (u32, u32) {
        (make$f1(), make$f2())
    }
    fn make$f1() -> u32 {
        limit$f3()
    }
    fn make$f2() -> u32 {
        limit$f4()
    }
    fn limit$f3() -> u32 {
        7
    }
    fn limit$f4() -> u32 {
        5
    }
    ");
}

#[test]
fn inherited_static_default_method_on_generic_impl_is_ambiguous_with_another_trait() {
    let src = r#"
    pub trait A { fn limit() -> u32; fn make() -> u32 { Self::limit() } }
    pub trait B { fn make() -> u32; }
    pub struct G<let N: u32> {}
    impl<let N: u32> A for G<N> { fn limit() -> u32 { N } }
    impl<let N: u32> B for G<N> { fn make() -> u32 { 1000 } }
    fn main() -> pub u32 { G::<7>::make() }
                                   ^^^^ Multiple applicable items in scope
                                   ~~~~ Multiple traits which provide `make` are implemented and in scope: `A`, `B`
    "#;
    check_errors(src);
}

#[test]
fn inherited_self_default_method_on_generic_impl_keeps_the_receiver_generic() {
    let src = r#"
    pub trait Lim { fn limit(self) -> u32; fn me(self) -> u32 { self.limit() } }
    pub struct G<let N: u32> {}
    impl<let N: u32> Lim for G<N> { fn limit(self) -> u32 { N } }
    fn main() -> pub u32 {
        let g = G {};
        let r = g.me();
        let _h: G<7> = g;
        r
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u32 {
        let g$l0 = {
            ()
        };
        let r$l1 = me$f1(g$l0);
        let _h$l2 = g$l0;
        r$l1
    }
    fn me$f1(self$l3: ()) -> u32 {
        limit$f2(self$l3)
    }
    fn limit$f2(self$l4: ()) -> u32 {
        7
    }
    ");
}

#[test]
fn inherited_self_default_method_on_type_generic_impl_keeps_the_receiver_generic() {
    let src = r#"
    pub trait Lim { fn limit(self) -> u32; fn me(self) -> u32 { self.limit() } }
    pub struct H<T> { x: [T; 0] }
    impl<T> Lim for H<T> { fn limit(self) -> u32 { 5 } }
    fn main() -> pub u32 {
        let h = H { x: [] };
        let r = h.me();
        let _k: H<u8> = h;
        r
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn inherited_self_default_method_on_generic_impl_needs_the_receiver_type_known() {
    let src = r#"
    pub trait Lim { fn limit(self) -> u32; fn me(self) -> u32 { self.limit() } }
    pub struct G<let N: u32> {}
    impl<let N: u32> Lim for G<N> { fn limit(self) -> u32 { N } }
    fn main() -> pub u32 {
        let g = G {};
                ^ Type annotation needed
                ~ Could not determine the value of the generic argument `N` declared on the struct `G`
        g.me()
    }
    "#;
    check_errors(src);
}

#[test]
fn inherited_self_default_method_inside_generic_impl_does_not_infer_the_receiver_from_self() {
    let src = r#"
    pub trait Lim { fn limit(self) -> u32; fn me(self) -> u32 { self.limit() } fn two(self) -> u32; }
    pub struct G<let N: u32> {}
    impl<let N: u32> Lim for G<N> {
        fn limit(self) -> u32 { N }
        fn two(self) -> u32 { let g = G {}; g.me() }
                                      ^ Type annotation needed
                                      ~ Could not determine the value of the generic argument `N` declared on the struct `G`
    }
    fn main() -> pub u32 { G::<3> {}.two() * 100 + G::<9> {}.two() }
    "#;
    check_errors(src);
}

#[test]
fn method_call_that_a_blanket_impl_with_a_default_body_also_serves_is_ambiguous() {
    let src = r#"
    trait Marker {}
    trait Foo {
        fn value(self) -> Field {
            let _ = self;
            1
        }
    }
    struct S<U> { x: U }
    impl<T> Foo for T where T: Marker {}
    impl<U> Foo for S<U> {
        fn value(self) -> Field { 2 }
    }

    pub fn value_of<U>(x: U) -> Field where S<U>: Marker {
        S { x }.value()
        ^^^^^^^^^^^^^ Multiple trait impls match the object type `S<U>`
        ~~~~~~~~~~~~~ Ambiguous impl
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn method_call_that_a_blanket_impl_also_serves_under_a_where_clause_is_ambiguous() {
    let src = r#"
    trait Marker {}
    trait Foo {
        fn value(self) -> Field;
    }
    struct S<U> { x: U }
    impl<T> Foo for T where T: Marker {
        fn value(self) -> Field { 1 }
    }
    impl<U> Foo for S<U> {
        fn value(self) -> Field { 2 }
    }

    pub fn value_of<U>(x: U) -> Field where S<U>: Marker {
        S { x }.value()
        ^^^^^^^^^^^^^ Multiple trait impls match the object type `S<U>`
        ~~~~~~~~~~~~~ Ambiguous impl
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn type_path_call_that_a_blanket_impl_also_serves_under_a_where_clause_is_ambiguous() {
    let src = r#"
    trait Marker {}
    trait Foo {
        fn value(self) -> Field;
    }
    struct S<U> { x: U }
    impl<T> Foo for T where T: Marker {
        fn value(self) -> Field { 1 }
    }
    impl<U> Foo for S<U> {
        fn value(self) -> Field { 2 }
    }

    pub fn value_of<U>(x: U) -> Field where S<U>: Marker {
        S::value(S { x })
        ^^^^^^^^ Multiple trait impls match the object type `S<U>`
        ~~~~~~~~ Ambiguous impl
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn turbofish_path_call_that_a_blanket_impl_also_serves_under_a_where_clause_is_ambiguous() {
    let src = r#"
    trait Marker {}
    trait Foo {
        fn value(self) -> Field;
    }
    struct S<U> { x: U }
    impl<T> Foo for T where T: Marker {
        fn value(self) -> Field { 1 }
    }
    impl<U> Foo for S<U> {
        fn value(self) -> Field { 2 }
    }

    pub fn value_of<U>(x: U) -> Field where S<U>: Marker {
        S::<U>::value(S { x })
        ^^^^^^^^^^^^^ Multiple trait impls match the object type `S<U>`
        ~~~~~~~~~~~~~ Ambiguous impl
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn method_call_selects_a_types_own_impl_over_a_blanket_impl_with_an_undecided_where_clause() {
    let src = r#"
    trait From<T> {
        fn from(input: T) -> Self;
    }
    trait Into<T> {
        fn into(self) -> T;
    }
    impl<T, U> Into<T> for U where T: From<U> {
        fn into(self) -> T { T::from(self) }
    }
    impl<T> From<T> for T {
        fn from(input: T) -> T { input }
    }

    struct S {}
    impl Into<Field> for S {
        fn into(self) -> Field { 2 }
    }

    fn main() {
        let _ = S {}.into();
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _$l0 = into$f1({
            ()
        })
    }
    fn into$f1(self$l1: ()) -> Field {
        2
    }
    ");
}

#[test]
fn calls_select_a_types_own_impl_over_a_blanket_impl_whose_where_clause_fails() {
    let src = r#"
    trait Marker {}
    trait Foo {
        fn value(self) -> Field;
    }
    struct S {}
    struct G<T> {}
    impl<T> Foo for T where T: Marker {
        fn value(self) -> Field { 1 }
    }
    impl Foo for S {
        fn value(self) -> Field { 2 }
    }
    impl Foo for G<u8> {
        fn value(self) -> Field { 3 }
    }

    fn main() -> pub Field {
        S {}.value() + S::value(S {}) + G::<u8>::value(G {})
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        ((value$f1({
            ()
        }) + value$f1({
            ()
        })) + value$f2({
            ()
        }))
    }
    fn value$f1(self$l0: ()) -> Field {
        2
    }
    fn value$f2(self$l1: ()) -> Field {
        3
    }
    ");
}

#[test]
fn type_path_call_selects_a_types_own_impl_over_a_blanket_impl_of_another_trait() {
    let src = r#"
    trait From<T> {
        fn from(input: T) -> Self;
    }
    impl<T> From<T> for T {
        fn from(input: T) -> T { input }
    }

    trait Parse {
        fn from(x: Field) -> Self;
    }
    struct S { x: Field }
    impl Parse for S {
        fn from(x: Field) -> Self { S { x } }
    }

    fn main() -> pub Field {
        S::from(1).x
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        from$f1(1).0
    }
    fn from$f1(x$l0: Field) -> (Field,) {
        {
            let x$l1 = x$l0;
            (x$l1)
        }
    }
    ");
}

#[test]
fn method_call_selects_a_types_own_impl_over_a_blanket_impl_at_other_trait_arguments() {
    let src = r#"
    struct W { v: Field }
    trait Scale<X> {
        fn scale(self, k: X) -> Field;
    }
    impl<T> Scale<Field> for T {
        fn scale(self, k: Field) -> Field { k }
    }
    impl Scale<u32> for W {
        fn scale(self, k: u32) -> Field { self.v * (k as Field) }
    }

    fn main(x: Field) -> pub Field {
        let w = W { v: x };
        w.scale(5)
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(x$l0: Field) -> pub Field {
        let w$l2 = {
            let v$l1 = x$l0;
            (v$l1)
        };
        scale$f1(w$l2, 5)
    }
    fn scale$f1(self$l3: (Field,), k$l4: u32) -> Field {
        (self$l3.0 * (k$l4 as Field))
    }
    ");
}

#[test]
fn type_path_call_selects_a_types_own_impl_over_an_identity_blanket_impl() {
    let src = r#"
    trait Conv<X> {
        fn conv(x: X) -> Self;
    }
    impl<T> Conv<T> for T {
        fn conv(x: T) -> T { x }
    }
    impl Conv<u8> for u64 {
        fn conv(x: u8) -> u64 { x as u64 + 100 }
    }

    fn main() -> pub u64 {
        u64::conv(3)
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u64 {
        conv$f1(3)
    }
    fn conv$f1(x$l0: u8) -> u64 {
        ((x$l0 as u64) + 100)
    }
    ");
}

#[test]
fn method_call_selects_a_blanket_impl_when_a_types_own_impl_has_a_failing_where_clause() {
    let src = r#"
    trait Marker {}
    trait Other {}
    trait Foo {
        fn value(self) -> Field;
    }
    struct S<A> { x: A }
    impl<A> Foo for S<A> where A: Marker {
        fn value(self) -> Field { 1 }
    }
    impl<T> Foo for T where T: Other {
        fn value(self) -> Field { 2 }
    }
    impl Other for S<Field> {}

    fn main() -> pub Field {
        S { x: 1 as Field }.value()
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        value$f1({
            let x$l0 = (1 as Field);
            (x$l0)
        })
    }
    fn value$f1(self$l1: (Field,)) -> Field {
        2
    }
    ");
}

#[test]
fn type_path_call_under_where_clause_selects_the_assumed_impl_over_a_types_own_impl() {
    let src = r#"
    trait Small {}
    trait Limit { fn max() -> u64; }
    pub struct S<T> {}
    impl Small for S<u8> {}
    impl Limit for S<u64> { fn max() -> u64 { 18446744073709551615 } }
    impl<T: Small> Limit for T { fn max() -> u64 { 255 } }

    fn f<T>() -> u64 where S<T>: Limit { S::max() }

    fn main() -> pub u64 { f::<u8>() }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u64 {
        f$f1()
    }
    fn f$f1() -> u64 {
        max$f2()
    }
    fn max$f2() -> u64 {
        18446744073709551615
    }
    ");
}

#[test]
fn type_path_call_under_where_clause_selects_the_assumed_impl_when_a_blanket_where_clause_is_undecided()
 {
    let src = r#"
    trait Small {}
    trait Limit { fn max() -> u64; }
    pub struct S<T> {}
    impl Small for S<u8> {}
    impl Small for S<u16> {}
    impl Limit for S<u64> { fn max() -> u64 { 18446744073709551615 } }
    impl<T: Small> Limit for T { fn max() -> u64 { 255 } }

    fn f<T>() -> u64 where S<T>: Limit { S::max() }

    fn main() -> pub u64 { f::<u8>() }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u64 {
        f$f1()
    }
    fn f$f1() -> u64 {
        max$f2()
    }
    fn max$f2() -> u64 {
        18446744073709551615
    }
    ");
}

#[test]
fn self_type_path_call_in_a_bounded_impl_selects_the_assumed_impl_over_a_types_own_impl() {
    let src = r#"
    trait Small {}
    trait Limit { fn max() -> u64; }
    pub struct S<T> {}
    impl Small for S<u8> {}
    impl Small for S<u16> {}
    impl Limit for S<u64> { fn max() -> u64 { 18446744073709551615 } }
    impl<T: Small> Limit for T { fn max() -> u64 { 255 } }

    trait Validate { fn limit() -> u64; }
    impl<T> Validate for S<T> where S<T>: Limit {
        fn limit() -> u64 { Self::max() }
    }

    fn main() -> pub u64 { S::<u8>::limit() }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u64 {
        limit$f1()
    }
    fn limit$f1() -> u64 {
        max$f2()
    }
    fn max$f2() -> u64 {
        18446744073709551615
    }
    ");
}

#[test]
fn method_call_under_where_clause_selects_the_assumed_impl_over_a_types_own_impl() {
    let src = r#"
    trait Baz {}
    trait Foo { fn foo(self) -> Field; }
    pub struct Wrapper<T> {}
    impl Foo for Wrapper<u32> { fn foo(self) -> Field { 32 } }
    impl<X: Baz> Foo for X { fn foo(self) -> Field { 99 } }
    impl Baz for Wrapper<u8> {}

    fn f<T>() -> Field where Wrapper<T>: Foo {
        let w = Wrapper {};
        w.foo()
    }

    fn main() -> pub Field { f::<u8>() }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        f$f1()
    }
    fn f$f1() -> Field {
        let w$l0 = {
            ()
        };
        foo$f2(w$l0)
    }
    fn foo$f2(self$l1: ()) -> Field {
        32
    }
    ");
}

#[test]
fn type_path_call_with_an_unknown_generic_and_two_applicable_impls_is_ambiguous() {
    let src = r#"
    trait Small {}
    trait Limit { fn max() -> u64; }
    pub struct S<T> {}
    impl Small for S<u8> {}
    impl Limit for S<u64> { fn max() -> u64 { 18446744073709551615 } }
    impl<T: Small> Limit for T { fn max() -> u64 { 255 } }

    fn main() -> pub u64 { S::max() }
    "#;
    check_errors(src);
}

#[test]
fn type_path_call_selects_a_blanket_impl_when_a_types_own_impl_has_a_failing_where_clause() {
    let src = r#"
    trait Marker {}
    trait Other {}
    trait Foo {
        fn value(self) -> Field;
    }
    struct S<A> { x: A }
    impl<A> Foo for S<A> where A: Marker {
        fn value(self) -> Field { 1 }
    }
    impl<T> Foo for T where T: Other {
        fn value(self) -> Field { 2 }
    }
    impl Other for S<Field> {}

    fn main() -> pub Field {
        S::value(S { x: 1 as Field })
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        value$f1({
            let x$l0 = (1 as Field);
            (x$l0)
        })
    }
    fn value$f1(self$l1: (Field,)) -> Field {
        2
    }
    ");
}

#[test]
fn type_path_call_with_an_unknown_generic_selects_a_types_only_impl() {
    let src = r#"
    trait Limit { fn max() -> u64; }
    pub struct S<T> {}
    impl Limit for S<u64> { fn max() -> u64 { 18446744073709551615 } }

    fn main() -> pub u64 { S::max() }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u64 {
        max$f1()
    }
    fn max$f1() -> u64 {
        18446744073709551615
    }
    ");
}
