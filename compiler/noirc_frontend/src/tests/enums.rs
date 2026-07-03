use crate::elaborator::UnstableFeature;

use crate::tests::{
    assert_no_errors, assert_no_errors_using_features, check_errors, check_errors_using_features,
    get_program_using_features,
};

#[test]
fn error_with_duplicate_enum_variant() {
    let src = r#"
    pub enum Foo {
        Bar(i32),
        ~~~ First definition found here
        ~~~ Previous impl defined here
        Bar(u8),
        ^^^ Duplicate definitions of enum variant with name Bar found
        ~~~ Second definition found here
        ^^^ Impl for type `Foo` overlaps with existing impl
        ~~~ Overlapping impl
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
fn struct_pattern_non_alphabetical_field_order_is_not_unreachable() {
    // Regression test for https://github.com/noir-lang/noir-claude/issues/1068.
    // `S` declares its fields in non-alphabetical order (`z` before `a`). The
    // struct pattern's args used to be ordered by field name while the match
    // branch variables follow declaration order, so each pattern was paired with
    // the wrong variable. The arm was then silently dropped and reported as an
    // unreachable case even though it can match.
    assert_no_errors(
        r#"
        struct S { z: bool, a: i32 }

        fn main(x: S) -> pub i32 {
            match x {
                S { z: false, a: 42 } => 100,
                _ => 0,
            }
        }
    "#,
    );
}

#[test]
fn redundant_struct_pattern_is_still_unreachable() {
    // The fix must not suppress genuine unreachability: a second, identical arm on
    // a non-alphabetically-declared struct is still redundant and must be reported.
    check_errors(
        r#"
        struct S { z: bool, a: i32 }

        fn main(x: S) -> pub i32 {
            match x {
                S { z: false, a: 42 } => 100,
                S { z: false, a: 42 } => 200,
                ^^^^^^^^^^^^^^^^^^^^^ Unreachable match case
                ~~~~~~~~~~~~~~~~~~~~~ This pattern is redundant with one or more prior patterns
                _ => 0,
            }
        }
    "#,
    );
}

#[test]
fn struct_pattern_args_bind_by_field_name_not_written_order() {
    // The pattern lists fields in yet another order, distinct from both the
    // declaration order and alphabetical order. Each literal sub-pattern must
    // still be tested against its own field's branch variable.
    assert_no_errors(
        r#"
        struct S { z: bool, a: i32 }

        fn main(x: S) -> pub i32 {
            match x {
                S { a: 42, z: false } => 100,
                _ => 0,
            }
        }
    "#,
    );
}

#[test]
fn struct_pattern_many_fields_scrambled_declaration_order() {
    // Several fields declared in a deliberately scrambled order, with `bool` and
    // integer fields interleaved so that ordering args by field name pairs a bool
    // literal against an integer branch variable (and vice-versa). Such a
    // mispairing makes `compile_rows` drop the arm and report it as unreachable.
    assert_no_errors(
        r#"
        struct S { d: i32, c: bool, b: i32, a: bool }

        fn main(x: S) -> pub i32 {
            match x {
                S { d: 1, c: true, b: 2, a: false } => 100,
                _ => 0,
            }
        }
    "#,
    );
}

#[test]
fn nested_struct_pattern_non_alphabetical_field_order() {
    assert_no_errors(
        r#"
        struct Inner { y: bool, x: i32 }
        struct Outer { tag: u8, inner: Inner }

        fn main(o: Outer) -> pub i32 {
            match o {
                Outer { tag: 9, inner: Inner { y: true, x: 5 } } => 100,
                _ => 0,
            }
        }
    "#,
    );
}

#[test]
fn nested_enum_variant_with_payload_is_not_unreachable() {
    // Regression test for https://github.com/noir-lang/noir/issues/7637.
    // Matching a payload-carrying variant at more than one nesting level (here
    // both elements of the tuple) let-binds the payload, rewriting the arm's
    // body. Reconstructing the row used to reset `original_body` to the rewritten
    // body, so the arm was never pruned from `unreachable_cases` and was falsely
    // reported as redundant.
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(
        r#"
        pub enum Foo { Bar, Baz(()) }

        pub fn foo(x: Foo, y: Foo) {
            match (x, y) {
                (Foo::Bar, Foo::Bar) => (),
                (Foo::Baz(_x), Foo::Baz(_y)) => (),
                _ => (),
            }
        }

        fn main() {}
        "#,
        &features,
    );
}

#[test]
fn nested_enum_variant_with_non_unit_payload_is_not_unreachable() {
    // The same bug is not specific to unit payloads: any variant with arguments
    // matched across nesting levels must remain reachable.
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(
        r#"
        pub enum Foo { Bar, Baz(u32) }

        pub fn foo(x: Foo, y: Foo) -> u32 {
            match (x, y) {
                (Foo::Bar, Foo::Bar) => 1,
                (Foo::Baz(_x), Foo::Baz(_y)) => 2,
                _ => 3,
            }
        }

        fn main() {}
        "#,
        &features,
    );
}

#[test]
fn redundant_nested_enum_variant_with_payload_is_still_unreachable() {
    // The fix must not suppress genuine unreachability: a second, identical
    // payload-carrying arm in a nested match is still redundant and must warn.
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(
        r#"
        pub enum Foo { Bar, Baz(u32) }

        pub fn foo(x: Foo, y: Foo) -> u32 {
            match (x, y) {
                (Foo::Baz(_x), Foo::Baz(_y)) => 1,
                (Foo::Baz(_a), Foo::Baz(_b)) => 2,
                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Unreachable match case
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~ This pattern is redundant with one or more prior patterns
                _ => 3,
            }
        }

        fn main() {}
        "#,
        &features,
    );
}

#[test]
fn missing_field_in_non_alphabetical_match_struct_pattern() {
    // The missing-field diagnostic must name the field that is actually absent
    // even when the struct is not declared in alphabetical order.
    let src = r#"
    fn main() {
        let foo = Foo { y: 10, x: 20 };
        match foo {
            Foo { y: _ } => {}
            ^^^ missing field x in struct Foo
        }
    }

    struct Foo {
        y: i32,
        x: Field,
    }
    "#;
    check_errors(src);
}

#[test]
fn duplicate_field_in_non_alphabetical_match_struct_pattern() {
    // `Foo` declares `y` before `x`, and the duplicated field (`x`) is not the
    // first declared field. Duplicate detection keys off the field's
    // declaration-order slot, so it must still flag the repeat regardless of
    // declaration order or which slot the field occupies.
    let src = r#"
    fn main() {
        let foo = Foo { y: 10, x: 20 };
        match foo {
            Foo { x: _, x: _, y: _ } => {}
                        ^ duplicate field x
        }
    }

    struct Foo {
        y: i32,
        x: Field,
    }
    "#;
    check_errors(src);
}

#[test]
fn duplicate_field_with_missing_field_in_match_struct_pattern() {
    // Both fields of `Pair` have the same type, so duplicating one and omitting the
    // other cannot be caught by a later type error. Struct patterns must bind every
    // field (there is no partial or `..` rest pattern), so this must report both the
    // duplicate field and the missing field.
    let src = r#"
    fn main() {
        let pair = Pair { a: 1, b: 2 };
        match pair {
            Pair { a: _, a: _ } => {}
            ^^^^ missing field b in struct Pair
                         ^ duplicate field a
        }
    }

    struct Pair {
        a: i32,
        b: i32,
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
                ^^^^^^^^^^ Expected a struct, enum, or literal pattern, but found function `foo`
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

#[test]
fn regression_7651() {
    let src = r#"
    pub enum Foo {
        Bar,
    }

    fn main(_x: Foo) { }
                ^^^ Invalid type found in the entry point to a program
                ~~~ Enum is not yet allowed as an entry point type. Found: Foo
    "#;

    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn cannot_return_enum_from_unconstrained_to_constrained() {
    // An enum's tag is an unconstrained `Field` witness when it crosses from an
    // unconstrained runtime into a constrained one. Mirrors the entry-point rule
    // (`regression_7651`) so a prover cannot supply an out-of-range tag and force
    // the wrong match arm in the constrained caller.
    let src = r#"
    pub enum Foo {
        Bar,
        Baz,
    }

    fn main() {
        // safety:
        unsafe {
            let _foo = make_foo();
                       ^^^^^^^^^^ Enums cannot be returned from an unconstrained runtime to a constrained runtime
        }
    }

    unconstrained fn make_foo() -> Foo {
        Foo::Bar
    }
    "#;

    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn cannot_return_enum_nested_in_struct_from_unconstrained_to_constrained() {
    let src = r#"
    pub enum Foo {
        Bar,
        Baz,
    }

    pub struct Wrapper {
        foo: Foo,
    }

    fn main() {
        // safety:
        unsafe {
            let _w = make_wrapper();
                     ^^^^^^^^^^^^^^ Enums cannot be returned from an unconstrained runtime to a constrained runtime
        }
    }

    unconstrained fn make_wrapper() -> Wrapper {
        Wrapper { foo: Foo::Bar }
    }
    "#;

    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn can_pass_enum_from_constrained_to_unconstrained() {
    // The reverse direction is fine: the enum is built in the constrained caller, so its tag
    // is already valid by construction. Only returning an enum the other way is rejected.
    let src = r#"
    pub enum Foo {
        Bar,
        Baz,
    }

    unconstrained fn consume(_foo: Foo) {}

    fn main() {
        // safety:
        unsafe {
            consume(Foo::Bar);
        }
    }
    "#;

    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn can_return_enum_from_unconstrained_to_unconstrained() {
    // Returning an enum is only rejected when crossing into a constrained runtime.
    // A purely unconstrained call chain has no constraints to subvert.
    let src = r#"
    pub enum Foo {
        Bar,
        Baz,
    }

    unconstrained fn make_foo() -> Foo {
        Foo::Bar
    }

    unconstrained fn main() {
        let _foo = make_foo();
    }
    "#;

    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn errors_if_using_comptime_type_in_non_comptime_enum() {
    let src = r#"
    pub enum Foo {
        Quoted(Quoted),
               ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime enum
    }
    "#;
    check_errors(src);
}

#[test]
fn mutually_recursive_types_error() {
    // cSpell:disable
    let src = "
    fn main() {
        let _zero = Even::Zero;
    }

    enum Even {
        Zero,
        Succ(Odd),
    }

    enum Odd {
         ^^^ Dependency cycle found
         ~~~ 'Odd' recursively depends on itself: Odd -> Even -> Odd
        One,
        Succ(Even),
    }
    ";
    // cSpell:enable
    check_errors_using_features(src, &[UnstableFeature::Enums]);
}

#[test]
fn mutually_recursive_types_with_structs_error() {
    // cSpell:disable
    let src = "
    fn main() {
        let _zero = Even::Zero;
    }

    enum Even {
         ^^^^ Dependency cycle found
         ~~~~ 'Even' recursively depends on itself: Even -> EvenSucc -> Odd -> OddSucc -> Even
        Zero,
        Succ(EvenSucc),
    }

    pub struct EvenSucc { inner: Odd }

    enum Odd {
        One,
        Succ(OddSucc),
    }

    pub struct OddSucc { inner: Even }
    ";
    // cSpell:enable
    check_errors_using_features(src, &[UnstableFeature::Enums]);
}

#[test]
fn match_constructor_pattern_on_integer_gives_type_error() {
    let src = "
    struct S {}

    fn main(x: u32, _b: S) {
        match x {
            S {} => (),
            ^^^^ Expected type u32, found type S
        }
    }
    ";
    check_errors_using_features(src, &[UnstableFeature::Enums]);
}

#[test]
fn struct_pattern_on_enum_variant_errors() {
    let src = "
    enum E {
        A,
        B(u32),
    }

    fn foo(e: E) -> u32 {
        match e {
            E::A { x } => x,
            ^^^^^^^^^^ Cannot use `{ }` pattern syntax on enum variant `E::A`
            ~~~~~~~~~~ Use parentheses `()` for enum variants with fields, or no arguments for fieldless variants
                          ^ cannot find `x` in this scope
                          ~ not found in this scope
            _ => 0,
        }
    }

    fn main() {
        let _ = foo(E::A);
    }
    ";
    check_errors_using_features(src, &[UnstableFeature::Enums]);
}

// Regression test: comptime enum variant without fields on a generic enum
// should not crash follow_bindings with a Forall type.
#[test]
fn comptime_generic_enum_variant() {
    let src = r#"
    enum Foo<T> {
        A(T),
        B,
    }

    fn main() {
        comptime let fb: Foo<str<5>> = Foo::B;
        foo(fb);
    }

    fn foo(_f: Foo<str<5>>) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn enum_variant_with_fields_type_turbofish() {
    let src = r#"
    enum Foo<T> {
        Spam,
        Eggs(T),
    }

    fn main() {
        let _ = Foo::<u32>::Eggs(0);
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn enum_variant_with_fields_type_turbofish_binds_type() {
    let src = r#"
    enum Foo<T> {
        Eggs(T),
    }

    fn main() {
        let _ = Foo::<u32>::Eggs(true);
                                 ^^^^ Expected type u32, found type bool
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn enum_variant_with_fields_multiple_type_turbofish() {
    let src = r#"
    enum Foo<A, B> {
        Spam(A),
        Eggs(B),
    }

    fn main() {
        let _ = Foo::<bool, u32>::Eggs(0);
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn enum_variant_segment_turbofish_still_works() {
    let src = r#"
    enum Foo<A, B> {
        Spam(A),
        Eggs(B),
    }

    fn main() {
        let _ = Foo::Eggs::<bool, u32>(0);
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn fieldless_enum_variant_type_turbofish_binds_type() {
    let src = r#"
    enum Foo<T> {
        Spam,
        Eggs(T),
    }

    fn main() {
        let _ = Foo::<u32>::Spam;
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn fieldless_enum_variant_segment_turbofish_binds_type() {
    let src = r#"
    enum Foo<T> {
        Spam,
        Eggs(T),
    }

    fn main() {
        let _ = Foo::Spam::<u32>;
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn fieldless_enum_variant_segment_turbofish_count_mismatch() {
    let src = r#"
    enum Foo<T> {
        Spam,
        Eggs(T),
    }

    fn main() {
        let _ = Foo::Spam::<u32, bool>;
                ^^^^^^^^^^^^^^^^^^^^^^ enum `Foo` expects 1 generic but 2 were given
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn turbofish_not_allowed_on_global_holding_enum_value() {
    let src = r#"
    enum Foo<T> {
        Spam,
        Eggs(T),
    }

    global Bar: Foo<u32> = Foo::Spam;

    fn main() {
        let _ = Bar::<bool>;
                   ^^^^^^^^ turbofish (`::<_>`) not allowed on globals
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn fieldless_enum_variant_type_turbofish_on_non_generic_enum() {
    let src = r#"
    enum Foo {
        Spam,
        Eggs(bool),
    }

    fn main() {
        let _ = Foo::<u32>::Spam;
                ^^^^^^^^^^^^^^^^ enum `Foo` expects 0 generics but 1 was given
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn errors_on_segment_after_enum_variant() {
    let src = r#"
    pub enum E { A, B }
    pub fn f(_x: E::A::Bar) {}
                    ^ enum variant `A` has no associated items
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_segment_after_associated_constant() {
    let src = r#"
    pub trait T { let C: u32; }
    pub struct Foo {}
    impl T for Foo { let C: u32 = 1; }
    pub fn f(_x: Foo::C::Bar) {}
                      ^ associated constant `C` has no associated items
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_enum_generics_specified_on_both_type_and_variant() {
    // In `<E<u32>>::A::<bool>` the enum's generics are given both on the type (`<u32>`)
    // and on the variant turbofish (`<bool>`); specifying them twice is an error.
    let src = r#"
    pub enum E<T> { A, B(T) }
    fn main() {
        let _ = <E<u32>>::A::<bool>;
                              ^^^^ generic arguments are not allowed on both an enum and its variant's path segments simultaneously; they are only valid in one place or the other
                              ~~~~ remove the generics arguments from one of the path segments
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::Enums]);
}

#[test]
fn turbofish_on_match_pattern_variant_binds_payload_type() {
    // Regression test for https://github.com/noir-lang/noir/issues/7430.
    // The scrutinee's generic is left undetermined so only the turbofish can pin
    // the payload's type.
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    fn main() {
        let f = Foo::Baz;
        match f {
            Foo::Bar::<i32>(x) => {
                let _: i32 = x;
            }
            Foo::Baz => {}
        }
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    assert_no_errors_using_features(src, &features);
}

#[test]
fn turbofish_on_match_pattern_variant_conflicting_type() {
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    fn main() {
        let f = Foo::Baz;
        match f {
            Foo::Bar::<i32>(x) => {
                let _: bool = x;
                              ^ Expected type bool, found type i32
            }
            Foo::Baz => {}
        }
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn turbofish_on_fieldless_match_pattern_variant_conflicting_type() {
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    fn main() {
        let f: Foo<Field> = Foo::Baz;
        match f {
            Foo::Baz::<i32> => {}
            ^^^^^^^^^^^^^^^ Expected type Foo<Field>, found type Foo<i32>
            Foo::Bar(_) => {}
        }
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn turbofish_on_match_pattern_variant_type_segment() {
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    fn main() {
        let f = Foo::Baz;
        match f {
            Foo::<i32>::Bar(x) => {
                let _: bool = x;
                              ^ Expected type bool, found type i32
            }
            Foo::Baz => {}
        }
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn turbofish_on_match_pattern_variant_count_mismatch() {
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    fn main() {
        let f: Foo<i32> = Foo::Baz;
        match f {
            Foo::Bar::<i32, bool>(x) => {
            ^^^^^^^^^^^^^^^^^^^^^ Expected 1 generic from this function, but 2 were provided
                let _ = x;
            }
            Foo::Baz => {}
        }
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn errors_on_turbofish_on_both_type_and_variant_in_match_pattern() {
    // The enum's generics can be given on the type segment (`Foo::<i32>`) or on the variant
    // segment (`Bar::<i32>`), but giving them in both places specifies them twice.
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    fn main() {
        let f: Foo<i32> = Foo::Baz;
        match f {
            Foo::<i32>::Bar::<i32>(x) => {
                              ^^^ Generic arguments for the enum were specified more than once
                              ~~~ Specify the enum's generic arguments in only one place
                let _ = x;
            }
            Foo::Baz => {}
        }
    }
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}
