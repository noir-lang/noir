use super::*;
use crate::ast::BinaryOpKind;
use crate::ast::ConstrainStatement;

pub struct ConstrainParser;

// XXX: For now we disallow statement of the form `constrain x / y` as their meaning is a bit ambiguous
// In the future, if there is a disallowed operator, we will modify the AST so that it's RHS is zero
// We will also do it for the other expressions
// Example1 `constrain x` becomes `constrain x == 0`
// Example2 `constrain x / y` becomes `constrain x/y == 0`
// XXX: However, I'm wondering if we should avoid doing anything under the hood and just have users explicitly writing what they mean?
fn disallowed_operators() -> Vec<BinaryOpKind> {
    vec![
        BinaryOpKind::And,
        BinaryOpKind::Or,
        BinaryOpKind::Divide,
        BinaryOpKind::Multiply,
    ]
}

impl ConstrainParser {
    // Since == is an infix operator
    // The pratt parser will do most of the job, we just need to check that everything was correct
    pub(crate) fn parse_constrain_statement(parser: &mut Parser) -> Result<ConstrainStatement, ParserError> {
        parser.advance_tokens();
        
        let expr = parser.parse_expression(Precedence::Lowest)?;
        // XXX: We do this so that the first == sign in the constraint statement is not seen as a predicate
        let infix = match expr.kind.infix() {
            Some(infix) => infix,
            None => {
                let message = format!("Expected an infix expression since this is a constrain statement. You cannot assign values");
                return Err(ParserErrorKind::UnstructuredError{message, span: expr.span}.into_err(parser.file_id))
            },
        };
        if infix.operator.contents == BinaryOpKind::Assign {
            // XXX: We need to add span to BinaryOps, currently we use the span of the expression
            let message = format!("Cannot use '=' with a constrain statement");
            return Err(ParserErrorKind::UnstructuredError{message, span: expr.span}.into_err(parser.file_id))
        }
        
        if disallowed_operators().contains(&infix.operator.contents) {
            let message = format!("Cannot use the {:?} operator in a constraint statement.",&infix.operator);
            return Err(ParserErrorKind::UnstructuredError{message, span: expr.span}.into_err(parser.file_id))
        }
        
        Ok(ConstrainStatement(infix))
        
    }
}
