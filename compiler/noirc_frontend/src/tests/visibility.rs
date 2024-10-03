use crate::{
    hir::{
        def_collector::{dc_crate::CompilationError, errors::DefCollectorErrorKind},
        resolution::{errors::ResolverError, import::PathResolutionError},
    },
    tests::get_program_errors,
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
#[test]
fn errors_if_type_alias_aliases_more_private_type() {
    let src = r#"
    struct Foo {}
    pub type Bar = Foo;
    pub fn no_unused_warnings(_b: Bar) {
        let _ = Foo {};
    }
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::TypeIsMorePrivateThenItem {
        typ, item, ..
    }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(typ, "Foo");
    assert_eq!(item, "Bar");
}

#[test]
fn errors_if_type_alias_aliases_more_private_type_in_generic() {
    let src = r#"
    pub struct Generic<T> { value: T }
    struct Foo {}
    pub type Bar = Generic<Foo>;
    pub fn no_unused_warnings(_b: Bar) {
        let _ = Foo {};
        let _ = Generic { value: 1 };
    }
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::TypeIsMorePrivateThenItem {
        typ, item, ..
    }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(typ, "Foo");
    assert_eq!(item, "Bar");
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
    assert_eq!(errors.len(), 2); // There's a bug that duplicates this error

    let CompilationError::ResolverError(ResolverError::PathResolutionError(
        PathResolutionError::Private(ident),
    )) = &errors[0].0
    else {
        panic!("Expected a private error");
    };

    assert_eq!(ident.to_string(), "bar");
}
