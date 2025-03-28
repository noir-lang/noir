use crate::tests::{assert_no_errors, check_errors};

#[test]
fn errors_on_unused_private_import() {
    let src = r#"
    mod foo {
        pub fn bar() {}
        pub fn baz() {}

        pub trait Foo {
        }
    }

    use foo::bar;
             ^^^ unused import bar
             ~~~ unused import
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_pub_crate_import() {
    let src = r#"
    mod foo {
        pub fn bar() {}
        pub fn baz() {}

        pub trait Foo {
        }
    }

    pub(crate) use foo::bar;
                        ^^^ unused import bar
                        ~~~ unused import
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_function() {
    let src = r#"
    contract some_contract {
        // This function is unused, but it's a contract entrypoint
        // so it should not produce a warning
        fn foo() -> pub Field {
            1
        }
    }


    fn foo() {
       ^^^ unused function foo
       ~~~ unused function
        bar();
    }

    fn bar() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_struct() {
    let src = r#"
    struct Foo {}
           ^^^ struct `Foo` is never constructed
           ~~~ struct is never constructed
    struct Bar {}

    fn main() {
        let _ = Bar {};
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_trait() {
    let src = r#"
    trait Foo {}
          ^^^ unused trait Foo
          ~~~ unused trait
    trait Bar {}

    pub struct Baz {
    }

    impl Bar for Baz {}

    fn main() {
    }
    "#;
    check_errors(src);
}

#[test]
fn silences_unused_variable_warning() {
    let src = r#"
    fn main() {
        #[allow(unused_variables)]
        let x = 1;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_on_unused_type_alias() {
    let src = r#"
    type Foo = Field;
         ^^^ unused type alias Foo
         ~~~ unused type alias
    type Bar = Field;
    pub fn bar(_: Bar) {}
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn warns_on_unused_global() {
    let src = r#"
    global foo: u32 = 1;
           ^^^ unused global foo
           ~~~ unused global
    global bar: Field = 1;

    fn main() {
        let _ = bar;
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_warn_on_unused_global_if_it_has_an_abi_attribute() {
    let src = r#"
    contract foo {
        #[abi(notes)]
        global bar: u64 = 1;
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unused_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    #[abi(dummy)]
    struct Foo { bar: u8 }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unused_function_if_it_has_an_export_attribute() {
    let src = r#"
    #[export]
    fn foo() {}

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_inner_struct_when_parent_is_used() {
    let src = r#"
    struct Bar {
        inner: [Field; 3],
    }

    struct Foo {
        a: Field,
        bar: Bar,
    }

    fn main(foos: [Foo; 1]) {
        assert_eq(foos[0].a, 10);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    #[abi(functions)]
    struct Foo {
        a: Field,
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_indirect_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    struct Bar {
        field: Field,
    }

    #[abi(functions)]
    struct Foo {
        bar: Bar,
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_self_in_trait_impl() {
    let src = r#"
    struct Bar {}

    trait Foo {
        fn foo(self, a: u64);
    }

    impl Foo for Bar {
        fn foo(self, _a: u64) {}
    }

    fn main() {
        let _ = Bar {};
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolves_trait_where_clause_in_the_correct_module() {
    // This is a regression test for https://github.com/noir-lang/noir/issues/6479
    let src = r#"
    mod foo {
        pub trait Foo {}
    }

    use foo::Foo;

    pub trait Bar<T>
    where
        T: Foo,
    {}

    fn main() {}
    "#;
    assert_no_errors(src);
}
