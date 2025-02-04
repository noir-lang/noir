use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::def_collector::errors::DefCollectorErrorKind;
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::import::PathResolutionError;
use crate::hir::type_check::TypeCheckError;
use crate::tests::{get_program_errors, get_program_with_maybe_parser_errors};

use super::assert_no_errors;

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

        fn main() {}
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

        fn main() {}
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

        fn main() {}
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

        fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_dependency_cycle() {
    let src = r#"
        trait Foo: Bar {}
        trait Bar: Foo {}
        fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::DependencyCycle { .. })
    ));
}

#[test]
fn trait_inheritance_missing_parent_implementation() {
    let src = r#"
        pub trait Foo {}

        pub trait Bar: Foo {}

        pub struct Struct {}

        impl Bar for Struct {}

        fn main() {
            let _ = Struct {}; // silence Struct never constructed warning
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::TraitNotImplemented {
        impl_trait,
        missing_trait: the_trait,
        type_missing_trait: typ,
        ..
    }) = &errors[0].0
    else {
        panic!("Expected a TraitNotImplemented error, got {:?}", &errors[0].0);
    };

    assert_eq!(the_trait, "Foo");
    assert_eq!(typ, "Struct");
    assert_eq!(impl_trait, "Bar");
}

#[test]
fn errors_on_unknown_type_in_trait_where_clause() {
    let src = r#"
        pub trait Foo<T> where T: Unknown {}

        fn main() {
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
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

        fn main() {}
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

        fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::TraitNotImplemented {
        impl_trait,
        missing_trait: the_trait,
        type_missing_trait: typ,
        ..
    }) = &errors[0].0
    else {
        panic!("Expected a TraitNotImplemented error, got {:?}", &errors[0].0);
    };

    assert_eq!(the_trait, "Greeter");
    assert_eq!(typ, "SomeGreeter");
    assert_eq!(impl_trait, "Foo");
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

// TODO(https://github.com/noir-lang/noir/issues/6467): currently fails with the
// same errors as the desugared version
#[test]
fn trait_alias_polymorphic_where_clause() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar<T> {
            fn bar(self) -> T;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T> = Foo + Bar<T> where T: Baz;

        fn qux<T, U>(x: T) -> bool where T: Qux<U> {
            x.foo().bar().baz()
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

        impl Baz for bool {
            fn baz(self) -> bool {
                self
            }
        }

        fn main() {
            assert(0.foo().bar().baz() == qux(0));
        }
    "#;

    // TODO(https://github.com/noir-lang/noir/issues/6467)
    // assert_no_errors(src);
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);

    match &errors[0].0 {
        CompilationError::TypeError(TypeCheckError::UnresolvedMethodCall {
            method_name, ..
        }) => {
            assert_eq!(method_name, "baz");
        }
        other => {
            panic!("expected UnresolvedMethodCall, but found {:?}", other);
        }
    }

    match &errors[1].0 {
        CompilationError::TypeError(TypeCheckError::NoMatchingImplFound(err)) => {
            assert_eq!(err.constraints.len(), 2);
            assert_eq!(err.constraints[0].1, "Baz");
            assert_eq!(err.constraints[1].1, "Qux<_>");
        }
        other => {
            panic!("expected NoMatchingImplFound, but found {:?}", other);
        }
    }
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
        }

        fn main() {}
    "#;

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
        }

        fn main() {}
    "#;

    let errors = get_program_errors(src);
    let alias_errors = get_program_errors(alias_src);

    assert_eq!(errors.len(), 1);
    assert_eq!(alias_errors.len(), 1);

    match (&errors[0].0, &alias_errors[0].0) {
        (
            CompilationError::TypeError(TypeCheckError::UnresolvedMethodCall {
                method_name,
                object_type,
                ..
            }),
            CompilationError::TypeError(TypeCheckError::UnresolvedMethodCall {
                method_name: alias_method_name,
                object_type: alias_object_type,
                ..
            }),
        ) => {
            assert_eq!(method_name, alias_method_name);
            assert_eq!(object_type, alias_object_type);
        }
        other => {
            panic!("expected UnresolvedMethodCall, but found {:?}", other);
        }
    }
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

    fn main() {}
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

        fn main() {}
    "#;

    let (_, _, errors) = get_program_with_maybe_parser_errors(
        src, true, // allow parser errors
    );
    assert!(!errors.is_empty());
}

#[test]
fn warns_if_trait_is_not_in_scope_for_function_call_and_there_is_only_one_trait_method() {
    let src = r#"
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
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::TraitMethodNotInScope { ident, trait_name },
    )) = &errors[0].0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    assert_eq!(trait_name, "private_mod::Foo");
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
    assert_no_errors(src);
}

#[test]
fn errors_if_trait_is_not_in_scope_for_function_call_and_there_are_multiple_candidates() {
    let src = r#"
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
    let mut errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::UnresolvedWithPossibleTraitsToImport { ident, mut traits },
    )) = errors.remove(0).0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    traits.sort();
    assert_eq!(traits, vec!["private_mod::Foo", "private_mod::Foo2"]);
}

#[test]
fn errors_if_multiple_trait_methods_are_in_scope_for_function_call() {
    let src = r#"
    use private_mod::Foo;
    use private_mod::Foo2;

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
    let mut errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::MultipleTraitsInScope { ident, mut traits },
    )) = errors.remove(0).0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    traits.sort();
    assert_eq!(traits, vec!["private_mod::Foo", "private_mod::Foo2"]);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_method_call_and_there_is_only_one_trait_method() {
    let src = r#"
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
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::TraitMethodNotInScope { ident, trait_name },
    )) = &errors[0].0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    assert_eq!(trait_name, "private_mod::Foo");
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
    let mut errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::UnresolvedWithPossibleTraitsToImport { ident, mut traits },
    )) = errors.remove(0).0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    traits.sort();
    assert_eq!(traits, vec!["private_mod::Foo", "private_mod::Foo2"]);
}

#[test]
fn errors_if_multiple_trait_methods_are_in_scope_for_method_call() {
    let src = r#"
    use private_mod::Foo;
    use private_mod::Foo2;

    fn main() {
        let bar = Bar { x : 42 };
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
    let mut errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::MultipleTraitsInScope { ident, mut traits },
    )) = errors.remove(0).0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    traits.sort();
    assert_eq!(traits, vec!["private_mod::Foo", "private_mod::Foo2"]);
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
                let _ = self;
                true
            }
        }

        fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::TypeMismatchWithSource {
        expected,
        actual,
        ..
    }) = &errors[0].0
    else {
        panic!("Expected a type mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(expected.to_string(), "i32");
    assert_eq!(actual.to_string(), "bool");
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

        fn main() {}
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

        fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn warns_if_trait_is_not_in_scope_for_primitive_function_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let _ = Field::foo();
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
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::TraitMethodNotInScope { ident, trait_name },
    )) = &errors[0].0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    assert_eq!(trait_name, "private_mod::Foo");
}

#[test]
fn warns_if_trait_is_not_in_scope_for_primitive_method_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let x: Field = 1;
        let _ = x.foo();
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
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::TraitMethodNotInScope { ident, trait_name },
    )) = &errors[0].0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    assert_eq!(trait_name, "private_mod::Foo");
}

#[test]
fn warns_if_trait_is_not_in_scope_for_generic_function_call_and_there_is_only_one_trait_method() {
    let src = r#"
    fn main() {
        let x: i32 = 1;
        let _ = x.foo();
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
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::TraitMethodNotInScope { ident, trait_name },
    )) = &errors[0].0
    else {
        panic!("Expected a 'trait method not in scope' error");
    };
    assert_eq!(ident.to_string(), "foo");
    assert_eq!(trait_name, "private_mod::Foo");
}

#[test]
fn error_on_duplicate_impl_with_associated_type() {
    let src = r#"
        trait Foo {
            type Bar;
        }

        impl Foo for i32 {
            type Bar = u32;
        }

        impl Foo for i32 {
            type Bar = u8;
        }

        fn main() {}
    "#;

    // Expect "Impl for type `i32` overlaps with existing impl"
    //    and "Previous impl defined here"
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);

    use CompilationError::DefinitionError;
    use DefCollectorErrorKind::*;
    assert!(matches!(&errors[0].0, DefinitionError(OverlappingImpl { .. })));
    assert!(matches!(&errors[1].0, DefinitionError(OverlappingImplNote { .. })));
}

#[test]
fn error_on_duplicate_impl_with_associated_constant() {
    let src = r#"
        trait Foo {
            let Bar: u32;
        }

        impl Foo for i32 {
            let Bar = 5;
        }

        impl Foo for i32 {
            let Bar = 6;
        }

        fn main() {}
    "#;

    // Expect "Impl for type `i32` overlaps with existing impl"
    //    and "Previous impl defined here"
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);

    use CompilationError::DefinitionError;
    use DefCollectorErrorKind::*;
    assert!(matches!(&errors[0].0, DefinitionError(OverlappingImpl { .. })));
    assert!(matches!(&errors[1].0, DefinitionError(OverlappingImplNote { .. })));
}

// See https://github.com/noir-lang/noir/issues/6530
#[test]
fn regression_6530() {
    let src = r#"
    pub trait From<T> {
        fn from(input: T) -> Self;
    }
    
    pub trait Into<T> {
        fn into(self) -> T;
    }
    
    impl<T, U> Into<T> for U
    where
        T: From<U>,
    {
        fn into(self) -> T {
            T::from(self)
        }
    }
    
    struct Foo {
        inner: Field,
    }
    
    impl Into<Field> for Foo {
        fn into(self) -> Field {
            self.inner
        }
    }
    
    fn main() {
        let foo = Foo { inner: 0 };
    
        // This works:
        let _: Field = Into::<Field>::into(foo);
    
        // This was failing with 'No matching impl':
        let _: Field = foo.into();
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 0);
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
fn calls_trait_method_using_struct_name_when_multiple_impls_exist_and_errors_turbofish() {
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
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::TypeMismatch { .. })
    ));
}
