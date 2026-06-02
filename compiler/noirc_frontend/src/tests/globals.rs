use crate::elaborator::FrontendOptions;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::resolution::errors::ResolverError;
use crate::test_utils::{GetProgramOptions, get_program_with_options};
use crate::tests::{
    assert_no_errors, check_errors, check_monomorphization_error, get_program_errors,
};

/// Compiles `src` with the given `--define`/`-D` global overrides and returns any errors.
fn compile_with_defines(src: &str, defines: &[(String, String)]) -> Vec<CompilationError> {
    get_program_with_options(
        src,
        GetProgramOptions {
            frontend_options: FrontendOptions {
                global_overrides: defines,
                ..FrontendOptions::test_default()
            },
            ..Default::default()
        },
    )
    .2
}

fn has_invalid_global_override(errors: &[CompilationError]) -> bool {
    errors.iter().any(|error| {
        matches!(
            error,
            CompilationError::ResolverError(ResolverError::InvalidGlobalOverride { .. })
        )
    })
}

#[test]
fn deny_cyclic_globals() {
    let src = r#"
        global A: u32 = B;
               ^ Dependency cycle found
               ~ 'A' recursively depends on itself: A -> B -> A
        global B: u32 = A;
                        ^ Dependency cycle found
                        ~ 'A' recursively depends on itself: the variable definition type hasn't been resolved yet
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
        pub global NESTED_STR: [str<_>] = @["hi"];
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
            (@["hi"], [[]; 3]);
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

#[test]
fn global_fn_using_quoted() {
    let src = "
    global foo: fn() = || {
        let _ = quote { 1 };
                ^^^^^^^^^^^ Comptime-only type `Quoted` used in runtime code
                ~~~~~~~~~~~ Comptime type used here
    };

    fn main() {
        foo();
    }
    ";
    check_monomorphization_error(src);
}

#[test]
fn global_using_nested_quoted_type() {
    let src = "
    global foo: [Quoted; 1] = [quote { 1 }];
                 ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime global

    fn main() {
        let _ = foo;
    }
    ";
    check_errors(src);
}

#[test]
fn comptime_global_using_nested_quoted_type() {
    let src = "
    comptime global foo: [Quoted; 1] = [quote { 1 }];

    fn main() {
        let _ = comptime { foo };
    }
    ";
    assert_no_errors(src);
}

#[test]
fn global_closure_with_undefined_variable_method_call() {
    // A global contained a closure with a method call on an undefined variable.
    // It should report an error, and not panic.
    let src = r#"
    global foo: fn() -> Field = || {
        v0.bar()
        ^^ cannot find `v0` in this scope
        ~~ not found in this scope
    };
    fn main() {
        let _ = foo;
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_mutually_recursive_function() {
    let src = r#"
    global foo: Field = bar();
           ^^^ Dependency cycle found
           ~~~ 'foo' recursively depends on itself: foo -> bar -> foo
                        ^^^^^ Expected a function, but found a(n) Field

    global bar: Field = foo();
                        ^^^ Dependency cycle found
                        ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_mutually_recursive_typed_function() {
    let src = r#"
    global foo: fn() = bar;
           ^^^ Dependency cycle found
           ~~~ 'foo' recursively depends on itself: foo -> bar -> foo

    global bar: fn() = foo;
                       ^^^ Dependency cycle found
                       ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_function() {
    let src = r#"
    global foo: Field = foo();
                        ^^^ Dependency cycle found
                        ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_typed_function() {
    let src = r#"
    global foo: fn() = foo();
                       ^^^ Dependency cycle found
                       ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_value() {
    let src = r#"
    global foo: Field = foo;
                        ^^^ Dependency cycle found
                        ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_mutually_recursive_value() {
    let src = r#"
    global foo: Field = bar;
           ^^^ Dependency cycle found
           ~~~ 'foo' recursively depends on itself: foo -> bar -> foo

    global bar: Field = foo;
                        ^^^ Dependency cycle found
                        ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_comptime_function() {
    let src = r#"
    comptime global foo: Field = foo();
                                 ^^^ Dependency cycle found
                                 ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_11489_comptime_value() {
    let src = r#"
    comptime global foo: Field = foo;
                                 ^^^ Dependency cycle found
                                 ~~~ 'foo' recursively depends on itself: the variable definition type hasn't been resolved yet
    "#;
    check_errors(src);
}

#[test]
fn regression_5626_global_annotation_flows_into_block() {
    // https://github.com/noir-lang/noir/issues/5626
    // The type annotation on a global must propagate into the assigned block
    // expression so that method calls on a generic type with trait bounds
    // can be resolved using the annotated type as inference context.
    let src = r#"
    trait Bound {
        fn make() -> Self;
    }

    pub struct Wrapper<T> {
        value: T,
    }

    impl<T> Wrapper<T>
    where
        T: Bound,
    {
        pub fn new() -> Self {
            Wrapper { value: T::make() }
        }

        pub fn push(self, _x: u32) -> Self {
            self
        }
    }

    pub struct Inner {}
    impl Bound for Inner {
        fn make() -> Self {
            Inner {}
        }
    }

    global G: Wrapper<Inner> = {
        let mut x = Wrapper::new();
        x = x.push(1);
        x
    };

    fn main() {
        let _ = G;
        let _local: Wrapper<Inner> = {
            let mut x = Wrapper::new();
            x = x.push(1);
            x
        };
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn can_refer_to_complex_global_in_function_signature() {
    let src = r#"
    global LENGTH: u32 = init();

    pub fn another(_array: [Field; LENGTH]) {}

    fn init() -> u32 {
        10
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn can_refer_to_complex_global_in_method_signature() {
    let src = r#"
    global LENGTH: u32 = init();

    pub struct Foo {}

    impl Foo {
        pub fn another(_array: [Field; LENGTH]) {}
    }

    fn init() -> u32 {
        10
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_global_is_needed_in_initialize_and_function_signature() {
    let src = r#"
    global FOO: u32 = init([0; 10]);
           ^^^ Dependency cycle found
           ~~~ 'FOO' recursively depends on itself: FOO -> init -> FOO
                      ^^^^^^^^^^^^^ Expected type u32, found type ()

    fn init(_array: [Field; FOO]) {}
                            ^^^ Cannot find a global or generic type parameter named `FOO`
                            ~~~ Only globals or generic type parameters are allowed to be used as an array type's length
                            ^^^ expected type, found global `FOO`

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn cli_define_overrides_integer_global() {
    // `N` is declared as 2, so `[0; N]` would normally be a `[Field; 2]` and
    // disagree with the `[Field; 3]` return type. Overriding `N` to 3 makes the
    // program type-check, which only happens if the CLI value actually replaces
    // the global's initializer.
    let src = "
        global N: u32 = 2;
        fn main() -> pub [Field; 3] {
            [0; N]
        }
    ";

    assert!(!get_program_errors(src).is_empty(), "expected a length mismatch without the override");

    let defines = vec![("N".to_string(), "3".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(errors.is_empty(), "expected override to make the program compile, got: {errors:?}");
}

#[test]
fn cli_define_overrides_field_global() {
    let src = "
        global X: Field = 1;
        fn main() -> pub Field {
            X
        }
    ";
    let defines = vec![("X".to_string(), "42".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(errors.is_empty(), "expected a `Field` override to compile, got: {errors:?}");
}

#[test]
fn cli_define_overrides_bool_global() {
    let src = "
        global ENABLED: bool = false;
        fn main() -> pub bool {
            ENABLED
        }
    ";
    let defines = vec![("ENABLED".to_string(), "true".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(errors.is_empty(), "expected a `bool` override to compile, got: {errors:?}");
}

#[test]
fn cli_define_ignores_unknown_global() {
    let src = "
        global N: u32 = 2;
        fn main() {
            let _ = N;
        }
    ";
    let defines = vec![("DOES_NOT_EXIST".to_string(), "5".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(
        errors.is_empty(),
        "an override for an unknown global should be ignored, got: {errors:?}"
    );
}

#[test]
fn cli_define_rejects_malformed_value() {
    let src = "
        global N: u32 = 2;
        fn main() {
            let _ = N;
        }
    ";
    let defines = vec![("N".to_string(), "not_a_number".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(
        has_invalid_global_override(&errors),
        "expected an InvalidGlobalOverride error, got: {errors:?}"
    );
}

#[test]
fn cli_define_rejects_out_of_range_value() {
    let src = "
        global N: u8 = 2;
        fn main() {
            let _ = N;
        }
    ";
    let defines = vec![("N".to_string(), "256".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(
        has_invalid_global_override(&errors),
        "expected an InvalidGlobalOverride error for an out-of-range `u8`, got: {errors:?}"
    );
}

#[test]
fn cli_define_rejects_unsupported_type() {
    let src = "
        pub struct Point { x: Field, y: Field }
        global P: Point = Point { x: 1, y: 2 };
        fn main() {
            let _ = P;
        }
    ";
    let defines = vec![("P".to_string(), "3".to_string())];
    let errors = compile_with_defines(src, &defines);
    assert!(
        has_invalid_global_override(&errors),
        "expected an InvalidGlobalOverride error for a struct global, got: {errors:?}"
    );
}
