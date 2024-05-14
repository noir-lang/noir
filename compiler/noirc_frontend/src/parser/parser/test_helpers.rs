use chumsky::primitive::just;
use chumsky::Parser;
use iter_extended::vecmap;
use noirc_errors::CustomDiagnostic;

use crate::{
    lexer::Lexer,
    parser::{force, NoirParser},
    token::Token,
};

pub(crate) fn parse_with<P, T>(parser: P, program: &str) -> Result<T, Vec<CustomDiagnostic>>
where
    P: NoirParser<T>,
{
    let (tokens, lexer_errors) = Lexer::lex(program);
    if !lexer_errors.is_empty() {
        return Err(vecmap(&lexer_errors, Into::into));
    }
    parser.then_ignore(just(Token::EOF)).parse(tokens).map_err(|errors| vecmap(&errors, Into::into))
}

pub(crate) fn parse_recover<P, T>(parser: P, program: &str) -> (Option<T>, Vec<CustomDiagnostic>)
where
    P: NoirParser<T>,
{
    let (tokens, lexer_errors) = Lexer::lex(program);
    let (opt, errs) = parser.then_ignore(force(just(Token::EOF))).parse_recovery(tokens);

    let mut errors = vecmap(&lexer_errors, Into::into);
    errors.extend(errs.iter().map(Into::into));

    (opt, errors)
}

pub(crate) fn parse_all<P, T>(parser: P, programs: Vec<&str>) -> Vec<T>
where
    P: NoirParser<T>,
{
    vecmap(programs, move |program| {
        let message = format!("Failed to parse:\n{program}");
        let (op_t, diagnostics) = parse_recover(&parser, program);
        diagnostics.iter().for_each(|diagnostic| {
            if diagnostic.is_error() {
                panic!("{} with error {}", &message, diagnostic);
            }
        });
        op_t.expect(&message)
    })
}

pub(crate) fn parse_all_failing<P, T>(parser: P, programs: Vec<&str>) -> Vec<CustomDiagnostic>
where
    P: NoirParser<T>,
    T: std::fmt::Display,
{
    programs
            .into_iter()
            .flat_map(|program| match parse_with(&parser, program) {
                Ok(expr) => {
                    unreachable!(
                        "Expected this input to fail:\n{}\nYet it successfully parsed as:\n{}",
                        program, expr
                    )
                }
                Err(diagnostics) => {
                    if diagnostics.iter().all(|diagnostic: &CustomDiagnostic| diagnostic.is_warning()) {
                        unreachable!(
                            "Expected at least one error when parsing:\n{}\nYet it successfully parsed without errors:\n",
                            program
                        )
                    };
                    diagnostics
                }
            })
            .collect()
}

#[derive(Copy, Clone)]
pub(crate) struct Case {
    pub(crate) source: &'static str,
    pub(crate) errors: usize,
    pub(crate) expect: &'static str,
}

pub(crate) fn check_cases_with_errors<T, P>(cases: &[Case], parser: P)
where
    P: NoirParser<T> + Clone,
    T: std::fmt::Display,
{
    let show_errors = |v| vecmap(&v, ToString::to_string).join("\n");

    let results = vecmap(cases, |&case| {
        let (opt, errors) = parse_recover(parser.clone(), case.source);
        let actual = opt.map(|ast| ast.to_string());
        let actual = if let Some(s) = &actual { s.to_string() } else { "(none)".to_string() };

        let result = ((errors.len(), actual.clone()), (case.errors, case.expect.to_string()));
        if result.0 != result.1 {
            let num_errors = errors.len();
            let shown_errors = show_errors(errors);
            eprintln!(
                concat!(
                    "\nExpected {expected_errors} error(s) and got {num_errors}:",
                    "\n\n{shown_errors}",
                    "\n\nFrom input:   {src}",
                    "\nExpected AST: {expected_result}",
                    "\nActual AST:   {actual}\n",
                ),
                expected_errors = case.errors,
                num_errors = num_errors,
                shown_errors = shown_errors,
                src = case.source,
                expected_result = case.expect,
                actual = actual,
            );
        }
        result
    });

    assert_eq!(vecmap(&results, |t| t.0.clone()), vecmap(&results, |t| t.1.clone()),);
}
