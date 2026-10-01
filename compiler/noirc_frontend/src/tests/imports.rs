use crate::tests::{assert_no_errors, check_errors};

#[test]
fn use_super() {
    let src = r#"
    fn some_func() {}

    mod foo {
        use super::some_func;

        #[allow(dead_code)]
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
        #[allow(dead_code)]
        pub fn func() {
            super::some_func();
        }
    }

    fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_super_super() {
    let src = r#"
    fn some_func() {}

    mod foo {
        mod bar {
            use super::super::some_func;

            #[allow(dead_code)]
            pub fn baz() {
                some_func();
            }
        }
    }

    fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_super_super_in_path() {
    let src = r#"
    fn some_func() {}

    mod foo {
        mod bar {
            #[allow(dead_code)]
            pub fn func() {
                super::super::some_func();
            }
        }
    }

    fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_super_super() {
    // `foo` is only one level deep, so `super::super` walks past the crate root.
    let src = "
    mod foo {
        use super::super::some_func;
            ^^^^^ There is no super module
    }
    ";
    check_errors(src);
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

#[test]
fn errors_if_importing_trait_method() {
    // Like Rust, a trait's methods can't be imported through the trait. The method is still
    // callable via a qualified path (`Trait::method(..)`).
    let src = r#"
    mod foo {
        pub trait Trait {
            fn method(self);
        }
    }

    use foo::Trait::method;
             ^^^^^ Trait is a trait, not a module

    fn main() {
    }
    "#;
    check_errors(src);
}

#[test]
fn private_use_reexports_that_comes_later() {
    let src = r#"
    mod history {
        use crate::note::Note;

        pub fn foo() {
            let _ = Note {};
        }
    }
    mod note {
        mod retrieved_history {
            pub struct Note {}
        }
        pub use retrieved_history::Note;
    }

    fn main() {
        history::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn pub_use_reexports_that_comes_later() {
    let src = r#"
    mod history {
        pub use crate::note::Note;
    }
    mod note {
        mod retrieved_history {
            pub struct Note {}
        }
        pub use retrieved_history::Note;
    }

    fn main() {
        let _ = history::Note {};
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn invalid_mod_crate_path() {
    let src = r#"
mod crate::mod;
    ^^^^^ Expected an identifier but found 'crate'
         ^^ Expected an item but found '::'
              ^ Expected an identifier but found ';'

fn main() {
}
"#;
    check_errors(src);
}
