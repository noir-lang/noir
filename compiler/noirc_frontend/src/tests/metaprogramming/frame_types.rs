//! Types solved inside one comptime frame must be visible wherever the value goes next.
//!
//! The interpreter records a type it solves while running (for example the result type of a
//! macro call) in the substitution of the frame that solved it, not on the type variable. A value
//! keeps the type it was built with, so a value built before the solve still mentions the
//! unsolved type variable, and only resolves against a substitution that holds the solution.
//! Every route that carries such a value into another frame, or out of the interpreter, must
//! therefore either resolve it on the way or make the solution visible on the other side.
//! Otherwise reflection such as `type_of` sees `_` where the source has a concrete type, and code
//! generated from it differs.
//!
//! Each case below builds a vector whose element type is solved to `S` only after the vector
//! exists. Then it moves the vector along one route and checks `type_of` in a frame that has
//! not solved the element type itself, so the check passes only if the route resolved it. The
//! builtin-argument case is the exception: `type_of` is itself the route, called from the frame
//! that solved the type.

use noirc_errors::CustomDiagnostic;

use crate::hir::comptime::{ComptimeError, InterpreterError};
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::test_utils::{GetProgramOptions, get_program_with_options};

const PRELUDE: &str = r#"
    #[builtin(type_of)]
    pub comptime fn type_of<T>(_x: T) -> Type {}

    impl Type {
        #[builtin(type_eq)]
        pub comptime fn eq(self, _other: Self) -> bool {}
    }

    impl Quoted {
        #[builtin(quoted_as_type)]
        pub comptime fn as_type(self) -> Type {}
    }

    struct S { a: u8 }

    comptime fn make_s() -> Quoted {
        quote { S { a: 1 } }
    }

    struct Wrap<T> { inner: T }
"#;

/// Asserts that `src` compiles and that none of its comptime `assert`s fail.
fn check(src: &str) {
    let src = format!("{PRELUDE}\n{src}");
    let options = GetProgramOptions {
        allow_elaborator_errors: true,
        root_and_stdlib: true,
        ..Default::default()
    };
    let errors = get_program_with_options(&src, options).2;
    let errors = errors.into_iter().filter(|error| CustomDiagnostic::from(error).is_error());

    let (failed_checks, other_errors): (Vec<_>, Vec<_>) = errors.partition(is_failing_constraint);
    assert!(other_errors.is_empty(), "unexpected errors: {other_errors:?}");

    assert!(failed_checks.is_empty(), "type check failed: {failed_checks:?}");
}

fn is_failing_constraint(error: &CompilationError) -> bool {
    match error {
        CompilationError::InterpreterError(InterpreterError::FailingConstraint { .. }) => true,
        CompilationError::ComptimeError(ComptimeError::ErrorRunningAttribute { error, .. }) => {
            matches!(
                **error,
                CompilationError::InterpreterError(InterpreterError::FailingConstraint { .. })
            )
        }
        _ => false,
    }
}

/// Built in the current frame: `e: [?]`, then `?` is solved to `S`.
const SOLVE_HERE: &str = r#"
            let e = @[];
            let x = make_s!();
            let _ = [e, @[x]];
"#;

#[test]
fn builtin_argument_in_the_solving_frame() {
    // `type_of` runs in its own frame, and builtin arguments are not resolved on the way in.
    let src = format!(
        r#"
    fn main() {{
        comptime {{
            {SOLVE_HERE}
            assert(type_of(e).eq(quote {{ [S] }}.as_type()));
        }}
    }}
    "#
    );
    check(&src);
}

#[test]
fn function_argument() {
    let src = format!(
        r#"
    comptime fn is_vec_of_s<U>(x: U) -> bool {{
        type_of(x).eq(quote {{ [S] }}.as_type())
    }}
    fn main() {{
        comptime {{
            {SOLVE_HERE}
            assert(is_vec_of_s(e));
        }}
    }}
    "#
    );
    check(&src);
}

#[test]
fn method_argument() {
    let src = format!(
        r#"
    struct Checker {{}}
    impl Checker {{
        comptime fn is_vec_of_s<U>(_self: Self, x: U) -> bool {{
            type_of(x).eq(quote {{ [S] }}.as_type())
        }}
    }}
    fn main() {{
        comptime {{
            {SOLVE_HERE}
            assert(Checker {{}}.is_vec_of_s(e));
        }}
    }}
    "#
    );
    check(&src);
}

#[test]
fn type_value_argument() {
    // The value carried across is a `Type` taken before the solve, not the vector itself.
    let src = r#"
    comptime fn is_vec_of_s(t: Type) -> bool {
        t.eq(quote { [S] }.as_type())
    }
    fn main() {
        comptime {
            let e = @[];
            let t = type_of(e);
            let x = make_s!();
            let _ = [e, @[x]];
            assert(is_vec_of_s(t));
        }
    }
    "#;
    check(src);
}

#[test]
fn function_value_argument() {
    // `f`'s instantiation bindings are taken when `f` is evaluated, before the solve.
    let src = r#"
    comptime fn elem_is_s<T>(_witness: [T]) -> bool {
        let fresh: [T] = @[];
        type_of(fresh).eq(quote { [S] }.as_type())
    }
    comptime fn call_it<U>(f: fn([U]) -> bool, e: [U]) -> bool {
        f(e)
    }
    fn main() {
        comptime {
            let e = @[];
            let f = elem_is_s;
            let x = make_s!();
            let _ = [e, @[x]];
            assert(call_it(f, e));
        }
    }
    "#;
    check(src);
}

#[test]
fn function_result() {
    let src = r#"
    comptime fn make<T>() -> [T] {
        let e: [T] = @[];
        let _: T = make_s!();
        e
    }
    fn main() {
        comptime {
            let v = make();
            assert(type_of(v).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn function_result_inside_a_struct() {
    let src = r#"
    comptime fn make<T>() -> Wrap<[T]> {
        let e: [T] = @[];
        let _: T = make_s!();
        Wrap { inner: e }
    }
    fn main() {
        comptime {
            let w = make();
            assert(type_of(w.inner).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn array_literal_result() {
    // The array literal reads its own type before `f()` solves `T`; the array then leaves `apply`
    // as its result.
    let src = r#"
    comptime fn apply<T>(f: fn() -> T) -> [T; 1] {
        [f()]
    }
    fn main() {
        comptime {
            let c = || make_s!();
            let v = apply(c);
            assert(type_of(v).eq(quote { [S; 1] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn repeated_array_literal_result() {
    let src = r#"
    comptime fn apply<T>(f: fn() -> T) -> [T; 2] {
        [f(); 2]
    }
    fn main() {
        comptime {
            let c = || make_s!();
            let v = apply(c);
            assert(type_of(v).eq(quote { [S; 2] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn vector_literal_result() {
    let src = r#"
    comptime fn apply<T>(f: fn() -> T) -> [T] {
        @[f()]
    }
    fn main() {
        comptime {
            let c = || make_s!();
            let v = apply(c);
            assert(type_of(v).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn closure_result() {
    let src = format!(
        r#"
    fn main() {{
        comptime {{
            let c = || {{
                {SOLVE_HERE}
                e
            }};
            let v = c();
            assert(type_of(v).eq(quote {{ [S] }}.as_type()));
        }}
    }}
    "#
    );
    check(&src);
}

#[test]
fn closure_argument() {
    // The closure is created before the solve, so its own substitution does not have it either.
    let src = format!(
        r#"
    fn main() {{
        comptime {{
            let is_vec_of_s = |x| type_of(x).eq(quote {{ [S] }}.as_type());
            {SOLVE_HERE}
            assert(is_vec_of_s(e));
        }}
    }}
    "#
    );
    check(&src);
}

#[test]
fn closure_capture() {
    let src = r#"
    fn main() {
        comptime {
            let e = @[];
            let is_vec_of_s = || type_of(e).eq(quote { [S] }.as_type());
            let x = make_s!();
            let _ = [e, @[x]];
            assert(is_vec_of_s());
        }
    }
    "#;
    check(src);
}

#[test]
fn write_through_mutable_reference() {
    let src = r#"
    comptime fn fill<T>(r: &mut [T]) {
        let e: [T] = @[];
        let _: T = make_s!();
        *r = e;
    }
    fn main() {
        comptime {
            let mut v = @[];
            fill(&mut v);
            assert(type_of(v).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn write_through_mutable_reference_to_a_field() {
    let src = r#"
    comptime fn fill<T>(r: &mut Wrap<[T]>) {
        let e: [T] = @[];
        let _: T = make_s!();
        r.inner = e;
    }
    fn main() {
        comptime {
            let mut w = Wrap { inner: @[] };
            fill(&mut w);
            assert(type_of(w.inner).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn read_through_mutable_reference() {
    // The caller solves the type; the callee reads the pointee, which arguments never resolve.
    let src = r#"
    comptime fn is_vec_of_s<U>(r: &mut U) -> bool {
        type_of(*r).eq(quote { [S] }.as_type())
    }
    fn main() {
        comptime {
            let mut v = @[];
            let x = make_s!();
            let _ = [v, @[x]];
            assert(is_vec_of_s(&mut v));
        }
    }
    "#;
    check(src);
}

#[test]
fn unquote() {
    let src = r#"
    comptime fn wrap<T>(_f: fn() -> T) -> Quoted {
        let e: [T] = @[];
        let _: T = make_s!();
        quote { $e }
    }
    fn main() {
        comptime {
            let c = || make_s!();
            let v = wrap!(c);
            assert(type_of(v).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn function_result_solved_differently_on_each_loop_iteration() {
    let src = r#"
    comptime fn pick(n: u32) -> Quoted {
        if n == 0 { quote { S { a: 1 } } } else { quote { 1_u8 } }
    }
    comptime fn make<T>(n: u32) -> [T] {
        let e: [T] = @[];
        let _: T = pick!(n);
        e
    }
    fn main() {
        comptime {
            for i in 0..2 {
                let v = make(i);
                if i == 0 {
                    assert(type_of(v).eq(quote { [S] }.as_type()));
                } else {
                    assert(type_of(v).eq(quote { [u8] }.as_type()));
                }
            }
        }
    }
    "#;
    check(src);
}

#[test]
fn function_result_through_nested_generic_calls() {
    let src = r#"
    comptime fn nest<T>(n: u32) -> [T] {
        if n == 0 {
            let e: [T] = @[];
            let _: T = make_s!();
            e
        } else {
            nest(n - 1)
        }
    }
    fn main() {
        comptime {
            let v = nest(2);
            assert(type_of(v).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn recursive_call_solving_its_own_body_differently_keeps_the_callers_solution() {
    let src = r#"
    comptime fn pick(n: u32) -> Quoted {
        if n == 0 { quote { S { a: 1 } } } else { quote { 1_u8 } }
    }
    comptime fn rec<T>(n: u32) -> bool {
        let e = @[];
        let x = pick!(n);
        let _ = [e, @[x]];
        if n > 0 {
            let _: bool = rec::<T>(n - 1);
        }
        if n == 0 {
            type_of(e).eq(quote { [S] }.as_type())
        } else {
            type_of(e).eq(quote { [u8] }.as_type())
        }
    }
    fn main() {
        comptime {
            assert(rec::<Field>(1));
        }
    }
    "#;
    check(src);
}

#[test]
fn closure_body_solving_a_captured_variable() {
    // The closure's body solves the element type of `e`, which belongs to the frame that created
    // the closure.
    let src = r#"
    fn main() {
        comptime {
            let e = @[];
            let c = || {
                let x = make_s!();
                let _ = [e, @[x]];
            };
            c();
            assert(type_of(e).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn closure_body_solving_a_captured_variable_called_from_a_function() {
    let src = r#"
    comptime fn run<Env>(f: fn[Env]() -> ()) {
        f()
    }
    fn main() {
        comptime {
            let e = @[];
            let c = || {
                let x = make_s!();
                let _ = [e, @[x]];
            };
            run(c);
            assert(type_of(e).eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}

#[test]
fn comptime_block_value_inlined_into_runtime_code() {
    // The struct is inlined with the type stored on it, so it must be resolved before the
    // interpreter that solved its generic is dropped.
    let src = r#"
    fn main() {
        let w = comptime {
            let w = Wrap { inner: @[] };
            let x = make_s!();
            let _ = [w, Wrap { inner: @[x] }];
            w
        };
        let _ = w;
    }
    "#;
    check(src);
}

#[test]
fn type_value_result() {
    // `t` is taken before the solve and the element type is local to `element_type_of_made`, so
    // the caller never learns it; only the callee can resolve `t`.
    let src = r#"
    comptime fn element_type_of_made() -> Type {
        let e = @[];
        let t = type_of(e);
        let x = make_s!();
        let _ = [e, @[x]];
        t
    }
    fn main() {
        comptime {
            assert(element_type_of_made().eq(quote { [S] }.as_type()));
        }
    }
    "#;
    check(src);
}
