use crate::{assert_no_errors, check_errors};

#[named]
#[test]
fn errors_once_on_unused_import_that_is_not_accessible() {
    // Tests that we don't get an "unused import" here given that the import is not accessible
    let src = r#"
        mod moo {
            struct Foo {}
        }
        use moo::Foo;
                 ^^^ Foo is private and not visible from the current module
                 ~~~ Foo is private
        fn main() {
            let _ = Foo {};
        }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_type_alias_aliases_more_private_type() {
    let src = r#"
    struct Foo {}
    pub type Bar = Foo;
    ^^^^^^^^^^^^^^^^^^ Type `Foo` is more private than item `Bar`
    pub fn no_unused_warnings() {
        let _: Bar = Foo {};
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_type_alias_aliases_more_private_type_in_generic() {
    let src = r#"
    pub struct Generic<T> { value: T }
    struct Foo {}
    pub type Bar = Generic<Foo>;
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^ Type `Foo` is more private than item `Bar`
    pub fn no_unused_warnings() {
        let _ = Foo {};
        let _: Bar = Generic { value: Foo {} };
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_pub_type_alias_leaks_private_type_in_generic() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub struct Foo<T> { pub value: T }
        pub type FooBar = Foo<Bar>;
        ^^^^^^^^^^^^^^^^^^^^^^^^^^ Type `Bar` is more private than item `FooBar`

        pub fn no_unused_warnings() {
            let _: FooBar = Foo { value: Bar {} };
        }
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_pub_struct_field_leaks_private_type_in_generic() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub struct Foo<T> { pub value: T }
        pub struct FooBar { pub value: Foo<Bar> }
                                ^^^^^ Type `Bar` is more private than item `FooBar::value`

        pub fn no_unused_warnings() {
            let _ = FooBar { value: Foo { value: Bar {} } };
        }
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_pub_function_leaks_private_type_in_return() {
    let src = r#"
    pub mod moo {
        struct Bar {}

        pub fn bar() -> Bar {
               ^^^ Type `Bar` is more private than item `bar`
            Bar {}
        }
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_pub_function_leaks_private_type_in_arg() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub fn bar(_bar: Bar) {}
               ^^^ Type `Bar` is more private than item `bar`

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_pub_function_is_on_private_struct() {
    let src = r#"
    pub mod moo {
        struct Bar {}

        impl Bar { 
            pub fn bar() -> Bar { 
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_pub_function_on_pub_struct_returns_private() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub struct Foo {}

        impl Foo { 
            pub fn bar() -> Bar { 
                   ^^^ Type `Bar` is more private than item `bar`
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Foo {};            
        }
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_pub_trait_is_defined_on_private_struct() {
    let src = r#"
    pub mod moo {
        struct Bar {}

        pub trait Foo { 
            fn foo() -> Self;
        }

        impl Foo for Bar {
            fn foo() -> Self { 
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_pub_trait_returns_private_struct() {
    let src = r#"
    pub mod moo {
        struct Bar {}

        pub trait Foo { 
            fn foo() -> Bar;
               ^^^ Type `Bar` is more private than item `foo`
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_trait_with_default_visibility_returns_struct_with_default_visibility() {
    let src = r#"
    struct Foo {}

    trait Bar {
        fn bar(self) -> Foo;
    }

    impl Bar for Foo {
        fn bar(self) -> Foo {
            self
        }
    }

    fn main() {
        let foo = Foo {};
        let _ = foo.bar();
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_trying_to_access_public_function_inside_private_module() {
    let src = r#"
    mod foo {
        mod bar {
            pub fn baz() {}
        }
    }
    fn main() {
        foo::bar::baz()
             ^^^ bar is private and not visible from the current module
             ~~~ bar is private
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_calling_private_struct_method() {
    let src = r#"
    mod moo {
        pub struct Foo {}

        impl Foo {
            fn bar(self) {
                let _ = self;
            }
        }
    }

    pub fn method(foo: moo::Foo) {
        foo.bar()
            ^^^ bar is private and not visible from the current module
            ~~~ bar is private
    }

    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_warn_if_calling_pub_crate_struct_method_from_same_crate() {
    let src = r#"
    mod moo {
        pub struct Foo {}

        impl Foo {
            pub(crate) fn bar(self) {
                let _ = self;
            }
        }
    }

    pub fn method(foo: moo::Foo) {
        foo.bar()
    }

    fn main() {}
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_calling_private_struct_function_from_same_struct() {
    let src = r#"
    struct Foo {

    }

    impl Foo {
        fn foo() {
            Foo::bar()
        }

        fn bar() {}
    }

    fn main() {
        let _ = Foo {};
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_calling_private_struct_function_from_same_module() {
    let src = r#"
    struct Foo;

    impl Foo {
        fn bar() -> Field {
            0
        }
    }

    fn main() {
        let _ = Foo {};
        assert_eq(Foo::bar(), 0);
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn error_when_accessing_private_struct_field() {
    let src = r#"
    mod moo {
        pub struct Foo {
            x: Field
        }
    }

    fn foo(foo: moo::Foo) -> Field {
        foo.x
            ^ x is private and not visible from the current module
            ~ x is private
    }

    fn main() {}
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_error_when_accessing_private_struct_field_from_nested_module() {
    let src = r#"
    struct Foo {
        x: Field
    }

    mod nested {
        fn foo(foo: super::Foo) -> Field {
            foo.x
        }
    }

    fn main() {
        let _ = Foo { x: 1 };
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn does_not_error_when_accessing_pub_crate_struct_field_from_nested_module() {
    let src = r#"
    mod moo {
        pub(crate) struct Foo {
            pub(crate) x: Field
        }
    }

    fn foo(foo: moo::Foo) -> Field {
        foo.x
    }

    fn main() {
        let _ = moo::Foo { x: 1 };
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn error_when_using_private_struct_field_in_constructor() {
    let src = r#"
    mod moo {
        pub struct Foo {
            x: Field
        }
    }

    fn main() {
        let _ = moo::Foo { x: 1 };
                           ^ x is private and not visible from the current module
                           ~ x is private
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn error_when_using_private_struct_field_in_struct_pattern() {
    let src = r#"
    mod moo {
        pub struct Foo {
            x: Field
        }
    }

    fn foo(foo: moo::Foo) -> Field {
        let moo::Foo { x } = foo;
                       ^ x is private and not visible from the current module
                       ~ x is private
        x
    }

    fn main() {
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn does_not_error_if_referring_to_top_level_private_module_via_crate() {
    let src = r#"
    mod foo {
        pub fn bar() {}
    }

    use crate::foo::bar;

    fn main() {
        bar()
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn visibility_bug_inside_comptime() {
    let src = r#"
    mod foo {
        pub struct Foo {
            inner: Field,
        }
    
        impl Foo {
            pub fn new(inner: Field) -> Self {
                Self { inner }
            }
        }
    }
    
    use foo::Foo;
    
    fn main() {
        let _ = Foo::new(5);
        let _ = comptime { Foo::new(5) };
    }
    "#;
    assert_no_errors!(src);
}

#[named]
#[test]
fn errors_if_accessing_private_struct_member_inside_comptime_context() {
    let src = r#"
    mod foo {
        pub struct Foo {
            inner: Field,
        }
    
        impl Foo {
            pub fn new(inner: Field) -> Self {
                Self { inner }
            }
        }
    }
    
    use foo::Foo;
    
    fn main() {
        comptime { 
            let foo = Foo::new(5);
            let _ = foo.inner;
                        ^^^^^ inner is private and not visible from the current module
                        ~~~~~ inner is private
        };
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_if_accessing_private_struct_member_inside_function_generated_at_comptime() {
    let src = r#"
    mod foo {
        pub struct Foo {
            foo_inner: Field,
        }
    }

    use foo::Foo;

    #[generate_inner_accessor]
    ~~~~~~~~~~~~~~~~~~~~~~~~~~ While running this function attribute
    struct Bar {
        bar_inner: Foo,
    }

    comptime fn generate_inner_accessor(_s: TypeDefinition) -> Quoted {
        quote {
            fn bar_get_foo_inner(x: Bar) -> Field {
                x.bar_inner.foo_inner
                            ^^^^^^^^^ foo_inner is private and not visible from the current module
                            ~~~~~~~~~ foo_inner is private
            }
        }
    }

    fn main(x: Bar) {
        let _ = bar_get_foo_inner(x);
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn errors_on_use_of_private_exported_item() {
    let src = r#"
    mod foo {
        mod bar {
            pub fn baz() {}
        }

        use bar::baz;

        pub fn qux() {
            baz();
        }
    }

    fn main() {
        foo::baz();
             ^^^ baz is private and not visible from the current module
             ~~~ baz is private
    }
    "#;
    check_errors!(src);
}
