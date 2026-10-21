//! The expected behavior of every open trait-resolution issue in noir-lang/noir-claude, one test
//! per issue (or per program of an issue). Each test states the correct outcome, matching Rust's
//! where Rust has the same program.
//!
//! A test for an issue that isn't fixed yet is `#[ignore]`d with the issue as the reason, so
//! `cargo test -p noirc_frontend --lib known_issues -- --include-ignored` shows which issues are
//! still open. Once a change makes such a test pass, its `#[ignore]` is removed and it stays as a
//! regression test.

use noirc_errors::CustomDiagnostic;

use crate::elaborator::UnstableFeature;
use crate::test_utils::get_monomorphized;
use crate::tests::{get_program_errors, get_program_using_features};

/// The messages of the errors (not warnings) the frontend reports for `src`.
fn errors(src: &str) -> Vec<String> {
    get_program_errors(src)
        .iter()
        .map(CustomDiagnostic::from)
        .filter(CustomDiagnostic::is_error)
        .map(|diagnostic| diagnostic.message)
        .collect()
}

fn errors_using_features(src: &str, features: &[UnstableFeature]) -> Vec<String> {
    get_program_using_features(src, features)
        .2
        .iter()
        .map(CustomDiagnostic::from)
        .filter(CustomDiagnostic::is_error)
        .map(|diagnostic| diagnostic.message)
        .collect()
}

fn assert_accepted(src: &str) {
    let errors = errors(src);
    assert!(errors.is_empty(), "expected the program to be accepted, got {errors:?}");
}

fn assert_rejected(src: &str) {
    assert!(!errors(src).is_empty(), "expected the program to be rejected");
}

/// The program is rejected with an error whose message contains `message`.
fn assert_rejected_with(src: &str, message: &str) {
    let errors = errors(src);
    assert!(
        errors.iter().any(|error| error.contains(message)),
        "expected an error containing `{message}`, got {errors:?}"
    );
}

/// The monomorphized program, which must have no frontend errors.
fn monomorphized(src: &str) -> String {
    match get_monomorphized(src) {
        Ok(program) => program.to_string(),
        Err(error) => panic!("expected the program to monomorphize, got {error:?}"),
    }
}

/// The program compiles, and its monomorphized form contains every one of `present` and none of
/// `absent`: the bodies that are expected to run, and the ones that are expected not to.
fn assert_runs(src: &str, present: &[&str], absent: &[&str]) {
    let program = monomorphized(src);
    for text in present {
        assert!(program.contains(text), "expected `{text}` in the program:\n{program}");
    }
    for text in absent {
        assert!(!program.contains(text), "expected no `{text}` in the program:\n{program}");
    }
}

/// The program is either rejected with an error or compiles, but never crashes the compiler.
fn assert_rejected_or_compiles(src: &str) {
    if errors(src).is_empty() {
        monomorphized(src);
    }
}

// --- A. Method and path candidate selection ---------------------------------------------------

/// https://github.com/noir-lang/noir-claude/issues/2742
#[test]
fn i2742_method_call_uses_the_enclosing_where_clause() {
    let src = r#"
    trait A {}
    trait Foo { fn foo(self) -> Field; }
    pub struct W<T> { v: Field }
    impl A for u8 {}
    impl<T> Foo for W<T> where T: A { fn foo(self) -> Field { self.v } }
    fn g<T>(w: W<T>) -> Field where W<T>: Foo { w.foo() }
    fn main(x: Field) { assert(g(W::<u8> { v: x }) == x); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2742
#[test]
fn i2742_method_call_under_a_where_clause_runs_each_instantiations_impl() {
    let src = r#"
    trait A { fn a() -> Field; }
    trait Foo { fn foo(self) -> Field; }
    pub struct W<T> { v: Field }
    impl A for u8 { fn a() -> Field { 1 } }
    impl<T> Foo for W<T> where T: A { fn foo(self) -> Field { 100 + T::a() + self.v } }
    impl Foo for W<u16> { fn foo(self) -> Field { 200 + self.v } }
    fn gen_where<T>(w: W<T>) -> Field where W<T>: Foo { w.foo() }
    fn main(x: Field) -> pub (Field, Field) {
        (gen_where(W::<u16> { v: x }), gen_where(W::<u8> { v: x }))
    }
    "#;
    assert_runs(src, &["100", "200"], &[]);
}

const OWN_AND_BLANKET_LIMIT: &str = r#"
    trait Small {}
    trait Limit { fn max() -> u64; }
    pub struct S<T> {}
    impl Small for S<u8> {}
    impl Small for S<u16> {}
    impl Limit for S<u64> { fn max() -> u64 { 18446744073709551615 } }
    impl<T: Small> Limit for T { fn max() -> u64 { 255 } }
    "#;

/// https://github.com/noir-lang/noir-claude/issues/2792 (e11)
#[test]
fn i2792_type_path_call_on_an_unknown_generic_uses_the_where_clause() {
    let src = format!(
        "{OWN_AND_BLANKET_LIMIT}
        fn f<T>() -> u64 where S<T>: Limit {{ S::max() }}
        fn main() -> pub u64 {{ f::<u8>() }}"
    );
    assert_runs(&src, &["255"], &["18446744073709551615"]);
}

/// https://github.com/noir-lang/noir-claude/issues/2792 (e0)
#[test]
fn i2792_self_path_call_in_a_generic_impl_uses_the_where_clause() {
    let src = format!(
        "{OWN_AND_BLANKET_LIMIT}
        trait Validate {{ fn limit() -> u64; }}
        impl<T> Validate for S<T> where S<T>: Limit {{ fn limit() -> u64 {{ Self::max() }} }}
        fn main() -> pub u64 {{ S::<u8>::limit() }}"
    );
    assert_runs(&src, &["255"], &["18446744073709551615"]);
}

/// https://github.com/noir-lang/noir-claude/issues/2792 (e12), rustc: E0283
#[test]
fn i2792_type_path_call_that_several_impls_can_serve_is_ambiguous() {
    let src = format!("{OWN_AND_BLANKET_LIMIT}\nfn main() -> pub u64 {{ S::max() }}");
    assert_rejected(&src);
}

const BOUND_AND_CHECKER: &str = r#"
    trait Bound { fn max() -> u64; }
    impl Bound for u8 { fn max() -> u64 { 255 } }
    impl Bound for u64 { fn max() -> u64 { 18446744073709551615 } }
    struct W<T> {}
    trait Checker<T> { fn check(tag: T, v: u64); }
    impl<T: Bound> Checker<T> for W<T> {
        fn check(_tag: T, v: u64) { assert(v <= T::max(), "out of range"); }
    }
    "#;

/// https://github.com/noir-lang/noir-claude/issues/1308: `Self` is `W<T>`, so `check` takes a `T`.
#[test]
fn i1308_self_path_call_in_a_trait_impl_is_typed_by_the_impls_self() {
    let src = format!(
        "{BOUND_AND_CHECKER}
        trait Api {{ fn validate(v: u64); }}
        impl<T: Bound> Api for W<T> {{ fn validate(v: u64) {{ Self::check(v, v) }} }}
        fn main(v: u64) {{ W::<u8>::validate(v); }}"
    );
    assert_rejected(&src);
}

/// https://github.com/noir-lang/noir-claude/issues/1308
#[test]
fn i1308_self_path_call_in_an_inherent_impl_is_typed_by_the_impls_self() {
    let src = format!(
        "{BOUND_AND_CHECKER}
        impl<T: Bound> W<T> {{ fn validate(v: u64) {{ Self::check(v, v) }} }}
        fn main(v: u64) {{ W::<u8>::validate(v); }}"
    );
    assert_rejected(&src);
}

/// https://github.com/noir-lang/noir-claude/issues/1308
#[test]
fn i1308_self_path_call_in_one_of_several_impls_uses_that_impl() {
    let src = r#"
    struct S<T> { x: T }
    trait DefaultTag { fn static_tag() -> Field; fn impl_tag() -> Field; }
    impl DefaultTag for S<u32> {
        fn static_tag() -> Field { Self::impl_tag() }
        fn impl_tag() -> Field { 132 }
    }
    impl DefaultTag for S<u64> {
        fn static_tag() -> Field { Self::impl_tag() }
        fn impl_tag() -> Field { 164 }
    }
    fn main() -> pub (Field, Field) { (S::<u32>::static_tag(), S::<u64>::static_tag()) }
    "#;
    assert_runs(src, &["132", "164"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/1950
#[test]
fn i1950_method_call_that_several_inherent_impls_can_serve_is_ambiguous() {
    let src = r#"
    pub struct S<T> { v: T }
    impl S<u8> { pub fn m(self) -> Field { let _ = self; 8 } }
    impl S<u16> { pub fn m(self) -> Field { let _ = self; 16 } }
    fn main() -> pub Field { let s = S { v: 5 }; s.m() }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2801: `Lib`'s `check` belongs to `u16`.
#[test]
fn i2801_method_call_ignores_a_trait_method_of_another_types_impl() {
    let src = r#"
    trait Check { fn check(self) -> bool; }
    impl<T> Check for T { fn check(self) -> bool { false } }
    trait Lib { fn check(x: u8) -> bool; }
    impl Lib for u16 { fn check(_x: u8) -> bool { true } }
    fn main(a: u8) -> pub bool { a.check() }
    "#;
    assert_runs(src, &["false"], &["true"]);
}

/// https://github.com/noir-lang/noir-claude/issues/2801 (the prelude's `From<u8> for u16`)
#[test]
fn i2801_method_call_ignores_a_static_function_of_another_types_impl() {
    let src = r#"
    trait From<T> { fn from(input: T) -> Self; }
    impl From<u8> for u16 { fn from(input: u8) -> u16 { input as u16 } }
    trait Conv { fn from(self) -> u8; }
    impl<T> Conv for T { fn from(self) -> u8 { 7 } }
    fn main(a: u8) -> pub u8 { let b: u8 = a.from(); b }
    "#;
    assert_runs(src, &["7"], &["as u16"]);
}

/// https://github.com/noir-lang/noir-claude/issues/2801 (inherent route)
#[test]
fn i2801_method_call_ignores_an_inherent_function_of_another_instantiation() {
    let src = r#"
    pub struct W<T> { x: T }
    impl W<u8> { pub fn m(_x: W<u16>) -> u32 { 111 } }
    trait Tr { fn m(self) -> u32; }
    impl Tr for W<u16> { fn m(self) -> u32 { 222 } }
    fn main() -> pub u32 { W::<u16> { x: 0 }.m() }
    "#;
    assert_runs(src, &["222"], &["111"]);
}

/// https://github.com/noir-lang/noir-claude/issues/2802, rustc: E0034
#[test]
fn i2802_method_of_a_blanket_impl_and_a_types_impl_of_another_trait_is_ambiguous() {
    let src = r#"
    trait Check { fn check(self) -> bool; }
    impl<T> Check for T { fn check(self) -> bool { false } }
    trait Lib { fn check(self) -> bool; }
    impl Lib for u8 { fn check(self) -> bool { true } }
    fn main(a: u8) -> pub bool { a.check() }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2802
#[test]
fn i2802_method_of_a_trait_out_of_scope_does_not_hide_a_blanket_impl_in_scope() {
    let src = r#"
    mod lib {
        pub trait Lib { fn check(self) -> bool; }
        impl Lib for u8 { fn check(self) -> bool { true } }
    }
    trait Check { fn check(self) -> bool; }
    impl<T> Check for T { fn check(self) -> bool { false } }
    fn main(a: u8) -> pub bool { a.check() }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2803 (w1): `L<T>`'s `check` needs `T: LX`.
#[test]
fn i2803_method_call_ignores_an_inherent_method_whose_where_clause_fails() {
    let src = r#"
    pub struct L<T> { pub v: u32 }
    pub trait LX {}
    impl<T: LX> L<T> { pub fn check(_self: Self) -> bool { true } }
    trait Check { fn check(self) -> bool; }
    impl Check for L<u64> { fn check(self) -> bool { self.v < 1000 } }
    fn main(x: u32) -> pub bool { let a = L::<u64> { v: x }; a.check() }
    "#;
    assert_runs(src, &["1000"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/2803 (w2)
#[test]
fn i2803_method_call_ignores_a_trait_impl_whose_where_clause_fails() {
    let src = r#"
    pub struct L<T> { pub v: u32 }
    pub trait LX {}
    trait Other { fn check(self) -> bool; }
    impl<T: LX> Other for L<T> { fn check(self) -> bool { true } }
    trait Check { fn check(self) -> bool; }
    impl Check for L<u64> { fn check(self) -> bool { self.v < 1000 } }
    fn main(x: u32) -> pub bool { let a = L::<u64> { v: x }; a.check() }
    "#;
    assert_runs(src, &["1000"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/2803 (w3)
#[test]
fn i2803_method_call_ignores_a_blanket_impl_whose_where_clause_fails() {
    let src = r#"
    pub trait LX {}
    trait Eq { fn eq(self, other: Self) -> bool; }
    impl Eq for u32 { fn eq(self, other: u32) -> bool { self == other } }
    trait Other { fn check(self) -> bool; }
    impl<T: LX> Other for T { fn check(self) -> bool { true } }
    trait Check { fn check(self) -> bool; }
    impl<T> Check for T where T: Eq { fn check(self) -> bool { false } }
    fn main(x: u32) -> pub bool { x.check() }
    "#;
    assert_runs(src, &["false"], &["true"]);
}

const INTO_AND_FROM: &str = r#"
    trait From<T> { fn from(input: T) -> Self; }
    impl<T> From<T> for T { fn from(input: T) -> T { input } }
    trait Into<T> { fn into(self) -> T; }
    impl<T, U> Into<T> for U where T: From<U> { fn into(self) -> T { T::from(self) } }
    "#;

/// https://github.com/noir-lang/noir-claude/issues/2804
#[test]
fn i2804_method_call_on_a_generic_finds_a_blanket_impl() {
    let src = format!(
        "{INTO_AND_FROM}
        fn g<T>(t: T) -> T {{ t.into() }}
        fn main(x: Field) -> pub Field {{ g(x) }}"
    );
    assert_accepted(&src);
}

/// https://github.com/noir-lang/noir-claude/issues/2804
#[test]
fn i2804_method_call_on_a_bounded_generic_finds_a_blanket_impl() {
    let src = r#"
    trait Base { fn base(self) -> Field; }
    impl Base for Field { fn base(self) -> Field { self } }
    trait Ext { fn ext(self) -> Field; }
    impl<T> Ext for T where T: Base { fn ext(self) -> Field { self.base() + 1 } }
    fn g<T: Base>(t: T) -> Field { t.ext() }
    fn main(x: Field) -> pub Field { g(x) }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2772, rustc: E0034
#[test]
fn i2772_self_path_call_of_a_method_that_a_supertrait_also_declares_is_ambiguous() {
    let src = r#"
    trait Validate { fn check(x: Field) -> Field; }
    trait Process: Validate {
        fn check(x: Field) -> Field;
        fn process(x: Field) -> Field { Self::check(x) }
    }
    struct S {}
    impl Validate for S { fn check(x: Field) -> Field { assert(x != 0); x } }
    impl Process for S { fn check(x: Field) -> Field { x } }
    fn main(x: Field) -> pub Field { S::process(x) }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2772, rustc: E0034
#[test]
fn i2772_self_path_call_of_a_method_that_a_where_clause_also_declares_is_ambiguous() {
    let src = r#"
    trait Validate { fn check(x: Field) -> Field; }
    trait Process {
        fn check(x: Field) -> Field;
        fn process(x: Field) -> Field where Self: Validate { Self::check(x) }
    }
    struct S {}
    impl Validate for S { fn check(x: Field) -> Field { assert(x != 0); x } }
    impl Process for S { fn check(x: Field) -> Field { x } }
    fn main(x: Field) -> pub Field { S::process(x) }
    "#;
    assert_rejected(src);
}

const CHECKED_AND_RAW: &str = r#"
    trait Checked { fn get(self) -> u8; }
    trait Raw { fn get(self) -> u8; }
    impl Checked for u8 { fn get(self) -> u8 { assert(self < 100); self } }
    impl Raw for u8 { fn get(self) -> u8 { self } }
    type Id<T> = T;
    "#;

/// https://github.com/noir-lang/noir-claude/issues/2793, rustc: E0034
#[test]
fn i2793_method_call_sees_a_where_clause_spelled_through_an_alias() {
    let src = format!(
        "{CHECKED_AND_RAW}
        fn read<T>(t: T) -> u8 where Id<T>: Checked, T: Raw {{ t.get() }}
        fn main(x: u8) -> pub u8 {{ read(x) }}"
    );
    assert_rejected(&src);
}

/// https://github.com/noir-lang/noir-claude/issues/2793, rustc: E0034
#[test]
fn i2793_generic_path_call_sees_a_where_clause_spelled_through_an_alias() {
    let src = format!(
        "{CHECKED_AND_RAW}
        fn read<T>(t: T) -> u8 where Id<T>: Checked, T: Raw {{ T::get(t) }}
        fn main(x: u8) -> pub u8 {{ read(x) }}"
    );
    assert_rejected(&src);
}

/// https://github.com/noir-lang/noir-claude/issues/2769
#[test]
fn i2769_type_path_call_skips_an_inaccessible_inherent_method_for_a_trait_method() {
    let src = r#"
    mod inner {
        pub struct S { pub v: Field }
        impl S { fn m(_self: Self) -> Field { 50 } }
        pub trait Tr { fn m(self) -> Field; }
        impl Tr for S { fn m(self) -> Field { 10 } }
    }
    use inner::S;
    use inner::Tr;
    fn main() -> pub Field { let s = S { v: 1 }; S::m(s) }
    "#;
    assert_runs(src, &["10"], &["50"]);
}

/// https://github.com/noir-lang/noir-claude/issues/1999
#[test]
fn i1999_self_path_call_of_an_inherent_method_uses_the_impls_self() {
    let src = r#"
    struct Foo<T> { x: T }
    impl Foo<u32> { fn tag() -> Field { 3232 } }
    impl Foo<u64> {
        fn tag() -> Field { 6464 }
        fn get_tag(_self: Self) -> Field { Self::tag() }
    }
    fn main(x: u64) -> pub Field { let foo: Foo<u64> = Foo { x }; foo.get_tag() }
    "#;
    assert_runs(src, &["6464"], &["3232"]);
}

/// https://github.com/noir-lang/noir-claude/issues/1055: the inherent method takes `&Foo`, the
/// receiver's own type, so it is found before the trait method that needs a dereference.
#[test]
fn i1055_method_call_finds_an_inherent_method_whose_self_is_an_aliased_reference() {
    let src = r#"
    struct Foo { x: Field }
    type FooRef = &Foo;
    impl Foo { fn pick(_self: FooRef) -> Field { 100 } }
    trait Pickable { fn pick(self) -> Field; }
    impl Pickable for Foo { fn pick(self) -> Field { 200 } }
    fn main() -> pub Field { let foo = Foo { x: 7 }; let r = &foo; r.pick() }
    "#;
    assert_runs(src, &["100"], &["200"]);
}

/// https://github.com/noir-lang/noir-claude/issues/2078
#[test]
fn i2078_type_path_call_finds_a_method_of_a_blanket_impl() {
    let src = r#"
    struct S { n: u32 }
    trait Op { fn tag(self) -> u32; }
    impl<T> Op for T { fn tag(self) -> u32 { 60 } }
    fn main(n: u32) -> pub u32 { S::tag(S { n }) }
    "#;
    assert_runs(src, &["60"], &[]);
}

const GUARD: &str = r#"
    trait Bar { fn check(x: Field) -> Self; }
    impl Bar for S { fn check(x: Field) -> Self { assert(x == 0); S { x } } }
    impl Bar for u32 { fn check(_x: Field) -> Self { 0 } }
    trait Guard { fn guard(x: Field) -> Self; }
    impl<T: Bar> Guard for T { fn guard(x: Field) -> Self { T::check(x) } }
    pub struct S { x: Field }
    "#;

/// https://github.com/noir-lang/noir-claude/issues/2797: `S::guard` is `<S as Guard>::guard`.
#[test]
fn i2797_type_path_call_of_a_blanket_impl_method_has_the_paths_type() {
    let src = format!("{GUARD}\nfn main(x: Field) {{ let _: u32 = S::guard(x); }}");
    assert_rejected(&src);
}

/// https://github.com/noir-lang/noir-claude/issues/2797
#[test]
fn i2797_type_path_call_of_a_blanket_impl_method_runs_the_paths_types_impl() {
    let src = format!("{GUARD}\nfn main(x: Field) -> pub Field {{ S::guard(x).x }}");
    assert_runs(&src, &["assert"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/1718: whether an impl overrides a default
/// method doesn't decide an integer literal's type.
#[test]
fn i1718_default_method_call_on_an_integer_literal_does_not_depend_on_overrides() {
    let overrides_u16 = r#"
    trait Foo { fn foo(self) -> Field { 0 } }
    impl Foo for u8 { }
    impl Foo for u16 { fn foo(self) -> Field { 1 } }
    fn main() -> pub Field { let x = 200; let _ = x.foo(); (!x) as Field }
    "#;
    let overrides_u8 = r#"
    trait Foo { fn foo(self) -> Field { 0 } }
    impl Foo for u8 { fn foo(self) -> Field { 1 } }
    impl Foo for u16 { }
    fn main() -> pub Field { let x = 200; let _ = x.foo(); (!x) as Field }
    "#;
    let width = |src: &str| {
        let errors = errors(src);
        if errors.is_empty() {
            monomorphized(src).contains("u8").to_string()
        } else {
            "rejected".to_string()
        }
    };
    assert_eq!(width(overrides_u16), width(overrides_u8));
}

/// https://github.com/noir-lang/noir-claude/issues/1062 (case A)
#[test]
fn i1062_second_parent_path_to_one_trait_at_other_arguments_is_found() {
    let src = r#"
    trait Quux<X> { fn quux_method() -> X; }
    trait Bar: Quux<u8> {}
    trait Baz: Quux<u16> {}
    trait Foo: Bar + Baz {}
    fn use_only_u16<T: Foo>() -> u16 { <T as Quux<u16>>::quux_method() }
    struct S {}
    impl Quux<u8> for S { fn quux_method() -> u8 { 7 } }
    impl Quux<u16> for S { fn quux_method() -> u16 { 13 } }
    impl Bar for S {}
    impl Baz for S {}
    impl Foo for S {}
    fn main() { let _ = use_only_u16::<S>(); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1062 (associated-type equality)
#[test]
fn i1062_parent_bound_fixing_an_associated_type_is_used_in_a_default_method() {
    let src = r#"
    trait Parent { type Item; }
    trait Child: Parent<Item = Field> {
        fn to_field(self, item: Self::Item) -> Field { item }
    }
    struct S {}
    impl Parent for S { type Item = Field; }
    impl Child for S {}
    fn main() {}
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1062
#[test]
fn i1062_grandparent_associated_type_fixed_by_a_parent_bound_is_used_in_a_default_method() {
    let src = r#"
    trait Grand { type Item; }
    trait Parent: Grand<Item = Field> {}
    trait Child: Parent {
        fn to_field(self, item: Self::Item) -> Field { item }
    }
    struct S {}
    impl Grand for S { type Item = Field; }
    impl Parent for S {}
    impl Child for S {}
    fn main() {}
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2778: a generic shadows a module of its name.
#[test]
fn i2778_generic_associated_type_path_is_not_a_same_named_module() {
    let src = r#"
    trait Check { fn check(x: u32); }
    pub struct Lt10 {}
    impl Check for Lt10 { fn check(x: u32) { assert(x < 10, "x out of range"); } }
    trait Validator { type C: Check; }
    pub struct Strict {}
    impl Validator for Strict { type C = Lt10; }
    mod V {
        pub struct C {}
        impl C { pub fn check(_x: u32) {} }
    }
    fn verify<V: Validator>(x: u32) { V::C::check(x); }
    fn main(x: u32) { verify::<Strict>(x); }
    "#;
    assert_runs(src, &["x out of range"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/2778
#[test]
fn i2778_generic_in_a_where_clause_is_not_a_same_named_module() {
    let src = r#"
    trait Check<K> { fn check(x: u32); }
    pub struct Lt10 {}
    trait Validator { type K; }
    pub struct Strict {}
    impl Validator for Strict { type K = Lt10; }
    pub struct Ver {}
    impl Check<Lt10> for Ver { fn check(x: u32) { assert(x < 10, "x out of range"); } }
    mod V { pub struct K {} }
    impl Check<V::K> for Ver { fn check(_x: u32) {} }
    fn verify<V: Validator, C>(x: u32) where C: Check<V::K> { C::check(x); }
    fn main(x: u32) { verify::<Strict, Ver>(x); }
    "#;
    assert_runs(src, &["x out of range"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/2778
#[test]
fn i2778_method_of_a_generics_associated_type_is_found() {
    let src = r#"
    trait Mk { fn mk() -> Field; }
    struct P {}
    impl Mk for P { fn mk() -> Field { 42 } }
    trait Tr { type A: Mk; }
    struct S {}
    impl Tr for S { type A = P; }
    fn g<T: Tr>() -> Field { T::A::mk() }
    fn main() -> pub Field { g::<S>() }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2778
#[test]
fn i2778_method_of_selfs_associated_type_in_a_default_method_is_found() {
    let src = r#"
    trait Mk { fn mk() -> Field; }
    struct P {}
    impl Mk for P { fn mk() -> Field { 42 } }
    trait Tr { type A: Mk; fn g() -> Field { Self::A::mk() } }
    struct S {}
    impl Tr for S { type A = P; }
    fn main() -> pub Field { S::g() }
    "#;
    assert_accepted(src);
}

// --- B. Impl search, where clauses, overlap --------------------------------------------------

/// https://github.com/noir-lang/noir-claude/issues/2743: each of the 37 distinct bounds is solved
/// once, not once per path to it (about 4^10).
#[test]
fn i2743_a_bound_reached_through_many_impls_is_solved_once() {
    let src = r#"
    pub struct Wrap<T> { inner: T }
    pub trait Tr0 {}
    impl Tr0 for Field {}
    impl<T> Tr0 for Wrap<T> where T: Tr0, T: Tr1, T: Tr2, T: Tr3 {}
    pub trait Tr1 {}
    impl Tr1 for Field {}
    impl<T> Tr1 for Wrap<T> where T: Tr0, T: Tr1, T: Tr2, T: Tr3 {}
    pub trait Tr2 {}
    impl Tr2 for Field {}
    impl<T> Tr2 for Wrap<T> where T: Tr0, T: Tr1, T: Tr2, T: Tr3 {}
    pub trait Tr3 {}
    impl Tr3 for Field {}
    impl<T> Tr3 for Wrap<T> where T: Tr0, T: Tr1, T: Tr2, T: Tr3 {}
    fn need<X>(x: X) -> X where X: Tr0 { x }
    fn main(x: Field) -> pub Field {
        let w: Wrap<Wrap<Wrap<Wrap<Wrap<Wrap<Wrap<Wrap<Wrap<Field>>>>>>>>> = Wrap { inner: Wrap { inner: Wrap { inner: Wrap { inner: Wrap { inner: Wrap { inner: Wrap { inner: Wrap { inner: Wrap { inner: x } } } } } } } } };
        let _ = need(w);
        x
    }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1958 (R1), rustc: E0119
#[test]
fn i1958_impl_overlapping_a_blanket_impl_through_a_generic_is_rejected() {
    let src = r#"
    trait Small {}
    trait Checked { fn checked(self) -> u8; }
    struct Wrap<T> { v: T }
    impl Small for Wrap<u8> {}
    impl Small for Wrap<u16> {}
    impl<T> Checked for T where T: Small { fn checked(self) -> u8 { 0 } }
    impl<U> Checked for Wrap<U> { fn checked(self) -> u8 { 1 } }
    fn main() {}
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1958 (R2), rustc: E0119
#[test]
fn i1958_blanket_impls_with_where_clauses_one_type_satisfies_both_overlap() {
    let src = r#"
    trait Bounded {}
    trait Audited {}
    trait Validate { fn validate(self); }
    impl<T> Validate for T where T: Bounded { fn validate(self) {} }
    impl<T> Validate for T where T: Audited { fn validate(self) { assert(false, "rejected"); } }
    struct Amount { v: u8 }
    impl Bounded for Amount {}
    impl Audited for Amount {}
    fn main() {}
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2798 (d1)
#[test]
fn i2798_impl_for_an_arithmetic_array_length_overlaps_a_concrete_length() {
    let src = r#"
    trait Validate { fn validate(self); }
    impl Validate for [u8; 4] { fn validate(self) { for b in self { assert(b < 128); } } }
    impl<let N: u32> Validate for [u8; N + N] { fn validate(self) {} }
    fn main() {}
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2798 (a1)
#[test]
fn i2798_impl_for_an_arithmetic_generic_overlaps_a_concrete_one() {
    let src = r#"
    struct S<let N: u32> {}
    trait Tr { fn m(self) -> u32; }
    impl<let N: u32> Tr for S<N + N> { fn m(self) -> u32 { 1 } }
    impl Tr for S<4> { fn m(self) -> u32 { 2 } }
    fn main() {}
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2771: `S: Baz` fails whatever `T` is.
#[test]
fn i2771_impl_with_a_failing_where_bound_after_an_undecided_one_does_not_apply() {
    let src = r#"
    struct S { v: u32 }
    trait Conv<T> { fn conv(self) -> T; }
    trait Mk<U> { fn mk(u: U) -> Self; }
    trait Baz {}
    impl Conv<u32> for S { fn conv(self) -> u32 { self.v + 1 } }
    impl<T, U> Conv<T> for U where T: Mk<U>, U: Baz { fn conv(self) -> T { T::mk(self) } }
    fn main(x: u32) { let s = S { v: x }; assert(Conv::conv(s) == 2); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2771 (bounds swapped)
#[test]
fn i2771_impl_with_a_failing_where_bound_before_an_undecided_one_does_not_apply() {
    let src = r#"
    struct S { v: u32 }
    trait Conv<T> { fn conv(self) -> T; }
    trait Mk<U> { fn mk(u: U) -> Self; }
    trait Baz {}
    impl Conv<u32> for S { fn conv(self) -> u32 { self.v + 1 } }
    impl<T, U> Conv<T> for U where U: Baz, T: Mk<U> { fn conv(self) -> T { T::mk(self) } }
    fn main(x: u32) { let s = S { v: x }; assert(Conv::conv(s) == 2); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2773 (c3a): the bounds decide every generic
/// whatever order they're written in.
#[test]
fn i2773_chain_of_bounds_written_against_resolution_order_is_solved() {
    let src = r#"
    struct S {}
    trait A<T, V> { fn a(self, t: T) -> V; }
    trait B<T, U> { fn b(self, t: T) -> U; }
    trait C<T, U> { fn c(self, t: T) -> U; }
    trait D<T> { fn d(self) -> T; }
    impl D<u32> for S { fn d(self) -> u32 { 1 } }
    impl C<u32, u8> for S { fn c(self, t: u32) -> u8 { t as u8 + 1 } }
    impl C<u64, u16> for S { fn c(self, t: u64) -> u16 { t as u16 + 1 } }
    impl B<u8, u16> for S { fn b(self, t: u8) -> u16 { t as u16 + 1 } }
    impl B<u16, u8> for S { fn b(self, t: u16) -> u8 { t as u8 + 1 } }
    impl A<u16, u32> for S { fn a(self, t: u16) -> u32 { t as u32 + 1 } }
    impl A<u8, u64> for S { fn a(self, t: u8) -> u64 { t as u64 + 1 } }
    fn g<X, T1, T2, T3, V>(x: X) -> V
    where X: A<T3, V> + B<T2, T3> + C<T1, T2> + D<T1> { x.a(x.b(x.c(x.d()))) }
    fn main() { let s = S {}; let _ = g(s); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2799
#[test]
fn i2799_impl_for_n_plus_one_does_not_apply_to_zero() {
    let src = r#"
    struct S<let N: u32> {}
    trait Tr { fn v() -> u32; }
    impl<let N: u32> Tr for S<N + 1> { fn v() -> u32 { 7 } }
    fn main() -> pub u32 { <S<0> as Tr>::v() }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2233
#[test]
fn i2233_entry_point_where_clause_that_no_impl_satisfies_is_rejected() {
    let src = r#"
    trait Small {}
    struct Raw { x: u32 }
    fn spend<T: Small>(_t: T, x: u32) -> u32 { x * 2 }
    fn main(x: u32) -> pub u32 where Raw: Small { spend(Raw { x }, x) }
    "#;
    assert_rejected(src);
}

// --- C. Associated items and parent bounds ---------------------------------------------------

/// https://github.com/noir-lang/noir-claude/issues/1811: `<S as Par>::N` is 3, not 5.
#[test]
fn i1811_elided_parent_item_is_not_decided_by_a_function_body() {
    let src = r#"
    trait Par { let N: u32; }
    trait Sub: Par {}
    #[gen_sub]
    struct S {}
    impl Par for S { let N: u32 = 3; }
    comptime fn gen_sub(_s: TypeDefinition) -> Quoted { quote { impl Sub for S {} } }
    pub fn poison<T, let M: u32>(xs: [Field; M]) -> [Field; M] where T: Sub {
        let ys: [Field; <T as Par>::N] = xs;
        ys
    }
    fn main() -> pub Field {
        let a: [Field; 5] = [1, 2, 3, 4, 5];
        let b = poison::<S, 5>(a);
        b[4]
    }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2549 (o1)
#[test]
fn i2549_qualified_call_in_a_default_method_substitutes_the_traits_generic() {
    let src = r#"
    trait Foo<T> {
        fn m(self) -> T;
        fn d(self) -> T where Self: Foo<[T; 2]> { let a: T = <Self as Foo<[T; 2]>>::m(self); a }
    }
    struct X { v: u16 }
    impl Foo<u16> for X { fn m(self) -> u16 { self.v } }
    impl Foo<[u16; 2]> for X { fn m(self) -> [u16; 2] { [self.v + 1, self.v + 2] } }
    fn main(x: u16) -> pub u16 { let s = X { v: x }; Foo::<u16>::d(s) }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2549 (o2)
#[test]
fn i2549_well_typed_qualified_call_in_a_default_method_is_accepted() {
    let src = r#"
    trait Foo<T> {
        fn m(self) -> T;
        fn d(self) -> [T; 2] where Self: Foo<[T; 2]> {
            let a: [T; 2] = <Self as Foo<[T; 2]>>::m(self);
            a
        }
    }
    struct X { v: u16 }
    impl Foo<u16> for X { fn m(self) -> u16 { self.v } }
    impl Foo<[u16; 2]> for X { fn m(self) -> [u16; 2] { [self.v + 1, self.v + 2] } }
    fn main(x: u16) -> pub [u16; 2] { let s = X { v: x }; Foo::<u16>::d(s) }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2741
#[test]
fn i2741_generic_impl_after_a_concrete_one_of_the_same_type_is_registered() {
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
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2752 (w1)
#[test]
#[ignore = "noir-claude#2752"]
fn i2752_trait_where_self_bound_eliding_an_associated_type_can_be_implemented() {
    let src = r#"
    trait Lim { type Out; }
    struct A {}
    impl Lim for A { type Out = u8; }
    trait Chk where Self: Lim { fn chk() {} }
    impl Chk for A {}
    fn main() { A::chk(); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2752 (w3)
#[test]
#[ignore = "noir-claude#2752"]
fn i2752_default_method_returning_an_elided_associated_type_of_a_where_self_bound() {
    let src = r#"
    trait Lim { type Out; fn get(self) -> Self::Out; }
    struct A { v: u8 }
    impl Lim for A { type Out = u8; fn get(self) -> u8 { self.v } }
    trait Chk where Self: Lim { fn chk(self) -> <Self as Lim>::Out { self.get() } }
    impl Chk for A {}
    fn main(x: u8) -> pub u8 { A { v: x }.chk() }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2237 (m1)
#[test]
#[ignore = "noir-claude#2237"]
fn i2237_impl_method_returning_a_parent_projection_matches_the_trait() {
    let src = r#"
    trait Source { type Out; fn get(self) -> Self::Out; }
    trait Checked: Source {}
    trait Foo { fn g(self) -> <Self as Source>::Out where Self: Checked; }
    impl Source for u32 { type Out = u8; fn get(self) -> u8 { self as u8 } }
    impl Checked for u32 {}
    impl Foo for u32 { fn g(self) -> <Self as Source>::Out { self.get() } }
    fn main(x: u32) { let r: u8 = x.g(); assert(r == 3); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2237 (m2)
#[test]
#[ignore = "noir-claude#2237"]
fn i2237_impl_method_restating_an_implied_parent_bound_matches_the_trait() {
    let src = r#"
    trait Source { type Out; fn get(self) -> Self::Out; }
    trait Checked: Source {}
    trait Foo { fn f<T>(x: T) -> Field where T: Checked; }
    struct S {}
    impl Source for u32 { type Out = u8; fn get(self) -> u8 { self as u8 } }
    impl Checked for u32 {}
    impl Foo for S { fn f<U>(x: U) -> Field where U: Checked, U: Source { let _ = x; 3 } }
    fn main(x: u32) { assert(S::f(x) == 3); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2761
#[test]
#[ignore = "noir-claude#2761"]
fn i2761_impl_method_restating_a_self_projection_bound_matches_the_trait() {
    let src = r#"
    trait Eq { fn eq(self, other: Self) -> bool; }
    trait C { type N; }
    pub struct V<T> { t: T }
    impl<T> C for V<T> { type N = T; }
    trait D { fn d(_self: Self) -> Field where Self: C, <Self as C>::N: Eq; }
    impl<T> D for V<T> { fn d(_self: Self) -> Field where Self: C, <Self as C>::N: Eq { 1 } }
    fn main() {}
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2756
#[test]
fn i2756_impl_where_clause_projecting_through_a_later_impl_matches_the_trait() {
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
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2735
#[test]
fn i2735_impl_header_projecting_through_a_later_impl_is_resolved() {
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
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2735
#[test]
fn i2735_impl_for_an_array_length_from_a_later_impl_is_resolved() {
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
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2709
#[test]
fn i2709_where_clause_on_a_concrete_type_decides_an_associated_type() {
    let src = r#"
    pub trait Tr { type Out; fn mk() -> Self::Out; }
    impl Tr for u8 { type Out = u16; fn mk() -> u16 { 7 } }
    fn f<T>() -> T where u8: Tr<Out = T> { <u8 as Tr>::mk() }
    fn main() { let _: u16 = f::<u16>(); }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2705
#[test]
fn i2705_associated_type_result_decided_by_an_earlier_constraint() {
    let src = r#"
    pub trait First { type Out; fn mk() -> Self::Out; }
    pub struct K {}
    impl First for K { type Out = u8; fn mk() -> u8 { 7 } }
    pub struct Wrap<T> { x: T }
    pub struct A { v: u32 }
    pub struct B { w: u64 }
    pub trait Second { type R; fn get(self) -> Self::R; }
    impl Second for Wrap<u8> { type R = A; fn get(self) -> A { A { v: self.x as u32 } } }
    impl Second for Wrap<u16> { type R = B; fn get(self) -> B { B { w: self.x as u64 } } }
    fn first<T: First>() -> Wrap<T::Out> { Wrap { x: T::mk() } }
    fn second<W: Second>(w: W) -> W::R { w.get() }
    fn main() {
        let w = first::<K>();
        let b = second(w);
        let _: u32 = b.v;
    }
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2645
#[test]
fn i2645_associated_constant_shorthand_solves_the_impls_where_clause() {
    let src = r#"
    pub trait Foo { let K: u32; }
    pub trait Tr { let C: u32; }
    pub struct H<T> {}
    impl<T> Tr for H<T> where T: Foo { let C: u32 = T::K * 10; }
    pub struct A {}
    impl Foo for A { let K: u32 = 1; }
    fn main() -> pub u32 { H::<A>::C }
    "#;
    assert_runs(src, &["10"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/1619
#[test]
fn i1619_associated_constant_of_a_primitive_type_is_found() {
    let src = r#"
    trait HasN { let N: u32; }
    impl HasN for u32 { let N: u32 = 9; }
    fn main() -> pub u32 { u32::N }
    "#;
    assert_runs(src, &["9"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/2001: an inherent impl has no associated types.
#[test]
fn i2001_self_associated_type_in_an_inherent_impl_is_rejected() {
    let src = r#"
    trait Tr { type u8; }
    struct Foo { x: Field }
    impl Tr for Foo { type u8 = Field; }
    impl Foo {
        fn g(v: u64) -> Field { let y: Self::u8 = v as Self::u8; y as Field + 1 }
    }
    fn main() -> pub Field { Foo::g(1000) }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2000, rustc: E0207
#[test]
fn i2000_impl_generic_that_its_self_type_does_not_determine_is_rejected() {
    let src = r#"
    trait Foo { fn foo(self) -> u32; }
    impl<let N: u32> Foo for [u8; N * 0] { fn foo(self) -> u32 { 7 } }
    fn main() -> pub u32 { let a: [u8; 0] = []; a.foo() }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1935
#[test]
fn i1935_trait_where_clause_naming_a_later_trait_is_accepted() {
    let src = r#"
    trait B {}
    trait A<U> where U: B {}
    fn main() {}
    "#;
    assert_accepted(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1935
#[test]
fn i1935_impl_of_a_trait_whose_where_clause_names_a_later_trait() {
    let src = r#"
    trait Sz<U> where U: N2 { fn base(self) -> Field; }
    trait N2 { fn name2(self) -> Field; }
    impl N2 for u8 { fn name2(self) -> Field { 10 } }
    struct B {}
    impl Sz<u8> for B { fn base(self) -> Field { 1 } }
    fn main() -> pub Field { <B as Sz<u8>>::base(B {}) }
    "#;
    assert_accepted(src);
}

// --- D. Declarations that crash or are never checked -----------------------------------------

/// https://github.com/noir-lang/noir-claude/issues/1711
#[test]
#[ignore = "noir-claude#1711"]
fn i1711_impl_method_generic_of_another_kind_than_the_traits_is_rejected() {
    let src = r#"
    trait Marker {}
    trait MyT { fn f<T>(self, x: T) where T: Marker; }
    struct B { x: Field }
    impl MyT for B { fn f<let Q: u32>(_self: Self, _x: Field) where Q: Marker {} }
    fn main() {}
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2624
#[test]
fn i2624_trait_method_returning_impl_trait_does_not_crash_the_compiler() {
    let src = r#"
    trait Show { fn show(self) -> Field; }
    impl Show for u8 { fn show(self) -> Field { self as Field + 1000 } }
    trait Mk { fn mk(self) -> impl Show; }
    struct A {}
    impl Mk for A { fn mk(self) -> impl Show { 5_u8 } }
    fn g<X: Mk>(x: X) -> Field { let _ = x.mk(); 0 }
    fn main() -> pub Field { g(A {}) }
    "#;
    assert_rejected_or_compiles(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1918: `a::<u8>()` is an `A` and `b::<u8>()` a
/// `B`, as for non-generic functions.
#[test]
fn i1918_generic_functions_returning_impl_trait_have_their_concrete_types_at_the_call() {
    let src = r#"
    trait Foo { fn get(self) -> Field; }
    struct A { x: u8 }
    struct B { x: Field, y: Field }
    impl Foo for A { fn get(self) -> Field { self.x as Field } }
    impl Foo for B { fn get(self) -> Field { self.x + self.y } }
    fn a<T>() -> impl Foo { A { x: 7 } }
    fn b<T>() -> impl Foo { B { x: 100, y: 200 } }
    fn main(c: bool) -> pub Field { let v = if c { a::<u8>() } else { b::<u8>() }; v.get() }
    "#;
    assert_rejected_with(src, "found type B");
}

/// https://github.com/noir-lang/noir-claude/issues/1904
#[test]
fn i1904_impl_trait_nested_in_a_return_type_is_rejected() {
    let src = r#"
    trait Foo { fn f(self) -> Field; }
    impl Foo for Field { fn f(self) -> Field { self } }
    fn mk() -> [impl Foo; 2] { mk() }
    fn main(x: Field) -> pub Field { let _a = mk(); x }
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/1904
#[test]
fn i1904_impl_trait_nested_in_a_parameter_type_is_rejected() {
    let src = r#"
    trait Foo { fn f(self) -> Field; }
    impl Foo for Field { fn f(self) -> Field { self } }
    fn take(a: (impl Foo, Field)) -> Field { a.1 }
    fn main(x: Field) -> pub Field { take((x, x)) }
    "#;
    assert_rejected_with(src, "`impl Trait` is not allowed");
}

/// https://github.com/noir-lang/noir-claude/issues/1904
#[test]
fn i1904_impl_trait_in_a_let_annotation_is_rejected() {
    let src = r#"
    trait Foo { fn f(self) -> Field; }
    impl Foo for Field { fn f(self) -> Field { self } }
    fn main(x: Field) -> pub Field { let a: impl Foo = x; a.f() }
    "#;
    assert_rejected_with(src, "`impl Trait` is not allowed");
}

/// https://github.com/noir-lang/noir-claude/issues/2689
#[test]
fn i2689_impl_trait_return_is_checked_after_the_bodys_constraints_are_solved() {
    let src = r#"
    trait Foo { fn v(self) -> u32; }
    impl Foo for u8 { fn v(self) -> u32 { 8 } }
    impl Foo for u64 { fn v(self) -> u32 { 64 } }
    fn use_foo<T: Foo>(x: T) -> u32 { x.v() }
    trait Mk { type Out; fn mk(self) -> Self::Out; }
    impl Mk for bool { type Out = u64; fn mk(self) -> u64 { 5 } }
    fn a() -> impl Foo { Mk::mk(true) }
    fn main() -> pub u32 { use_foo(a()) }
    "#;
    assert_runs(src, &["64"], &[]);
}

/// https://github.com/noir-lang/noir-claude/issues/2049
#[test]
#[ignore = "noir-claude#2049"]
fn i2049_trait_method_with_an_impl_trait_parameter_can_be_implemented() {
    let src = r#"
    trait Bar { fn get(self) -> Field; }
    impl Bar for Field { fn get(self) -> Field { self + 1 } }
    trait Tr { fn f(a: impl Bar) -> Field; }
    struct S {}
    impl Tr for S { fn f(a: impl Bar) -> Field { a.get() } }
    fn main(x: Field) -> pub Field { S::f(x) }
    "#;
    let errors = errors_using_features(src, &[UnstableFeature::TraitAsType]);
    assert!(errors.is_empty(), "expected the program to be accepted, got {errors:?}");
}

/// https://github.com/noir-lang/noir-claude/issues/2746
#[test]
fn i2746_runtime_impl_of_a_comptime_trait_method_is_rejected() {
    let src = r#"
    trait Foo { comptime fn foo(x: Field) -> Field; }
    struct S {}
    impl Foo for S { fn foo(x: Field) -> Field { x + 1 } }
    fn main() {}
    "#;
    assert_rejected(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2746: a type that only exists at compile time
/// implements a runtime trait with `comptime` methods, as the standard library's `Quoted: Eq` does.
#[test]
fn i2746_comptime_impl_of_a_runtime_trait_method_is_accepted() {
    let src = r#"
    trait Foo { fn foo(x: Field) -> Field; }
    struct S {}
    impl Foo for S { comptime fn foo(x: Field) -> Field { x + 1 } }
    fn main() {}
    "#;
    assert_accepted(src);
}
