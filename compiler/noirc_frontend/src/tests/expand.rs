//! Tests for `nargo expand` output (via the HIR printer), focusing on faithfully
//! reconstructing impls: their generics and where clauses.

use crate::tests::assert_no_errors_and_to_string;

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
        let _ = impls::Foo::bar(Foo {});
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
