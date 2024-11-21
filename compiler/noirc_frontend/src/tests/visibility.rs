use crate::{
    hir::{
        def_collector::{dc_crate::CompilationError, errors::DefCollectorErrorKind},
        resolution::{errors::ResolverError, import::PathResolutionError},
    },
    tests::{assert_no_errors, get_program_errors},
};

#[test]
fn errors_once_on_unused_import_that_is_not_accessible() {
    // Tests that we don't get an "unused import" here given that the import is not accessible
    let src = r#"
        mod moo {
            struct Foo {}
        }
        use moo::Foo;
        fn main() {
            let _ = Foo {};
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::DefinitionError(DefCollectorErrorKind::PathResolutionError(
            PathResolutionError::Private { .. }
        ))
    ));
}

fn assert_type_is_more_private_than_item_error(src: &str, private_typ: &str, public_item: &str) {
    let errors = get_program_errors(src);

    assert!(!errors.is_empty(), "expected visibility error, got nothing");
    for (error, _) in &errors {
        let CompilationError::ResolverError(ResolverError::TypeIsMorePrivateThenItem {
            typ,
            item,
            ..
        }) = error
        else {
            panic!("Expected a type vs item visibility error, got {}", error);
        };

        assert_eq!(typ, private_typ);
        assert_eq!(item, public_item);
    }
    assert_eq!(errors.len(), 1, "only expected one error");
}

#[test]
fn errors_if_type_alias_aliases_more_private_type() {
    let src = r#"
    struct Foo {}
    pub type Bar = Foo;
    pub fn no_unused_warnings() {
        let _: Bar = Foo {};
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Foo", "Bar");
}

#[test]
fn errors_if_type_alias_aliases_more_private_type_in_generic() {
    let src = r#"
    pub struct Generic<T> { value: T }
    struct Foo {}
    pub type Bar = Generic<Foo>;
    pub fn no_unused_warnings() {
        let _ = Foo {};
        let _: Bar = Generic { value: Foo {} };
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Foo", "Bar");
}

#[test]
fn errors_if_pub_type_alias_leaks_private_type_in_generic() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub struct Foo<T> { pub value: T }
        pub type FooBar = Foo<Bar>;

        pub fn no_unused_warnings() {
            let _: FooBar = Foo { value: Bar {} };
        }
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Bar", "FooBar");
}

#[test]
fn errors_if_pub_struct_field_leaks_private_type_in_generic() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub struct Foo<T> { pub value: T }
        pub struct FooBar { pub value: Foo<Bar> }

        pub fn no_unused_warnings() {
            let _ = FooBar { value: Foo { value: Bar {} } };
        }
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Bar", "FooBar::value");
}

#[test]
fn errors_if_pub_function_leaks_private_type_in_return() {
    let src = r#"
    pub mod moo {
        struct Bar {}

        pub fn bar() -> Bar {
            Bar {}
        }
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Bar", "bar");
}

#[test]
fn errors_if_pub_function_leaks_private_type_in_arg() {
    let src = r#"
    pub mod moo {
        struct Bar {}
        pub fn bar(_bar: Bar) {}

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Bar", "bar");
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
    fn main() {}
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
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Foo {};            
        }
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Bar", "bar");
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
    fn main() {}
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
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    "#;
    assert_type_is_more_private_than_item_error(src, "Bar", "foo");
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
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "bar");
}

#[test]
fn warns_if_calling_private_struct_method() {
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
    }

    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "bar");
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

    fn main() {}
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
    }

    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "x");
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
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "x");
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
        x
    }

    fn main() {
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "x");
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
        };
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "inner");
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
    struct Bar {
        bar_inner: Foo,
    }

    comptime fn generate_inner_accessor(_s: StructDefinition) -> Quoted {
        quote {
            fn bar_get_foo_inner(x: Bar) -> Field {
                x.bar_inner.foo_inner
            }
        }
    }

    fn main(x: Bar) {
        let _ = bar_get_foo_inner(x);
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "foo_inner");
}
