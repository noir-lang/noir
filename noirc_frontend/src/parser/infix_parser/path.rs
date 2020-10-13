use super::*;
use crate::parser::prefix_parser::NameParser;
use crate::parser::PrefixParser;
use crate::ast::Ident;

pub struct PathParser;

impl InfixParser for PathParser {
    // XXX: Currently we only support importing for functions
    // We will introduce importing for constants later.
    // Ultimately, paths will be used to call static methods on a struct, external funcs and external global constants
    fn parse(parser: &mut Parser, root_module: Expression) -> Expression {
        // Current token is now the `::`. We need to advance and peek to see if there is another `::`
        parser.advance_tokens();

        // Current token should now be an identifier. Lets peek at the next token to check if the path is finished
        let mut parsed_path = vec![root_module];
        while parser.peek_token == Token::DoubleColon {
            let name = NameParser::parse(parser);
            parsed_path.push(name);
            parser.advance_tokens();
        }
        // Current token will either be an identifier or an `::` at this point. Lets conditionally advance the parser and parse the method
        // XXX: We parse a general expression, and have the analyser restrict it to be a method. In the future, we will accept global constants along with methods.
        if parser.curr_token == Token::DoubleColon {
            parser.advance_tokens()
        };
        let method = parser.parse_expression(Precedence::Lowest).unwrap();

        // Convert path into strings
        let mut path_idents = Vec::new();
        for path in parsed_path.into_iter() {
            let path_ident = match path {
                Expression::Ident(x) => Ident(x),
                _ => unimplemented!("name space should only be an identifier"),
            };
            path_idents.push(path_ident);
        }

        // By default CallParser with set the path as Current
        // We extract the Call expression and set the path correctly here
        let call_expr = match method {
            Expression::Call(_, call_expr) => call_expr,
            k => unimplemented!("Currently you can only access external functions {:?}", k),
        };
        Expression::Call(path_idents.into(), call_expr)
    }
}
