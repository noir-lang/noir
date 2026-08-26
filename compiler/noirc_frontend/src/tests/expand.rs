//! Tests for `nargo expand` output (via the HIR printer), focusing on faithfully
//! reconstructing impls: their generics and where clauses.

use crate::tests::{assert_no_errors_and_to_string, assert_no_errors_and_to_string_using_features};

#[test]
fn expands_inherent_impl_with_where_clause() {
    let src = r#"
    trait Bar {}

    struct Foo<T> {
        x: T,
    }

    impl<T> Foo<T>
    where
        T: Bar,
    {
        fn get(self) -> T {
            self.x
        }
    }

    impl Bar for Field {}

    fn main() {
        let foo = Foo { x: 1 };
        let _ = foo.get();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Bar {

    }

    impl Bar for Field {

    }

    struct Foo<T> {
        x: T,
    }

    impl<T> Foo<T> where T: Bar {
        fn get(self) -> T {
            self.x
        }
    }

    fn main() {
        let foo: Foo<Field> = Foo::<Field> { x: 1_Field};
        let _: Field = foo.get();
    }
    ");
}

#[test]
fn expands_inherent_impl_with_colon_bound_generic() {
    let src = r#"
    trait Bar {}

    struct Foo<T> {
        x: T,
    }

    impl<T: Bar> Foo<T> {
        fn get(self) -> T {
            self.x
        }
    }

    impl Bar for Field {}

    fn main() {
        let foo = Foo { x: 1 };
        let _ = foo.get();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Bar {

    }

    impl Bar for Field {

    }

    struct Foo<T> {
        x: T,
    }

    impl<T> Foo<T> where T: Bar {
        fn get(self) -> T {
            self.x
        }
    }

    fn main() {
        let foo: Foo<Field> = Foo::<Field> { x: 1_Field};
        let _: Field = foo.get();
    }
    ");
}

#[test]
fn expands_inherent_impl_method_with_own_where_clause() {
    let src = r#"
    trait Bar {}
    trait Baz {}

    struct Foo<T> {
        x: T,
    }

    impl<T> Foo<T>
    where
        T: Bar,
    {
        fn convert<U>(self, _other: U) -> T
        where
            U: Baz,
        {
            self.x
        }
    }

    impl Bar for Field {}
    impl Baz for Field {}

    fn main() {
        let foo = Foo { x: 1 };
        let _ = foo.convert(2);
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Bar {

    }

    impl Bar for Field {

    }

    trait Baz {

    }

    impl Baz for Field {

    }

    struct Foo<T> {
        x: T,
    }

    impl<T> Foo<T> where T: Bar {
        fn convert<U>(self, _other: U) -> T where U: Baz {
            self.x
        }
    }

    fn main() {
        let foo: Foo<Field> = Foo::<Field> { x: 1_Field};
        let _: Field = foo.convert(2_Field);
    }
    ");
}

#[test]
fn expands_separate_inherent_impl_blocks_separately() {
    let src = r#"
    struct Foo {
        x: Field,
    }

    impl Foo {
        fn a(self) -> Field {
            self.x
        }
    }

    impl Foo {
        fn b(self) -> Field {
            self.x
        }
    }

    fn main() {
        let foo = Foo { x: 1 };
        let _ = foo.a();
        let _ = foo.b();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    struct Foo {
        x: Field,
    }

    impl Foo {
        fn a(self) -> Field {
            self.x
        }
    }

    impl Foo {
        fn b(self) -> Field {
            self.x
        }
    }

    fn main() {
        let foo: Foo = Foo { x: 1_Field};
        let _: Field = foo.a();
        let _: Field = foo.b();
    }
    ");
}

#[test]
fn expands_inherent_impl_with_doc_comment() {
    let src = r#"
    struct Foo {
        x: Field,
    }

    /// Methods for Foo.
    impl Foo {
        fn get(self) -> Field {
            self.x
        }
    }

    fn main() {
        let foo = Foo { x: 1 };
        let _ = foo.get();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    struct Foo {
        x: Field,
    }

    /// Methods for Foo.
    impl Foo {
        fn get(self) -> Field {
            self.x
        }
    }

    fn main() {
        let foo: Foo = Foo { x: 1_Field};
        let _: Field = foo.get();
    }
    ");
}

// A method that binds an associated type on the impl's generic (`T: Foo<Assoc = Field>`)
// keeps that binding: it is distinct from the impl's own `T: Foo` and must not be deduplicated
// against it just because they share the same type and trait.
#[test]
fn expands_method_where_clause_with_associated_type_binding() {
    let src = r#"
    trait Foo {
        type Assoc;
        fn get(self) -> Self::Assoc;
    }

    impl Foo for Field {
        type Assoc = Field;
        fn get(self) -> Field {
            self
        }
    }

    struct W<T> {
        x: T,
    }

    impl<T> W<T>
    where
        T: Foo,
    {
        fn m(self) -> Field
        where
            T: Foo<Assoc = Field>,
        {
            self.x.get()
        }
    }

    fn main() {
        let w = W { x: 1 };
        let _ = w.m();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded);
}

#[test]
fn expands_inherent_impl_inside_module() {
    // An inherent impl declared inside a submodule must be printed inside that module, not
    // hoisted to the type's module. Hoisting it would change method-resolution visibility.
    let src = r#"
    pub struct Foo {}

    mod impls {
        use super::Foo;

        impl Foo {
            pub fn bar(self) -> u32 {
                let _ = self;
                1
            }
        }
    }

    fn main() {
        let _ = Foo::bar(Foo {});
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    pub struct Foo {
    }

    mod impls {
        use crate::Foo;

        impl Foo {
            pub fn bar(self) -> u32 {
                let _: Self = self;
                1_u32
            }
        }
    }

    fn main() {
        let _: u32 = Foo { }.bar();
    }
    ");
}

#[test]
fn expands_trait_and_inherent_impl_inside_module() {
    // Both the inherent `impl Foo` and the trait `impl Bar for Foo` are declared inside the
    // `impls` module; they must be printed there so the private inherent method stays private.
    let src = r#"
    pub struct Foo {}

    trait Bar {
        fn bar(self) -> u32;
    }

    mod impls {
        use super::{Bar, Foo};

        impl Foo {
            fn bar(self) -> u32 {
                let _ = self;
                1
            }
        }

        impl Bar for Foo {
            fn bar(self) -> u32 {
                let _ = self;
                2
            }
        }

        pub fn calls_inherent_bar(foo: Foo) -> u32 {
            foo.bar()
        }
    }

    fn main() {
        let _ = (Foo {}).bar();
        let _ = impls::calls_inherent_bar(Foo {});
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    pub struct Foo {
    }

    trait Bar {
        fn bar(self) -> u32;
    }

    mod impls {
        use crate::Bar;
        use crate::Foo;

        impl Foo {
            fn bar(self) -> u32 {
                let _: Self = self;
                1_u32
            }
        }

        impl Bar for Foo {
            fn bar(self) -> u32 {
                let _: Self = self;
                2_u32
            }
        }

        pub fn calls_inherent_bar(foo: Foo) -> u32 {
            foo.bar()
        }
    }

    fn main() {
        let _: u32 = Foo { }.bar();
        let _: u32 = impls::calls_inherent_bar(Foo { });
    }
    ");
}

#[test]
fn expands_trait_impl_calling_private_inherent_method_inside_module() {
    // The trait method body calls a module-private inherent method (`secret`). Both impls must be
    // emitted inside `impls` so the call stays visible; hoisting the trait impl to the root would
    // make the expanded source fail to compile. `assert_no_errors_and_to_string` re-checks that the
    // expanded output compiles, so this also guards the round-trip.
    let src = r#"
    pub struct Foo {}

    trait Bar {
        fn bar(self) -> u32;
    }

    mod impls {
        use super::{Bar, Foo};

        impl Foo {
            fn secret(self) -> u32 {
                let _ = self;
                42
            }
        }

        impl Bar for Foo {
            fn bar(self) -> u32 {
                self.secret()
            }
        }
    }

    fn main() {
        let _ = (Foo {}).bar();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    pub struct Foo {
    }

    trait Bar {
        fn bar(self) -> u32;
    }

    mod impls {
        use crate::Bar;
        use crate::Foo;

        impl Foo {
            fn secret(self) -> u32 {
                let _: Self = self;
                42_u32
            }
        }

        impl Bar for Foo {
            fn bar(self) -> u32 {
                self.secret()
            }
        }
    }

    fn main() {
        let _: u32 = Foo { }.bar();
    }
    ");
}

/// A trait method and an inherent method can share a name, and the four ways to call one resolve
/// to different methods:
///
/// - `foo.method()` and `Foo::method(foo)` both prefer the *inherent* method.
/// - `Trait::method(foo)` and `<Foo as Trait>::method(foo)` select the *trait* method.
///
/// The HIR printer must keep each faithful. Rendering a trait call back as `foo.method()` (or
/// `Foo::method(foo)`) would make the expanded source re-resolve to the inherent method, silently
/// changing which method runs. The fully-qualified `<Foo as Trait>::method(foo)` is the only form
/// that round-trips to the trait method regardless of the inherent one.
///
/// This test pins the *buggy* output (all four calls collapse to `foo.method()`) so the fix is
/// visible as a snapshot change.
#[test]
fn expands_trait_method_call_shadowed_by_inherent_method() {
    let src = r#"
    trait Trait {
        fn method(self);
    }

    struct Foo {}

    impl Foo {
        fn method(self) {
            let _ = self;
        }
    }

    impl Trait for Foo {
        fn method(self) {
            let _ = self;
        }
    }

    fn main() {
        let foo = Foo {};
        foo.method();
        Foo::method(foo);
        Trait::method(foo);
        <Foo as Trait>::method(foo);
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Trait {
        fn method(self);
    }

    struct Foo {
    }

    impl Foo {
        fn method(self) {
            let _: Self = self;
        }
    }

    impl Trait for Foo {
        fn method(self) {
            let _: Self = self;
        }
    }

    fn main() {
        let foo: Foo = Foo { };
        foo.method();
        foo.method();
        <Foo as Trait>::method(foo);
        <Foo as Trait>::method(foo);
    }
    ");
}

#[test]
fn expands_self_static_trait_method_call_in_default_method() {
    let src = r#"
    trait ATrait {
        fn static_method() -> Field {
            Self::static_method_2()
        }

        fn static_method_2() -> Field {
            100
        }
    }

    struct Foo {}

    impl ATrait for Foo {
        fn static_method_2() -> Field {
            200
        }
    }

    fn main() {
        let _ = Foo::static_method();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait ATrait {
        fn static_method() -> Field {
            Self::static_method_2()
        }

        fn static_method_2() -> Field {
            100_Field
        }
    }

    struct Foo {
    }

    impl ATrait for Foo {
        fn static_method_2() -> Field {
            200_Field
        }
    }

    fn main() {
        let _: Field = <Foo as ATrait>::static_method();
    }
    ");
}

#[test]
fn expands_impl_trait_parameter_without_where_clause() {
    let src = r#"
    trait SomeTrait {
        fn get_value(self) -> Field;
    }

    struct AType {}

    impl SomeTrait for AType {
        fn get_value(self) -> Field {
            1
        }
    }

    fn take(x: impl SomeTrait) -> Field {
        x.get_value()
    }

    fn main() {
        let _ = take(AType {});
    }
    "#;
    let expanded = assert_no_errors_and_to_string_using_features(
        src,
        &[crate::elaborator::UnstableFeature::TraitAsType],
    );
    insta::assert_snapshot!(expanded, @r"
    trait SomeTrait {
        fn get_value(self) -> Field;
    }

    struct AType {
    }

    impl SomeTrait for AType {
        fn get_value(self) -> Field {
            1_Field
        }
    }

    fn take(x: impl SomeTrait) -> Field {
        x.get_value()
    }

    fn main() {
        let _: Field = take(AType { });
    }
    ");
}

#[test]
fn expands_impl_trait_parameter_with_generics() {
    let src = r#"
    trait Foo<let N: u32> {}

    impl<let N: u32> Foo<N> for [Field; N] {}

    fn my_fn<let N: u32>(_input: impl Foo<N>) {}

    fn main() {
        my_fn::<0>([]);
    }
    "#;
    let expanded = assert_no_errors_and_to_string_using_features(
        src,
        &[crate::elaborator::UnstableFeature::TraitAsType],
    );
    insta::assert_snapshot!(expanded, @r"
    trait Foo<let N: u32> {

    }

    impl<let N: u32> Foo<N> for [Field; N] {

    }

    fn my_fn<let N: u32>(_input: impl Foo<N>) {
    }

    fn main() {
        my_fn::<0>([]);
    }
    ");
}

#[test]
fn expands_global_whose_value_has_private_fields_as_its_initializer_expression() {
    let src = r#"
    mod foo {
        pub struct Bar {
            value: Field,
        }

        pub fn make_bar() -> Bar {
            Bar { value: 1 }
        }
    }

    global B: foo::Bar = foo::make_bar();

    fn main() {
        let _ = B;
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    mod foo {
        pub struct Bar {
            value: Field,
        }

        pub fn make_bar() -> Bar {
            Bar { value: 1_Field}
        }
    }

    global B: foo::Bar = foo::make_bar();

    fn main() {
        let _: foo::Bar = B;
    }
    ");
}

#[test]
fn expands_associated_constant_reference_in_impl_method() {
    let src = r#"
    trait Trait {
        let N: u32;

        fn foo() -> u32;
    }

    struct Foo {}

    impl Trait for Foo {
        let N: u32 = 30;

        fn foo() -> u32 {
            Self::N
        }
    }

    fn main() {
        let _ = Foo::foo();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Trait {
        let N: u32;

        fn foo() -> u32;
    }

    struct Foo {
    }

    impl Trait for Foo {
        let N: u32 = 30_u32;

        fn foo() -> u32 {
            Self::N
        }
    }

    fn main() {
        let _: u32 = Foo::foo();
    }
    ");
}

#[test]
fn expands_numeric_type_alias_with_its_numeric_type() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;

    fn main() {
        let arr: [Field; Double::<2>] = [0; 4];
        let _ = arr;
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    type Double<let N: u32>: u32 = N * 2;

    fn main() {
        let arr: [Field; 2 * 2] = [0_Field; 4];
        let _: [Field; 2 * 2] = arr;
    }
    ");
}

#[test]
fn expands_numeric_type_alias_used_as_value_with_turbofish() {
    let src = r#"
    type AliasN<let N: u32>: u32 = N;

    global N: u32 = 100;

    fn main() {
        let a: u32 = AliasN::<1>;
        assert(a == 1);
        assert(N == 100);
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    type AliasN<let N: u32>: u32 = N;

    global N: u32 = 100;

    fn main() {
        let a: u32 = 1_u32;
        assert(a == 1_u32);
        assert(N == 100_u32);
    }
    ");
}

#[test]
fn expands_associated_constant_over_self_type_with_concrete_annotation() {
    let src = r#"
    trait Foo {
        let N: i32;

        fn n() -> i32 {
            Self::N
        }
    }

    impl Foo for i32 {
        let N: i32 = -12345i32;
    }

    fn main() {
        let _ = i32::n();
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    trait Foo {
        let N: i32;

        fn n() -> i32 {
            Self::N
        }
    }

    impl Foo for i32 {
        let N: i32 = -12345_i32;
    }

    fn main() {
        let _: i32 = <i32 as Foo>::n();
    }
    ");
}

#[test]
fn expands_global_capturing_closure_as_its_initializer_expression() {
    let src = r#"
    fn make() -> fn[(Field,)](Field) -> Field {
        let x: Field = 3;
        |y: Field| -> Field { y + x }
    }

    global F: fn[(Field,)](Field) -> Field = make();

    fn main() {
        let _ = F;
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    fn make() -> fn[(Field,)](Field) -> Field {
        let x: Field = 3_Field;
        |y: Field| -> Field {
            y + x
        }
    }

    global F: fn[(Field,)](Field) -> Field = make();

    fn main() {
        let _: fn[(Field,)](Field) -> Field = F;
    }
    ");
}

#[test]
fn expands_string_literal_with_noir_escapes() {
    let src = r#"
    global S: str<6> = "\r\n\t\0\"\\";

    fn main() {
        let _ = S;
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r#"
    global S: str<6> = "\r\n\t\0\"\\";

    fn main() {
        let _: str<6> = S;
    }
    "#);
}

#[test]
fn expands_string_literal_with_control_character() {
    // Noir's lexer accepts a raw control character inside a string literal, and has no
    // numeric escape to write one with, so it has to be printed back raw.
    let src = "fn main() { let _ = \"a\u{7}b\"; }";
    let expanded = assert_no_errors_and_to_string(src);
    assert!(expanded.contains("let _: str<3> = \"a\u{7}b\";"), "{expanded}");
}

#[test]
fn expands_string_literal_with_combining_mark() {
    let src = "fn main() { let _ = \"e\u{301}\"; }";
    let expanded = assert_no_errors_and_to_string(src);
    assert!(expanded.contains("let _: str<3> = \"e\u{301}\";"), "{expanded}");
}

#[test]
fn expands_global_string_with_control_character_as_its_initializer_expression() {
    // The value prints as a literal holding a raw control character, so the initializer is
    // preferred - which is that same literal, and must still round-trip.
    let src = "global S: str<3> = \"a\u{7}b\";\n\nfn main() { let _ = S; }";
    let expanded = assert_no_errors_and_to_string(src);
    assert!(expanded.contains("global S: str<3> = \"a\u{7}b\";"), "{expanded}");
}

#[test]
fn expands_format_string_with_quote_and_escapes() {
    let src = r#"
    fn main() {
        let _ = f"a\"b{{c}}d\\e\nf";
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r#"
    fn main() {
        let _: fmtstr<13, ()> = f"a\"b{{c}}d\\e\nf";
    }
    "#);
}

#[test]
fn expands_global_format_string_value_with_quote_and_escapes() {
    let src = r#"
    global S: fmtstr<13, ()> = f"a\"b{{c}}d\\e\nf";

    fn main() {
        let _ = S;
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r#"
    global S: fmtstr<13, ()> = f"a\"b{{c}}d\\e\nf";

    fn main() {
        let _: fmtstr<13, ()> = S;
    }
    "#);
}
