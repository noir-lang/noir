use crate::tests::check_errors;

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_when_passed_as_type() {
    let src = r#"
    mod moo {
        pub struct Foo {}

        pub fn foo(_: Foo) {}
    }

    mod moo2 {
        pub struct Foo {}
    }

    fn main() {
        moo::foo(moo2::Foo {});
                 ^^^^^^^^^^^^ Expected type Foo, found type Foo
                 ~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_when_passed_as_generics() {
    let src = r#"
    pub struct Gen<T> {}

    mod moo {
        use super::Gen;

        pub struct Foo {}

        pub fn foo(_: Gen<Foo>) {}
    }

    mod moo2 {
        pub struct Foo {}
    }

    fn main() {
        moo::foo(Gen::<moo2::Foo> {});
                 ^^^^^^^^^^^^^^^^^^^ Expected type Gen<Foo>, found type Gen<Foo>
                 ~~~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}
