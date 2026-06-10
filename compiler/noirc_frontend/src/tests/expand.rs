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
    insta::assert_snapshot!(expanded);
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
    insta::assert_snapshot!(expanded);
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
    insta::assert_snapshot!(expanded);
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
    insta::assert_snapshot!(expanded);
}
