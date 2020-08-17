use super::*;
use librasac_ast::BinaryOp;
use librasac_ast::{ConstrainStatement, InfixExpression};

/// The constrain parser is used to parse keywords which directly and only apply constraints
/// XXX(med) : The 'as' keyword would fall under this category, but it would probably be implemented as an infix operator
/// Possibly could have an invariant for these types of Statements, so that the evaluator can be less complex
pub struct ConstraintParser;

impl ConstraintParser {
    //Since == is an infix operator
    // The pratt parser will do most of the job, we just need to check that everything was correct
    pub(crate) fn parse_constrain_statement(parser: &mut Parser) -> Box<ConstrainStatement> {
        parser.advance_tokens();

        let expr = parser.parse_expression(Precedence::Lowest).unwrap();
        let infix = match expr.infix() {
            Some(infix) => infix,
            None => panic!("Expected an infix expression since this is a constrain statement. You cannot assign values"),
        };

        if infix.operator != BinaryOp::Equal {
            panic!("Can only use == with a constrain statement for now")
        }

        if parser.peek_token == Token::Semicolon {
            parser.advance_tokens();
        }

        let stmt = ConstrainStatement(infix);
        Box::new(stmt)
    }
}
