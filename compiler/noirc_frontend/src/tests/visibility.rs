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
fn errors_on_private_module_accessed_via_use_and_path() {
    let src = r#"
    pub mod foo {
        mod bar {
            pub fn baz() {}
        }
    }

    use foo::bar::baz;
             ^^^ bar is private and not visible from the current module
             ~~~ bar is private

    fn main() {
        foo::bar::baz();
             ^^^ bar is private and not visible from the current module
             ~~~ bar is private
        baz();
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
fn errors_if_trait_impl_associated_type_leaks_private_type() {
    let src = r#"
    struct Priv {}

    pub trait T {
        type Item;
    }

    impl T for u32 {
        type Item = Priv;
             ^^^^ Type `Priv` is more private than item `T::Item`
    }

    pub fn no_unused_warnings() {
        let _ = Priv {};
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_error_if_private_trait_impl_associated_type_uses_private_type() {
    let src = r#"
    struct Priv {}

    trait T {
        type Item;
    }

    impl T for u32 {
        type Item = Priv;
    }

    fn main() {
        let _ = Priv {};
    }
    "#;
    assert_no_errors(src);
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
        pub fn foo() {
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
            pub fn e(self: Self) {
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
fn error_if_calling_private_struct_function_via_self_from_extension() {
    let src = r#"
    mod foo {
        pub struct Foo {}

        impl Foo {
            fn secret() -> u32 {
                42
            }
        }
    }

    mod ext {
        use super::foo::Foo;

        impl Foo {
            pub fn calls_secret_via_self() -> u32 {
                Self::secret()
                      ^^^^^^ secret is private and not visible from the current module
                      ~~~~~~ secret is private
            }
        }
    }

    fn main() {
        let _ = foo::Foo::calls_secret_via_self();
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

        pub fn new() -> Foo {
            Foo { x: 1 }
        }
    }

    fn foo(foo: moo::Foo) -> Field {
        foo.x
            ^ x is private and not visible from the current module
            ~ x is private
    }

    fn main() {
        let _ = foo(moo::new());
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

        pub fn run() -> Field {
            foo(super::Foo { x: 1 })
        }
    }

    fn main() {
        let _ = nested::run();
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
        let _ = foo(moo::Foo { x: 1 });
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

        pub fn new() -> Foo {
            Foo { x: 1 }
        }
    }

    fn foo(foo: moo::Foo) -> Field {
        let moo::Foo { x } = foo;
                       ^ x is private and not visible from the current module
                       ~ x is private
        x
    }

    fn main() {
        let _ = foo(moo::new());
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

        #[allow(dead_code)]
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

        pub fn bar(self) {
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
        pub fn bar(self) {
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
    // `moo::foo {}` constructs the struct (type namespace); the same-named `fn foo` (value
    // namespace) is never called, so it is correctly reported as unused — the two namespaces
    // are tracked independently.
    let src = "
    mod moo {
        struct foo {}

        fn foo() {}
           ^^^ unused function foo
           ~~~ unused function
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
            ~~~~~~~~~~~~ unnecessary call_data(0)
            ^^^^^^^^^^^^ unnecessary call_data(0) attribute for function inner
                                 ~~~~~~~~~~~ unnecessary return_data
                                 ^^^^^^^^^^^ unnecessary return_data attribute for function inner
    a
}
    ";
    check_errors(src);
}

#[test]
fn return_data_not_allowed_on_parameter() {
    let src = "
fn main(a: return_data u32) -> pub u32 {
           ~~~~~~~~~~~ return_data is only allowed on the return value
           ^^^^^^^^^^^ return_data attribute is not allowed on a parameter
    a
}
    ";
    check_errors(src);
}

#[test]
fn call_data_not_allowed_on_return_value() {
    let src = "
fn main(a: u32) -> call_data(0) u32 {
                   ~~~~~~~~~~~~ call_data(0) is only allowed on a parameter
                   ^^^^^^^^^^^^ call_data(0) attribute is not allowed on the return value
    a
}
    ";
    check_errors(src);
}

#[test]
fn unnecessary_pub_on_return_type() {
    let src = "
    pub fn foo() -> pub u32 {
                    ^^^ unnecessary pub keyword on return type for function foo
                    ~~~ unnecessary pub return type
        0
    }
    ";
    check_errors(src);
}

#[test]
fn unnecessary_pub_on_argument() {
    let src = "
    pub fn foo(_: pub u32) {
                  ^^^ unnecessary pub keyword on parameter for function foo
                  ~~~ unnecessary pub parameter
    }
    ";
    check_errors(src);
}

#[test]
fn unnecessary_pub_on_fold_function_parameter() {
    let src = "
    fn main(x: Field) -> pub Field {
        foo(x)
    }

    #[fold]
    fn foo(x: pub Field) -> Field {
              ^^^ unnecessary pub keyword on parameter for function foo
              ~~~ unnecessary pub parameter
        x + 1
    }
    ";
    check_errors(src);
}

#[test]
fn unnecessary_pub_on_fold_function_return_type() {
    let src = "
    fn main(x: Field) -> pub Field {
        foo(x)
    }

    #[fold]
    fn foo(x: Field) -> pub Field {
                        ^^^ unnecessary pub keyword on return type for function foo
                        ~~~ unnecessary pub return type
        x + 1
    }
    ";
    check_errors(src);
}

#[test]
fn errors_if_calling_private_inherent_impl_method_from_outside_impl_module() {
    // Regression test: inherent impl methods defined in a submodule on a type from the parent
    // module should not be callable from outside the impl's defining module.
    let src = r#"
    struct S {}

    mod private {
        struct R { pub x: u32 }

        impl super::S {
            fn get_r() -> R {
                R { x: 1 }
            }
        }
    }

    fn main() {
        let _ = S::get_r();
                   ^^^^^ get_r is private and not visible from the current module
                   ~~~~~ get_r is private
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_calling_private_inherent_impl_method_via_dot_notation_from_outside_impl_module() {
    let src = r#"
    struct S { x: u32 }

    mod private {
        impl super::S {
            fn secret(self) -> u32 {
                self.x
            }
        }
    }

    fn main() {
        let s = S { x: 1 };
        let _ = s.secret();
                  ^^^^^^ secret is private and not visible from the current module
                  ~~~~~~ secret is private
    }
    "#;
    check_errors(src);
}

#[test]
fn allows_pub_inherent_impl_method_from_outside_impl_module() {
    // Public methods on inherent impls in submodules should remain callable.
    let src = r#"
    struct S {}

    mod private {
        impl super::S {
            pub fn public_method() -> u32 {
                42
            }
        }
    }

    fn main() {
        let _ = S::public_method();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn allows_private_inherent_impl_method_via_dot_from_within_impl_block() {
    // Private methods should still be callable via dot notation from within the same impl block.
    let src = r#"
    struct S {}

    mod private {
        impl super::S {
            fn secret(self) -> u32 {
                let _ = self;
                42
            }

            pub fn public_wrapper(self) -> u32 {
                self.secret()
            }
        }
    }

    fn main() {
        let s = S {};
        let _ = s.public_wrapper();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_once_not_twice_for_private_inherent_impl_method_in_separate_modules() {
    // Regression test: when the struct and impl are in different modules (neither is the
    // caller's module), we should only get ONE "private" error, not two.
    let src = r#"
    mod types {
        pub struct S {}
    }

    mod impls {
        impl super::types::S {
            fn secret() -> u32 {
                42
            }
        }
    }

    fn main() {
        let _ = types::S::secret();
                          ^^^^^^ secret is private and not visible from the current module
                          ~~~~~~ secret is private
    }
    "#;
    check_errors(src);
}

#[test]
fn private_inherent_impl_method_accessible_via_dot_notation_from_impl_module() {
    let src = r#"
    mod types {
        pub struct S {
            pub x: u32,
        }
    }

    mod impls {
        impl super::types::S {
            fn secret(self) -> u32 {
                self.x
            }
        }

        pub fn caller() -> u32 {
            let s = super::types::S { x: 1 };
            s.secret()
        }
    }

    fn main() {
        let _ = impls::caller();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn private_inherent_impl_method_accessible_via_qualified_path_from_impl_module() {
    let src = r#"
    mod types {
        pub struct S {}
    }

    mod impls {
        impl super::types::S {
            fn secret() -> u32 {
                42
            }
        }

        pub fn caller() -> u32 {
            super::types::S::secret()
        }
    }

    fn main() {
        let _ = impls::caller();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn private_inherent_impl_method_accessible_from_nested_child_of_impl_module() {
    let src = r#"
    mod moo {
        pub struct S {
            pub x: u32,
        }
    }

    mod private {
        impl crate::moo::S {
            fn one() -> u32 {
                1
            }
            fn two(self) -> u32 {
                self.x
            }
        }

        mod nested {
            pub fn foo() -> u32 {
                let s = crate::moo::S { x: 1 };
                crate::moo::S::one() + s.two()
            }
        }

        pub fn caller() -> u32 {
            nested::foo()
        }
    }

    fn main() {
        let _ = private::caller();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_calling_private_methods_from_nested_extension_module() {
    // Private methods defined in an extension `impl` in module `inner` are callable from
    // another extension `impl` in `inner2`, because `inner2` is a descendant of `inner`. This
    // matches Rust: a descendant module can see its ancestors' private associated items
    // regardless of which `impl` block holds them.
    //
    // Each kind of method is checked by a different mechanism: the no-`self` associated
    // function (`Foo::inner_x()`) during path resolution, and the `self` method
    // (`self.inner_y()`) during method-call resolution. The parent/child rule must be enforced
    // identically by both.
    //
    // The `self` half is the reason this test exists alongside
    // `private_inherent_impl_method_accessible_from_nested_child_of_impl_module`: there the
    // descendant caller is a free function, so `self_type` is `None` and the dot-call takes the
    // `struct_member_is_visible` branch. Here the caller is itself inside `impl Foo`, so
    // `self_type` is `Some`, exercising the strict parent/child case of the `self_type` branch
    // in `method_call_is_visible` that no other test covers.
    let src = r#"
    mod foo {
        pub struct Foo {}

        mod inner {
            use crate::foo::Foo;

            impl Foo {
                fn inner_x() -> u32 {
                    0
                }

                fn inner_y(self) -> u32 {
                    let _ = self;
                    0
                }
            }

            mod inner2 {
                use crate::foo::Foo;

                impl Foo {
                    pub fn x(self) -> u32 {
                        Foo::inner_x() + self.inner_y()
                    }
                }
            }
        }
    }

    fn main() {
        let f = foo::Foo {};
        let _ = f.x();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_calling_private_methods_from_sibling_extension_module() {
    // Mirror of the descendant case: when the calling extension `impl` is in `inner2`, a
    // sibling of `inner` rather than a descendant, neither the no-`self` associated function
    // (checked during path resolution) nor the `self` method (checked during method-call
    // resolution) is visible. Both mechanisms enforce the same parent/child rule.
    let src = r#"
    mod foo {
        pub struct Foo {}

        mod inner {
            use crate::foo::Foo;

            impl Foo {
                fn inner_x() -> u32 {
                    0
                }

                fn inner_y(self) -> u32 {
                    let _ = self;
                    0
                }
            }
        }

        mod inner2 {
            use crate::foo::Foo;

            impl Foo {
                pub fn x(self) -> u32 {
                    let a = Foo::inner_x();
                                 ^^^^^^^ inner_x is private and not visible from the current module
                                 ~~~~~~~ inner_x is private
                    let b = self.inner_y();
                                 ^^^^^^^ inner_y is private and not visible from the current module
                                 ~~~~~~~ inner_y is private
                    a + b
                }
            }
        }
    }

    fn main() {
        let f = foo::Foo {};
        let _ = f.x();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_when_using_private_type_imported_via_value_name_collision() {
    // A module has a private type and a public value sharing the same name.
    // Importing the name is allowed (the public value is visible), but using
    // the private type must still be rejected.
    let src = r#"
    mod moo {
        struct Foo {}

        #[allow(dead_code)]
        pub fn Foo() {}
    }

    use moo::Foo;

    fn main() {
        let _ = Foo {};
                ^^^ Foo is private and not visible from the current module
                ~~~ Foo is private
    }
    "#;
    check_errors(src);
}

#[test]
fn allows_importing_value_when_colliding_type_is_public() {
    // The mirror of the collision case: when both the type and the value are
    // visible, importing and using either must keep working.
    let src = r#"
    mod moo {
        pub struct Foo {}

        pub fn Foo() {}
    }

    use moo::Foo;

    fn main() {
        let _ = Foo {};
        Foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn allows_calling_value_when_using_private_type_imported_via_collision_still_errors() {
    // The public value of the collision is still usable; only the private type is rejected.
    let src = r#"
    mod moo {
        struct Foo {}

        #[allow(dead_code)]
        pub fn Foo() {}
    }

    use moo::Foo;

    fn main() {
        Foo();
        let _ = Foo {};
                ^^^ Foo is private and not visible from the current module
                ~~~ Foo is private
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_when_using_private_value_imported_via_type_name_collision() {
    // Mirror of the type/value collision: a private value and a public type share a name.
    // Importing is allowed (the type is visible), but calling the private value is rejected.
    let src = r#"
    mod moo {
        pub struct Foo {}

        fn Foo() {}
    }

    use moo::Foo;

    fn main() {
        let _ = Foo {};
        Foo();
        ^^^ Foo is private and not visible from the current module
        ~~~ Foo is private
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_when_using_private_type_imported_via_aliased_collision() {
    // Aliasing the import must not launder the private type into scope either.
    let src = r#"
    mod moo {
        struct Foo {}

        #[allow(dead_code)]
        pub fn Foo() {}
    }

    use moo::Foo as Leaked;

    fn main() {
        let _ = Leaked {};
                ^^^^^^ Leaked is private and not visible from the current module
                ~~~~~~ Leaked is private
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_qualified_access_to_private_type_colliding_with_public_value() {
    // Direct qualified access to the private type is rejected regardless of the import path.
    let src = r#"
    mod moo {
        struct Foo {}

        #[allow(dead_code)]
        pub fn Foo() {}
    }

    fn main() {
        let _ = moo::Foo {};
                     ^^^ Foo is private and not visible from the current module
                     ~~~ Foo is private
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_when_using_private_type_imported_via_collision_as_path_prefix() {
    // The private type must be rejected even when it only appears as an intermediate path
    // prefix (calling an inherent associated function), not just as the final path item.
    let src = r#"
    mod moo {
        struct Foo {}

        impl Foo {
            pub fn make() -> Self { Foo {} }
        }

        pub fn Foo() {}
    }

    use moo::Foo;

    fn main() {
        let _ = Foo::make();
                ^^^ Foo is private and not visible from the current module
                ~~~ Foo is private
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_at_import_when_both_colliding_items_are_private() {
    // When a name resolves to a private item in both namespaces there is no visible item to make
    // the import legal, so the error is reported at the `use` itself (and only once), rather than
    // being deferred to the use site.
    let src = r#"
    mod moo {
        struct Foo {}

        fn Foo() {}
    }

    use moo::Foo;
             ^^^ Foo is private and not visible from the current module
             ~~~ Foo is private

    fn main() {
        let _ = Foo {};
        Foo();
    }
    "#;
    check_errors(src);
}

#[test]
fn calls_public_trait_method_when_inherent_method_is_private() {
    // `Foo` has both an inherent `bar` (private to `impls`) and a trait `Bar::bar` (public and
    // in scope). The inherent method is not accessible from the root, so the call must resolve
    // to the public trait method instead of erroring that the inherent method is private.
    // The inherent method is still callable from within `impls`, where it is visible.
    let src = r#"
    pub struct Foo {}

    trait Bar {
        fn bar(self) -> u32;
    }

    mod impls {
        use super::{Bar, Foo};

        impl Foo {
            fn bar(self) -> u32 {
                let _ = self;
                1
            }
        }

        impl Bar for Foo {
            fn bar(self) -> u32 {
                let _ = self;
                2
            }
        }

        pub fn calls_inherent_bar(foo: Foo) -> u32 {
            foo.bar()
        }
    }

    fn main() {
        let _ = (Foo {}).bar();
        let _ = impls::calls_inherent_bar(Foo {});
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn calls_in_scope_trait_method_when_clashing_trait_is_out_of_scope() {
    // Two traits define `bar` for `Foo`, but only `A` is in scope at the call site (the trait
    // `B` is declared inside `hidden` and never imported). The call resolves to `A::bar`
    // without a "multiple applicable items" error.
    let src = r#"
    struct Foo {}

    trait A { fn bar(self) -> u32; }

    impl A for Foo { fn bar(self) -> u32 { 1 } }

    mod hidden {
        use super::Foo;
        pub trait B { fn bar(self) -> u32; }
        impl B for Foo { fn bar(self) -> u32 { 2 } }
    }

    fn main() {
        let _ = (Foo {}).bar();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn private_inherent_method_falls_back_to_out_of_scope_trait_error() {
    // The inherent `bar` is private to `impls` and the only trait providing `bar` is not in
    // scope. Because the inaccessible inherent method is not a resolution candidate here, the
    // call is treated as if only the trait method existed: the user is told to import the
    // trait rather than getting a misleading "method is private" error.
    let src = r#"
    pub struct Foo {}

    mod hidden {
        use super::Foo;
        pub trait B { fn bar(self) -> u32; }
        impl B for Foo { fn bar(self) -> u32 { let _ = self; 2 } }
    }

    mod impls {
        use super::Foo;
        impl Foo {
            fn bar(self) -> u32 { let _ = self; 1 }
        }

        pub fn calls_inherent_bar(foo: Foo) -> u32 {
            foo.bar()
        }
    }

    fn main() {
        let _ = impls::calls_inherent_bar(Foo {});
        let _ = (Foo {}).bar();
                ^^^^^^^^^^^^^^ trait `hidden::B` which provides `bar` is implemented but not in scope, please import it
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_when_two_inherent_methods_with_same_name_exist_in_different_modules() {
    // Two inherent `impl Foo` blocks in different modules both defining `bar` is an overlapping
    // impl error, regardless of where they live; there is no single inherent method to call.
    let src = r#"
    struct Foo {}

    mod a {
        impl super::Foo {
            pub fn bar(self) -> u32 {
                   ~~~ Previous impl defined here
                let _ = self;
                1
            }
        }
    }

    mod b {
        impl super::Foo {
            pub fn bar(self) -> u32 {
                   ^^^ Impl for type `Foo` overlaps with existing impl
                   ~~~ Overlapping impl
                let _ = self;
                2
            }
        }
    }

    fn main() {
        let _ = (Foo {}).bar();
    }
    "#;
    check_errors(src);
}

#[test]
fn private_module_is_accessible_from_within_its_parent() {
    // A private module's public items are reachable from the module it is declared in (and that
    // module's descendants), exactly like any other private item. Here `inner` is private to
    // `outer`, and `outer::bar` (inside `outer`) accesses `inner::foo` through a fully-qualified
    // path. This is allowed in Rust, but is currently rejected with "inner is private".
    let src = r#"
    mod outer {
        mod inner {
            pub fn foo() {}
        }

        pub fn bar() {
            crate::outer::inner::foo();
        }
    }

    fn main() {
        outer::bar();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_path_can_access_private_intermediate_module_from_within_its_parent() {
    // The `use` resolver must apply the same intermediate-segment visibility rule as qualified
    // paths: a private module is reachable from the module it is declared in. Here `inner` is
    // private to `outer`, so `use crate::outer::inner::foo;` inside `outer` is valid (the direct
    // path `crate::outer::inner::foo()` already compiles). This was rejected as "inner is private".
    let src = r#"
    mod outer {
        mod inner {
            pub fn foo() {}
        }

        use crate::outer::inner::foo;

        pub fn bar() {
            foo();
        }
    }

    fn main() {
        outer::bar();
    }
    "#;
    assert_no_errors(src);
}
