use crate::tests::{assert_no_errors, check_errors};

#[test]
fn use_super() {
    let src = r#"
    fn some_func() {}

    mod foo {
        use super::some_func;

        pub fn bar() {
            some_func();
        }
    }

    fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_super() {
    let src = "
    use super::some_func;
        ^^^^^ There is no super module
    ";
    check_errors(src);
}

#[test]
fn use_super_in_path() {
    let src = r#"
    fn some_func() {}

    mod foo {
        pub fn func() {
            super::some_func();
        }
    }

    fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
fn can_use_pub_use_item() {
    let src = r#"
    mod foo {
        mod bar {
            pub fn baz() {}
        }

        pub use bar::baz;
    }

    fn main() {
        foo::baz();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn warns_on_re_export_of_item_with_less_visibility() {
    let src = r#"
    mod foo {
        mod bar {
            pub(crate) fn baz() {}
        }

        pub use bar::baz;
                     ^^^ cannot re-export baz because it has less visibility than this use statement
                     ~~~ consider marking baz as pub
    }

    fn main() {
        foo::baz();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_using_alias_in_import() {
    let src = r#"
    mod foo {
        pub type bar = i32;
    }

    use foo::bar::baz;
             ^^^ bar is a type alias, not a module

    fn main() {
    }
    "#;
    check_errors(src);
}
