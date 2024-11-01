use std::collections::BTreeMap;

use acvm::{AcirField, FieldElement};
use noirc_errors::Span;

use crate::{BinaryTypeOperator, Type, TypeBindings, UnificationError};

impl Type {
    /// Try to canonicalize the representation of this type.
    /// Currently the only type with a canonical representation is
    /// `Type::Infix` where for each consecutive commutative operator
    /// we sort the non-constant operands by `Type: Ord` and place all constant
    /// operands at the end, constant folded.
    ///
    /// For example:
    /// - `canonicalize[((1 + N) + M) + 2] = (M + N) + 3`
    /// - `canonicalize[A + 2 * B + 3 - 2] = A + (B * 2) + 3 - 2`
    pub fn canonicalize(&self) -> Type {
        match self.follow_bindings() {
            Type::CheckedCast { from, to } => Type::CheckedCast {
                from: Box::new(from.canonicalize_checked()),
                to: Box::new(to.canonicalize_unchecked()),
            },

            other => {
                let non_checked_cast = false;
                let run_simplifications = true;
                other.canonicalize_helper(non_checked_cast, run_simplifications)
            }
        }
    }

    /// Only simplify constants and drop/skip any CheckedCast's
    pub(crate) fn canonicalize_checked(&self) -> Type {
        let found_checked_cast = true;
        let skip_simplifications = false;
        self.canonicalize_helper(found_checked_cast, skip_simplifications)
    }

    /// Run all simplifications and drop/skip any CheckedCast's
    fn canonicalize_unchecked(&self) -> Type {
        let found_checked_cast = true;
        let run_simplifications = true;
        self.canonicalize_helper(found_checked_cast, run_simplifications)
    }

    /// If found_checked_cast, then drop additional CheckedCast's
    ///
    /// If run_simplifications is false, then only:
    /// - Attempt to evaluate each sub-expression to a constant
    /// - Drop nested CheckedCast's
    ///
    /// Otherwise also attempt try_simplify_partial_constants, sort_commutative,
    /// and other simplifications
    pub(crate) fn canonicalize_helper(
        &self,
        found_checked_cast: bool,
        run_simplifications: bool,
    ) -> Type {
        match self.follow_bindings() {
            Type::InfixExpr(lhs, op, rhs) => {
                let kind = lhs.infix_kind(&rhs);
                let dummy_span = Span::default();
                // evaluate_to_field_element also calls canonicalize so if we just called
                // `self.evaluate_to_field_element(..)` we'd get infinite recursion.
                if let (Ok(lhs_value), Ok(rhs_value)) = (
                    lhs.evaluate_to_field_element_helper(&kind, dummy_span, run_simplifications),
                    rhs.evaluate_to_field_element_helper(&kind, dummy_span, run_simplifications),
                ) {
                    if let Ok(result) = op.function(lhs_value, rhs_value, &kind, dummy_span) {
                        return Type::Constant(result, kind);
                    }
                }

                let lhs = lhs.canonicalize_helper(found_checked_cast, run_simplifications);
                let rhs = rhs.canonicalize_helper(found_checked_cast, run_simplifications);

                if !run_simplifications {
                    return Type::InfixExpr(Box::new(lhs), op, Box::new(rhs));
                }

                if let Some(result) = Self::try_simplify_non_constants_in_lhs(&lhs, op, &rhs) {
                    return result.canonicalize_unchecked();
                }

                if let Some(result) = Self::try_simplify_non_constants_in_rhs(&lhs, op, &rhs) {
                    return result.canonicalize_unchecked();
                }

                // Try to simplify partially constant expressions in the form `(N op1 C1) op2 C2`
                // where C1 and C2 are constants that can be combined (e.g. N + 5 - 3 = N + 2)
                if let Some(result) = Self::try_simplify_partial_constants(&lhs, op, &rhs) {
                    return result.canonicalize_unchecked();
                }

                if op.is_commutative() {
                    return Self::sort_commutative(&lhs, op, &rhs);
                }

                Type::InfixExpr(Box::new(lhs), op, Box::new(rhs))
            }
            Type::CheckedCast { from, to } => {
                let inner_found_checked_cast = true;
                let to = to.canonicalize_helper(inner_found_checked_cast, run_simplifications);

                if found_checked_cast {
                    return to;
                }

                let from = from.canonicalize_checked();

                Type::CheckedCast { from: Box::new(from), to: Box::new(to) }
            }
            other => other,
        }
    }

    fn sort_commutative(lhs: &Type, op: BinaryTypeOperator, rhs: &Type) -> Type {
        let mut queue = vec![lhs.clone(), rhs.clone()];

        // Maps each term to the number of times that term was used.
        let mut sorted = BTreeMap::new();

        let zero_value = if op == BinaryTypeOperator::Addition {
            FieldElement::zero()
        } else {
            FieldElement::one()
        };
        let mut constant = zero_value;

        // Push each non-constant term to `sorted` to sort them. Recur on InfixExprs with the same operator.
        while let Some(item) = queue.pop() {
            match item.canonicalize_unchecked() {
                Type::InfixExpr(lhs_inner, new_op, rhs_inner) if new_op == op => {
                    queue.push(*lhs_inner);
                    queue.push(*rhs_inner);
                }
                Type::Constant(new_constant, new_constant_kind) => {
                    let dummy_span = Span::default();
                    if let Ok(result) =
                        op.function(constant, new_constant, &new_constant_kind, dummy_span)
                    {
                        constant = result;
                    } else {
                        let constant = Type::Constant(new_constant, new_constant_kind);
                        *sorted.entry(constant).or_default() += 1;
                    }
                }
                other => {
                    *sorted.entry(other).or_default() += 1;
                }
            }
        }

        if let Some(first) = sorted.pop_first() {
            let (mut typ, first_type_count) = first.clone();

            // - 1 since `typ` already is set to the first instance
            for _ in 0..first_type_count - 1 {
                typ = Type::InfixExpr(Box::new(typ), op, Box::new(first.0.clone()));
            }

            for (rhs, rhs_count) in sorted {
                for _ in 0..rhs_count {
                    typ = Type::InfixExpr(Box::new(typ), op, Box::new(rhs.clone()));
                }
            }

            if constant != zero_value {
                let constant = Type::Constant(constant, lhs.infix_kind(rhs));
                typ = Type::InfixExpr(Box::new(typ), op, Box::new(constant));
            }

            typ
        } else {
            // Every type must have been a constant
            Type::Constant(constant, lhs.infix_kind(rhs))
        }
    }

    /// Try to simplify non-constant expressions in the form `(N op1 M) op2 M`
    /// where the two `M` terms are expected to cancel out.
    /// Precondition: `lhs & rhs are in canonical form`
    ///
    /// - Simplifies `(N +/- M) -/+ M` to `N`
    /// - Simplifies `(N * M) ÷ M` to `N`
    fn try_simplify_non_constants_in_lhs(
        lhs: &Type,
        op: BinaryTypeOperator,
        rhs: &Type,
    ) -> Option<Type> {
        let Type::InfixExpr(l_lhs, l_op, l_rhs) = lhs.follow_bindings() else {
            return None;
        };

        // Note that this is exact, syntactic equality, not unification.
        // `rhs` is expected to already be in canonical form.
        if l_op.approx_inverse() != Some(op)
            || l_op == BinaryTypeOperator::Division
            || l_rhs.canonicalize_unchecked() != *rhs
        {
            return None;
        }

        Some(*l_lhs)
    }

    /// Try to simplify non-constant expressions in the form `N op1 (M op1 N)`
    /// where the two `M` terms are expected to cancel out.
    /// Precondition: `lhs & rhs are in canonical form`
    ///
    /// Unlike `try_simplify_non_constants_in_lhs` we can't simplify `N / (M * N)`
    /// Since that should simplify to `1 / M` instead of `M`.
    ///
    /// - Simplifies `N +/- (M -/+ N)` to `M`
    /// - Simplifies `N * (M ÷ N)` to `M`
    fn try_simplify_non_constants_in_rhs(
        lhs: &Type,
        op: BinaryTypeOperator,
        rhs: &Type,
    ) -> Option<Type> {
        let Type::InfixExpr(r_lhs, r_op, r_rhs) = rhs.follow_bindings() else {
            return None;
        };

        // `N / (M * N)` should be simplified to `1 / M`, but we only handle
        // simplifying to `M` in this function.
        if op == BinaryTypeOperator::Division && r_op == BinaryTypeOperator::Multiplication {
            return None;
        }

        // Note that this is exact, syntactic equality, not unification.
        // `lhs` is expected to already be in canonical form.
        if r_op.inverse() != Some(op) || *lhs != r_rhs.canonicalize_unchecked() {
            return None;
        }

        Some(*r_lhs)
    }

    /// Given:
    ///   lhs = `N op C1`
    ///   rhs = C2
    /// Returns: `(N, op, C1, C2)` if C1 and C2 are constants.
    ///   Note that the operator here is within the `lhs` term, the operator
    ///   separating lhs and rhs is not needed.
    /// Precondition: `lhs & rhs are in canonical form`
    fn parse_partial_constant_expr(
        lhs: &Type,
        rhs: &Type,
    ) -> Option<(Box<Type>, BinaryTypeOperator, FieldElement, FieldElement)> {
        let kind = lhs.infix_kind(rhs);
        let dummy_span = Span::default();
        let rhs = rhs.evaluate_to_field_element(&kind, dummy_span).ok()?;

        let Type::InfixExpr(l_type, l_op, l_rhs) = lhs.follow_bindings() else {
            return None;
        };

        let dummy_span = Span::default();
        let l_rhs = l_rhs.evaluate_to_field_element(&kind, dummy_span).ok()?;
        Some((l_type, l_op, l_rhs, rhs))
    }

    /// Try to simplify partially constant expressions in the form `(N op1 C1) op2 C2`
    /// where C1 and C2 are constants that can be combined (e.g. N + 5 - 3 = N + 2)
    /// Precondition: `lhs & rhs are in canonical form`
    ///
    /// - Simplifies `(N +/- C1) +/- C2` to `N +/- (C1 +/- C2)` if C1 and C2 are constants.
    /// - Simplifies `(N * C1) ÷ C2` to `N * (C1 ÷ C2)` if C1 and C2 are constants which divide
    ///   without a remainder.
    fn try_simplify_partial_constants(
        lhs: &Type,
        mut op: BinaryTypeOperator,
        rhs: &Type,
    ) -> Option<Type> {
        use BinaryTypeOperator::*;
        let (l_type, l_op, l_const, r_const) = Type::parse_partial_constant_expr(lhs, rhs)?;

        match (l_op, op) {
            (Addition | Subtraction, Addition | Subtraction) => {
                // If l_op is a subtraction we want to inverse the rhs operator.
                if l_op == Subtraction {
                    op = op.inverse()?;
                }
                let dummy_span = Span::default();
                let result =
                    op.function(l_const, r_const, &lhs.infix_kind(rhs), dummy_span).ok()?;
                let constant = Type::Constant(result, lhs.infix_kind(rhs));
                Some(Type::InfixExpr(l_type, l_op, Box::new(constant)))
            }
            (Multiplication, Division) => {
                // We need to ensure the result divides evenly to preserve integer division semantics
                let divides_evenly = !lhs.infix_kind(rhs).is_type_level_field_element()
                    && l_const.to_i128().checked_rem(r_const.to_i128()) == Some(0);

                // If op is a division we need to ensure it divides evenly
                if op == Division && (r_const == FieldElement::zero() || !divides_evenly) {
                    None
                } else {
                    let dummy_span = Span::default();
                    let result =
                        op.function(l_const, r_const, &lhs.infix_kind(rhs), dummy_span).ok()?;
                    let constant = Box::new(Type::Constant(result, lhs.infix_kind(rhs)));
                    Some(Type::InfixExpr(l_type, l_op, constant))
                }
            }
            _ => None,
        }
    }

    /// Try to unify equations like `(..) + 3 = (..) + 1`
    /// by transforming them to `(..) + 2 =  (..)`
    pub(super) fn try_unify_by_moving_constant_terms(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        if let Type::InfixExpr(lhs_a, op_a, rhs_a) = self {
            if let Some(inverse) = op_a.approx_inverse() {
                let kind = lhs_a.infix_kind(rhs_a);
                let dummy_span = Span::default();
                if let Ok(rhs_a_value) = rhs_a.evaluate_to_field_element(&kind, dummy_span) {
                    let rhs_a = Box::new(Type::Constant(rhs_a_value, kind));
                    let new_other = Type::InfixExpr(Box::new(other.clone()), inverse, rhs_a);

                    let mut tmp_bindings = bindings.clone();
                    if lhs_a.try_unify(&new_other, &mut tmp_bindings).is_ok() {
                        *bindings = tmp_bindings;
                        return Ok(());
                    }
                }
            }
        }

        if let Type::InfixExpr(lhs_b, op_b, rhs_b) = other {
            if let Some(inverse) = op_b.approx_inverse() {
                let kind = lhs_b.infix_kind(rhs_b);
                let dummy_span = Span::default();
                if let Ok(rhs_b_value) = rhs_b.evaluate_to_field_element(&kind, dummy_span) {
                    let rhs_b = Box::new(Type::Constant(rhs_b_value, kind));
                    let new_self = Type::InfixExpr(Box::new(self.clone()), inverse, rhs_b);

                    let mut tmp_bindings = bindings.clone();
                    if new_self.try_unify(lhs_b, &mut tmp_bindings).is_ok() {
                        *bindings = tmp_bindings;
                        return Ok(());
                    }
                }
            }
        }

        Err(UnificationError)
    }
}
