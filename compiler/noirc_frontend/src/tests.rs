#![cfg(test)]

mod aliases;
mod arithmetic_generics;
mod arrays;
mod bound_checks;
mod cast;
mod control_flow;
mod enums;
mod expressions;
mod functions;
mod globals;
mod imports;
mod lambdas;
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
use std::collections::HashMap;

use crate::elaborator::{FrontendOptions, UnstableFeature};
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

fn assert_no_errors(src: &str) -> Context<'_, '_> {
    let (_, context, errors) = get_program(src);
    if !errors.is_empty() {
        let errors = errors.iter().map(CustomDiagnostic::from).collect::<Vec<_>>();
        report_all(context.file_manager.as_file_map(), &errors, false, false);
        panic!("Expected no errors");
    }
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
    let mut primary_spans_with_errors: HashMap<Span, String> =
        primary_spans_with_errors.into_iter().collect();

    let mut secondary_spans_with_errors: HashMap<Span, String> =
        secondary_spans_with_errors.into_iter().collect();

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
        let span = secondary.location.span;
        let message = &error.message;
        let Some(expected_message) = primary_spans_with_errors.remove(&span) else {
            if let Some(message) = secondary_spans_with_errors.get(&span) {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                panic!(
                    "Error at {span:?} with message {message:?} is annotated as secondary but should be primary"
                );
            } else {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                panic!(
                    "Couldn't find primary error at {span:?} with message {message:?}.\nAll errors: {errors:?}"
                );
            }
        };

        if message != &expected_message {
            report_all(context.file_manager.as_file_map(), &errors, false, false);
            assert_eq!(
                message, &expected_message,
                "Primary error at {span:?} has unexpected message"
            );
        }

        for secondary in &error.secondaries {
            let message = &secondary.message;
            if message.is_empty() {
                continue;
            }

            let span = secondary.location.span;
            let Some(expected_message) = secondary_spans_with_errors.remove(&span) else {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                if let Some(message) = primary_spans_with_errors.get(&span) {
                    panic!(
                        "Error at {span:?} with message {message:?} is annotated as primary but should be secondary"
                    );
                } else {
                    panic!(
                        "Couldn't find secondary error at {span:?} with message {message:?}.\nAll errors: {errors:?}"
                    );
                };
            };

            if message != &expected_message {
                report_all(context.file_manager.as_file_map(), &errors, false, false);
                assert_eq!(
                    message, &expected_message,
                    "Secondary error at {span:?} has unexpected message"
                );
            }
        }
    }

    if !primary_spans_with_errors.is_empty() {
        report_all(context.file_manager.as_file_map(), &errors, false, false);
        panic!("These primary errors didn't happen: {primary_spans_with_errors:?}");
    }

    if !secondary_spans_with_errors.is_empty() {
        report_all(context.file_manager.as_file_map(), &errors, false, false);
        panic!("These secondary errors didn't happen: {secondary_spans_with_errors:?}");
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
