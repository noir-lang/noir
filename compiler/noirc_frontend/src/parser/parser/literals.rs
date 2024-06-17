use chumsky::Parser;

use crate::{
    ast::ExpressionKind,
    parser::NoirParser,
    token::{Token, TokenKind},
};

use super::primitives::token_kind;

pub(super) fn literal() -> impl NoirParser<ExpressionKind> {
    token_kind(TokenKind::Literal).map(|token| match token {
        Token::Int(x) => ExpressionKind::integer(x),
        Token::Bool(b) => ExpressionKind::boolean(b),
        Token::Str(s) => ExpressionKind::string(s),
        Token::RawStr(s, hashes) => ExpressionKind::raw_string(s, hashes),
        Token::FmtStr(s) => ExpressionKind::format_string(s),
        unexpected => unreachable!("Non-literal {} parsed as a literal", unexpected),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::Literal;
    use crate::parser::parser::{
        expression, expression_no_constructors, fresh_statement, term, test_helpers::*,
    };

    fn expr_to_lit(expr: ExpressionKind) -> Literal {
        match expr {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!("expected a literal"),
        }
    }

    #[test]
    fn parse_int() {
        let int = parse_with(literal(), "5").unwrap();
        let hex = parse_with(literal(), "0x05").unwrap();

        match (expr_to_lit(int), expr_to_lit(hex)) {
            (Literal::Integer(int, false), Literal::Integer(hex, false)) => assert_eq!(int, hex),
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_string() {
        let expr = parse_with(literal(), r#""hello""#).unwrap();
        match expr_to_lit(expr) {
            Literal::Str(s) => assert_eq!(s, "hello"),
            _ => unreachable!(),
        };
    }

    #[test]
    fn parse_bool() {
        let expr_true = parse_with(literal(), "true").unwrap();
        let expr_false = parse_with(literal(), "false").unwrap();

        match (expr_to_lit(expr_true), expr_to_lit(expr_false)) {
            (Literal::Bool(t), Literal::Bool(f)) => {
                assert!(t);
                assert!(!f);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn parse_unary() {
        parse_all(
            term(expression(), expression_no_constructors(expression()), fresh_statement(), true),
            vec!["!hello", "-hello", "--hello", "-!hello", "!-hello"],
        );
        parse_all_failing(
            term(expression(), expression_no_constructors(expression()), fresh_statement(), true),
            vec!["+hello", "/hello"],
        );
    }

    #[test]
    fn parse_raw_string_expr() {
        let cases = vec![
            Case { source: r#" r"foo" "#, expect: r#"r"foo""#, errors: 0 },
            Case { source: r##" r#"foo"# "##, expect: r##"r#"foo"#"##, errors: 0 },
            // backslash
            Case { source: r#" r"\\" "#, expect: r#"r"\\""#, errors: 0 },
            Case { source: r##" r#"\"# "##, expect: r##"r#"\"#"##, errors: 0 },
            Case { source: r##" r#"\\"# "##, expect: r##"r#"\\"#"##, errors: 0 },
            Case { source: r##" r#"\\\"# "##, expect: r##"r#"\\\"#"##, errors: 0 },
            // escape sequence
            Case {
                source: r##" r#"\t\n\\t\\n\\\t\\\n\\\\"# "##,
                expect: r##"r#"\t\n\\t\\n\\\t\\\n\\\\"#"##,
                errors: 0,
            },
            Case { source: r##" r#"\\\\\\\\"# "##, expect: r##"r#"\\\\\\\\"#"##, errors: 0 },
            // mismatch - errors:
            Case { source: r###" r#"foo"## "###, expect: r##"r#"foo"#"##, errors: 1 },
            Case { source: r##" r##"foo"# "##, expect: "(none)", errors: 2 },
            // mismatch: short:
            Case { source: r##" r"foo"# "##, expect: r#"r"foo""#, errors: 1 },
            Case { source: r#" r#"foo" "#, expect: "(none)", errors: 2 },
            // empty string
            Case { source: r#"r"""#, expect: r#"r"""#, errors: 0 },
            #[allow(clippy::needless_raw_string_hashes)]
            Case { source: r####"r###""###"####, expect: r####"r###""###"####, errors: 0 },
            // miscellaneous
            Case { source: r##" r#\"foo\"# "##, expect: "plain::r", errors: 2 },
            Case { source: r#" r\"foo\" "#, expect: "plain::r", errors: 1 },
            Case { source: r##" r##"foo"# "##, expect: "(none)", errors: 2 },
            // missing 'r' letter
            Case { source: r##" ##"foo"# "##, expect: r#""foo""#, errors: 2 },
            Case { source: r#" #"foo" "#, expect: "plain::foo", errors: 2 },
            // whitespace
            Case { source: r##" r #"foo"# "##, expect: "plain::r", errors: 2 },
            Case { source: r##" r# "foo"# "##, expect: "plain::r", errors: 3 },
            Case { source: r#" r#"foo" # "#, expect: "(none)", errors: 2 },
            // after identifier
            Case { source: r##" bar#"foo"# "##, expect: "plain::bar", errors: 2 },
            // nested
            Case {
                source: r###"r##"foo r#"bar"# r"baz" ### bye"##"###,
                expect: r###"r##"foo r#"bar"# r"baz" ### bye"##"###,
                errors: 0,
            },
        ];

        check_cases_with_errors(&cases[..], expression());
    }

    #[test]
    fn parse_raw_string_lit() {
        let lit_cases = vec![
            Case { source: r#" r"foo" "#, expect: r#"r"foo""#, errors: 0 },
            Case { source: r##" r#"foo"# "##, expect: r##"r#"foo"#"##, errors: 0 },
            // backslash
            Case { source: r#" r"\\" "#, expect: r#"r"\\""#, errors: 0 },
            Case { source: r##" r#"\"# "##, expect: r##"r#"\"#"##, errors: 0 },
            Case { source: r##" r#"\\"# "##, expect: r##"r#"\\"#"##, errors: 0 },
            Case { source: r##" r#"\\\"# "##, expect: r##"r#"\\\"#"##, errors: 0 },
            // escape sequence
            Case {
                source: r##" r#"\t\n\\t\\n\\\t\\\n\\\\"# "##,
                expect: r##"r#"\t\n\\t\\n\\\t\\\n\\\\"#"##,
                errors: 0,
            },
            Case { source: r##" r#"\\\\\\\\"# "##, expect: r##"r#"\\\\\\\\"#"##, errors: 0 },
            // mismatch - errors:
            Case { source: r###" r#"foo"## "###, expect: r##"r#"foo"#"##, errors: 1 },
            Case { source: r##" r##"foo"# "##, expect: "(none)", errors: 2 },
        ];

        check_cases_with_errors(&lit_cases[..], literal());
    }
}
