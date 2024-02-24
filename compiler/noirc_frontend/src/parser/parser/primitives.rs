use chumsky::prelude::*;

use crate::{
    parser::{labels::ParsingRuleLabel, ExprParser, NoirParser, ParserError},
    token::{Keyword, Token, TokenKind},
    ExpressionKind, Ident, UnaryOp,
};

use super::path;

/// This parser always parses no input and fails
pub(super) fn nothing<T>() -> impl NoirParser<T> {
    one_of([]).map(|_| unreachable!("parser should always error"))
}

pub(super) fn keyword(keyword: Keyword) -> impl NoirParser<Token> {
    just(Token::Keyword(keyword))
}

pub(super) fn token_kind(token_kind: TokenKind) -> impl NoirParser<Token> {
    filter_map(move |span, found: Token| {
        if found.kind() == token_kind {
            Ok(found)
        } else {
            Err(ParserError::expected_label(
                ParsingRuleLabel::TokenKind(token_kind.clone()),
                found,
                span,
            ))
        }
    })
}

pub(super) fn ident() -> impl NoirParser<Ident> {
    token_kind(TokenKind::Ident).map_with_span(Ident::from_token)
}

// Right-shift (>>) is issued as two separate > tokens by the lexer as this makes it easier
// to parse nested generic types. For normal expressions however, it means we have to manually
// parse two greater-than tokens as a single right-shift here.
pub(super) fn right_shift_operator() -> impl NoirParser<Token> {
    just(Token::Greater).then(just(Token::Greater)).to(Token::ShiftRight)
}

pub(super) fn not<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Bang).ignore_then(term_parser).map(|rhs| ExpressionKind::prefix(UnaryOp::Not, rhs))
}

pub(super) fn negation<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Minus)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Minus, rhs))
}

pub(super) fn mutable_reference<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Ampersand)
        .ignore_then(keyword(Keyword::Mut))
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::MutableReference, rhs))
}

pub(super) fn dereference<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Star)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Dereference { implicitly_added: false }, rhs))
}

pub(super) fn variable() -> impl NoirParser<ExpressionKind> {
    path().map(ExpressionKind::Variable)
}

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
    use crate::parser::parser::{
        expression, expression_no_constructors, fresh_statement, parser_test_helpers::*, term,
    };
    use crate::Literal;

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
