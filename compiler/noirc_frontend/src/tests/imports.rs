use crate::hir::{
    def_collector::{dc_crate::CompilationError, errors::DefCollectorErrorKind},
    resolution::{errors::ResolverError, import::PathResolutionError},
};

use super::{assert_no_errors, get_program_errors};

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
    "#;
    assert_no_errors(src);
}

#[test]
fn no_super() {
    let src = "use super::some_func;";
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::DefinitionError(DefCollectorErrorKind::PathResolutionError(
        PathResolutionError::NoSuper(span),
    )) = &errors[0].0
    else {
        panic!("Expected a 'no super' error, got {:?}", errors[0].0);
    };

    assert_eq!(span.start(), 4);
    assert_eq!(span.end(), 9);
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
    "#;
    assert_no_errors(src);
}

#[test]
fn warns_on_use_of_private_exported_item() {
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
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(
        &errors[0].0,
        CompilationError::ResolverError(ResolverError::PathResolutionError(
            PathResolutionError::Private(..),
        ))
    ));
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
    }

    fn main() {
        foo::baz();
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(
        &errors[0].0,
        CompilationError::DefinitionError(
            DefCollectorErrorKind::CannotReexportItemWithLessVisibility { .. }
        )
    ));
}

#[test]
fn errors_if_using_alias_in_import() {
    let src = r#"
    mod foo {
        pub type bar = i32;
    }

    use foo::bar::baz;

    fn main() {
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::DefinitionError(DefCollectorErrorKind::PathResolutionError(
        PathResolutionError::NotAModule { ident, kind },
    )) = &errors[0].0
    else {
        panic!("Expected a 'not a module' error, got {:?}", errors[0].0);
    };

    assert_eq!(ident.to_string(), "bar");
    assert_eq!(*kind, "type alias");
}
