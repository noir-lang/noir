use crate::{
    ssa::acir_gen::{
        const_from_expression, constraints::ACIRState, expression_to_witness,
        optional_expression_to_witness,
    },
    ssa::node::NodeId,
};
use acvm::{
    acir::native_types::{Expression, Witness},
    FieldElement,
};

#[derive(Clone, Debug, Eq)]
pub struct InternalVar {
    // A multi-variate degree-2 polynomial
    expression: Expression,
    // A witness associated to `expression`.
    // semantically `cached_witness` and `expression`
    // should be the equal.
    //
    // Example: `z = x + y`
    // `z` can be seen as the same to `x + y`
    // ie if `z = 10` , then `x+y` must be equal to
    // 10.
    // There are places where we can use `cached_witness`
    // in place of `expression` for performance reasons
    // due to the fact that `cached_witness` is a single variable
    // whereas `expression` is a multi-variate polynomial which can
    // contain many degree-2 terms.
    //
    // Note that we also protect against the case that `expression`
    // changes and `cached_witness` becomes "dirty" or outdated.
    // This is because one is not allowed to modify the `expression`
    // once set. Additionally, there are no methods that allow one
    // to modify the `cached_witness`
    cached_witness: Option<Witness>,
    id: Option<NodeId>,
}

impl InternalVar {
    pub(crate) fn expression(&self) -> &Expression {
        &self.expression
    }
    pub(crate) fn set_id(&mut self, id: NodeId) {
        match self.id {
            Some(existing_id) => {
                assert_eq!(existing_id, id, "ICE: changing node id to a different value")
            }
            None => self.id = Some(id),
        }

        assert!(self.id.is_some(), "ICE: node id has already been set for this `InternalVar`");
    }
    pub(crate) fn cached_witness(&self) -> &Option<Witness> {
        &self.cached_witness
    }

    /// If the InternalVar holds a constant expression
    /// Return that constant.Otherwise, return None.
    pub(super) fn to_const(&self) -> Option<FieldElement> {
        const_from_expression(&self.expression)
    }

    /// The expression term is degree-2 multi-variate polynomial, so
    /// in order to check if if represents a constant,
    /// we need to check that the degree-2 terms `mul_terms`
    /// and the degree-1 terms `linear_combinations` do not exist.
    ///
    /// Example: f(x,y) = xy + 5
    /// `f` is not a constant expression because there
    /// is a bi-variate term `xy`.
    /// Example: f(x,y) = x + y + 5
    /// `f` is not a constant expression because there are
    /// two uni-variate terms `x` and `y`
    /// Example: f(x,y) = 10
    /// `f` is a constant expression because there are no
    /// bi-variate or uni-variate terms, just a constant.
    fn is_const_expression(&self) -> bool {
        self.expression.is_const()
    }

    /// Creates an `InternalVar` from an `Expression`.
    /// If `Expression` represents a degree-1 polynomial
    /// then we also assign it to the `cached_witness`
    pub(crate) fn from_expression(expression: Expression) -> InternalVar {
        let witness = optional_expression_to_witness(&expression);
        InternalVar { expression, cached_witness: witness, id: None }
    }

    pub(crate) fn zero_expr() -> InternalVar {
        InternalVar::from_expression(Expression::zero())
    }

    /// Creates an `InternalVar` from a `Witness`.
    /// Since a `Witness` can alway be coerced into an
    /// Expression, this method is infallible.
    pub(crate) fn from_witness(witness: Witness) -> InternalVar {
        InternalVar {
            expression: Expression::from(&witness),
            cached_witness: Some(witness),
            id: None,
        }
    }

    /// Creates an `InternalVar` from a `FieldElement`.
    pub(crate) fn from_constant(constant: FieldElement) -> InternalVar {
        InternalVar { expression: Expression::from_field(constant), cached_witness: None, id: None }
    }

    /// Generates a `Witness` that is equal to the `expression`.
    /// - If a `Witness` has previously been generated
    /// we return that.
    /// - If the Expression represents a constant, we return None.
    pub(crate) fn get_or_compute_witness<A: ACIRState>(
        &mut self,
        evaluator: &mut A,
        create_witness_for_const: bool,
    ) -> Option<Witness> {
        // Check if we've already generated a `Witness` which is equal to
        // the stored `Expression`
        if let Some(witness) = self.cached_witness {
            return Some(witness);
        }

        // There are cases where we need to convert a constant expression
        // into a witness.
        if !create_witness_for_const && self.is_const_expression() {
            return None;
        }

        self.cached_witness = Some(expression_to_witness(self.expression.clone(), evaluator));

        self.cached_witness
    }
}

impl PartialEq for InternalVar {
    fn eq(&self, other: &Self) -> bool {
        // An InternalVar is Equal to another InternalVar if _any_ of the fields
        // in the InternalVar are equal.

        let expressions_are_same = self.expression == other.expression;

        // Check if `cached_witnesses` are the same
        //
        // This may happen if the expressions are the same
        // but one is simplified and the other is not.
        //
        // The caller whom created both `InternalVar` objects
        // may have known this and set their cached_witnesses to
        // be the same.
        // Example:
        // z = 2*x + y
        // t = x + x + y
        // After simplification, it is clear that both RHS are the same
        // However, since when we check for equality, we do not check for
        // simplification, or in particular, we may eagerly assume
        // the two expressions of the RHS are different.
        //
        // The caller can notice this and thus do:
        // InternalVar::new(expr: 2*x + y, witness: z)
        // InternalVar::new(expr: x + x + y, witness: z)
        let cached_witnesses_same =
            self.cached_witness.is_some() && self.cached_witness == other.cached_witness;

        let node_ids_same = self.id.is_some() && self.id == other.id;

        expressions_are_same || cached_witnesses_same || node_ids_same
    }
}

impl From<Expression> for InternalVar {
    fn from(expression: Expression) -> InternalVar {
        InternalVar::from_expression(expression)
    }
}

impl From<Witness> for InternalVar {
    fn from(witness: Witness) -> InternalVar {
        InternalVar::from_witness(witness)
    }
}

impl From<FieldElement> for InternalVar {
    fn from(constant: FieldElement) -> InternalVar {
        InternalVar::from_constant(constant)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ssa::acir_gen::InternalVar, Evaluator};
    use acvm::{acir::native_types::Witness, FieldElement};

    #[test]
    fn internal_var_const_expression() {
        let mut evaluator = Evaluator::new();

        let expected_constant = FieldElement::from(123456789u128);

        // Initialize an InternalVar with a FieldElement
        let mut internal_var = InternalVar::from_constant(expected_constant);

        // We currently do not create witness when the InternalVar was created using a constant
        let witness = internal_var.get_or_compute_witness(&mut evaluator, false);
        assert!(witness.is_none());

        match internal_var.to_const() {
            Some(got_constant) => assert_eq!(got_constant, expected_constant),
            None => {
                panic!("`InternalVar` was initialized with a constant, so a field element was expected")
            }
        }
    }
    #[test]
    fn internal_var_from_witness() {
        let mut evaluator = Evaluator::new();

        let expected_witness = Witness(1234);
        // Initialize an InternalVar with a `Witness`
        let mut internal_var = InternalVar::from_witness(expected_witness);

        // We should get back the same `Witness`
        let got_witness = internal_var.get_or_compute_witness(&mut evaluator, false);
        match got_witness {
            Some(got_witness) => assert_eq!(got_witness, expected_witness),
            None => panic!("expected a `Witness` value"),
        }
    }
}
