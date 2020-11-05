use crate::object::Object;
use crate::{Environment, Evaluator, Expression};
use noirc_frontend::ast::ArrayLiteral;
use super::{EvaluatorError, ArrayError};

#[derive(Clone, Debug)]
pub struct Array {
    pub contents: Vec<Object>,
    pub length: u128,
}

impl Array {
    pub fn from(evaluator: &mut Evaluator, env: &mut Environment, arr_lit: ArrayLiteral) -> Result<Array, EvaluatorError> {
        // Take each element in the array and turn it into an object
        // We do not check that the array is homogenous, this is done by the type checker.
        // We could double check here, however with appropriate tests, it should not be needed.
        let (objects, mut errs) = evaluator.expression_list_to_objects(env, &arr_lit.contents);
        if !errs.is_empty() {
            return Err(errs.pop().unwrap())
        }

        Ok(Array {
            contents: objects,
            length: arr_lit.length,
        })
    }
    pub fn get(&self, index: u128) -> Result<Object, EvaluatorError> {
        if index >= self.length {
            return Err(EvaluatorError::ArrayError(ArrayError::OutOfBounds{index, bound : self.length}));
        };

        Ok(self.contents[index as usize].clone())
    }

    pub fn from_expression(evaluator : &mut Evaluator, env : &mut Environment, expr : Expression) -> Result<Array, EvaluatorError> {
        let object = evaluator.expression_to_object(env, expr)?;
        match object {
            Object::Array(arr) => Ok(arr),
            _=> Err(EvaluatorError::expected_type("array", object.r#type()))
        }
    }
}
