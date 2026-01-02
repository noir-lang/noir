use crate::elaborator::UnstableFeature;

use crate::{
    parser::ParserErrorReason,
    tests::{
        CompilationError, assert_no_errors, check_errors, check_errors_using_features,
        get_program_using_features,
    },
};

#[test]
fn error_with_duplicate_enum_variant() {
    let src = r#"
    pub enum Foo {
        Bar(i32),
        ~~~ First enum variant found here
        Bar(u8),
        ^^^ Duplicate definitions of enum variant with name Bar found
        ~~~ Second enum variant found here
    }
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
    let src = r#"
    fn main() {
        match 3 {
        ^^^^^ This requires the unstable feature 'enums' which is not enabled
        ~~~~~ Pass -Zenums to nargo to enable this feature at your own risk.
            _ => (),
        }
    }
    "#;
    let no_features = &[];
    check_errors_using_features(src, no_features);
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

#[test]
fn unreachable_match_case() {
    check_errors(
        r#"
        fn main() {
            match Opt::Some(Opt::Some(3)) {
                Opt::Some(_) => (),
                Opt::None => (),
                Opt::Some(Opt::Some(_)) => (),
                ^^^^^^^^^^^^^^^^^^^^^^^ Unreachable match case
                ~~~~~~~~~~~~~~~~~~~~~~~ This pattern is redundant with one or more prior patterns
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    "#,
    );
}

#[test]
fn match_reachability_errors_ignored_when_there_is_a_type_error() {
    // No comment on the second `None` case.
    // Type errors in general mess up reachability errors in match cases.
    // If we naively change to catch this case (which is easy) we also end up
    // erroring that the `3 => ()` case is unreachable as well, which is true
    // but we don't want to annoy users with an extra obvious error. This
    // behavior matches Rust as well.
    check_errors(
        "
        fn main() {
            match Opt::Some(3) {
                Opt::None => (),
                Opt::Some(_) => {},
                Opt::None => (),
                3 => (),
                ^ Expected type Opt<Field>, found type Field
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    ",
    );
}

#[test]
fn missing_single_case() {
    check_errors(
        "
        fn main() {
            match Opt::Some(3) {
                  ^^^^^^^^^^^^ Missing case: `Some(_)`
                Opt::None => (),
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    ",
    );
}

#[test]
fn missing_many_cases() {
    check_errors(
        "
        fn main() {
            match Abc::A {
                  ^^^^^^ Missing cases: `C`, `D`, `E`, and 21 more not shown
                Abc::A => (),
                Abc::B => (),
            }
        }

        enum Abc {
            A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z
        }
    ",
    );
}

#[test]
fn missing_int_ranges() {
    check_errors(
        "
        fn main() {
            let x: i8 = 3;
            match Opt::Some(x) {
                  ^^^^^^^^^^^^ Missing cases: `None`, `Some(-128..=3)`, `Some(5)`, and 1 more not shown
                Opt::Some(4) => (),
                Opt::Some(6) => (),
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    ");
}

#[test]
fn missing_int_ranges_with_negatives() {
    check_errors(
        "
        fn main() {
            let x: i32 = -4;
            match x {
                  ^ Missing cases: `-2147483648..=-6`, `-4..=-1`, `1..=2`, and 1 more not shown
                -5 => (),
                0 => (),
                3 => (),
            }
        }
    ",
    );
}

#[test]
fn missing_cases_with_empty_match() {
    check_errors(
        "
        fn main() {
            match Abc::A {}
                  ^^^^^^ Missing cases: `A`, `B`, `C`, and 23 more not shown
        }

        enum Abc {
            A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z
        }
    ",
    );
}

#[test]
fn missing_integer_cases_with_empty_match() {
    check_errors(
        "
        fn main() {
            let x: i8 = 3;
            match x {}
                  ^ Missing cases: `i8` is non-empty
                  ~ Try adding a match-all pattern: `_`
        }
    ",
    );
}

#[test]
fn match_on_empty_enum() {
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(
        "
        pub fn foo(v: Void) {
            match v {}
        }
        pub enum Void {}
        ",
        &features,
    );
}

#[test]
fn cannot_determine_type_of_generic_argument_in_enum_constructor() {
    let src = r#"
    enum Foo<T> {
        Bar,
    }

    fn main()
    {
        let _ = Foo::Bar;
                     ^^^ Type annotation needed
                     ~~~ Could not determine the type of the generic argument `T` declared on the enum `Foo`
    }

    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn errors_on_comptime_enum() {
    let src = r#"
    comptime enum Foo {
        Bar,
    }
    fn main() { }
    "#;

    let features = vec![UnstableFeature::Enums];
    let errors = get_program_using_features(src, &features).2;
    assert_eq!(errors.len(), 1);

    let CompilationError::ParseError(error) = &errors[0] else {
        panic!("Expected a ParseError experimental feature error: {errors:?}");
    };

    assert!(matches!(error.reason(), Some(ParserErrorReason::ComptimeNotApplicable)));
}

#[test]
fn impl_on_enum() {
    let src = r#"
    enum Foo { Bar }

    impl Foo {
        fn foo(self) -> Self { self }
    }

    fn main() {
        let _ = Foo::Bar.foo();
    }
    "#;

    let features = vec![UnstableFeature::Enums];
    let errors = get_program_using_features(src, &features).2;
    assert!(errors.is_empty());
}

#[test]
fn impl_eq_for_enum() {
    let src = r#"
    enum Foo { Bar }

    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool {
            match (self, other) {
                (Foo::Bar, Foo::Bar) => true,
            }
        }
    }

    fn main() {
        assert(Foo::Bar.eq(Foo::Bar));
    }
    "#;

    let features = vec![UnstableFeature::Enums];
    let errors = get_program_using_features(src, &features).2;
    assert!(errors.is_empty());
}
