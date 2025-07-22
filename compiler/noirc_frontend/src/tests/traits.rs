use crate::{
    assert_no_errors, check_errors, check_monomorphization_error, elaborator::FrontendOptions,
    get_program_with_options, tests::Expect,
};

#[named]
#[test]
fn trait_inheritance() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> Field;
        }

        pub trait Bar {
            fn bar(self) -> Field;
        }

        pub trait Baz: Foo + Bar {
            fn baz(self) -> Field;
        }

        pub fn foo<T>(baz: T) -> (Field, Field, Field) where T: Baz {
            (baz.foo(), baz.bar(), baz.baz())
        }

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_inheritance_with_generics() {
    let src = r#"
        trait Foo<T> {
            fn foo(self) -> T;
        }

        trait Bar<U>: Foo<U> {
            fn bar(self);
        }

        pub fn foo<T>(x: T) -> i32 where T: Bar<i32> {
            x.foo()
        }

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_inheritance_with_generics_2() {
    let src = r#"
        pub trait Foo<T> {
            fn foo(self) -> T;
        }

        pub trait Bar<T, U>: Foo<T> {
            fn bar(self) -> (T, U);
        }

        pub fn foo<T>(x: T) -> i32 where T: Bar<i32, i32> {
            x.foo()
        }

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_inheritance_with_generics_3() {
    let src = r#"
        trait Foo<A> {}

        trait Bar<B>: Foo<B> {}

        impl Foo<i32> for () {}

        impl Bar<i32> for () {}

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_inheritance_with_generics_4() {
    let src = r#"
        trait Foo { type A; }

        trait Bar<B>: Foo<A = B> {}

        impl Foo for () { type A = i32; }

        impl Bar<i32> for () {}

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_inheritance_dependency_cycle() {
    let src = r#"
        trait Foo: Bar {}
              ^^^ Dependency cycle found
              ~~~ 'Foo' recursively depends on itself: Foo -> Bar -> Foo
        trait Bar: Foo {}
        fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn trait_inheritance_missing_parent_implementation() {
    let src = r#"
        pub trait Foo {}

        pub trait Bar: Foo {}
                       ~~~ required by this bound in `Bar`

        pub struct Struct {}

        impl Bar for Struct {}
                     ^^^^^^ The trait bound `Struct: Foo` is not satisfied
                     ~~~~~~ The trait `Foo` is not implemented for `Struct`

        fn main() {
        }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_on_unknown_type_in_trait_where_clause() {
    let src = r#"
        pub trait Foo<T> where T: Unknown {}
                                  ^^^^^^^ Could not resolve 'Unknown' in path

        fn main() {
        }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_impl_trait_constraint_is_satisfied_for_concrete_type() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T>
        where
            T: Greeter,
        {
            fn greet<U>(object: U)
            where
                U: Greeter,
            {
                object.greet();
            }
        }

        pub struct SomeGreeter;
        impl Greeter for SomeGreeter {
            fn greet(self) {}
        }

        pub struct Bar;

        impl Foo<SomeGreeter> for Bar {}

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_impl_trait_constraint_is_satisfied_for_type_variable() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T> where T: Greeter {
            fn greet(object: T) {
                object.greet();
            }
        }

        pub struct Bar;

        impl<T> Foo<T> for Bar where T: Greeter {
        }

        fn main() {
        }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_impl_trait_constraint_is_not_satisfied() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T>
        where
            T: Greeter,
               ~~~~~~~ required by this bound in `Foo`
        {
            fn greet<U>(object: U)
            where
                U: Greeter,
            {
                object.greet();
            }
        }

        pub struct SomeGreeter;

        pub struct Bar;

        impl Foo<SomeGreeter> for Bar {}
                                  ^^^ The trait bound `SomeGreeter: Greeter` is not satisfied
                                  ~~~ The trait `Greeter` is not implemented for `SomeGreeter`

        fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
// Regression test for https://github.com/noir-lang/noir/issues/6314
// Baz inherits from a single trait: Foo
fn regression_6314_single_inheritance() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Baz: Foo {}

        impl<T> Baz for T where T: Foo {}

        fn main() { }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
// Regression test for https://github.com/noir-lang/noir/issues/6314
// Baz inherits from two traits: Foo and Bar
fn regression_6314_double_inheritance() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz: Foo + Bar {}

        impl<T> Baz for T where T: Foo + Bar {}

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar for Field {
            fn bar(self) -> Self {
                self + 2
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }"#;

    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_alias_single_member() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Baz = Foo;

        impl Foo for Field {
            fn foo(self) -> Self { self }
        }

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo()
        }

        fn main() {
            let x: Field = 0;
            let _ = baz(x);
        }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_alias_two_members() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> Self;
        }

        pub trait Bar {
            fn bar(self) -> Self;
        }

        pub trait Baz = Foo + Bar;

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar for Field {
            fn bar(self) -> Self {
                self + 2
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }"#;

    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_alias_polymorphic_inheritance() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar<T> {
            fn bar(self) -> T;
        }

        trait Baz<T> = Foo + Bar<T>;

        fn baz<T, U>(x: T) -> U where T: Baz<U> {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar<bool> for Field {
            fn bar(self) -> bool {
                true
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }"#;

    assert_no_errors!(src);
}

// TODO(https://github.com/noir-lang/noir/issues/6467): currently failing, so
// this just tests that the trait alias has an equivalent error to the expected
// desugared version
#[named]
#[test]
fn trait_alias_with_where_clause_has_equivalent_errors() {
    let src = r#"
        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T>: Bar where T: Baz {}

        impl<T, U> Qux<T> for U where
            U: Bar,
            T: Baz,
        {}

        pub fn qux<T, U>(x: T, _: U) -> bool where U: Qux<T> {
            x.baz()
            ^^^^^^^ No method named 'baz' found for type 'T'
        }

        fn main() {}
    "#;
    check_errors!(src);
}

// TODO(https://github.com/noir-lang/noir/issues/6467): currently failing, so
// this just tests that the trait alias has an equivalent error to the expected
// desugared version
#[named]
#[test]
fn trait_alias_with_where_clause_has_equivalent_errors_2() {
    let alias_src = r#"
        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T> = Bar where T: Baz;

        pub fn qux<T, U>(x: T, _: U) -> bool where U: Qux<T> {
            x.baz()
            ^^^^^^^ No method named 'baz' found for type 'T'
        }

        fn main() {}
    "#;
    check_errors!(alias_src);
}

#[named]
#[test]
fn removes_assumed_parent_traits_after_function_ends() {
    let src = r#"
    trait Foo {}
    trait Bar: Foo {}

    pub fn foo<T>()
    where
        T: Bar,
    {}

    pub fn bar<T>()
    where
        T: Foo,
    {}

    fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_bounds_which_are_dependent_on_generic_types_are_resolved_correctly() {
    // Regression test for https://github.com/noir-lang/noir/issues/6420
    let src = r#"
        trait Foo {
            fn foo(self) -> Field;
        }

        trait Bar<T>: Foo {
            fn bar(self) -> Field {
                self.foo()
            }
        }

        struct MyStruct<T> {
            inner: Field,
        }

        trait MarkerTrait {}
        impl MarkerTrait for Field {}

        // `MyStruct<T>` implements `Foo` only when its generic type `T` implements `MarkerTrait`.
        impl<T> Foo for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn foo(self) -> Field {
                let _ = self;
                42
            }
        }

        // We expect this to succeed as `MyStruct<T>` satisfies `Bar`'s trait bounds
        // of implementing `Foo` when `T` implements `MarkerTrait`.
        impl<T> Bar<T> for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn bar(self) -> Field {
                31415
            }
        }

        fn main() {
            let foo: MyStruct<Field> = MyStruct { inner: 42 };
            let _ = foo.bar();
        }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn does_not_crash_on_as_trait_path_with_empty_path() {
    let src = r#"
        struct Foo {
            x: <N>,
        }

        fn main() {}
    "#;

    let allow_parser_errors = true;
    let options = FrontendOptions::test_default();
    let (_, _, errors) =
        get_program_with_options!(src, Expect::Error, allow_parser_errors, options);
    assert!(!errors.is_empty());
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn calls_trait_function_if_it_is_only_candidate_in_scope_in_nested_module_using_super() {
    let src = r#"
    mod moo {
        use super::public_mod::Foo;

        pub fn method() {
            let _ = super::Bar::foo();
        }
    }

    fn main() {}

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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn type_checks_trait_default_method_and_errors() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                            ^^^ expected type i32, found type bool
                            ~~~ expected i32 because of return type
                let _ = self;
                true
                ~~~~ bool returned here
            }
        }

        fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn type_checks_trait_default_method_and_does_not_error() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                let _ = self;
                1
            }
        }

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn type_checks_trait_default_method_and_does_not_error_using_self() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                self.bar()
            }

            fn bar(self) -> i32 {
                let _ = self;
                1
            }
        }

        fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
#[test]
fn error_on_duplicate_impl_with_associated_type() {
    let src = r#"
        trait Foo {
            type Bar;
        }

        impl Foo for i32 {
             ~~~ Previous impl defined here
            type Bar = u32;
        }

        impl Foo for i32 {
                     ^^^ Impl for type `i32` overlaps with existing impl
                     ~~~ Overlapping impl
            type Bar = u8;
        }

        fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn error_on_duplicate_impl_with_associated_constant() {
    let src = r#"
        trait Foo {
            let Bar: u32;
        }

        impl Foo for i32 {
             ~~~ Previous impl defined here
            let Bar: u32 = 5;
        }

        impl Foo for i32 {
                     ^^^ Impl for type `i32` overlaps with existing impl
                     ~~~ Overlapping impl
            let Bar: u32 = 6;
        }

        fn main() {}
    "#;
    check_errors!(src);
}

// See https://github.com/noir-lang/noir/issues/6530
#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn as_trait_path_in_expression() {
    let src = r#"
        fn main() {
            cursed::<S>();
        }

        fn cursed<T>()
            where T: Foo + Foo2
        {
            <T as Foo>::bar(1);
            <T as Foo2>::bar(());

            // Use each function with different generic arguments
            <T as Foo>::bar(());
        }

        trait Foo  { fn bar<U>(x: U); }
        trait Foo2 { fn bar<U>(x: U); }

        pub struct S {}

        impl Foo for S {
            fn bar<Z>(_x: Z) {}
        }

        impl Foo2 for S {
            fn bar<Z>(_x: Z) {}
        }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn allows_renaming_trait_during_import() {
    // Regression test for https://github.com/noir-lang/noir/issues/7632
    let src = r#"
    mod trait_mod {
        pub trait Foo {
            fn foo(_: Self) {}
        }

        impl Foo for Field {}
    }

    use trait_mod::Foo as FooTrait;

    fn main(x: Field) {
        x.foo();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn renaming_trait_avoids_name_collisions() {
    // Regression test for https://github.com/noir-lang/noir/issues/7632
    let src = r#"
    mod trait_mod {
        pub trait Foo {
            fn foo(_: Self) {}
        }

        impl Foo for Field {}
    }

    use trait_mod::Foo as FooTrait;

    pub struct Foo {}

    fn main(x: Field) {
        x.foo();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn passes_trait_with_associated_number_to_generic_function() {
    let src = "
    trait Trait {
        let N: u32;
    }

    pub struct Foo {}

    impl Trait for Foo {
        let N: u32 = 1;
    }

    fn main() {
        foo::<Foo>();
    }

    fn foo<T>()
    where
        T: Trait,
    {}
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn passes_trait_with_associated_number_to_generic_function_inside_struct_impl() {
    let src = "
    trait Trait {
        let N: u32;
    }

    pub struct Foo {}

    impl Trait for Foo {
        let N: u32 = 1;
    }

    pub struct Bar<T> {}

    impl<T> Bar<T> {
        fn bar<U>(self) where U: Trait {
            let _ = self;
        }
    }

    fn main() {
        let bar = Bar::<i32> {};
        bar.bar::<Foo>();
    }
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn returns_self_in_trait_method_1() {
    let src = "
    pub trait MagicNumber {
        fn from_magic_value() -> Self;
        fn from_value() -> Self;
    }

    pub struct Foo {}

    impl MagicNumber for Foo {
        fn from_magic_value() -> Foo {
            Self::from_value()
        }
        fn from_value() -> Self {
            Self {}
        }
    }

    pub struct Bar {}

    impl MagicNumber for Bar {
        fn from_magic_value() -> Bar {
            Self::from_value()
        }
        fn from_value() -> Self {
            Self {}
        }
    }

    fn main() {}
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn returns_self_in_trait_method_2() {
    let src = "
    pub trait MagicNumber {
        fn from_magic_value() -> Self {
            Self::from_value()
        }
        fn from_value() -> Self;
    }

    pub struct Foo {}

    impl MagicNumber for Foo {
        fn from_value() -> Self {
            Self {}
        }
    }

    pub struct Bar {}

    impl MagicNumber for Bar {
        fn from_value() -> Self {
            Self {}
        }
    }

    fn main() {}
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn returns_self_in_trait_method_3() {
    let src = "
    pub trait MagicNumber {
        fn from_magic_value() -> Self {
            Self::from_value()
        }
        fn from_value() -> Self;
    }

    impl MagicNumber for i32 {
        fn from_value() -> Self {
            0
        }
    }

    impl MagicNumber for i64 {
        fn from_value() -> Self {
            0
        }
    }

    fn main() {}
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_impl_with_where_clause_with_trait_with_associated_numeric() {
    let src = "
    trait Bar {
        let N: Field;
    }

    impl Bar for Field {
        let N: Field = 42;
    }

    trait Foo {
        fn foo<B>(b: B) where B: Bar; 
    }

    impl Foo for Field{
        fn foo<B>(_: B) where B: Bar {} 
    }

    fn main() {}
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_impl_with_where_clause_with_trait_with_associated_type() {
    let src = "
    trait Bar {
        type typ;
    }

    impl Bar for Field {
        type typ = Field;
    }

    trait Foo {
        fn foo<B>(b: B) where B: Bar; 
    }

    impl Foo for Field{
        fn foo<B>(_: B) where B: Bar {} 
    }

    fn main() {}
    ";
    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_constrained_trait_definition_has_unconstrained_impl() {
    let src = r#"
    pub trait Foo {
        fn foo() -> Field;
    }

    impl Foo for Field {
        unconstrained fn foo() -> Field {
                         ^^^ foo is not expected to be unconstrained
            42
        }
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_unconstrained_trait_definition_has_constrained_impl() {
    let src = r#"
    pub trait Foo {
        unconstrained fn foo() -> Field;
    }

    impl Foo for Field {
        fn foo() -> Field {
           ^^^ foo is expected to be unconstrained
            42
        }
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn accesses_associated_type_inside_trait_impl_using_self() {
    let src = r#"
    pub trait Trait {
        let N: u32;

        fn foo() -> u32;
    }

    impl Trait for i32 {
        let N: u32 = 10;

        fn foo() -> u32 {
            Self::N
        }
    }

    fn main() {
        let _ = i32::foo();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn accesses_associated_type_inside_trait_using_self() {
    let src = r#"
    pub trait Trait {
        let N: u32;

        fn foo() -> u32 {
            Self::N
        }
    }

    impl Trait for i32 {
        let N: u32 = 10;
    }

    fn main() {
        let _ = i32::foo();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn serialize_test_with_a_previous_unrelated_definition() {
    let src = r#"
    // There used to be a bug where this unrelated definition would cause compilation to fail
    // with a "No impl found" error.
    pub trait Trait {}

    trait Serialize {
        let Size: u32;

        fn serialize(self);
    }

    impl<A, B> Serialize for (A, B)
    where
        A: Serialize,
        B: Serialize,
    {
        let Size: u32 = <A as Serialize>::Size + <B as Serialize>::Size;

        fn serialize(self: Self) {
            self.0.serialize();
        }
    }

    impl Serialize for Field {
        let Size: u32 = 1;

        fn serialize(self) { }
    }

    fn main() {
        let x = (((1, 2), 5), 9);
        x.serialize();
    }
    "#;
    check_monomorphization_error!(&src);
}

#[named]
#[test]
fn errors_on_incorrect_generics_in_type_trait_call() {
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
    check_errors!(src);
}

#[named]
#[test]
fn trait_impl_with_child_constraint() {
    let src = r#"
    trait Parent {}

    trait Child: Parent {
        fn child() {}
    }

    pub struct Struct<T> {}

    impl<T: Parent> Parent for Struct<T> {}
    impl<T: Child> Child for Struct<T> {}

    fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn trait_with_same_generic_in_different_default_methods() {
    let src = r#"
    pub trait Trait {
        fn foo<let U: u32>(self, _msg: str<U>) { 
            let _ = self; 
        }

        fn bar<let U: u32>(self, _msg: str<U>) {
            let _ = self;
        }
    }

    pub struct Struct {}

    impl Trait for Struct {}

    pub fn main() {
        Struct {}.bar("Hello");
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn associated_constant_of_generic_type_used_in_another_associated_constant() {
    let src = r#"
    trait Serialize {
        let N: u32;

        fn serialize(self) -> [Field; N];
    }

    impl<let M: u32> Serialize for [Field; M] {
        let N: u32 = M;

        fn serialize(self) -> [Field; Self::N] {
            self
        }
    }

    struct Foo {}

    impl Serialize for Foo {
        let N: u32 = <[Field; 3] as Serialize>::N;

        fn serialize(self) -> [Field; Self::N] {
            [0; Self::N]
        }
    }

    fn main() {
        let _ = Foo {}.serialize();
    }
    "#;
    check_monomorphization_error!(src);
}

#[named]
#[test]
fn associated_constant_of_generic_type_used_in_expression() {
    let src = r#"
    trait Serialize {
        let N: u32;
    }

    impl<let M: u32> Serialize for [Field; M] {
        let N: u32 = M;
    }

    fn main() {
        let _ = <[Field; 3] as Serialize>::N;
    }
    "#;
    check_monomorphization_error!(src);
}

#[named]
#[test]
fn as_trait_path_called_multiple_times_for_different_t_1() {
    let src = r#"
    pub trait Trait {
        let N: u32;
    }
    impl Trait for Field {
        let N: u32 = 1;
    }
    impl Trait for i32 {
        let N: u32 = 999;
    }
    pub fn load<T>()
    where
        T: Trait,
    {
        let _ = <T as Trait>::N;
    }
    fn main() {
        let _ = load::<Field>();
        let _ = load::<i32>();
    }
    "#;
    check_monomorphization_error!(src);
}

#[named]
#[test]
fn as_trait_path_called_multiple_times_for_different_t_2() {
    let src = r#"
    pub trait Trait {
        let N: u32;
    }
    impl Trait for Field {
        let N: u32 = 1;
    }
    impl Trait for i32 {
        let N: u32 = 999;
    }
    pub fn load<T>()
    where
        T: Trait,
    {
        let _ = T::N;
    }
    fn main() {
        let _ = load::<Field>();
        let _ = load::<i32>();
    }
    "#;
    check_monomorphization_error!(src);
}

#[named]
#[test]
fn ambiguous_associated_type() {
    let src = r#"
    trait MyTrait {
        type X;
    }

    fn main() {
        let _: MyTrait::X = 1;
               ^^^^^^^^^^ Ambiguous associated type
               ~~~~~~~~~~ If there were a type named `Example` that implemented `MyTrait`, you could use the fully-qualified path: `<Example as MyTrait>::X`
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn as_trait_path_self_type() {
    let src = r#"
    pub trait BigCurve<B> {
        fn one() -> Self;
    }

    struct Bn254 {}

    impl<B> BigCurve<B> for Bn254 {
        fn one() -> Self { Bn254 {} }
    }

    fn main() {
        let _ = <Bn254 as BigCurve<()>>::one();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
#[test]
fn associated_constant_sum_of_other_constants() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; Self::N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    struct Gen<T> {}

    impl<T> Deserialize for Gen<T>
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + <T as Deserialize>::N;

        fn deserialize(_: [Field; Self::N]) {}
    }

    fn main() {
        let f = <Gen<Field> as Deserialize>::deserialize;
        f([0; 2]);
    }
    "#;
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn associated_constant_sum_of_other_constants_2() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + M;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X + 1]);
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn associated_constant_sum_of_other_constants_3() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + M - 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X]);
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn associated_constant_mul_of_other_constants() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N * M;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X]);
    }
    "#;
    assert_no_errors!(src);
}
