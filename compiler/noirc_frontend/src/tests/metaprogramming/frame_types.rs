//! Types solved inside one comptime frame must be visible wherever the value goes next.
//!
//! The interpreter records a type it solves while running (for example the result type of a
//! macro call) only in the substitution of the frame that solved it. A value keeps the type it
//! was built with, so a value built before the solve still mentions the unsolved type variable,
//! and only resolves when it passes through `Interpreter::value` while that frame is current.
//! Every route that carries a value from the frame that solved its type into another frame, or
//! out of the interpreter, must therefore resolve it on the way. Otherwise reflection such as
//! `type_of` sees `_` where the source has a concrete type, and code generated from it differs.
//!
//! Each case below builds a vector whose element type is solved to `S` only after the vector
//! exists. Then it moves the vector along one route and checks `type_of` in a frame that has
//! not solved the element type itself, so the check passes only if the route resolved it. The
//! builtin-argument case is the exception: `type_of` is itself the route, called from the frame
//! that solved the type.
//!
//! Routes that do not resolve types yet are listed as [`Expect::Stale`]. Their test asserts that
//! the check still fails, so fixing a route makes its test fail until it is moved to
//! [`Expect::Solved`].

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

enum Expect {
    /// The route resolves the types its frame solved.
    Solved,
    /// The route carries the unsolved type variable along; the reason names the code location.
    Stale(&'static str),
}

fn check(src: &str, expect: Expect) {
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

    match expect {
        Expect::Solved => {
            assert!(failed_checks.is_empty(), "type check failed: {failed_checks:?}");
        }
        Expect::Stale(reason) => assert!(
            !failed_checks.is_empty(),
            "this route now resolves types, so change it to `Expect::Solved` (was: {reason})"
        ),
    }
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
    check(&src, Expect::Stale("call_function: arguments of builtins (call_special)"));
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
    check(&src, Expect::Stale("call_function: arguments of user-defined functions"));
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
    check(&src, Expect::Stale("evaluate_method_call -> call_function: method arguments"));
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
    check(src, Expect::Stale("call_function: arguments holding a `Value::Type`"));
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
    check(src, Expect::Stale("call_function: bindings inside a `Value::Function` argument"));
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
    check(src, Expect::Stale("call_function: result of user-defined functions"));
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
    check(src, Expect::Stale("call_function: result of user-defined functions, nested"));
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
    check(src, Expect::Stale("evaluate_array type read before elements, then function result"));
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
    check(&src, Expect::Solved);
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
    check(&src, Expect::Stale("call_closure: arguments"));
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
    check(src, Expect::Stale("evaluate_lambda: captured values and substitution snapshot"));
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
    check(src, Expect::Solved);
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
    check(src, Expect::Solved);
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
    check(src, Expect::Stale("call_function: pointee of a `&mut` argument"));
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
    check(src, Expect::Solved);
}
