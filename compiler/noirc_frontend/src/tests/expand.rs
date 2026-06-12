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
