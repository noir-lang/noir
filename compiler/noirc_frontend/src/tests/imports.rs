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
    assert_eq!(errors.len(), 2); // An existing bug causes this error to be duplicated

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
