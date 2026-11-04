//! Tests for trait implementation validation.
//! Validates duplicate impls, impl target correctness, missing associated items, and generic counts.

use iter_extended::vecmap;
use noirc_errors::CustomDiagnostic;

use crate::test_utils::get_program_with_stdlib_dependency;
use crate::tests::{assert_no_errors, check_errors};

#[test]
fn check_trait_impl_for_non_type() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    impl Default2 for main {
                      ^^^^ expected type, found function `main`
        fn default(x: Field, y: Field) -> Field {
            x + y
        }
    }

    fn main() {}
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
fn trait_impl_generics_count_mismatch() {
    let src = r#"
    trait Foo {}

    impl Foo<()> for Field {}
         ^^^ Foo expects 0 generics but 1 was given
    "#;
    check_errors(src);
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
          ~~~~~~~~ First definition found here
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
          ~~~~~~~~ Second definition found here
        fn default(x: Field) -> Self;
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
fn trait_impl_associated_type_without_body() {
    let src = "
    pub trait Trait {
        type Assoc;
    }

    impl Trait for Field {
        type Assoc;
             ^^^^^ Associated type in impl without body
             ~~~~~ Provide a definition for the type: ` = <type>;`
    }

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn trait_impl_overlap() {
    let src = r#"
    trait Trait { }

    impl<T> Trait for T { }
            ~~~~~ Previous impl defined here
    impl Trait for u32 { }
                   ^^^ Impl for type `u32` overlaps with existing impl
                   ~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_6581_impl_only() {
    let src = "
    trait Foo {
        fn foo(self) -> Self;
    }

    impl<T, let N: u32> Foo for () {
         ^ The type parameter `T` is not constrained by the impl trait, self type, or predicates
         ~ Hint: remove the `T` type parameter
                ^ The type parameter `N` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `N` type parameter
        fn foo(self) -> Self {
            ()
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_6581_using_impl_method() {
    let src = "
    trait Foo {
        fn foo(self) -> Self;
    }

    impl<T, let N: u32> Foo for () {
         ^ The type parameter `T` is not constrained by the impl trait, self type, or predicates
         ~ Hint: remove the `T` type parameter
                ^ The type parameter `N` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `N` type parameter
        fn foo(self) -> Self {
            ()
        }
    }

    fn println<T>(_x: T) {}

    fn main() {
        println(().foo());
    }
    ";
    check_errors(src);
}

#[test]
fn multiple_trait_impls_different_type_params() {
    let src = r#"
    struct Container<T> {
        value: T,
    }

    trait Convert {
        fn convert(self) -> Field;
    }

    impl Convert for Container<Field> {
        fn convert(self) -> Field {
            self.value
        }
    }

    impl Convert for Container<bool> {
        fn convert(self) -> Field {
            if self.value { 1 } else { 0 }
        }
    }

    fn main() {
        let c1 = Container { value: 42 as Field };
        let c2 = Container { value: true };
        let _ = c1.convert();
        let _ = c2.convert();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn multiple_trait_impls_numeric_and_type_params() {
    let src = r#"
    struct Buffer<let N: u32, T> {
        data: [T; N],
    }

    trait Sum {
        fn sum(self) -> Field;
    }

    impl<let N: u32> Sum for Buffer<N, Field> {
        fn sum(self) -> Field {
            let mut s: Field = 0;
            for i in 0..N {
                s += self.data[i];
            }
            s
        }
    }

    impl<let N: u32> Sum for Buffer<N, bool> {
        fn sum(self) -> Field {
            let mut s: Field = 0;
            for i in 0..N {
                if self.data[i] { s += 1; }
            }
            s
        }
    }

    fn main() {
        let b1 = Buffer { data: [1, 2, 3] };
        let b2 = Buffer { data: [true, false, true] };
        assert(b1.sum() == 6);
        assert(b2.sum() == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn specialized_impls_for_different_concrete_types() {
    let src = r#"
    trait Describe {
        fn describe(self) -> Field;
    }

    struct Tagged<T> {
        tag: Field,
        inner: T,
    }

    impl Describe for Tagged<Field> {
        fn describe(self) -> Field {
            self.tag + self.inner
        }
    }

    impl Describe for Tagged<bool> {
        fn describe(self) -> Field {
            if self.inner { self.tag } else { 0 }
        }
    }

    impl Describe for Tagged<u32> {
        fn describe(self) -> Field {
            self.tag + self.inner as Field
        }
    }

    fn main() {
        let t1 = Tagged { tag: 1, inner: 2 as Field };
        let t2 = Tagged { tag: 3, inner: true };
        let t3 = Tagged { tag: 4, inner: 5 as u32 };
        assert(t1.describe() == 3);
        assert(t2.describe() == 3);
        assert(t3.describe() == 9);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_struct_implementing_generic_trait() {
    let src = r#"
    trait Convert<U> {
        fn convert(self) -> U;
    }

    struct Wrapper<T> {
        val: T,
    }

    impl Convert<Field> for Wrapper<u32> {
        fn convert(self) -> Field {
            self.val as Field
        }
    }

    impl Convert<Field> for Wrapper<bool> {
        fn convert(self) -> Field {
            if self.val { 1 } else { 0 }
        }
    }

    fn to_field<T>(w: T) -> Field where T: Convert<Field> {
        w.convert()
    }

    fn main() {
        let w1 = Wrapper { val: 42 as u32 };
        let w2 = Wrapper { val: true };
        assert(to_field(w1) == 42);
        assert(to_field(w2) == 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn overlapping_generic_impls() {
    let src = r#"
    pub struct Foo<T> {}
    pub trait Bar {}
    impl<T> Bar for Foo<T> {}
            ~~~ Previous impl defined here
    impl<T> Bar for Foo<T> {}
                    ^^^^^^ Impl for type `Foo<T>` overlaps with existing impl
                    ~~~~~~ Overlapping impl
    "#;
    check_errors(src);
}

#[test]
fn overlapping_blanket_impl_with_generic_struct() {
    let src = r#"
    pub trait Foo {}
    pub struct Bar<T> {}
    impl<T> Foo for Bar<T> {}
            ~~~ Previous impl defined here
    impl<T> Foo for T {}
                    ^ Impl for type `T` overlaps with existing impl
                    ~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn overlapping_impls_via_different_paths_to_same_trait_and_type_in_one_module() {
    // The two impls refer to the same trait and struct via different paths
    // (imported vs. fully-qualified), but still overlap.
    let src = r#"
    pub mod module1 {
        pub(crate) trait MyTrait {}
    }
    pub mod module2 {
        pub(crate) struct MyStruct {}
    }
    pub mod module3 {
        use crate::module1::MyTrait;
        use crate::module2::MyStruct;

        impl MyTrait for MyStruct {}
             ~~~~~~~ Previous impl defined here
        impl crate::module1::MyTrait for crate::module2::MyStruct {}
                                         ^^^^^^^^^^^^^^^^^^^^^^^^ Impl for type `MyStruct` overlaps with existing impl
                                         ~~~~~~~~~~~~~~~~~~~~~~~~ Overlapping impl
    }

    fn main() {
        let _ = crate::module2::MyStruct {};
    }
    "#;
    check_errors(src);
}

#[test]
fn overlapping_impls_via_different_paths_across_modules() {
    // The overlapping impls live in separate modules but target the same trait and struct.
    let src = r#"
    pub mod module1 {
        pub(crate) trait MyTrait {}
    }
    pub mod module2 {
        pub(crate) struct MyStruct {}
    }
    pub mod module3 {
        use crate::module1::MyTrait;
        use crate::module2::MyStruct;

        impl MyTrait for MyStruct {}
             ~~~~~~~ Previous impl defined here
    }
    pub mod module4 {
        impl crate::module1::MyTrait for crate::module2::MyStruct {}
                                         ^^^^^^^^^^^^^^^^^^^^^^^^ Impl for type `MyStruct` overlaps with existing impl
                                         ~~~~~~~~~~~~~~~~~~~~~~~~ Overlapping impl
    }

    fn main() {
        let _ = crate::module2::MyStruct {};
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_item_constraint_in_impl_trait_is_an_error() {
    let src = r#"
    trait Trait {
        type T;
    }

    impl Trait<T = i32> for bool {
         ^^^^^ `Trait` is missing the associated type `T`
               ^^^^^^^ Associated item constraints are not allowed here
               ~~~~~~~ Consider removing this associated item binding
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn does_not_crash_when_trait_impl_is_defined_multiple_times() {
    let src = r#"
    pub struct Wrap { }

    trait Marker1 { 
        fn mark(); 
    } 
    trait Marker2 { 
        fn mark(); 
    }

    impl Marker1 for Wrap { 
        fn mark() { } 
    } 
    impl Marker2 for Wrap { 
         ~~~~~~~ Previous impl defined here
        fn mark() {} 
    } 
    impl Marker2 for Wrap { 
                     ^^^^ Impl for type `Wrap` overlaps with existing impl
                     ~~~~ Overlapping impl
        fn mark() {} 
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_crash_on_broken_impl_header() {
    // Regression test: a truncated `impl` header used to crash the compiler.
    let src = "impl< Foo for";
    let errors = crate::tests::get_program_errors(src);
    assert!(!errors.is_empty(), "expected the broken impl header to produce errors");
}

/// https://github.com/noir-lang/noir-claude/issues/1958
#[test]
fn impl_overlapping_a_blanket_impl_through_a_generic_is_rejected() {
    let src = r#"
    trait Small {}
    trait Checked { fn checked(self) -> u8; }
    pub struct Wrap<T> { v: T }
    impl Small for Wrap<u8> {}
    impl Small for Wrap<u16> {}
    impl<T> Checked for T where T: Small { fn checked(self) -> u8 { 0 } }
            ~~~~~~~ Previous impl defined here
    impl<U> Checked for Wrap<U> { fn checked(self) -> u8 { 1 } }
                        ^^^^^^^ Impl for type `Wrap<U>` overlaps with existing impl
                        ~~~~~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1958
#[test]
fn blanket_impls_whose_where_clauses_one_type_satisfies_overlap() {
    let src = r#"
    trait Bounded {}
    trait Audited {}
    trait Validate { fn validate(self); }
    impl<T> Validate for T where T: Bounded { fn validate(self) {} }
            ~~~~~~~~ Previous impl defined here
    impl<T> Validate for T where T: Audited { fn validate(self) { assert(false, "rejected"); } }
                         ^ Impl for type `T` overlaps with existing impl
                         ~ Overlapping impl
    pub struct Amount { v: u8 }
    impl Bounded for Amount {}
    impl Audited for Amount {}
    fn main() {}
    "#;
    check_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2741
#[test]
fn generic_impl_after_a_concrete_one_of_the_same_type_is_registered() {
    let src = r#"
    trait Other {}
    impl Other for bool {}
    trait Tr { let N: u32; }
    struct Foo<T> { x: T }
    struct Z {}
    impl Tr for Z { let N: u32 = <Foo<bool> as Tr>::N; }
    impl Tr for Foo<u32> { let N: u32 = 1; }
    impl<T> Tr for Foo<T> where T: Other { let N: u32 = 2; }
    fn main() -> pub u32 { <Z as Tr>::N }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2756
#[test]
fn impl_where_clause_projecting_through_a_later_impl_is_resolved() {
    let src = r#"
    trait Eq { fn eq(self, other: Self) -> bool; }
    trait C { type N; }
    trait D { fn d(self) -> Field; }
    pub struct W<T> { t: T }
    pub struct V<T> { t: T }
    impl<T> D for V<T> where W<T>: C, <W<T> as C>::N: Eq { fn d(self) -> Field { 1 } }
    impl<T> C for W<T> { type N = T; }
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2735
#[test]
fn impl_header_projecting_through_a_later_impl_is_resolved() {
    let src = r#"
    trait C { type T; }
    trait D { type U; }
    trait E<X> { fn get(self) -> Field; }
    struct S {}
    struct Z {}
    struct Q {}
    impl C for S { type T = <Z as D>::U; }
    impl E<<S as C>::T> for Q { fn get(self) -> Field { 1 } }
    impl D for Z { type U = u8; }
    fn main() -> pub Field { let q = Q {}; E::<u8>::get(q) }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2735
#[test]
fn impl_for_an_array_length_from_a_later_impl_is_resolved() {
    let src = r#"
    trait C { let N: u32; }
    trait D { let M: u32; }
    trait E { fn get(self) -> u32; }
    struct S {}
    struct Z {}
    impl C for S { let N: u32 = <Z as D>::M; }
    impl E for [u8; <S as C>::N] { fn get(self) -> u32 { <S as C>::N } }
    impl D for Z { let M: u32 = 4; }
    fn main() -> pub u32 { let a: [u8; 4] = [0; 4]; a.get() }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2709
#[test]
fn where_clause_on_a_concrete_type_decides_its_associated_type() {
    let src = r#"
    pub trait Tr { type Out; fn mk() -> Self::Out; }
    impl Tr for u8 { type Out = u16; fn mk() -> u16 { 7 } }
    fn f<T>() -> T where u8: Tr<Out = T> { <u8 as Tr>::mk() }
    fn main() { let _: u16 = f::<u16>(); }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1935
#[test]
fn impl_of_a_trait_whose_where_clause_names_a_later_trait() {
    let src = r#"
    trait Sz<U> where U: N2 { fn base(self) -> Field; }
    trait N2 { fn name2(self) -> Field; }
    impl N2 for u8 { fn name2(self) -> Field { 10 } }
    struct B {}
    impl Sz<u8> for B { fn base(self) -> Field { 1 } }
    fn main() -> pub Field { <B as Sz<u8>>::base(B {}) }
    "#;
    assert_no_errors(src);
}

#[test]
fn blanket_impls_whose_where_clauses_both_have_impls_overlap() {
    // No type implements both `Marker` and `Same`, but each has impls, so the impls overlap, as
    // in rustc (E0119).
    let src = r#"
    trait Same {}
    pub struct One<A> { a: A }
    pub struct Two<A> { a: A }
    impl<A, let N: u32> Same for [A; N] where A: Same {}
    impl<A> Same for [A] where A: Same {}
    impl<A> Same for One<A> where A: Same {}
    impl<A> Same for Two<A> where A: Same {}
    impl<A, B> Same for (A, B) where A: Same, B: Same {}
    impl<A, B, C> Same for (A, B, C) where A: Same, B: Same, C: Same {}
    impl Same for Field {}

    trait Marker {}
    pub struct S {}
    impl Marker for S {}

    trait Show { fn show(self); }
    impl<T> Show for T where T: Marker { fn show(self) {} }
            ~~~~ Previous impl defined here
    impl<T> Show for T where T: Same { fn show(self) {} }
                     ^ Impl for type `T` overlaps with existing impl
                     ~ Overlapping impl

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn impl_header_projecting_through_a_later_impl_for_a_type_in_a_module() {
    let src = r#"
    mod m { pub struct S {} }
    trait C { type T; }
    trait E<X> { fn get(self) -> Field; }
    struct Q {}
    impl E<<m::S as C>::T> for Q { fn get(self) -> Field { 1 } }
    impl C for m::S { type T = u8; }
    fn main() -> pub Field { let q = Q {}; E::<u8>::get(q) }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2835: a bound on an unknown type may hold even
/// with no impl of its trait here, since a dependent crate may implement it for its own type, so
/// two blanket impls kept apart only by such bounds overlap, as in rustc (E0119).
#[test]
fn blanket_impls_whose_bounds_have_no_impls_yet_overlap() {
    let src = r#"
    pub trait Bounded {}
    pub trait Audited {}
    pub trait Validate { fn validate(self) -> u32; }
    impl<T> Validate for T where T: Bounded { fn validate(self) -> u32 { 0 } }
            ~~~~~~~~ Previous impl defined here
    impl<T> Validate for T where T: Audited { fn validate(self) -> u32 { 1 } }
                         ^ Impl for type `T` overlaps with existing impl
                         ~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn blanket_impls_where_one_bound_has_an_impl_for_another_type_overlap() {
    let src = r#"
    pub trait Bounded {}
    pub trait Audited {}
    impl Bounded for u8 {}
    pub trait Validate { fn validate(self) -> u32; }
    impl<T> Validate for T where T: Bounded { fn validate(self) -> u32 { 0 } }
            ~~~~~~~~ Previous impl defined here
    impl<T> Validate for T where T: Audited { fn validate(self) -> u32 { 1 } }
                         ^ Impl for type `T` overlaps with existing impl
                         ~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn trait_where_clause_naming_a_trait_declared_before_it_is_accepted() {
    let src = r#"
    pub trait B {}
    pub trait A<U> where U: B {}
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2835: the overlap is reported in the library
/// that declares the impls, before a dependent crate implements both bounds for its own type.
#[test]
fn blanket_impls_in_a_library_overlap_before_a_dependent_crate_implements_their_bounds() {
    let library = r#"
    pub trait Bounded {}
    pub trait Audited {}
    pub trait Validate { fn validate(self) -> u32; }
    impl<T> Validate for T where T: Bounded { fn validate(self) -> u32 { 0 } }
    impl<T> Validate for T where T: Audited { fn validate(self) -> u32 { assert(false); 1 } }
    pub fn check_bounded<T>(t: T) -> u32 where T: Bounded { Validate::validate(t) }
    "#;
    let application = r#"
    use std::{Bounded, Audited};
    pub struct Amount { v: u32 }
    impl Bounded for Amount {}
    impl Audited for Amount {}
    fn main(x: u32) -> pub u32 { let a = Amount { v: x }; std::check_bounded(a) + a.v }
    "#;
    let errors = get_program_with_stdlib_dependency(library, application);
    let messages = vecmap(&errors, |error| CustomDiagnostic::from(error).message);
    assert!(
        messages.iter().any(|message| message.contains("overlaps with existing impl")),
        "expected an overlap error, got {messages:?}"
    );
}

/// A bound served only by an impl whose where clause can't hold is known to fail, so the impls it
/// keeps apart don't overlap, as in rustc: `u32: One` needs `u32: Zero`, which has no impl.
#[test]
fn bound_served_only_by_an_impl_whose_where_clause_fails_keeps_impls_apart() {
    let src = r#"
    pub trait Zero {}
    pub trait One {}
    pub trait Two {}
    impl<U> One for U where U: Zero {}
    impl<T> Two for T where T: One {}
    impl Two for u32 {}
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// A bound served only by an impl for all types decides nothing more, so the overlap check ends.
#[test]
fn bound_served_only_by_an_impl_for_all_types_overlaps() {
    let src = r#"
    pub trait One {}
    pub trait Two {}
    impl<U> One for U {}
    impl<T> Two for T where T: One {}
            ~~~ Previous impl defined here
                               ^^^ Constraint for `T: One` is not needed, another matching impl is already in scope
                               ~~~ Unnecessary trait constraint in where clause
    impl Two for u32 {}
                 ^^^ Impl for type `u32` overlaps with existing impl
                 ~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

/// A projection in the arguments of a where-clause bound, or of an inline bound, also makes the
/// impl wait for the impls it projects through.
#[test]
fn impl_whose_bound_arguments_project_through_a_later_impl_is_resolved() {
    let src = r#"
    pub trait C { type X; }
    pub trait Foo<Y> {}
    pub trait Bar {}
    pub trait Baz {}
    pub struct S {}
    impl<T> Bar for T where T: Foo<<S as C>::X> {}
    impl<T: Foo<<S as C>::X>> Baz for [T; 1] {}
    impl C for S { type X = u8; }
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// A type a single-impl bound decides is seen through every unknown it was unified with, so the
/// overlap verdict doesn't depend on which impl comes first, as in rustc.
#[test]
fn type_decided_through_unified_unknowns_keeps_impls_apart_in_either_order() {
    let impls =
        ["impl<T> Foo for S<T> where S<T>: Other {}", "impl<A> Foo for S<A> where A: Marker {}"];
    for (first, second) in [(impls[0], impls[1]), (impls[1], impls[0])] {
        let src = format!(
            r#"
    pub trait Marker {{}}
    pub trait Other {{}}
    pub trait Foo {{}}
    pub struct S<A> {{ x: A }}
    {first}
    {second}
    impl Other for S<Field> {{}}
    fn main() {{}}
    "#
        );
        assert_no_errors(&src);
    }
}

/// A single-impl bound that makes an unknown fully known (`?u := Field` inside `S<W<?u>>`) after
/// another gave it a known head is progress, so a third bound can be known to fail
/// (`S<W<Field>>: Marker`) and keep the impls apart, as in rustc.
#[test]
fn unknown_made_fully_known_by_a_second_bound_keeps_impls_apart() {
    let src = r#"
    pub trait Marker {}
    pub trait One {}
    pub trait Two {}
    pub trait Foo {}
    pub struct S<A> { x: A }
    pub struct W<A> { x: A }
    impl<A> Foo for S<A> where S<A>: Marker {}
    impl<T> Foo for S<T> where S<T>: One, S<T>: Two {}
    impl<U> One for S<W<U>> {}
    impl Two for S<W<Field>> {}
    impl Marker for S<W<u8>> {}
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// Impls serving bounds with fresh unknowns each time (`impl<U> Other for S<W<U>>`,
/// `impl<U> Marker for W<U>`) decide each bound once, so the overlap check ends, in either order
/// of the impls. `S<W<X>>` meets both where clauses, so the impls overlap.
#[test]
fn bounds_served_by_generic_impls_are_decided_once_in_either_order() {
    let src = r#"
    pub trait Marker {}
    pub trait Other {}
    pub trait Foo {}
    pub struct S<A> { x: A }
    pub struct W<A> { x: A }
    impl<T> Foo for S<T> where S<T>: Other {}
            ~~~ Previous impl defined here
    impl<A> Foo for S<A> where A: Marker {}
                    ^^^^ Impl for type `S<A>` overlaps with existing impl
                    ~~~~ Overlapping impl
    impl<U> Other for S<W<U>> {}
    impl<U> Marker for W<U> {}
    fn main() {}
    "#;
    check_errors(src);

    let src = r#"
    pub trait Marker {}
    pub trait Other {}
    pub trait Foo {}
    pub struct S<A> { x: A }
    pub struct W<A> { x: A }
    impl<A> Foo for S<A> where A: Marker {}
            ~~~ Previous impl defined here
    impl<T> Foo for S<T> where S<T>: Other {}
                    ^^^^ Impl for type `S<T>` overlaps with existing impl
                    ~~~~ Overlapping impl
    impl<U> Other for S<W<U>> {}
    impl<U> Marker for W<U> {}
    fn main() {}
    "#;
    check_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2845: a where bound's associated type must
/// match the serving impl's, so `u16: Foo<Out = Field>` fails when `u16`'s `Out` is `u32`, and the
/// impls are disjoint, in either order.
#[test]
fn associated_type_bound_that_fails_for_a_type_keeps_impls_apart_in_either_order() {
    let impls = [
        "impl<T> Tr for S<T> where T: Foo<Out = Field> { fn get(self) -> Field { 5 } }",
        "impl Tr for S<u16> { fn get(self) -> Field { 7 } }",
    ];
    for (first, second) in [(impls[0], impls[1]), (impls[1], impls[0])] {
        let src = format!(
            r#"
    pub trait Foo {{ type Out; }}
    impl Foo for u8 {{ type Out = Field; }}
    impl Foo for u16 {{ type Out = u32; }}
    pub trait Tr {{ fn get(self) -> Field; }}
    pub struct S<A> {{ a: A }}
    {first}
    {second}
    fn main() -> pub (Field, Field) {{ (S {{ a: 1 as u8 }}.get(), S {{ a: 1 as u16 }}.get()) }}
    "#
        );
        assert_no_errors(&src);
    }
}

/// An associated type bound on another generic of the impl (`T: Foo<Out = U>`) is known to fail
/// once both are decided.
#[test]
fn associated_type_bound_naming_another_generic_keeps_impls_apart() {
    let src = r#"
    pub trait Foo { type Out; }
    impl Foo for u8 { type Out = Field; }
    impl Foo for u16 { type Out = u32; }
    pub trait Tr { fn get(self) -> Field; }
    pub struct S<A, B> { b: B }
    impl<T, U> Tr for S<T, U> where T: Foo<Out = U> { fn get(self) -> Field { 5 } }
    impl Tr for S<u16, Field> { fn get(self) -> Field { 7 } }
    fn g<C>(c: C) -> Field where C: Tr { c.get() }
    fn main() -> pub (Field, Field) { (g(S::<u8, Field> { b: 1 }), g(S::<u16, Field> { b: 1 })) }
    "#;
    assert_no_errors(src);
}

/// A bound whose associated type matches the impl's (`u16`'s `Out` is `Field`) holds, so the impls
/// overlap.
#[test]
fn associated_type_bound_that_holds_for_a_type_overlaps() {
    let src = r#"
    pub trait Foo { type Out; }
    impl Foo for u16 { type Out = Field; }
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl<T> Tr for S<T> where T: Foo<Out = Field> {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

/// A bound served only by an impl whose own where clause has a failing associated type bound is
/// known to fail.
#[test]
fn bound_served_only_by_an_impl_with_a_failing_associated_type_bound_keeps_impls_apart() {
    let src = r#"
    pub trait Foo { type Out; }
    impl Foo for u16 { type Out = u32; }
    pub struct W<T> { t: T }
    pub trait Mid {}
    impl<T> Mid for W<T> where T: Foo<Out = Field> {}
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl<T> Tr for S<T> where W<T>: Mid {}
    impl Tr for S<u16> {}
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// An impl collected after the impls being compared still serves the bound (`u16`'s `Out` is
/// `Field`), so the impls overlap.
#[test]
fn associated_type_bound_served_by_a_later_impl_overlaps() {
    let src = r#"
    pub trait Foo { type Out; }
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl<T> Tr for S<T> where T: Foo<Out = Field> {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    impl Foo for u16 { type Out = Field; }
    fn main() {}
    "#;
    check_errors(src);
}

/// A bound that leaves an associated type out puts no requirement on it, so the impls overlap.
#[test]
fn bound_leaving_out_an_associated_type_overlaps() {
    let src = r#"
    pub trait Foo { type Out; }
    impl Foo for u16 { type Out = u32; }
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl<T> Tr for S<T> where T: Foo {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

/// A bound on an associated type of an impl collected later (`<u16 as Foo>::Out: Bar`) holds, so
/// the impls overlap.
#[test]
fn bound_on_an_associated_type_of_a_later_impl_overlaps() {
    let src = r#"
    pub trait Foo { type Out; }
    pub trait Bar {}
    impl Bar for Field {}
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl<T> Tr for S<T> where T: Foo, <T as Foo>::Out: Bar {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    impl Foo for u16 { type Out = Field; }
    fn main() {}
    "#;
    check_errors(src);
}

/// An associated type given as another associated type of the same impl
/// (`type Out = Self::Other`), and read by an impl header before that impl is collected, is that
/// type (`Field`), so the impls overlap.
#[test]
fn associated_type_resolved_to_a_later_associated_type_overlaps() {
    let src = r#"
    pub trait Foo { type Out; type Other; }
    pub trait Marker {}
    pub trait Tr {}
    pub struct S<A> { a: A }
    pub struct W<A> { a: A }
    impl Marker for W<<u16 as Foo>::Out> {}
    impl<T> Tr for S<T> where T: Foo<Out = Field> {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    impl Foo for u16 { type Out = Self::Other; type Other = Field; }
    fn main() {}
    "#;
    check_errors(src);
}

/// The same with the other associated type nested in it (`type Out = W<Self::Other>`).
#[test]
fn associated_type_containing_a_later_associated_type_overlaps() {
    let src = r#"
    pub trait Foo { type Out; type Other; }
    pub trait Marker {}
    pub trait Tr {}
    pub struct S<A> { a: A }
    pub struct W<A> { a: A }
    impl Marker for W<<u16 as Foo>::Out> {}
    impl<T> Tr for S<T> where T: Foo<Out = W<Field>> {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    impl Foo for u16 { type Out = W<Self::Other>; type Other = Field; }
    fn main() {}
    "#;
    check_errors(src);
}

/// The impls of `Foo` come after the impls being compared, and still keep them apart.
#[test]
fn associated_type_bound_that_fails_for_a_later_impl_keeps_impls_apart() {
    let src = r#"
    pub trait Foo { type Out; }
    pub trait Tr { fn get(self) -> Field; }
    pub struct S<A> { a: A }
    impl<T> Tr for S<T> where T: Foo<Out = Field> { fn get(self) -> Field { 5 } }
    impl Tr for S<u16> { fn get(self) -> Field { 7 } }
    impl Foo for u8 { type Out = Field; }
    impl Foo for u16 { type Out = u32; }
    fn main() -> pub (Field, Field) { (S { a: 1 as u8 }.get(), S { a: 1 as u16 }.get()) }
    "#;
    assert_no_errors(src);
}

/// An associated type in a where bound that is an associated type of an impl collected later
/// (`<u8 as Bar>::B`, with `type B = Self::C`) is that type (`Field`), so the impls overlap.
#[test]
fn bound_whose_associated_type_names_a_later_impls_associated_type_overlaps() {
    let src = r#"
    pub trait Foo { type Out; }
    pub trait Bar { type B; type C; }
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl Foo for u16 { type Out = Field; }
    impl<T> Tr for S<T> where T: Foo<Out = <u8 as Bar>::B> {}
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    impl Bar for u8 { type B = Self::C; type C = Field; }
    fn main() {}
    "#;
    check_errors(src);
}

/// The same with the later impl's associated type as the bound's type (`<u8 as Bar>::B: Foo`).
#[test]
fn bound_on_a_later_impls_associated_type_overlaps() {
    let src = r#"
    pub trait Foo {}
    pub trait Bar { type B; type C; }
    pub trait Tr {}
    pub struct S<A> { a: A }
    impl Foo for Field {}
    impl<T> Tr for S<T> where <u8 as Bar>::B: Foo {}
                                              ^^^ Constraint for `Field: Foo` is not needed, another matching impl is already in scope
                                              ~~~ Unnecessary trait constraint in where clause
            ~~ Previous impl defined here
    impl Tr for S<u16> {}
                ^^^^^^ Impl for type `S<u16>` overlaps with existing impl
                ~~~~~~ Overlapping impl
    impl Bar for u8 { type B = Self::C; type C = Field; }
    fn main() {}
    "#;
    check_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2840: a where bound that only the impl being
/// checked serves (`Opt<T>: Validate`, through the blanket impl) holds, so the impls overlap
/// in either order.
#[test]
fn bound_served_only_by_the_impl_being_checked_overlaps_in_either_order() {
    let src = r#"
    pub trait Small {}
    pub trait Validate { fn validate(self) -> u32; }
    pub struct Wrap<T> { v: T }
    pub struct Opt<T> { v: T }
    impl Small for Wrap<u32> {}
    impl Small for Opt<u32> {}
    impl<T> Validate for Wrap<T> where Opt<T>: Validate { fn validate(self) -> u32 { 1 } }
            ~~~~~~~~ Previous impl defined here
    impl<T> Validate for T where T: Small { fn validate(self) -> u32 { 0 } }
                         ^ Impl for type `T` overlaps with existing impl
                         ~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);

    let src = r#"
    pub trait Small {}
    pub trait Validate { fn validate(self) -> u32; }
    pub struct Wrap<T> { v: T }
    pub struct Opt<T> { v: T }
    impl Small for Wrap<u32> {}
    impl Small for Opt<u32> {}
    impl<T> Validate for T where T: Small { fn validate(self) -> u32 { 0 } }
            ~~~~~~~~ Previous impl defined here
    impl<T> Validate for Wrap<T> where Opt<T>: Validate { fn validate(self) -> u32 { 1 } }
                         ^^^^^^^ Impl for type `Wrap<T>` overlaps with existing impl
                         ~~~~~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}
