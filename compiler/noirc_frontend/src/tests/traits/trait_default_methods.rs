//! Tests for default method implementations in trait definitions.
//! Validates type checking and usage of `Self` within default trait methods.

use crate::tests::{assert_no_errors, check_errors};

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

/// Regression test for <https://github.com/noir-lang/noir/issues/8632>
/// (tracked as part of <https://github.com/noir-lang/noir/issues/9020>).
///
/// A default method body must resolve paths relative to the trait's defining
/// module, not the impl's module. Here `helper` is defined in `my_trait` and
/// is not imported into the outer module; the default body must still find it.
#[test]
fn default_method_resolves_paths_in_trait_module() {
    let src = r#"
    mod my_trait {
        pub(crate) fn helper(value: Field) -> Field {
            value + 1
        }

        pub trait PartialTrait {
            fn required(self) -> Field;

            fn provided(self) -> Field {
                helper(self.required())
            }
        }
    }

    use my_trait::PartialTrait;

    pub struct Foo {}

    impl PartialTrait for Foo {
        fn required(self) -> Field {
            let _ = self;
            7
        }
    }

    fn main() {
        let f = Foo {};
        let _ = f.provided();
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for <https://github.com/noir-lang/noir/issues/9020>.
#[test]
fn default_method_type_error_reported_once() {
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

    pub struct A {}
    pub struct B {}

    impl Foo for A {}
    impl Foo for B {}

    fn main() {}
    "#;
    check_errors(src);
}

/// Multiple impls of the same trait inherit the trait's default method
/// (so they share the same `FuncId`). A dot-notation call on an explicitly-typed
/// receiver should resolve to the right impl. An ambiguous polymorphic receiver
/// should produce a "no matching impl" diagnostic (after kind-based defaulting),
/// not a panic and not "type annotations needed" for an internal type variable.
#[test]
fn shared_default_method_with_multiple_impls() {
    let src = r#"
    pub trait Identity {
        fn id(self) -> Self {
            self
        }
    }

    impl Identity for u32 {}
    impl Identity for u64 {}

    fn main() {
        // Explicit type annotations: each call resolves unambiguously.
        let _ = 2_u32.id();
        let _ = 2_u64.id();
    }
    "#;
    assert_no_errors(src);
}

/// When one of the impls is for the receiver's *default* type (`Field` for an
/// untyped integer literal), the polymorphic receiver defaults to `Field` and the
/// constraint check picks that impl — no annotation needed.
#[test]
fn shared_default_method_with_field_and_int_impls() {
    let src = r#"
    pub trait Identity {
        fn id(self) -> Self {
            self
        }
    }

    impl Identity for u32 {}
    impl Identity for Field {}

    fn main() {
        // Polymorphic literal defaults to `Field`, dispatches to the `Field` impl.
        let _ = 2.id();
        // Explicit `u32` steers to the `u32` impl.
        let _u: u32 = 2_u32.id();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn shared_default_method_with_multiple_impls_ambiguous_receiver() {
    let src = r#"
    pub trait Identity {
        fn id(self) -> Self {
            self
        }
    }

    impl Identity for u32 {}
    impl Identity for u64 {}

    fn main() {
        let _ = 2.id();
                ^^^^ No matching impl found for `Field: Identity`
                ~~~~ No impl for `Field: Identity`
    }
    "#;
    check_errors(src);
}

/// Regression test for <https://github.com/noir-lang/noir/issues/11552>.
/// A numeric generic on a generic trait must be visible in a default method body.
/// Was fixed as a side effect of #9020 (default bodies are now typed once at the
/// trait definition, so trait generics naturally flow through).
#[test]
fn generic_trait_numeric_generic_default_method() {
    let src = r#"
    trait Fillable<let N: u32> {
        fn value(self) -> Field;

        fn fill(self) -> [Field; N] {
            let mut arr = [0; N];
            let v = self.value();
            for i in 0..N {
                arr[i] = v;
            }
            arr
        }
    }

    struct Num {
        val: Field,
    }

    impl Fillable<4> for Num {
        fn value(self) -> Field {
            self.val
        }
    }

    fn main() {
        let n = Num { val: 7 };
        let arr = n.fill();
        assert(arr[0] == 7);
        assert(arr[3] == 7);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for <https://github.com/noir-lang/noir/issues/8687>.
#[test]
fn issue_8687_trait_default_method_return_type_is_trait_generic() {
    let src = r#"
    pub trait Trait<T> {
        fn one(self) -> T;

        fn foo(self) {
            let t = self.one();
            let _: i32 = t;
                         ^ Expected type i32, found type T
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

/// Regression test for <https://github.com/noir-lang/noir/issues/8687>.
#[test]
fn issue_8687_trait_default_method_return_type() {
    let src = r#"
    pub trait Trait<T> {
        fn one(self) -> T;

        fn foo(self) {
            let t = self.one();
            let _: T = t;
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn self_item_in_default_method_cannot_be_unified_with_concrete_type() {
    let src = r#"
    struct Narrow { v: Field }
    struct Wide { v: Field }

    trait Checked {
        fn make(v: Field) -> Self;
        fn validate(self) -> Field;
        fn unused_helper() -> Field {
            let w: Wide = Self::make(0);
                          ^^^^^^^^^^^^^ Expected type Wide, found type Self
            w.v
        }
        fn checked(self) -> Field {
            Checked::validate(self)
        }
    }

    impl Checked for Narrow {
        fn make(v: Field) -> Self { Narrow { v } }
        fn validate(self) -> Field { assert(self.v != 100); self.v }
    }

    impl Checked for Wide {
        fn make(v: Field) -> Self { Wide { v } }
        fn validate(self) -> Field { self.v }
    }

    fn main(w: Field) -> pub Field {
        Narrow { v: w }.checked()
    }
    "#;
    check_errors(src);
}

#[test]
fn self_item_in_default_method_has_self_type() {
    let src = r#"
    trait Mk {
        fn mk(v: Field) -> Self;
        fn val(self) -> Field;
        fn via_annotation(v: Field) -> Field {
            let s: Self = Self::mk(v);
            s.val()
        }
        fn via_method_call(v: Field) -> Field {
            Self::mk(v).val()
        }
        fn via_trait_path(v: Field) -> Field {
            let s = Self::mk(v);
            Mk::val(s)
        }
    }

    struct A { v: Field }

    impl Mk for A {
        fn mk(v: Field) -> Self { A { v } }
        fn val(self) -> Field { self.v }
    }

    impl Mk for u8 {
        fn mk(v: Field) -> Self { v as u8 }
        fn val(self) -> Field { self as Field }
    }

    fn main() {
        let _ = A::via_annotation(1) + u8::via_method_call(2) + A::via_trait_path(3);
    }
    "#;
    assert_no_errors(src);
}

// Regression tests for https://github.com/noir-lang/noir-claude/issues/2042 and
// https://github.com/noir-lang/noir-claude/issues/2061
//
// A default body sees `Self` through its method's meta (`ImplContext::of_function`), so that meta
// has to hold the trait's rigid `Self`, the same as the trait's own scope does.
//
// In the tests below `Wide` has no `Checked` impl, so naming `Wide` (or an unbounded `T`) as a
// `Checked` in a default body must be rejected. The assumed `Self: Checked` that holds inside the
// trait is about the trait's own `Self` and must not be satisfied by any other type: matching it
// would bind `Self` to that type for every default method, making `Narrow { v: w }.checked()`
// dispatch to `Wide`'s `validate` and drop `Narrow`'s assertion.

#[test]
fn trait_path_call_on_non_implementing_type_in_default_method() {
    let src = r#"
    struct Narrow { v: Field }
    struct Wide { v: Field }

    trait Other {
        fn validate(self) -> Field;
    }

    impl Other for Narrow {
        fn validate(self) -> Field { assert(self.v != 100); self.v }
    }

    impl Other for Wide {
        fn validate(self) -> Field { self.v }
    }

    trait Checked: Other {
        fn tag(self) -> Field { let _ = self; 0 }
        fn unused_helper() -> Field {
            Checked::tag(Wide { v: 0 })
            ^^^^^^^^^^^^ No matching impl found for `Wide: Checked`
            ~~~~~~~~~~~~ No impl for `Wide: Checked`
        }
        fn checked(self) -> Field { Other::validate(self) }
    }

    impl Checked for Narrow {}

    fn main(w: Field) -> pub Field {
        Narrow { v: w }.checked()
    }
    "#;
    check_errors(src);
}

#[test]
fn as_trait_path_call_on_non_implementing_type_in_default_method() {
    let src = r#"
    struct Narrow { v: Field }
    struct Wide { v: Field }

    trait Other {
        fn validate(self) -> Field;
    }

    impl Other for Narrow {
        fn validate(self) -> Field { assert(self.v != 100); self.v }
    }

    impl Other for Wide {
        fn validate(self) -> Field { self.v }
    }

    trait Checked: Other {
        fn tag(self) -> Field { let _ = self; 0 }
        fn unused_helper() -> Field {
            <Wide as Checked>::tag(Wide { v: 0 })
             ^^^^^^^^^^^^^^^ No matching impl found for `Wide: Checked`
             ~~~~~~~~~~~~~~~ No impl for `Wide: Checked`
        }
        fn checked(self) -> Field { Other::validate(self) }
    }

    impl Checked for Narrow {}

    fn main(w: Field) -> pub Field {
        Narrow { v: w }.checked()
    }
    "#;
    check_errors(src);
}

#[test]
fn bounded_generic_call_on_non_implementing_type_in_default_method() {
    let src = r#"
    struct Narrow { v: Field }
    struct Wide { v: Field }

    trait Other {
        fn validate(self) -> Field;
    }

    impl Other for Narrow {
        fn validate(self) -> Field { assert(self.v != 100); self.v }
    }

    impl Other for Wide {
        fn validate(self) -> Field { self.v }
    }

    trait Checked: Other {
        fn tag(self) -> Field { let _ = self; 0 }
        fn unused_helper() -> Field {
            g(Wide { v: 0 })
            ^ No matching impl found for `Wide: Checked`
            ~ No impl for `Wide: Checked`
        }
        fn checked(self) -> Field { Other::validate(self) }
    }

    impl Checked for Narrow {}

    fn g<T: Checked>(t: T) -> Field {
        t.tag()
    }

    fn main(w: Field) -> pub Field {
        Narrow { v: w }.checked()
    }
    "#;
    check_errors(src);
}

#[test]
fn macro_expanded_trait_path_call_on_non_implementing_type_in_default_method() {
    let src = r#"
    struct Narrow { v: Field }
    struct Wide { v: Field }

    trait Other {
        fn validate(self) -> Field;
    }

    impl Other for Narrow {
        fn validate(self) -> Field { assert(self.v != 100); self.v }
    }

    impl Other for Wide {
        fn validate(self) -> Field { self.v }
    }

    comptime fn mac() -> Quoted {
        quote { Checked::tag(Wide { v: 0 }) }
                ^^^^^^^^^^^^ No matching impl found for `Wide: Checked`
                ~~~~~~~~~~~~ No impl for `Wide: Checked`
    }

    trait Checked: Other {
        fn tag(self) -> Field { let _ = self; 0 }
        fn unused_helper() -> Field { mac!() }
        fn checked(self) -> Field { Other::validate(self) }
    }

    impl Checked for Narrow {}

    fn main(w: Field) -> pub Field {
        Narrow { v: w }.checked()
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_path_call_on_unbounded_generic_in_default_method() {
    let src = r#"
    struct Narrow { v: Field }

    trait Checked {
        fn tag(self) -> Field { let _ = self; 0 }
        fn unused_helper<T>(t: T) -> Field {
            Checked::tag(t)
            ^^^^^^^^^^^^ No matching impl found for `T: Checked`
            ~~~~~~~~~~~~ No impl for `T: Checked`
        }
    }

    impl Checked for Narrow {}

    fn main(w: Field) -> pub Field {
        Narrow { v: w }.tag()
    }
    "#;
    check_errors(src);
}

#[test]
fn explicit_self_bound_on_own_trait_in_default_method() {
    let src = r#"
    trait Checked {
        fn tag(self) -> Field;
        fn tagged(self) -> Field where Self: Checked {
            self.tag()
        }
        fn via_trait_path(self) -> Field {
            Checked::tag(self)
        }
    }

    struct A { v: Field }
    struct B { v: Field }

    impl Checked for A {
        fn tag(self) -> Field { self.v }
    }

    impl Checked for B {
        fn tag(self) -> Field { self.v + 1 }
    }

    fn main() {
        let _ = A { v: 1 }.tagged() + B { v: 2 }.tagged();
        let _ = A { v: 1 }.via_trait_path() + B { v: 2 }.via_trait_path();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn blanket_impl_of_current_trait_does_not_make_assumed_self_impl_redundant() {
    let src = r#"
    trait Tag {
        fn tag(self) -> Field {
            let _ = self;
            0
        }
        fn twice(self) -> Field {
            Tag::tag(self) + self.tag()
        }
    }

    impl<T> Tag for T {}

    struct A {}

    fn main() {
        let _ = A {}.twice() + 1.twice();
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/2080
///
/// A default method of `C: B` with `B: A` can use `A` on `Self`, through every spelling.
#[test]
fn default_method_sees_grandparent_trait_of_self() {
    let src = r#"
    trait A {
        fn a(self) -> Field {
            let _ = self;
            1
        }
    }
    trait B: A {}
    fn needs_a<T: A>(t: T) -> Field {
        t.a()
    }
    trait C: B {
        fn c(self) -> Field {
            needs_a(self) + A::a(self) + <Self as A>::a(self) + self.a()
        }
    }
    struct S {}
    impl A for S {}
    impl B for S {}
    impl C for S {}

    fn main() {
        let _ = S {}.c();
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/2080
///
/// A method-level `where Self: B` brings `B`'s parents into scope as well.
#[test]
fn default_method_sees_parents_of_where_self_bound() {
    let src = r#"
    trait A {
        fn a(self) -> Field {
            let _ = self;
            1
        }
    }
    trait B: A {}
    fn needs_a<T: A>(t: T) -> Field {
        t.a()
    }
    trait C {
        fn c(self) -> Field
        where
            Self: B,
        {
            needs_a(self)
        }
    }
    struct S {}
    impl A for S {}
    impl B for S {}
    impl C for S {}

    fn main() {
        let _ = S {}.c();
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/2080
///
/// `trait C<X>: Foo<Bar = X>` with `trait Foo { type Bar: HasQux; }` gives `X: HasQux` inside
/// `C`'s default methods.
#[test]
fn default_method_sees_associated_type_bound_of_parent() {
    let src = r#"
    trait HasQux {
        fn qux(self) -> Field;
    }
    trait Foo {
        type Bar: HasQux;
    }
    fn needs_q<T: HasQux>(t: T) -> Field {
        t.qux()
    }
    trait C<X>: Foo<Bar = X> {
        fn c(self, x: X) -> Field {
            let _ = self;
            needs_q(x)
        }
    }
    struct Q {}
    impl HasQux for Q {
        fn qux(self) -> Field {
            let _ = self;
            7
        }
    }
    struct S {}
    impl Foo for S {
        type Bar = Q;
    }
    impl C<Q> for S {}

    fn main() {
        let _ = S {}.c(Q {});
    }
    "#;
    assert_no_errors(src);
}

/// `A` reaches `C`'s default methods both directly and through `B`, and both routes name the
/// same bound, so calls through `A` are not ambiguous.
#[test]
fn default_method_sees_parent_reached_through_two_routes_once() {
    let src = r#"
    trait A {
        fn a(self) -> Field {
            let _ = self;
            1
        }
    }
    trait B: A {}
    fn needs_a<T: A>(t: T) -> Field {
        t.a()
    }
    trait C: B + A {
        fn c(self) -> Field {
            needs_a(self) + A::a(self) + <Self as A>::a(self)
        }
    }
    struct S {}
    impl A for S {}
    impl B for S {}
    impl C for S {}

    fn main() {
        let _ = S {}.c();
    }
    "#;
    assert_no_errors(src);
}
