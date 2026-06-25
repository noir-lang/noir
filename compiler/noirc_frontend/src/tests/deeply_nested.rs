use std::iter;

use itertools::Itertools;

use crate::{
    hir::{
        comptime::InterpreterError, def_collector::dc_crate::CompilationError,
        resolution::errors::ResolverError,
    },
    parser::ParserErrorReason,
    test_utils::{GetProgramOptions, get_program_with_options},
    tests::get_program_errors,
};

fn assert_parser_max_recursion_depth(src: &str, expected_error_count: Option<usize>) {
    let errors = get_program_errors(src);

    if let Some(expected_error_count) = expected_error_count {
        assert_eq!(errors.len(), expected_error_count, "Expected an exact number of errors");
    } else {
        // We didn't work out how many errors we will get exactly, but at there shouldn't
        // be an avalanche of them, which would indicate the error recovery does not work.
        assert!(errors.len() < 10, "Not expecting cascading errors");
    }

    for error in errors {
        if matches!(
            error,
            CompilationError::ParseError(parser_error)
                if parser_error.reason() == Some(&ParserErrorReason::MaximumRecursionDepthExceeded)
        ) {
            return;
        }
    }
    panic!("Expected a MaximumRecursionDepthExceeded error");
}

// stack overflow in the parser
#[test]
fn deeply_nested_tuples() {
    // Creates: (((((...((u8))...)))))
    const DEPTH: usize = 2000;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("({type_str})");
    }

    let src = format!(
        r#"
    pub fn main(x: {type_str}) -> pub u8 {{
        x
    }}
    "#
    );

    assert_parser_max_recursion_depth(&src, None);
}

#[test]
fn deeply_nested_arrays() {
    // Creates: [[[[...[[u8; 1]; 1]...; 1]; 1]; 1]
    const DEPTH: usize = 700;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("[{type_str}; 1]");
    }

    let src = format!(
        r#"
    pub fn main(_x: {type_str}) -> pub u8 {{
        0
    }}
    "#
    );

    assert_parser_max_recursion_depth(&src, None);
}

// stack overflow in the parser
#[test]
fn deeply_nested_generic_struct_parameter() {
    // Creates: Wrapper<Wrapper<Wrapper<...Wrapper<u8>...>>>
    const DEPTH: usize = 700;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("Wrapper<{type_str}>");
    }

    let src = format!(
        r#"
    pub struct Wrapper<T> {{ inner: T }}

    pub fn main(_x: {type_str}) -> pub u8 {{
        0
    }}
    "#
    );

    assert_parser_max_recursion_depth(&src, None);
}

#[test]
fn deeply_nested_generic_struct_field() {
    // Creates: Wrapper<Wrapper<Wrapper<...Wrapper<u8>...>>>
    const DEPTH: usize = 700;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("Wrapper<{type_str}>");
    }

    let src = format!(
        r#"
    pub struct Wrapper<T> {{ inner: T }}
    pub struct Root {{ inner: {type_str} }}

    pub fn main() {{ }}
    "#
    );

    assert_parser_max_recursion_depth(&src, None);
}

// Elaborator::mark_type_as_used was very slow on a deep struct like this.
#[test]
fn deeply_nested_generic_struct_parameter_elaboration() {
    // Creates: Wrapper<Wrapper<Wrapper<...Wrapper<u8>...>>>
    // Shouldn't run into parsing error due to recursion, but it is a big type.
    const DEPTH: usize = 99;
    let mut type_str = String::from("u8");

    for _ in 0..DEPTH {
        type_str = format!("Wrapper<{type_str}>");
    }

    let src = format!(
        r#"
    pub struct Wrapper<T> {{ inner: T }}

    pub fn main(_x: {type_str}) -> pub u8 {{
        0
    }}
    "#
    );

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let errors = get_program_with_options(
            &src,
            GetProgramOptions { allow_parser_errors: true, ..Default::default() },
        )
        .2;
        // Try to send, if the test is still waiting.
        let _ = tx.send(errors.len());
    });

    let error_count =
        rx.recv_timeout(std::time::Duration::from_secs(1)).expect("elaboration should be fast");

    assert_eq!(error_count, 0, "the program should to compile");
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
    for error in errors {
        if matches!(
            error,
            CompilationError::ResolverError(ResolverError::MaximumRecursionDepthExceeded { .. })
        ) {
            return;
        }
    }
    panic!("Expected a MaximumRecursionDepthExceeded error");
}

#[test]
fn deeply_nested_expression_evaluation_overflow() {
    // Build a deeply expression: (((1 + 2) + 3) + 4) ... + 50
    // This tests the comptime interpreter's evaluation depth limit.
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

    assert_parser_max_recursion_depth(&src, Some(1));
}
