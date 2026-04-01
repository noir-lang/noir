use crate::tests::check_errors;

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_struct_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: Foo) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
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
fn type_mismatch_same_name_different_fully_qualified_name_generic_case() {
    let src = r#"
    pub struct Gen<T> {}

    mod moo {
        use super::Gen;

        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: Gen<Foo>) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(Gen::<moo2::Foo> {});
                 ^^^^^^^^^^^^^^^^^^^ Expected type Gen<Foo>, found type Gen<Foo>
                 ~~~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_tuple_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: (Foo, i32)) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo((moo2::Foo {}, 1));
                 ^^^^^^^^^^^^^^^^^ Expected type (Foo, i32), found type (Foo, Field)
                 ~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_array_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: [Foo; 1]) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo([moo2::Foo {}]);
                 ^^^^^^^^^^^^^^ Expected type [Foo; 1], found type [Foo; 1]
                 ~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_vector_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: [Foo]) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(@[moo2::Foo {}]);
                 ^^^^^^^^^^^^^^^ Expected type [Foo], found type [Foo]
                 ~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_reference_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: &mut Foo) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(&mut moo2::Foo {});
                 ^^^^^^^^^^^^^^^^^ Expected type &mut Foo, found type &mut Foo
                 ~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}
