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
