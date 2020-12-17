use super::*;

pub struct NameParser;

impl NameParser {

    
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {

        let ident_str = match parser.curr_token.token() {
            Token::Ident(x) => x.clone(),
            _ => {
                let token_kind = parser.curr_token.kind();
                return Err(ParserErrorKind::UnexpectedTokenKind{span : parser.curr_token.into_span(), expected : TokenKind::Ident,found : token_kind }.into_err(parser.file_id))
            } 

        };
        Ok(ExpressionKind::Ident(ident_str))
    }

}