#![cfg(test)]

mod aliases;
mod arithmetic_generics;
mod arrays;
mod assignment;
mod bound_checks;
mod cast;
mod control_flow;
mod enums;
mod expressions;
mod functions;
mod globals;
mod imports;
mod lambdas;
mod meta_quote_roundtrip;
mod metaprogramming;
mod name_shadowing;
mod numeric_generics;
mod oracles;
mod references;
mod runtime;
mod structs;
mod traits;
mod turbofish;
mod unused_items;
mod visibility;

// XXX: These tests repeat a lot of code
// what we should do is have test cases which are passed to a test harness
// A test harness will allow for more expressive and readable tests
use std::collections::{HashMap, HashSet};

use crate::elaborator::{FrontendOptions, UnstableFeature};
use crate::hir::comptime::InterpreterError;
use crate::hir::printer::display_crate;
use crate::test_utils::{get_program, get_program_with_options};

use noirc_errors::reporter::report_all;
use noirc_errors::{CustomDiagnostic, Span};

use crate::hir::Context;
use crate::hir::def_collector::dc_crate::CompilationError;

use crate::ParsedModule;

pub(crate) fn get_program_using_features(
    src: &str,
    features: &[UnstableFeature],
) -> (ParsedModule, Context<'static, 'static>, Vec<CompilationError>) {
    let allow_parser_errors = false;
    let mut options = FrontendOptions::test_default();
    options.enabled_unstable_features = features;
    get_program_with_options(src, allow_parser_errors, options)
}

pub(crate) fn get_program_errors(src: &str) -> Vec<CompilationError> {
    get_program(src).2
}

pub(crate) fn assert_no_errors(src: &str) -> Context<'_, '_> {
    let (_, context, errors) = get_program(src);
    if !errors.is_empty() {
        let errors = errors.iter().map(CustomDiagnostic::from).collect::<Vec<_>>();
        report_all(context.file_manager.as_file_map(), &errors, false, false);
        panic!("Expected no errors");
    }
    context
}

pub fn assert_no_errors_without_report(src: &str) -> Context<'_, '_> {
    let (_, context, errors) = get_program(src);
    assert!(errors.is_empty(), "Expected no errors");
    context
}

fn assert_no_errors_and_to_string(src: &str) -> String {
    let context = assert_no_errors(src);
    display_crate(
        *context.crate_graph.root_crate_id(),
        &context.crate_graph,
        &context.def_maps,
        &context.def_interner,
    )
}

/// Given a source file with annotated errors, like this
///
/// fn main() -> pub i32 {
///                  ^^^ expected i32 because of return type
///     true
///     ~~~~ bool returned here
/// }
///
/// where:
/// - lines with "^^^" are primary errors
/// - lines with "~~~" are secondary errors
///
/// this method will check that compiling the program without those error markers
/// will produce errors at those locations and with/ those messages.
fn check_errors(src: &str) {
    let allow_parser_errors = false;
    let monomorphize = false;
    check_errors_with_options(
        src,
        allow_parser_errors,
        monomorphize,
        FrontendOptions::test_default(),
    );
}

fn check_errors_using_features(src: &str, features: &[UnstableFeature]) {
    let allow_parser_errors = false;
    let monomorphize = false;
    let options =
        FrontendOptions { enabled_unstable_features: features, ..FrontendOptions::test_default() };
    check_errors_with_options(src, allow_parser_errors, monomorphize, options);
}

pub(super) fn check_monomorphization_error(src: &str) {
    check_monomorphization_error_using_features(src, &[]);
}

pub(super) fn check_monomorphization_error_using_features(src: &str, features: &[UnstableFeature]) {
    let allow_parser_errors = false;
    let monomorphize = true;
    check_errors_with_options(
        src,
        allow_parser_errors,
        monomorphize,
        FrontendOptions { enabled_unstable_features: features, ..FrontendOptions::test_default() },
    );
}

fn check_errors_with_options(
    src: &str,
    allow_parser_errors: bool,
    monomorphize: bool,
    options: FrontendOptions,
) {
    let lines = src.lines().collect::<Vec<_>>();

    // Here we'll hold just the lines that are code
    let mut code_lines = Vec::new();
    // Here we'll capture lines that are primary error spans, like:
    //
    //   ^^^ error message
    let mut primary_spans_with_errors: Vec<(Span, String)> = Vec::new();
    // Here we'll capture lines that are secondary error spans, like:
    //
    //   ~~~ error message
    let mut secondary_spans_with_errors: Vec<(Span, String)> = Vec::new();

    // The byte at the start of this line
    let mut byte = 0;
    // The length of the last line, needed to go back to the byte at the beginning of the last line
    let mut last_line_length = 0;
    for line in lines {
        if let Some((span, message)) =
            get_error_line_span_and_message(line, '^', byte, last_line_length)
        {
            primary_spans_with_errors.push((span, message));
            continue;
        }

        if let Some((span, message)) =
            get_error_line_span_and_message(line, '~', byte, last_line_length)
        {
            secondary_spans_with_errors.push((span, message));
            continue;
        }

        code_lines.push(line);

        byte += line.len() + 1; // For '\n'
        last_line_length = line.len();
    }

    // We might see the same primary message multiple time with different secondary.
    fn to_message_map(messages: Vec<(Span, String)>) -> HashMap<Span, Vec<String>> {
        let mut map = HashMap::<_, Vec<String>>::new();
        for (span, msg) in messages {
            map.entry(span).or_default().push(msg);
        }
        map
    }

    // By the end we want to have seen all errors.
    let mut all_primaries: HashSet<(Span, String)> =
        HashSet::from_iter(primary_spans_with_errors.clone());
    let mut all_secondaries: HashSet<(Span, String)> =
        HashSet::from_iter(secondary_spans_with_errors.clone());

    let primary_spans_with_errors = to_message_map(primary_spans_with_errors);
    let secondary_spans_with_errors = to_message_map(secondary_spans_with_errors);

    let src = code_lines.join("\n");
    let (_, mut context, errors) = get_program_with_options(&src, allow_parser_errors, options);
    let mut errors = errors.iter().map(CustomDiagnostic::from).collect::<Vec<_>>();

    if monomorphize {
        if !errors.is_empty() {
            report_all(context.file_manager.as_file_map(), &errors, false, false);
            panic!("Expected no errors before monomorphization");
        }

        let main = context.get_main_function(context.root_crate_id()).unwrap_or_else(|| {
            panic!("get_monomorphized: test program contains no 'main' function")
        });

        let result = crate::monomorphization::monomorphize(main, &mut context.def_interner, false);
        match result {
            Ok(_) => {
                if primary_spans_with_errors.is_empty() {
                    return;
                }
                panic!("Expected a monomorphization error but got none")
            }
            Err(error) => {
                errors.push(error.into());
            }
        }
    }

    if errors.is_empty() && !primary_spans_with_errors.is_empty() {
        panic!("Expected some errors but got none");
    }

    for error in &errors {
        let secondary = error
            .secondaries
            .first()
            .unwrap_or_else(|| panic!("Expected {error:?} to have a secondary label"));

        let primary_message = &error.message;
        let span = secondary.location.span;

        let Some(expected_primaries) = primary_spans_with_errors.get(&span) else {
            if let Some(secondaries) = secondary_spans_with_errors.get(&span) {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                panic!(
                    "Error at {span:?} with message(s) {secondaries:?} is annotated as secondary but should be primary: {primary_message:?}"
                );
            } else {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                panic!(
                    "Couldn't find primary error at {span:?} with message {primary_message:?}.\nAll errors: {errors:?}"
                );
            }
        };

        if !expected_primaries.contains(primary_message) {
            report_all(context.file_manager.as_file_map(), &errors, false, false);
            panic!(
                "Primary error at {span:?} has unexpected message: {primary_message:?}; should be one of {expected_primaries:?}"
            );
        } else {
            all_primaries.remove(&(span, primary_message.clone()));
        }

        for secondary in &error.secondaries {
            let secondary_message = &secondary.message;
            if secondary_message.is_empty() {
                continue;
            }

            let span = secondary.location.span;
            let Some(expected_secondaries) = secondary_spans_with_errors.get(&span) else {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                if let Some(primaries) = primary_spans_with_errors.get(&span) {
                    panic!(
                        "Error at {span:?} with message(s) {primaries:?} is annotated as primary but should be secondary: {secondary_message:?}"
                    );
                } else {
                    panic!(
                        "Couldn't find secondary error at {span:?} with message {secondary_message:?}.\nAll errors: {errors:?}"
                    );
                };
            };

            if !expected_secondaries.contains(secondary_message) {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                panic!(
                    "Secondary error at {span:?} has unexpected message: {secondary_message:?}; should be one of {expected_secondaries:?}"
                );
            } else {
                all_secondaries.remove(&(span, secondary_message.clone()));
            }
        }
    }

    if !all_primaries.is_empty() {
        report_all(context.file_manager.as_file_map(), &errors, false, false);
        panic!("These primary errors didn't happen: {all_primaries:?}");
    }

    if !all_secondaries.is_empty() {
        report_all(context.file_manager.as_file_map(), &errors, false, false);
        panic!("These secondary errors didn't happen: {all_secondaries:?}");
    }
}

/// Helper function for `check_errors` that returns the span that
/// `^^^^` or `~~~~` occupy, together with the message that follows it.
fn get_error_line_span_and_message(
    line: &str,
    char: char,
    byte: usize,
    last_line_length: usize,
) -> Option<(Span, String)> {
    if !line.trim().starts_with(char) {
        return None;
    }

    let chars = line.chars().collect::<Vec<_>>();
    let first_caret = chars.iter().position(|c| *c == char).unwrap();
    let last_caret = chars.iter().rposition(|c| *c == char).unwrap(); // cSpell:disable-line
    let start = byte - last_line_length;
    let span = Span::from((start + first_caret - 1) as u32..(start + last_caret) as u32);
    let error = line.trim().trim_start_matches(char).trim().to_string();
    Some((span, error))
}

#[test]
fn uses_self_in_import() {
    let src = r#"
    mod moo {
        pub mod bar {
            pub fn foo() -> i32 {
                1
            }
        }
    }

    use moo::bar::{self};

    pub fn baz() -> i32 {
        bar::foo()
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_stack_overflow_on_many_comments_in_a_row() {
    let mut src = "//\n".repeat(10_000);
    src.push_str("fn main() { }");
    assert_no_errors(&src);
}

#[test]
fn wildcard_with_generic_argument() {
    let src = r#"
    struct Foo<T> {}

    pub fn println<T>(_input: T) { }
    
    fn main() {
      let x: _<_> = "123";
      let y: _<_> = Foo::<()> { };
      println(x);
      println(y);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10553() {
    let src = r#"
    pub fn println<T>(_input: T) { }
    fn main() {
        let x = &[false];
        let s = f"{x}";
        let _ = &[s];
                ^^^^ Nested vectors, i.e. vectors within an array or vector, are not supported
        println(s);
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn regression_10554() {
    let src = r#"
    pub fn println<T>(_input: T) { }
    fn main() {
        let x = &[false];
        let t = &[x];
                ^^^^ Nested vectors, i.e. vectors within an array or vector, are not supported
        let s = f"{t}";
        println(s);
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn deeply_nested_expression_overflow() {
    // Build a deeply expression: (((1 + 2) + 3) + 4) ... + 100
    // If we build it too deep (like 200), then even the parser gets stack overflow,
    // but `nargo` uses a larger stack size, so it can go higher than the test.
    // Instead we use it to build a mix of recursive calls and nested expressions,
    // so that we can provide an overall limit on evaluation depth.
    fn make_nested_expr(stem: &str) -> String {
        let mut expr = String::from(stem);
        for i in 2..=100 {
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

    println!("{src}");

    let errors = get_program_errors(&src);

    for error in errors {
        if matches!(
            error,
            CompilationError::InterpreterError(InterpreterError::EvaluationDepthOverflow { .. })
        ) {
            return;
        }
    }

    panic!("should have got a EvaluationDepthOverflow error");
}
