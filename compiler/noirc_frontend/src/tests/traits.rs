use crate::{
    elaborator::FrontendOptions,
    test_utils::get_program_with_options,
    tests::{assert_no_errors, check_errors, check_monomorphization_error},
};

#[test]
fn check_trait_implemented_for_all_t() {
    let src = "
    trait Default2 {
        fn default2() -> Self;
    }

    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    trait IsDefault {
        fn is_default(self) -> bool;
    }

    impl<T> IsDefault for T where T: Default2 + Eq2 {
        fn is_default(self) -> bool {
            self.eq2(T::default2())
        }
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Default2 for u64 {
        fn default2() -> Self {
            0
        }
    }

    impl Default2 for Foo {
        fn default2() -> Self {
            Foo { a: Default2::default2() }
        }
    }

    fn main(a: Foo) -> pub bool {
        a.is_default()
    }";
    assert_no_errors(src);
}

#[test]
fn trait_impl_for_a_type_that_implements_another_trait() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            self
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    pub fn use_it<T>(t: T) -> i32 where T: Two {
        Two::two(t)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_impl_for_a_type_that_implements_another_trait_with_another_impl_used() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            let _ = self;
            1
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    impl Two for u32 {
        fn two(self) -> i32 {
            let _ = self;
            0
        }
    }

    pub fn use_it(t: u32) -> i32 {
        Two::two(t)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn check_trait_implementation_duplicate_method() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
           ~~~~~~~ First trait associated item found here
            y + 2 * x
        }
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
           ^^^^^^^ Duplicate definitions of trait associated item with name default found
           ~~~~~~~ Second trait associated item found here
            x + 2 * y
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_return_type() {
    let src = "
    trait Default2 {
        fn default() -> Self;
    }

    struct Foo {
    }

    impl Default2 for Foo {
        fn default() -> Field {
                        ^^^^^ Expected type Foo, found type Field
            0
        }
    }

    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_return_type2() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field, _y: Field) -> Field {
                                           ^^^^^ Expected type Foo, found type Field
            x
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_return_type3() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(_x: Field, _y: Field) {
                                        ^ Expected type Foo, found type ()
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_missing_implementation() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;

        fn method2(x: Field) -> Field;

    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
                      ^^^ Method `method2` from trait `Default2` is not implemented
                      ~~~ Please implement method2 here
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_not_in_scope() {
    let src = "
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
         ^^^^^^^^ Trait Default2 not found
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_name() {
    let src = "
    trait Default2 {
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn does_not_exist(x: Field, y: Field) -> Self {
           ^^^^^^^^^^^^^^ Method with name `does_not_exist` is not part of trait `Default2`, therefore it can't be implemented
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameter() {
    let src = "
    trait Default2 {
        fn default(x: Field) -> Self;
    }

    struct Foo {
        bar: u32,
    }

    impl Default2 for Foo {
        fn default(x: u32) -> Self {
                      ^^^ Parameter #1 of method `default` must be of type Field, not u32
            Foo {bar: x}
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameter2() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field, y: Foo) -> Self {
                                ^^^ Parameter #2 of method `default` must be of type Field, not Foo
            Self { bar: x, array: [x, y.bar] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameter_type() {
    let src = "
    pub trait Default2 {
        fn default(x: Field, y: NotAType) -> Field;
                                ^^^^^^^^ Could not resolve 'NotAType' in path
    }

    fn main(x: Field, y: Field) {
        assert(y == x);
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameters_count() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field) -> Self {
           ^^^^^^^ `Default2::default` expects 2 parameters, but this method has 1
            Self { bar: x, array: [x, x] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_impl_for_non_type() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    impl Default2 for main {
                      ^^^^ expected type got function
        fn default(x: Field, y: Field) -> Field {
            x + y
        }
    }

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn check_impl_struct_not_trait() {
    let src = "
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    struct Default2 {
        x: Field,
        z: Field,
    }

    impl Default2 for Foo {
         ^^^^^^^^ Default2 is not a trait, therefore it can't be implemented
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
        let _ = Default2 { x: 1, z: 1 }; // silence Default2 never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_duplicate_declaration() {
    let src = "
    trait Default2 {
          ~~~~~~~~ First trait definition found here
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field,y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    trait Default2 {
          ^^^^^^^^ Duplicate definitions of trait definition with name Default2 found
          ~~~~~~~~ Second trait definition found here
        fn default(x: Field) -> Self;
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_duplicate_implementation() {
    let src = "
    trait Default2 {
    }
    struct Foo {
        bar: Field,
    }

    impl Default2 for Foo {
         ~~~~~~~~ Previous impl defined here
    }
    impl Default2 for Foo {
                      ^^^ Impl for type `Foo` overlaps with existing impl
                      ~~~ Overlapping impl
    }
    fn main() {
        let _ = Foo { bar: 1 }; // silence Foo never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_duplicate_implementation_with_alias() {
    let src = "
    trait Default2 {
    }

    struct MyStruct {
    }

    type MyType = MyStruct;

    impl Default2 for MyStruct {
         ~~~~~~~~ Previous impl defined here
    }

    impl Default2 for MyType {
                      ^^^^^^ Impl for type `MyType` overlaps with existing impl
                      ~~~~~~ Overlapping impl
    }

    fn main() {
        let _ = MyStruct {}; // silence MyStruct never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn test_impl_self_within_default_def() {
    let src = "
    trait Bar {
        fn ok(self) -> Self;

        fn ref_ok(self) -> Self {
            self.ok()
        }
    }

    impl<T> Bar for (T, T) where T: Bar {
        fn ok(self) -> Self {
            self
        }
    }
    ";
    assert_no_errors(src);
}

#[test]
fn check_trait_as_type_as_fn_parameter() {
    let src = "
    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    fn test_eq(x: impl Eq2) -> bool {
        x.eq2(x)
    }

    fn main(a: Foo) -> pub bool {
        test_eq(a)
    }";
    assert_no_errors(src);
}

#[test]
fn check_trait_as_type_as_two_fn_parameters() {
    let src = "
    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    trait Test {
        fn test(self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Test for u64 {
        fn test(self) -> bool { self == self }
    }

    fn test_eq(x: impl Eq2, y: impl Test) -> bool {
        x.eq2(x) == y.test()
    }

    fn main(a: Foo, b: u64) -> pub bool {
        test_eq(a, b)
    }";
    assert_no_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_generics_3() {
    let src = r#"
        trait Foo<A> {}

        trait Bar<B>: Foo<B> {}

        impl Foo<i32> for () {}

        impl Bar<i32> for () {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_generics_4() {
    let src = r#"
        trait Foo { type A; }

        trait Bar<B>: Foo<A = B> {}

        impl Foo for () { type A = i32; }

        impl Bar<i32> for () {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_dependency_cycle() {
    let src = r#"
        trait Foo: Bar {}
              ^^^ Dependency cycle found
              ~~~ 'Foo' recursively depends on itself: Foo -> Bar -> Foo
        trait Bar: Foo {}
    "#;
    check_errors(src);
}

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
    check_errors(src);
}

#[test]
fn errors_on_unknown_type_in_trait_where_clause() {
    let src = r#"
        pub trait Foo<T> where T: Unknown {}
                                  ^^^^^^^ Could not resolve 'Unknown' in path

        fn main() {
        }
    "#;
    check_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    assert_no_errors(src);
}

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

    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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

    assert_no_errors(src);
}

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

    assert_no_errors(src);
}

// TODO(https://github.com/noir-lang/noir/issues/6467): currently failing, so
// this just tests that the trait alias has an equivalent error to the expected
// desugared version
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
    "#;
    check_errors(src);
}

// TODO(https://github.com/noir-lang/noir/issues/6467): currently failing, so
// this just tests that the trait alias has an equivalent error to the expected
// desugared version
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
    "#;
    check_errors(alias_src);
}

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
    "#;
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

#[test]
fn does_not_crash_on_as_trait_path_with_empty_path() {
    let src = r#"
        struct Foo {
            x: <N>,
        }
    "#;

    let allow_parser_errors = true;
    let options = FrontendOptions::test_default();
    let (_, _, errors) = get_program_with_options(src, allow_parser_errors, options);
    assert!(!errors.is_empty());
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
    "#;
    check_errors(src);
}

#[test]
fn type_checks_trait_default_method_and_does_not_error() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                let _ = self;
                1
            }
        }
    "#;
    assert_no_errors(src);
}

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
    "#;
    assert_no_errors(src);
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
    "#;
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

// See https://github.com/noir-lang/noir/issues/6530
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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    ";
    assert_no_errors(src);
}

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
    ";
    assert_no_errors(src);
}

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
    ";
    assert_no_errors(src);
}

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
    ";
    assert_no_errors(src);
}

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
    ";
    assert_no_errors(src);
}

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
    check_errors(src);
}

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
    check_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    check_monomorphization_error(src);
}

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
    check_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    check_monomorphization_error(src);
}

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
    check_monomorphization_error(src);
}

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
    check_monomorphization_error(src);
}

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
    check_monomorphization_error(src);
}

#[test]
fn short_syntax_for_trait_constraint_on_trait_generic() {
    let src = r#"
    pub trait Other {
        fn other(self) {
            let _ = self;
        }
    }

    pub trait Trait<T: Other> {
        fn foo(x: T) {
            x.other();
        }
    }

    fn main() {}
    "#;
    check_monomorphization_error(src);
}

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
    check_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

#[test]
fn trait_bound_with_associated_constant() {
    let src = r#"
    pub trait Other {
        let N: u32;
    }

    pub trait Trait<T>
    where
        T: Other,
    {}

    impl Other for Field {
        let N: u32 = 1;
    }

    impl Trait<Field> for i32 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_method_call_when_it_has_bounds_on_generic() {
    let src = r#"
    trait BigNum {}

    trait BigCurve<B>
    where
        B: BigNum,
    {
        fn new() -> Self;
    }

    pub fn foo<B: BigNum, Curve: BigCurve<B>>() {
        let _: Curve = BigCurve::new();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_constraining_two_generics() {
    let src = r#"
    pub trait Foo<U> {}

    pub trait Baz<T, U>
    where
        T: Foo<U>,
    {}

    pub struct HasFoo1 {}
    impl Foo<()> for HasFoo1 {}

    pub struct HasBaz1 {}
    impl Baz<HasFoo1, ()> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_where_clause_associated_type_constraint_expected_order() {
    let src = r#"
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        T: Foo,
        <T as Foo>::Bar: BarTrait,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_where_clause_associated_type_constraint_unexpected_order() {
    let src = r#"
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        <T as Foo>::Bar: BarTrait,
        T: Foo,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_method_numeric_generic_on_function() {
    let src = r#"
    trait Bar {
        fn baz<let N: u32>();
    }

    impl Bar for Field {
        fn baz<let N: u32>() {
            let _ = N;
        }
    }

    fn foo<K: Bar>() {
        K::baz::<2>();
    }

    fn main() {
        foo::<Field>();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn trait_bound_on_implementing_type() {
    let src = r#"
    struct GenericStruct<T> {
        inner: T,
    }

    trait Foo {
        fn foo() {}
    }

    impl Foo for Field {}

    impl<T: Foo> Foo for GenericStruct<T> {}

    trait Bar {
        fn bar();
    }

    impl<T> Bar for GenericStruct<T>
    where
        GenericStruct<T>: Foo,
    {
        fn bar() {
            <Self as Foo>::foo()
        }
    }
    
    fn main() {
        GenericStruct::<Field>::bar();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn impl_stricter_than_trait_no_trait_method_constraints() {
    // This test ensures that the error we get from the where clause on the trait impl method
    // is a `DefCollectorErrorKind::ImplIsStricterThanTrait` error.
    let src = r#"
    trait Serialize<let N: u32> {
        // We want to make sure we trigger the error when override a trait method
        // which itself has no trait constraints.
        fn serialize(self) -> [Field; N];
           ~~~~~~~~~ definition of `serialize` from trait
    }

    trait ToField {
        fn to_field(self) -> Field;
    }

    fn process_array<let N: u32>(array: [Field; N]) -> Field {
        array[0]
    }

    fn serialize_thing<A, let N: u32>(thing: A) -> [Field; N] where A: Serialize<N> {
        thing.serialize()
    }

    struct MyType<T> {
        a: T,
        b: T,
    }

    impl<T> Serialize<2> for MyType<T> {
        fn serialize(self) -> [Field; 2] where T: ToField {
                                                  ^^^^^^^ impl has stricter requirements than trait
                                                  ~~~~~~~ impl has extra requirement `T: ToField`
            [ self.a.to_field(), self.b.to_field() ]
        }
    }

    impl<T> MyType<T> {
        fn do_thing_with_serialization_with_extra_steps(self) -> Field {
            process_array(serialize_thing(self))
        }
    }

    fn main() {
        let _ = MyType { a: 1, b: 1 }; // silence MyType never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_generics() {
    let src = r#"
    trait Default2 { }

    // Object type of the trait constraint differs
    trait Foo<T> {
        fn foo_good<U>() where T: Default2;

        fn foo_bad<U>() where T: Default2;
           ~~~~~~~ definition of `foo_bad` from trait
    }

    impl<A> Foo<A> for () {
        fn foo_good<B>() where A: Default2 {}

        fn foo_bad<B>() where B: Default2 {}
                                 ^^^^^^^^ impl has stricter requirements than trait
                                 ~~~~~~~~ impl has extra requirement `B: Default2`
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_object_generics() {
    let src = r#"
    trait MyTrait { }

    trait OtherTrait {}

    struct Option2<T> {
        inner: T
    }

    struct OtherOption<T> {
        inner: Option2<T>,
    }

    trait Bar<T> {
        fn bar_good<U>() where Option2<T>: MyTrait, OtherOption<Option2<T>>: OtherTrait;

        fn bar_bad<U>() where Option2<T>: MyTrait, OtherOption<Option2<T>>: OtherTrait;
           ~~~~~~~ definition of `bar_bad` from trait

        fn array_good<U>() where [T; 8]: MyTrait;

        fn array_bad<U>() where [T; 8]: MyTrait;
           ~~~~~~~~~ definition of `array_bad` from trait

        fn tuple_good<U>() where (Option2<T>, Option2<U>): MyTrait;

        fn tuple_bad<U>() where (Option2<T>, Option2<U>): MyTrait;
           ~~~~~~~~~ definition of `tuple_bad` from trait
    }

    impl<A> Bar<A> for () {
        fn bar_good<B>()
        where
            OtherOption<Option2<A>>: OtherTrait,
            Option2<A>: MyTrait { }

        fn bar_bad<B>()
        where
            OtherOption<Option2<A>>: OtherTrait,
            Option2<B>: MyTrait { }
                        ^^^^^^^ impl has stricter requirements than trait
                        ~~~~~~~ impl has extra requirement `Option2<B>: MyTrait`

        fn array_good<B>() where [A; 8]: MyTrait { }

        fn array_bad<B>() where [B; 8]: MyTrait { }
                                        ^^^^^^^ impl has stricter requirements than trait
                                        ~~~~~~~ impl has extra requirement `[B; 8]: MyTrait`

        fn tuple_good<B>() where (Option2<A>, Option2<B>): MyTrait { }

        fn tuple_bad<B>() where (Option2<B>, Option2<A>): MyTrait { }
                                                          ^^^^^^^ impl has stricter requirements than trait
                                                          ~~~~~~~ impl has extra requirement `(Option2<B>, Option2<A>): MyTrait`
    }

    fn main() {
        let _ = OtherOption { inner: Option2 { inner: 1 } }; // silence unused warnings
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_trait() {
    let src = r#"
    trait Default2 { }

    trait OtherDefault { }

    struct Option2<T> {
        inner: T
    }

    trait Bar<T> {
        fn bar<U>() where Option2<T>: Default2;
           ~~~ definition of `bar` from trait
    }

    impl<A> Bar<A> for () {
        // Trait constraint differs due to the trait even though the constraint
        // types are the same.
        fn bar<B>() where Option2<A>: OtherDefault {}
                                      ^^^^^^^^^^^^ impl has stricter requirements than trait
                                      ~~~~~~~~~~~~ impl has extra requirement `Option2<A>: OtherDefault`
    }

    fn main() {
        let _ = Option2 { inner: 1 }; // silence Option2 never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_impl_where_clause_stricter_pass() {
    let src = r#"
    trait MyTrait {
        fn good_foo<T, H>() where H: OtherTrait;

        fn bad_foo<T, H>() where H: OtherTrait;
           ~~~~~~~ definition of `bad_foo` from trait
    }

    trait OtherTrait {}

    struct Option2<T> {
        inner: T
    }

    impl<T> MyTrait for [T] where Option2<T>: MyTrait {
        fn good_foo<A, B>() where B: OtherTrait { }

        fn bad_foo<A, B>() where A: OtherTrait { }
                                    ^^^^^^^^^^ impl has stricter requirements than trait
                                    ~~~~~~~~~~ impl has extra requirement `A: OtherTrait`
    }

    fn main() {
        let _ = Option2 { inner: 1 }; // silence Option2 never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_trait_generics() {
    let src = r#"
    trait Foo<T> {
        fn foo<U>() where T: T2<T>;
           ~~~ definition of `foo` from trait
    }

    impl<A> Foo<A> for () {
        // Should be A: T2<A>
        fn foo<B>() where A: T2<B> {}
                             ^^ impl has stricter requirements than trait
                             ~~ impl has extra requirement `A: T2<B>`
    }

    trait T2<C> {}
    "#;
    check_errors(src);
}

#[test]
fn trait_impl_generics_count_mismatch() {
    let src = r#"
    trait Foo {}

    impl Foo<()> for Field {}
         ^^^ Foo expects 0 generics but 1 was given
    "#;
    check_errors(src);
}

#[test]
fn trait_constraint_on_tuple_type() {
    let src = r#"
    trait Foo<A> {
        fn foo(self, x: A) -> bool;
    }

    pub fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
        x.foo(y)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_constraint_on_tuple_type_pub_crate() {
    let src = r#"
    pub(crate) trait Foo<A> {
        fn foo(self, x: A) -> bool;
    }

    pub fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
        x.foo(y)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn uses_self_type_inside_trait() {
    let src = r#"
    trait Foo {
        fn foo() -> Self {
            Self::bar()
        }

        fn bar() -> Self;
    }

    impl Foo for Field {
        fn bar() -> Self {
            1
        }
    }

    fn main() {
        let _: Field = Foo::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn as_trait_path_syntax_resolves_outside_impl() {
    let src = r#"
    trait Foo {
        type Assoc;
    }

    struct Bar {}

    impl Foo for Bar {
        type Assoc = i32;
    }

    fn main() {
        // AsTraitPath syntax is a bit silly when associated types
        // are explicitly specified
        let _: i64 = 1 as <Bar as Foo<Assoc = i32>>::Assoc;
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected type i64, found type i32

        let _ = Bar {}; // silence Bar never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn as_trait_path_syntax_no_impl() {
    let src = r#"
    trait Foo {
        type Assoc;
    }

    struct Bar {}

    impl Foo for Bar {
        type Assoc = i32;
    }

    fn main() {
        let _: i64 = 1 as <Bar as Foo<Assoc = i8>>::Assoc;
                                  ^^^ No matching impl found for `Bar: Foo<Assoc = i8>`
                                  ~~~ No impl for `Bar: Foo<Assoc = i8>`

        let _ = Bar {}; // silence Bar never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn uses_self_type_in_trait_where_clause() {
    let src = r#"
    pub trait Trait {
        fn trait_func(self) -> bool;
    }

    pub trait Foo where Self: Trait {
                              ~~~~~ required by this bound in `Foo`
        fn foo(self) -> bool {
            self.trait_func()
            ^^^^^^^^^^^^^^^^^ No method named 'trait_func' found for type 'Bar'
        }
    }

    struct Bar {}

    impl Foo for Bar {
                 ^^^ The trait bound `_: Trait` is not satisfied
                 ~~~ The trait `Trait` is not implemented for `_`

    }

    fn main() {
        let _ = Bar {}; // silence Bar never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_missing_associated_type() {
    let src = r#"
    trait Foo {
        type Assoc;
    }

    impl Foo for () {}
         ^^^ `Foo` is missing the associated type `Assoc`
    "#;
    check_errors(src);
}

#[test]
fn unconstrained_type_parameter_in_trait_impl() {
    let src = r#"
        pub trait Trait<T> {}
        pub struct Foo<T> {}

        impl<T, U> Trait<T> for Foo<T> {}
                ^ The type parameter `U` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `U` type parameter
        "#;
    check_errors(src);
}

#[test]
fn does_not_error_if_type_parameter_is_used_in_trait_bound_named_generic() {
    let src = r#"
    pub trait SomeTrait {}
    pub trait AnotherTrait {
        type AssocType;
    }

    impl<T, U> SomeTrait for T where T: AnotherTrait<AssocType=U> {}
    "#;
    assert_no_errors(src);
}
