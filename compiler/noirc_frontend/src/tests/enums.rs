use crate::{
    hir::def_collector::dc_crate::CompilationError,
    parser::ParserErrorReason,
    tests::{assert_no_errors, get_program_using_features},
};

use super::{check_errors, check_errors_using_features};

#[test]
fn error_with_duplicate_enum_variant() {
    // TODO: the primary error should be on the second `Bar`
    let src = r#"
    pub enum Foo {
        Bar(i32),
        ^^^ Duplicate definitions of enum variant with name Bar found
        ~~~ First enum variant found here
        Bar(u8),
        ~~~ Second enum variant found here
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unspecified_unstable_enum() {
    // Enums are experimental - this will need to be updated when they are stabilized
    let src = r#"
    enum Foo { Bar }
         ^^^ This requires the unstable feature 'enums' which is not enabled
         ~~~ Pass -Zenums to nargo to enable this feature at your own risk.

    fn main() {
        let _x = Foo::Bar;
    }
    "#;
    let no_features = &[];
    check_errors_using_features(src, no_features);
}

#[test]
fn errors_on_unspecified_unstable_match() {
    // TODO: update this test. Right now it's hard to test because the span happens in the entire
    // `match` node but ideally it would be nice if it only happened in the `match` keyword.
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
                 ^^ Variable `_x` was already defined in the same match pattern
                 ~~ `_x` redefined here
             ~~ `_x` was previously defined here
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn duplicate_field_in_match_struct_pattern() {
    let src = r#"
    fn main() {
        let foo = Foo { x: 10, y: 20 };
        match foo {
            Foo { x: _, x: _, y: _ } => {}
                        ^ duplicate field x
        }
    }

    struct Foo {
        x: i32,
        y: Field,
    }
    "#;
    check_errors(src);
}

#[test]
fn missing_field_in_match_struct_pattern() {
    let src = r#"
    fn main() {
        let foo = Foo { x: 10, y: 20 };
        match foo {
            Foo { x: _ } => {}
            ^^^ missing field y in struct Foo
        }
    }

    struct Foo {
        x: i32,
        y: Field,
    }
    "#;
    check_errors(src);
}

#[test]
fn no_such_field_in_match_struct_pattern() {
    let src = r#"
    fn main() {
        let foo = Foo { x: 10, y: 20 };
        match foo {
            Foo { x: _, y: _, z: _ } => {}
                              ^ no such field z defined in struct Foo
        }
    }

    struct Foo {
        x: i32,
        y: Field,
    }
    "#;
    check_errors(src);
}

#[test]
fn match_integer_type_mismatch_in_pattern() {
    let src = r#"
        fn main() {
            match 2 {
                Foo::One(_) => (),
                ^^^^^^^^ Expected type Field, found type Foo
            }
        }

        enum Foo {
            One(i32),
        }
    "#;
    check_errors(src);
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
    assert_no_errors(src);
}

#[test]
fn match_no_shadow_global() {
    let src = r#"
        fn main() {
            match 2 {
                crate::foo => (),
                ^^^^^^^^^^ Expected a struct, enum, or literal pattern, but found a function
            }
        }

        fn foo() {}
    "#;
    check_errors(src);
}

#[test]
fn constructor_arg_arity_mismatch_in_pattern() {
    let src = r#"
        fn main() {
            match Foo::One(1) {
                Foo::One(_, _) => (),
                ^^^^^^^^ Expected 1 argument, but found 2
                Foo::Two(_) => (),
                ^^^^^^^^ Expected 2 arguments, but found 1
            }
        }

        enum Foo {
            One(i32),
            Two(i32, i32),
        }
    "#;
    check_errors(src);
}
