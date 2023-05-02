use crate::ssa::node::NodeId;
use acvm::{
    acir::native_types::{Expression, Witness},
    FieldElement,
};

#[derive(Clone, Debug, Eq)]
pub(crate) struct InternalVar {
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
        self.id = Some(id);
    }
    pub(crate) fn get_id(&self) -> Option<NodeId> {
        self.id
    }
    pub(crate) fn cached_witness(&self) -> &Option<Witness> {
        &self.cached_witness
    }
    pub(crate) fn set_witness(&mut self, w: Witness) {
        debug_assert!(self.cached_witness.is_none() || self.cached_witness == Some(w));
        self.cached_witness = Some(w);
    }

    pub(crate) fn to_expression(&self) -> Expression {
        if let Some(w) = self.cached_witness {
            w.into()
        } else {
            self.expression().clone()
        }
    }

    /// If the InternalVar holds a constant expression
    /// Return that constant.Otherwise, return None.
    pub(super) fn to_const(&self) -> Option<FieldElement> {
        self.expression.to_const()
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
    pub(crate) fn is_const_expression(&self) -> bool {
        self.expression.is_const()
    }

    /// Creates an `InternalVar` from an `Expression`.
    /// If `Expression` represents a degree-1 polynomial
    /// then we also assign it to the `cached_witness`
    pub(crate) fn from_expression(expression: Expression) -> InternalVar {
        let witness = expression.to_witness();
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
            expression: Expression::from(witness),
            cached_witness: Some(witness),
            id: None,
        }
    }

    /// Creates an `InternalVar` from a `FieldElement`.
    pub(crate) fn from_constant(constant: FieldElement) -> InternalVar {
        InternalVar { expression: Expression::from_field(constant), cached_witness: None, id: None }
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
    use crate::ssa::acir_gen::InternalVar;
    use acvm::FieldElement;

    #[test]
    fn internal_var_const_expression() {
        let expected_constant = FieldElement::from(123456789u128);

        // Initialize an InternalVar with a FieldElement
        let internal_var = InternalVar::from_constant(expected_constant);

        // We currently do not create witness when the InternalVar was created using a constant
        assert!(internal_var.cached_witness().is_none());

        match internal_var.to_const() {
            Some(got_constant) => assert_eq!(got_constant, expected_constant),
            None => {
                panic!("`InternalVar` was initialized with a constant, so a field element was expected")
            }
        }
    }
}
