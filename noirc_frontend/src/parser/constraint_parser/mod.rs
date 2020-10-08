use super::*;
use crate::ast::BinaryOp;
use crate::ast::ConstrainStatement;

/// The constrain parser is used to parse keywords which directly and only apply constraints
/// XXX(med) : The 'as' keyword would fall under this category, but it would probably be implemented as an infix operator
/// Possibly could have an invariant for these types of Statements, so that the evaluator can be less complex
pub struct ConstraintParser;

// XXX: For now we disallow statement of the form `constrain x / y` as their meaning is a bit ambiguous
// In the future, if there is a disallowed operator, we will modify the AST so that it's RHS is zero
// We will also do it for the other expressions
// Example1 `constrain x` becomes `constrain x == 0`
// Example2 `constrain x / y` becomes `constrain x/y == 0`
// XXX: However, I'm wondering if we should avoid doing anything under the hood and just have users explicitly writing what they mean?
fn disallowed_operators() -> Vec<BinaryOp> {
    vec![
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::Divide,
        BinaryOp::Multiply,
    ]
}

impl ConstraintParser {
    // Since == is an infix operator
    // The pratt parser will do most of the job, we just need to check that everything was correct
    pub(crate) fn parse_constrain_statement(parser: &mut Parser) -> Box<ConstrainStatement> {
        parser.advance_tokens();

        let expr = parser.parse_expression(Precedence::Lowest).unwrap();
        // XXX: We do this so that the first == sign in the constraint statement is not seen as a predicate
        let infix = match expr.infix() {
            Some(infix) => infix,
            None => panic!("Expected an infix expression since this is a constrain statement. You cannot assign values"),
        };
        if infix.operator == BinaryOp::Assign {
            panic!("Cannot use '=' with a constrain statement")
        }

        if disallowed_operators().contains(&infix.operator) {
            panic!(
                "Cannot use the {:?} operator in a constraint statement.",
                &infix.operator
            )
        }

        let stmt = ConstrainStatement(infix);
        Box::new(stmt)
    }
}
