use super::*;
use crate::parser::prefix_parser::PrefixParser;

pub struct PathParser;

impl PathParser {
    // XXX: Currently we only support importing for functions
    // We will introduce importing for constants later.
    // Ultimately, paths will be used to call static methods on a struct, external funcs and external global constants
    pub fn parse(parser: &mut Parser, root_module: Expression) -> ParserExprKindResult {
        // Current token is now the `::`. We need to advance and peek to see if there is another `::`
        parser.advance_tokens();

        // Check that the root module is an identifier
        let root_span = root_module.span;
        let root = match root_module.into_ident() {
            None => return Err(ParserError::UnstructuredError{message : format!("Expected an identifier as the root module"), span : root_span}),
            Some(ident) => ident,
        };

        // Current token should now be an identifier. Lets peek at the next token to check if the path is finished
        let mut parsed_path = vec![root];
        while parser.peek_token == Token::DoubleColon {
            let name = PrefixParser::Name.parse(parser)?.into_ident().unwrap(); // We can safely unwrap here, because the name parser checks if it is an identifier
            parsed_path.push(name);
            parser.advance_tokens(); // Advanced past the Identifier which is the current tokens
            parser.advance_tokens(); // Advanced past the :: which we peeked and know is there
        }

        // Current token will either be an identifier or an `::` at this point. Lets conditionally advance the parser and parse the method
        // XXX: We parse a general expression, and have the analyser restrict it to be a method. In the future, we will accept global constants along with methods.
        if parser.curr_token == Token::DoubleColon {
            parser.advance_tokens()
        };
        let method = parser.parse_expression(Precedence::Lowest)?;

        // By default CallParser with set the path as Current
        // We extract the Call expression and set the path correctly here
        let call_expr = match method.kind {
            ExpressionKind::Call(_, call_expr) => call_expr,
            k => return Err(ParserError::UnstructuredError{message : format!("Currently you can only access external functions {:?}", k), span :method.span}),
        };
       Ok( ExpressionKind::Call(parsed_path.into(), call_expr))
    }
}
