use crate::NoirStruct;
use super::*;

/// Parses a struct declaration
///
/// ```noir
/// struct IDENT  {
///     <IDENT>: <TYPE>,
///     ...
/// }
///```
///
/// Cursor Start : `struct`
/// Cursor End : `}`
pub fn parse(parser: &mut Parser) -> Result<NoirStruct, ParserErrorKind> {
    // Current token is `struct`
    //
    // Peek ahead and check if the next token is an identifier
    let start = parser.curr_token.to_span();

    parser.peek_check_kind_advance(TokenKind::Ident)?;
    let name: Ident = parser.curr_token.clone().into();

    // Current token is the loop identifier
    //
    // Peek ahead and check if the next token is '{'
    parser.peek_check_variant_advance(&Token::LeftBrace)?;

    let fields = parse_fields(parser)?;

    let end = parser.curr_token.to_span();
    let span = start.merge(end);

    // The cursor position is inherited from the block expression
    // parsing procedure which is `}`
    Ok(NoirStruct::new(name, fields, span))
}

/// Cursor Start : `{`
///
/// Cursor End : `}`
fn parse_fields(parser: &mut Parser) -> Result<Vec<(Ident, Type)>, ParserErrorKind> {
    // Current token is `{`, next should be an Ident or }
    let mut fields: Vec<(Ident, Type)> = Vec::new();
    parser.advance_tokens();

    while parser.curr_token.kind() == TokenKind::Ident {
        let name: Ident = parser.curr_token.clone().into();

        // Current tokens should be `IDENT` and ':""'
        // Advance past both
        parser.peek_check_variant_advance(&Token::Colon)?;
        parser.advance_tokens();

        let typ = parser.parse_type(true)?;
        parser.advance_tokens();

        // TODO: This also makes commas between fields optional
        if parser.curr_token.token() == &Token::Comma {
            parser.advance_tokens();
        }

        fields.push((name, typ));
    }

    Ok(fields)
}

// #[cfg(test)]
// mod test {
//     use crate::{parser::test_parse, token::Token};
// 
//     use super::ForParser;
// 
//     #[test]
//     fn valid_syntax() {
//         /// Why is this allowed?
//         ///
//         /// The Parser does not check the types of the loops,
//         /// it only checks for valid expressions in RANGE_START and
//         /// RANGE_END
//         const SRC_EXPR_LOOP: &str = r#"
//             for i in x+y..z {
// 
//             }
//         "#;
//         const SRC_CONST_LOOP: &str = r#"
//             for i in 0..100 {
// 
//             }
//         "#;
// 
//         let mut parser = test_parse(SRC_EXPR_LOOP);
//         let start = parser.curr_token.clone();
//         ForParser::parse(&mut parser).unwrap();
//         let end = parser.curr_token;
// 
//         ForParser::parse(&mut test_parse(SRC_CONST_LOOP)).unwrap();
// 
//         assert_eq!(start, Token::Keyword(crate::token::Keyword::For));
//         assert_eq!(end, Token::RightBrace);
//     }
// 
//     #[test]
//     fn invalid_syntax() {
//         /// Cannot have a literal as the loop identifier
//         const SRC_LITERAL_IDENT: &str = r#"
//             for 1 in x+y..z {
// 
//             }
//         "#;
//         /// Currently only the DoubleDot is supported
//         const SRC_INCLUSIVE_LOOP: &str = r#"
//             for i in 0...100 {
// 
//             }
//         "#;
//         /// Currently only the DoubleDot is supported
//         const SRC_INCLUSIVE_EQUAL_LOOP: &str = r#"
//             for i in 0..=100 {
// 
//             }
//         "#;
// 
//         ForParser::parse(&mut test_parse(SRC_LITERAL_IDENT)).unwrap_err();
//         ForParser::parse(&mut test_parse(SRC_INCLUSIVE_LOOP)).unwrap_err();
//         ForParser::parse(&mut test_parse(SRC_INCLUSIVE_EQUAL_LOOP)).unwrap_err();
//     }
// }
