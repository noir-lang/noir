use super::RuntimeErrorKind;
use crate::errors::RuntimeError;
use crate::{binary_op::maybe_equal, object::Object};
use crate::{Environment, Evaluator};
use acvm::FieldElement;
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
    ) -> Result<Array, RuntimeError> {
        // Take each element in the array and turn it into an object
        // We do not check that the array is homogeneous, this is done by the type checker.
        // We could double check here, however with appropriate tests, it should not be needed.
        let (objects, mut errs) = evaluator.expression_list_to_objects(env, &arr_lit.contents);
        if !errs.is_empty() {
            // XXX Should we make this return an RunTimeError? The problem is that we do not want the OPCODES
            // to return RunTimeErrors, because we do not want to deal with span there
            return Err(errs.pop().unwrap());
        }

        Ok(Array { contents: objects, length: arr_lit.length })
    }
    pub fn get(&self, index: u128) -> Result<Object, RuntimeErrorKind> {
        if index >= self.length {
            return Err(RuntimeErrorKind::ArrayOutOfBounds { index, bound: self.length });
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

        Ok(Array { contents, length: length as u128 })
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

        Ok(Array { contents, length: length as u128 })
    }

    fn check_arr_len(lhs: &Array, rhs: &Array) -> Result<usize, RuntimeErrorKind> {
        let lhs_len = lhs.num_elements();
        let rhs_len = rhs.num_elements();
        if lhs_len != rhs_len {
            return Err(RuntimeErrorKind::UnstructuredError {
                message: format!(
                    "arrays must contain the same number of elements. lhs : {} , rhs : {}",
                    lhs_len, rhs_len
                ),
            });
        }

        if lhs_len == 0 {
            return Err(RuntimeErrorKind::UnstructuredError {
                message: "arrays must contain at least one element".to_string(),
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
        let _ = Array::check_arr_len(&lhs, &rhs)?;
        for (lhs_element, rhs_element) in lhs.contents.into_iter().zip(rhs.contents.into_iter()) {
            let _ = crate::binary_op::handle_equal_op(lhs_element, rhs_element, evaluator)?;
        }
        Ok(())
    }
    /// Given two arrays A, B
    /// This method checks that A[i] != B[i] for some i.
    pub fn not_equal(
        lhs: Array,
        rhs: Array,
        evaluator: &mut Evaluator,
    ) -> Result<(), RuntimeErrorKind> {
        let length = Array::check_arr_len(&lhs, &rhs)?;

        let mut predicates: Vec<Object> = Vec::with_capacity(length);
        for (lhs_element, rhs_element) in lhs.contents.into_iter().zip(rhs.contents.into_iter()) {
            let pred_i = maybe_equal(lhs_element, rhs_element, evaluator)?;
            predicates.push(Object::from_witness(pred_i.witness));
        }

        // We now have a predicates vector, where 1 represents the elements were the same
        // and zero represents a difference.
        // We want the constraint to pass if there is at least 1 zero.
        // To accomplish this, we simply multiply all of the predicates
        // Then constrain the product to be equal to 0

        let mut predicates_iter = predicates.into_iter();
        let mut result =
            predicates_iter.next().expect("ice: arrays must have at least one element in them");

        for pred in predicates_iter {
            result = crate::binary_op::handle_mul_op(result, pred, evaluator)?;
        }

        crate::binary_op::handle_equal_op(
            result,
            Object::Constants(FieldElement::zero()),
            evaluator,
        )?;

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
    ) -> Result<Array, RuntimeError> {
        let object = evaluator.expression_to_object(env, expr_id)?;
        match object {
            Object::Array(arr) => Ok(arr),
            _ => {
                let span = evaluator.context.def_interner.expr_location(expr_id);
                Err(RuntimeErrorKind::expected_type("array", object.r#type()).add_location(span))
            }
        }
    }
}
