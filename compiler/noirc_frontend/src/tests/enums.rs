use crate::{
    hir::{
        def_collector::{dc_crate::CompilationError, errors::DefCollectorErrorKind},
        resolution::errors::ResolverError,
        type_check::TypeCheckError,
    },
    parser::ParserErrorReason,
    tests::{get_program_errors, get_program_using_features},
};
use CompilationError::*;
use DefCollectorErrorKind::Duplicate;
use ResolverError::*;
use TypeCheckError::{ArityMisMatch, TypeMismatch};

#[test]
fn error_with_duplicate_enum_variant() {
    let src = r#"
        enum Foo {
            Bar(i32),
            Bar(u8),
        }

        fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);
    assert!(matches!(&errors[0], DefinitionError(Duplicate { .. })));
    assert!(matches!(&errors[1], ResolverError(UnusedItem { .. })));
}

#[test]
fn errors_on_unspecified_unstable_enum() {
    // Enums are experimental - this will need to be updated when they are stabilized
    let src = r#"
        enum Foo { Bar }

        fn main() {
            let _x = Foo::Bar;
        }
    "#;

    let no_features = &[];
    let errors = get_program_using_features(src, no_features).2;
    assert_eq!(errors.len(), 1);

    let CompilationError::ParseError(error) = &errors[0] else {
        panic!("Expected a ParseError experimental feature error");
    };

    assert!(matches!(error.reason(), Some(ParserErrorReason::ExperimentalFeature(_))));
}

#[test]
fn errors_on_unspecified_unstable_match() {
    // Enums are experimental - this will need to be updated when they are stabilized
    let src = r#"
        fn main() {
            match 3 {
                _ => (),
            }
        }
    "#;

    let no_features = &[];
    let errors = get_program_using_features(src, no_features).2;
    assert_eq!(errors.len(), 1);

    let CompilationError::ParseError(error) = &errors[0] else {
        panic!("Expected a ParseError experimental feature error");
    };

    assert!(matches!(error.reason(), Some(ParserErrorReason::ExperimentalFeature(_))));
}

#[test]
fn errors_on_repeated_match_variables_in_pattern() {
    let src = r#"
        fn main() {
            match (1, 2) {
                (_x, _x) => (),
            }
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(&errors[0], ResolverError(VariableAlreadyDefinedInPattern { .. })));
}

#[test]
fn duplicate_field_in_match_struct_pattern() {
    let src = r#"
        fn main() {
            let foo = Foo { x: 10, y: 20 };
            match foo {
                Foo { x: _, x: _, y: _ } => {}
            }
        }

        struct Foo {
            x: i32,
            y: Field,
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(&errors[0], ResolverError(DuplicateField { .. })));
}

#[test]
fn missing_field_in_match_struct_pattern() {
    let src = r#"
        fn main() {
            let foo = Foo { x: 10, y: 20 };
            match foo {
                Foo { x: _ } => {}
            }
        }

        struct Foo {
            x: i32,
            y: Field,
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(&errors[0], ResolverError(MissingFields { .. })));
}

#[test]
fn no_such_field_in_match_struct_pattern() {
    let src = r#"
        fn main() {
            let foo = Foo { x: 10, y: 20 };
            match foo {
                Foo { x: _, y: _, z: _ } => {}
            }
        }

        struct Foo {
            x: i32,
            y: Field,
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(&errors[0], ResolverError(NoSuchField { .. })));
}

#[test]
fn match_integer_type_mismatch_in_pattern() {
    let src = r#"
        fn main() {
            match 2 {
                Foo::One(_) => (),
            }
        }

        enum Foo {
            One(i32),
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(&errors[0], TypeError(TypeMismatch { .. })));
}

#[test]
fn match_shadow_global() {
    let src = r#"
        fn main() {
            match 2 {
                foo => assert_eq(foo, 2),
            }
        }

        fn foo() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 0);
}

#[test]
fn match_no_shadow_global() {
    let src = r#"
        fn main() {
            match 2 {
                crate::foo => (),
            }
        }

        fn foo() {}
    "#;
    let errors = dbg!(get_program_errors(src));
    assert_eq!(errors.len(), 1);

    assert!(matches!(&errors[0], ResolverError(UnexpectedItemInPattern { .. })));
}

#[test]
fn constructor_arg_arity_mismatch_in_pattern() {
    let src = r#"
        fn main() {
            match Foo::One(1) {
                Foo::One(_, _) => (), // too many
                Foo::Two(_) => (),    // too few
            }
        }

        enum Foo {
            One(i32),
            Two(i32, i32),
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);

    assert!(matches!(&errors[0], TypeError(ArityMisMatch { .. })));
    assert!(matches!(&errors[1], TypeError(ArityMisMatch { .. })));
}
