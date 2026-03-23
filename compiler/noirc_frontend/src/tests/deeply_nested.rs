use std::iter;

use itertools::Itertools;

use crate::{
    hir::{comptime::InterpreterError, def_collector::dc_crate::CompilationError},
    tests::get_program_errors,
};

#[test]
fn deeply_nested_tuples() {
    // Creates: (((((...((u8))...)))))
    const DEPTH: usize = 2000;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("({})", type_str);
    }

    let src = format!(
        r#"
    pub fn main(x: {}) -> pub u8 {{
        x
    }}
    "#,
        type_str
    );

    let errors = get_program_errors(&src);
    assert!(errors.is_empty())
}

#[test]
fn deeply_nested_arrays() {
    // Creates: [[[[...[[u8; 1]; 1]...; 1]; 1]; 1]
    const DEPTH: usize = 700;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("[{}; 1]", type_str);
    }

    let src = format!(
        r#"
    pub fn main(x: {}) -> pub u8 {{
        0
    }}
    "#,
        type_str
    );

    let errors = get_program_errors(&src);
    assert!(errors.is_empty())
}

#[test]
fn deeply_nested_generic_structs() {
    // Creates: Wrapper<Wrapper<Wrapper<...Wrapper<u8>...>>>
    const DEPTH: usize = 700;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("Wrapper<{}>", type_str);
    }

    let src = format!(
        r#"
    struct Wrapper<T> {{ inner: T }}

    pub fn main(x: {}) -> pub u8 {{
        0
    }}
    "#,
        type_str
    );

    let errors = get_program_errors(&src);
    assert!(errors.is_empty())
}

#[test]
fn deeply_nested_terms() {
    // Creates: a + a + a + ... + a
    const DEPTH: usize = 2000;

    let terms = iter::repeat_n("a", DEPTH).join(" + ");

    let src = format!(
        r#"
    pub fn main(x: u64) -> pub u64 {{
        {terms}
    }}
    "#
    );

    let errors = get_program_errors(&src);
    assert!(errors.is_empty())
}

#[test]
fn deeply_nested_expression_evaluation_overflow() {
    // Build a deeply expression: (((1 + 2) + 3) + 4) ... + 50
    // This tests the interpreter's evaluation depth limit.
    // If we use an expression too deep (like 100), then we will reach the parser recursion limit.
    // We use fewer nesting levels (50) combined with recursive function calls
    // to trigger the interpreter's EvaluationDepthOverflow error.
    fn make_nested_expr(stem: &str) -> String {
        let mut expr = String::from(stem);
        for i in 2..=50 {
            expr = format!("({expr} + {i})");
        }
        expr
    }

    let expr = make_nested_expr("if max_depth == 0 { 1 } else { foo(max_depth - 1) }");

    let src = format!(
        "
      fn foo(max_depth: u32) -> u32 {{
        {expr}
      }}
      fn main() {{
          comptime {{
              let _ = foo(5);
          }}
      }}
      "
    );

    let errors = get_program_errors(&src);

    for error in errors {
        if matches!(
            error,
            CompilationError::InterpreterError(InterpreterError::EvaluationDepthOverflow { .. })
        ) {
            return;
        }
    }

    panic!("Expected a EvaluationDepthOverflow error");
}

#[test]
fn deeply_nested_expression_parser_overflow() {
    use crate::parser::ParserErrorReason;

    // Build expression: (((1 + 2) + 3) + 4) ... + 200
    // This should hit the parser's maximum recursion depth limit
    let mut expr = String::from("1");
    for i in 2..=200 {
        expr = format!("({expr} + {i})");
    }

    let src = format!(
        "
      fn main() {{
          comptime {{
              let _ = {expr};
          }}
      }}
      "
    );

    let errors = get_program_errors(&src);

    // We should get exactly one MaximumRecursionDepthExceeded error
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    let has_depth_error = matches!(
        &errors[0],
        CompilationError::ParseError(parser_error)
            if parser_error.reason() == Some(&ParserErrorReason::MaximumRecursionDepthExceeded)
    );
    assert!(has_depth_error, "Expected a MaximumRecursionDepthExceeded error");
}
