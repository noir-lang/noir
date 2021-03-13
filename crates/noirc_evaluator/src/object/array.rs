use super::RuntimeErrorKind;
use crate::object::Object;
use crate::{Environment, Evaluator};
use noirc_errors::Span;
use noirc_frontend::hir_def::expr::HirArrayLiteral;
use noirc_frontend::node_interner::ExprId;

#[derive(Clone, Debug)]
pub struct Array {
    pub contents: Vec<Object>,
    pub length: u128,
}

impl Array {
    pub fn from(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        arr_lit: HirArrayLiteral,
    ) -> Result<Array, RuntimeErrorKind> {
        // Take each element in the array and turn it into an object
        // We do not check that the array is homogenous, this is done by the type checker.
        // We could double check here, however with appropriate tests, it should not be needed.
        let (objects, mut errs) = evaluator.expression_list_to_objects(env, &arr_lit.contents);
        if !errs.is_empty() {
            return Err(errs.pop().unwrap());
        }

        Ok(Array {
            contents: objects,
            length: arr_lit.length,
        })
    }
    pub fn get(&self, index: u128, span: Span) -> Result<Object, RuntimeErrorKind> {
        if index >= self.length {
            return Err(RuntimeErrorKind::ArrayOutOfBounds {
                index,
                bound: self.length,
                span,
            });
        };

        Ok(self.contents[index as usize].clone())
    }

    pub fn num_elements(&self) -> usize {
        self.contents.len()
    }
    /// Given two arrays A, B
    /// This method creates a new array C
    /// such that C[i] = A[i] - B[i] for all i.
    pub fn sub(
        lhs: Array,
        rhs: Array,
        evaluator: &mut Evaluator,
    ) -> Result<Array, RuntimeErrorKind> {
        let length = Array::check_arr_len(&lhs, &rhs)?;
        let mut contents = Vec::with_capacity(length);
        for (lhs_element, rhs_element) in lhs.contents.into_iter().zip(rhs.contents.into_iter()) {
            let out_element = crate::binary_op::handle_sub_op(lhs_element, rhs_element, evaluator)?;
            contents.push(out_element);
        }

        Ok(Array {
            contents,
            length: length as u128,
        })
    }
    /// Given two arrays A, B
    /// This method creates a new array C
    /// such that C[i] = A[i] + B[i] for all i.
    pub fn add(
        lhs: Array,
        rhs: Array,
        evaluator: &mut Evaluator,
    ) -> Result<Array, RuntimeErrorKind> {
        let length = Array::check_arr_len(&lhs, &rhs)?;
        let mut contents = Vec::with_capacity(length);
        for (lhs_element, rhs_element) in lhs.contents.into_iter().zip(rhs.contents.into_iter()) {
            let out_element = crate::binary_op::handle_add_op(lhs_element, rhs_element, evaluator)?;
            contents.push(out_element);
        }

        Ok(Array {
            contents,
            length: length as u128,
        })
    }

    fn check_arr_len(lhs: &Array, rhs: &Array) -> Result<usize, RuntimeErrorKind> {
        let lhs_len = lhs.num_elements();
        let rhs_len = rhs.num_elements();
        if lhs_len != rhs_len {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Span::default(),
                message: format!(
                    "arrays must contain the same number of elements. lhs : {} , rhs : {}",
                    lhs_len, rhs_len
                ),
            });
        }
        Ok(lhs_len)
    }
    /// Given two arrays A, B
    /// This method checks that A[i] == B[i] for all i.
    pub fn equal(
        lhs: Array,
        rhs: Array,
        evaluator: &mut Evaluator,
    ) -> Result<(), RuntimeErrorKind> {
        let length = Array::check_arr_len(&lhs, &rhs)?;
        let mut contents = Vec::with_capacity(length);
        for (lhs_element, rhs_element) in lhs.contents.into_iter().zip(rhs.contents.into_iter()) {
            let out_element =
                crate::binary_op::handle_equal_op(lhs_element, rhs_element, evaluator)?;
            contents.push(out_element);
        }
        Ok(())
    }

    /// Constrains all elements in the array to be equal to zero
    pub fn constrain_zero(&self, evaluator: &mut Evaluator) {
        for element in self.contents.iter() {
            element.constrain_zero(evaluator)
        }
    }

    pub fn from_expression(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        expr_id: &ExprId,
    ) -> Result<Array, RuntimeErrorKind> {
        let object = evaluator.expression_to_object(env, expr_id)?;
        match object {
            Object::Array(arr) => Ok(arr),
            _ => Err(RuntimeErrorKind::expected_type("array", object.r#type())),
        }
    }
}
