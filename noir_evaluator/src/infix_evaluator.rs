use crate::{Environment, Evaluator};

/// Evaluate Infix Polynomials
pub struct InfixEvaluator<'a> {
    env: &'a mut Environment,
    evaluator: &'a mut Evaluator,
}
