use crate::{ssa::node::NodeId, Evaluator};
use acvm::{
    acir::native_types::{Expression, Witness},
    FieldElement,
};

#[derive(Default, Clone, Debug, Eq)]
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
    // TODO: What if the cached_witness becomes `dirty`
    // TODO ie the expression is updated, but the cached_witness
    // TODO refers to an old version of it. Can we add tests for this?
    // TODO: We can guarantee this, if an InternalVar is immutable after
    // TODO creation.
    cached_witness: Option<Witness>,
    id: Option<NodeId>,
}

impl InternalVar {
    pub(super) fn new(
        expression: Expression,
        cached_witness: Option<Witness>,
        id: Option<NodeId>,
    ) -> InternalVar {
        InternalVar { expression, cached_witness, id: id }
    }

    pub(crate) fn expression(&self) -> &Expression {
        &self.expression
    }
    pub(crate) fn id_mut(&mut self) -> &mut Option<NodeId> {
        &mut self.id
    }
    pub(crate) fn cached_witness(&self) -> &Option<Witness> {
        &self.cached_witness
    }
    pub(crate) fn cached_witness_mut(&mut self) -> &mut Option<Witness> {
        &mut self.cached_witness
    }

    /// If the InternalVar holds a constant expression
    /// Return that constant.Otherwise, return None.
    // TODO: we should have a method in ACVM
    // TODO which returns the constant term if its a constant
    // TODO expression. ie `self.expression.to_const()`
    pub(super) fn to_const(&self) -> Option<FieldElement> {
        if self.is_const_expression() {
            return Some(self.expression.q_c);
        }

        None
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
    fn from_expression(expression: Expression) -> InternalVar {
        let witness = witness_from_expression(&expression);
        InternalVar { expression, cached_witness: witness, id: None }
    }

    /// Creates an `InternalVar` from a `Witness`.
    /// Since a `Witness` can alway be coerced into an
    /// Expression, this method is infallible.
    fn from_witness(witness: Witness) -> InternalVar {
        InternalVar {
            expression: Expression::from(&witness),
            cached_witness: Some(witness),
            id: None,
        }
    }

    /// Creates an `InternalVar` from a `FieldElement`.
    fn from_constant(constant: FieldElement) -> InternalVar {
        InternalVar { expression: Expression::from_field(constant), cached_witness: None, id: None }
    }

    /// Generates a `Witness` that is equal to the `expression`.
    /// - If a `Witness` has previously been generated
    /// we return that.
    /// - If the Expression represents a constant, we return None.
    pub(crate) fn witness(&mut self, evaluator: &mut Evaluator) -> Option<Witness> {
        // Check if we've already generated a `Witness` which is equal to
        // the stored `Expression`
        if let Some(witness) = self.cached_witness {
            return Some(witness);
        }

        // If we have a constant expression, we do not
        // create a witness equal to it and instead
        // panic (TODO change)
        // TODO: why don't we create a witness for the constant expression here?
        if self.is_const_expression() {
            return None;
        }

        self.cached_witness =
            Some(InternalVar::expression_to_witness(self.expression.clone(), evaluator));

        self.cached_witness
    }

    /// Converts an `Expression` into a `Witness`
    /// - If the `Expression` is a degree-1 univariate polynomial
    /// then this conversion is a simple coercion.
    /// - Otherwise, we create a new `Witness` and set it to be equal to the
    /// `Expression`.
    pub(crate) fn expression_to_witness(expr: Expression, evaluator: &mut Evaluator) -> Witness {
        match witness_from_expression(&expr) {
            Some(witness) => witness,
            None => evaluator.create_intermediate_variable(expr),
        }
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

// Returns a `Witness` if the `Expression` can be represented as a degree-1
// univariate polynomial. Otherwise, Return None.
//
// Note that `Witness` is only capable of expressing polynomials of the form
// f(x) = x and not polynomials of the form f(x) = mx+c , so this method has
// extra checks to ensure that m=1 and c=0
//
// TODO: move to ACVM repo
fn witness_from_expression(arith: &Expression) -> Option<Witness> {
    let is_deg_one_univariate = expression_is_deg_one_univariate(arith);

    if is_deg_one_univariate {
        // If we get here, we know that our expression is of the form `f(x) = mx+c`
        // We want to now restrict ourselves to expressions of the form f(x) = x
        // ie where the constant term is 0 and the coefficient in front of the variable is
        // one.
        let coefficient = arith.linear_combinations[0].0;
        let variable = arith.linear_combinations[0].1;
        let constant = arith.q_c;

        let coefficient_is_one = coefficient.is_one();
        let constant_term_is_zero = constant.is_zero();

        if coefficient_is_one && constant_term_is_zero {
            return Some(variable);
        }
    }

    None
}
// Returns true if highest degree term in the expression is one.
//
// - `mul_term` in an expression contains degree-2 terms
// - `linear_combinations` contains degree-1 terms
// Hence, it is sufficient to check that there are no `mul_terms`
//
// Examples:
// -  f(x, y) = x + y would return true
// -  f(x, y) = xy would return false, the degree here is 2
// -  f(x,y) = 0 would return true, the degree is 0
//
// TODO: move to ACVM repo
fn expression_is_degree_1(expression: &Expression) -> bool {
    expression.mul_terms.is_empty()
}
// Returns true if the expression can be seen as a degree-1 univariate polynomial
//
// - `mul_terms` in an expression can be univariate, however unless the coefficient
// is zero, it is always degree-2.
// - `linear_combinations` contains the sum of degree-1 terms, these terms do not
// need to contain the same variable and so it can be multivariate. However, we
// have thus far only checked if `linear_combinations` contains one term, so this
// method will return false, if the `Expression` has not been simplified.
//
// Hence, we check in the simplest case if an expression is a degree-1 univariate,
// by checking if it contains no `mul_terms` and it contains one `linear_combination` term.
//
// Examples:
// - f(x,y) = x would return true
// - f(x,y) = x + 6 would return true
// - f(x,y) = 2*y + 6 would return true
// - f(x,y) = x + y would return false
// - f(x, y) = x + x should return true, but we return false *** (we do not simplify)
//
// TODO move to ACVM repo
// TODO: ACVM has a method called is_linear, we should change this to `max_degree_one`
fn expression_is_deg_one_univariate(expression: &Expression) -> bool {
    let has_one_univariate_term = expression.linear_combinations.len() == 1;
    expression_is_degree_1(expression) && has_one_univariate_term
}

#[test]
fn internal_var_const_expression() {
    let mut evaluator = Evaluator::new();

    let expected_constant = FieldElement::from(123456789u128);

    // Initialize an InternalVar with a FieldElement
    let mut internal_var = InternalVar::from_constant(expected_constant);

    // We currently do not create witness when the InternalVar was created using a constant
    let witness = internal_var.witness(&mut evaluator);
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
    let got_witness = internal_var.witness(&mut evaluator);
    match got_witness {
        Some(got_witness) => assert_eq!(got_witness, expected_witness),
        None => panic!("expected a `Witness` value"),
    }
}
