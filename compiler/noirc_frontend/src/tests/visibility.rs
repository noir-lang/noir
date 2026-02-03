use crate::tests::{assert_no_errors, check_errors};

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
    check_errors(src);
}

#[test]
fn errors_if_type_alias_aliases_more_private_type() {
    let src = r#"
    struct Foo {}
    pub type Bar = Foo;
    ^^^^^^^^^^^^^^^^^^ Type `Foo` is more private than item `Bar`
    pub fn no_unused_warnings() {
        let _: Bar = Foo {};
    }
    "#;
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    assert_no_errors(src);
}

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
    check_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    "#;
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

#[test]
fn error_if_calling_private_struct_function_from_extension() {
    let src = r#"
    mod foo {
        pub struct Foo {
            z: u32
        }

        impl Foo {
            pub fn new() -> Foo {
                Foo { z: 0 }
            }
            fn x() -> u32 {
                0
            }
            fn y(_self: Self) -> u32 {
                0
            }
            fn e(self: Self) {
                self.private_extension();
                     ^^^^^^^^^^^^^^^^^ private_extension is private and not visible from the current module
                     ~~~~~~~~~~~~~~~~~ private_extension is private
                Self::extension();
            }
        }
    }

    mod ext {
        use super::foo::Foo;
        impl Foo {
            pub fn extension() {
                let f = Foo::new();

                let _x = Foo::x();
                              ^ x is private and not visible from the current module
                              ~ x is private

                let _y = f.y();
                           ^ y is private and not visible from the current module
                           ~ y is private

                let _z = f.z;
                           ^ z is private and not visible from the current module
                           ~ z is private

                f.private_extension();
            }

            fn private_extension(_self: Self) {}
        }
    }

    fn main() {
        let _f = foo::Foo::new();
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_error_when_accessing_private_module_through_super() {
    let src = r#"
    mod foo {
        pub struct Foo {}
        pub struct Qux {}
    }

    mod bar {
        use super::foo::Qux;
        pub fn bar() {
            let _f = super::foo::Foo {};
            let _q = Qux {};
        }
    }

    fn main() {
        bar::bar();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_when_accessing_private_module_through_crate() {
    let src = r#"
    mod foo {
        pub struct Foo {}
        pub struct Qux {}
    }

    mod bar {
        use crate::foo::Qux;
        pub fn bar() {
            let _f = crate::foo::Foo {};
            let _q = Qux {};
        }
    }

    fn main() {
        bar::bar();
    }
    "#;
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    "#;
    check_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    check_errors(src);
}

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
    check_errors(src);
}

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
    assert_no_errors(src);
}

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
    assert_no_errors(src);
}

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
    check_errors(src);
}

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
    check_errors(src);
}

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
    check_errors(src);
}

#[test]
fn private_impl_method_on_another_module_1() {
    let src = r#"
    pub mod bar {
        pub struct Foo<T> {}
    }

    impl<T> bar::Foo<T> {
        fn foo(self) {
            let _ = self;
        }

        fn bar(self) {
            self.foo();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn private_impl_method_on_another_module_2() {
    let src = r#"
    pub mod bar {
        pub struct Foo<T> {}
    }

    impl bar::Foo<i32> {
        fn foo(self) {
            let _ = self;
        }
    }

    impl bar::Foo<i64> {
        fn bar(self) {
            let _ = self;
            let foo = bar::Foo::<i32> {};
            foo.foo();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn checks_visibility_of_trait_related_to_trait_impl_on_method_call() {
    let src = r#"
    mod moo {
        pub struct Bar {}
    }

    trait Foo {
        fn foo(self);
    }

    impl Foo for moo::Bar {
        fn foo(self) {}
    }

    fn main() {
        let bar = moo::Bar {};
        bar.foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn same_name_in_types_and_values_namespace_works() {
    let src = "
    struct foo {}

    fn foo(x: foo) -> foo {
        x
    }

    fn main() {
        let x: foo = foo {};
        let _ = foo(x);
    }
    ";
    assert_no_errors(src);
}

#[test]
fn only_one_private_error_when_name_in_types_and_values_namespace_collides() {
    let src = "
    mod moo {
        struct foo {}

        fn foo() {}
    }

    fn main() {
        let _ = moo::foo {};
                     ^^^ foo is private and not visible from the current module
                     ~~~ foo is private
        x
        ^ cannot find `x` in this scope
        ~ not found in this scope
    }
    ";
    check_errors(src);
}

#[test]
fn databus_only_allowed_in_main() {
    let src = "
fn main(a: u32) -> pub u32 {
    let a = inner(a);
    let c = a << 2;
    c
}

fn inner(a: call_data(0) u32) -> return_data u32 {
   ~~~~~ unnecessary call_data(0)
   ^^^^^ unnecessary call_data(0) attribute for function inner
    a
}
    ";
    check_errors(src);
}
