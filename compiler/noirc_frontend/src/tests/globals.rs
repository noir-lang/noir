use crate::tests::{assert_no_errors, check_errors};

#[test]
fn deny_cyclic_globals() {
    let src = r#"
        global A: u32 = B;
               ^ Dependency cycle found
               ~ 'A' recursively depends on itself: A -> B -> A
        global B: u32 = A;
                        ^ Failed to resolve this global
    "#;
    check_errors(src);
}

#[test]
fn ban_mutable_globals() {
    let src = r#"
        mut global FOO: Field = 0;
                   ^^^ Only `comptime` globals may be mutable
        fn main() {
            let _ = FOO; // silence FOO never used warning
        }
    "#;
    check_errors(src);
}

#[test]
fn do_not_infer_globals_to_u32_from_type_use() {
    let src = r#"
        global ARRAY_LEN = 3;
               ^^^^^^^^^ Globals must have a specified type
                           ~ Inferred type is `Field`
        global STR_LEN: _ = 2;
                        ^ The placeholder `_` is not allowed in global definitions
               ^^^^^^^ Globals must have a specified type
                            ~ Inferred type is `Field`
        global FMT_STR_LEN = 2;
               ^^^^^^^^^^^ Globals must have a specified type
                             ~ Inferred type is `Field`

        fn main() {
            let _a: [u32; ARRAY_LEN] = [1, 2, 3];
                    ^^^^^^^^^^^^^^^^ The numeric generic is not of type `u32`
                    ~~~~~~~~~~~~~~~~ expected `u32`, found `Field`
            let _b: str<STR_LEN> = "hi";
                        ^^^^^^^ The numeric generic is not of type `u32`
                        ~~~~~~~ expected `u32`, found `Field`
            let _c: fmtstr<FMT_STR_LEN, _> = f"hi";
                           ^^^^^^^^^^^ The numeric generic is not of type `u32`
                           ~~~~~~~~~~~ expected `u32`, found `Field`
        }
    "#;
    check_errors(src);
}

#[test]
fn do_not_infer_partial_global_types() {
    let src = r#"
        pub global ARRAY: [Field; _] = [0; 3];
                                  ^ The placeholder `_` is not allowed in global definitions
                   ^^^^^ Globals must have a specified type
                                       ~~~~~~ Inferred type is `[Field; 3]`
        pub global NESTED_ARRAY: [[Field; _]; 3] = [[]; 3];
                                          ^ The placeholder `_` is not allowed in global definitions
                   ^^^^^^^^^^^^ Globals must have a specified type
                                                   ~~~~~~~ Inferred type is `[[Field; 0]; 3]`
        pub global STR: str<_> = "hi";
                            ^ The placeholder `_` is not allowed in global definitions
                   ^^^ Globals must have a specified type
                                 ~~~~ Inferred type is `str<2>`
        pub global NESTED_STR: [str<_>] = &["hi"];
                                    ^ The placeholder `_` is not allowed in global definitions
                   ^^^^^^^^^^ Globals must have a specified type
                                          ~~~~~~~ Inferred type is `[str<2>]`
        pub global FORMATTED_VALUE: str<5> = "there";
        pub global FMT_STR: fmtstr<_, _> = f"hi {FORMATTED_VALUE}";
                                   ^ The placeholder `_` is not allowed in global definitions
                                      ^ The placeholder `_` is not allowed in global definitions
                   ^^^^^^^ Globals must have a specified type
                                           ~~~~~~~~~~~~~~~~~~~~~~~ Inferred type is `fmtstr<20, (str<5>,)>`
        pub global TUPLE_WITH_MULTIPLE: ([str<_>], [[Field; _]; 3]) = 
                                              ^ The placeholder `_` is not allowed in global definitions
                                                            ^ The placeholder `_` is not allowed in global definitions
                   ^^^^^^^^^^^^^^^^^^^ Globals must have a specified type
            (&["hi"], [[]; 3]);
            ~~~~~~~~~~~~~~~~~~ Inferred type is `([str<2>], [[Field; 0]; 3])`
        pub global FOO: [i32; 3] = [1, 2, 3];
    "#;
    check_errors(src);
}

#[test]
fn u32_globals_as_sizes_in_types() {
    let src = r#"
        global ARRAY_LEN: u32 = 3;
        global STR_LEN: u32 = 2;
        global FMT_STR_LEN: u32 = 2;

        fn main() {
            let _a: [u32; ARRAY_LEN] = [1, 2, 3];
            let _b: str<STR_LEN> = "hi";
            let _c: fmtstr<FMT_STR_LEN, _> = f"hi";
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn non_u32_global_as_array_length() {
    let src = r#"
        global ARRAY_LEN: u8 = 3;

        fn main() {
            let _a: [u32; ARRAY_LEN] = [1, 2, 3];
                    ^^^^^^^^^^^^^^^^ The numeric generic is not of type `u32`
                    ~~~~~~~~~~~~~~~~ expected `u32`, found `u8`
        }
    "#;
    check_errors(src);
}

#[test]
fn operators_in_global_used_in_type() {
    let src = r#"
        global ONE: u32 = 1;
        global COUNT: u32 = ONE + 2;
        fn main() {
            let _array: [Field; COUNT] = [1, 2, 3];
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn disallows_references_in_globals() {
    let src = r#"
    pub global mutable: &mut Field = &mut 0;
               ^^^^^^^ References are not allowed in globals
    "#;
    check_errors(src);
}

#[test]
fn int_min_global() {
    let src = r#"
        global MIN: i8 = -128;
        fn main() {
            let _x = MIN;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn lazy_literal_globals() {
    // We want to make sure that we successfully elaborate the literal global `foo`
    // even though it is defined after the global `bar` which uses `foo`.
    let src = "
    global bar: Foo = Foo::new();
    global foo: u32 = 1;
    
    struct Foo {
       foo: u32
    }
    
    impl Foo {
        fn new() -> Self {
            Self { foo }
        }
    }
    
    fn main() {
        let _ = bar;
    }
    ";
    assert_no_errors(src);
}
