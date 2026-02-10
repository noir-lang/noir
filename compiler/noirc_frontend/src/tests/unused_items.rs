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
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unused_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    #[abi(dummy)]
    struct Foo { bar: u8 }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unused_function_if_it_has_an_export_attribute() {
    let src = r#"
    #[export]
    fn foo() {}
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
    "#;
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_impl_method_is_called() {
    let src = "
    struct Bar {}

    impl Bar {
        fn foo() {}
    }

    pub fn main() {
        Bar::foo()
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_trait_method_is_called() {
    let src = "
    struct Bar {}

    pub trait Foo {
        fn foo();
    }

    impl Foo for Bar {
        fn foo() {}
    }

    pub fn main() {
        Bar::foo()
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_mentioned_in_let_type() {
    let src = "
    struct Bar {}

    fn foo(array: [Bar; 1]) {
        let _: Bar = array[0];
    }

    fn main() {
        let _ = foo;
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_mentioned_in_return_type() {
    let src = "
    struct Bar {}

    fn foo(array: [Bar; 1]) -> Bar {
        array[0]
    }

    fn main() {
        let _ = foo;
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_passed_in_generic_args_in_constructor() {
    let src = "
    struct Bar {}

    struct Generic<T> {}

    fn main() {
        let _ = Generic::<Bar> {};
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_passed_in_generic_args_in_function_call() {
    let src = "
    struct Bar {}

    fn foo<T>() {}

    fn main() {
        let _ = foo::<Bar>();
    }
    ";
    assert_no_errors(src);
}

#[test]
fn does_not_consider_struct_as_constructed_if_mentioned_in_function_argument() {
    let src = "
    struct Bar {}
           ^^^ struct `Bar` is never constructed
           ~~~ struct is never constructed

    fn foo(_: [Bar; 1]) {}

    fn main() {
        foo();
        ^^^^^ Function expects 1 parameter but 0 were given
    }
    ";
    check_errors(src);
}

#[test]
fn allow_dead_code_on_unused_function() {
    let src = "
    #[allow(dead_code)]
    fn foo() {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn allow_dead_code_on_unused_struct() {
    let src = "
    #[allow(dead_code)]
    struct Foo {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn allow_dead_code_on_unused_trait() {
    let src = "
    #[allow(dead_code)]
    trait Foo {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn allow_dead_code_on_unused_enum() {
    let src = "
    #[allow(dead_code)]
    enum Foo {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}
