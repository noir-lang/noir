use std::collections::BTreeMap;

use acvm::{AcirField, FieldElement};

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
        // TODO rename
        // let found_txm = true;
        // let result = self.canonicalize_helper(found_txm);

        // // TODO: simplify
        // let found_txm = matches!(self.follow_bindings(), Type::Txm(..));
        // let result = self.canonicalize_helper(found_txm);
        // result

        // TODO: simplify
        match self.follow_bindings() {
            Type::Txm(to, from) => {
                let found_txm = true;
                let run_simplifications = true;
                let skip_simplifications = false;
                Type::Txm(
                    Box::new(to.canonicalize_helper(found_txm, run_simplifications)),
                    Box::new(from.canonicalize_helper(found_txm, skip_simplifications)),
                )
            }
            other => {
                let non_txm = false;
                let run_simplifications = true;
                other.canonicalize_helper(non_txm, run_simplifications)
            }
        }
    }

    fn canonicalize_helper(&self, found_txm: bool, run_simplifications: bool) -> Type {
        match self.follow_bindings() {
            Type::InfixExpr(lhs, op, rhs) => {
                // TODO
                dbg!("canonicalize_helper: InfixExpr {:?}", (&lhs, &rhs));

                let kind = lhs.infix_kind(&rhs);
                // evaluate_to_field_element also calls canonicalize so if we just called
                // `self.evaluate_to_field_element(..)` we'd get infinite recursion.
                if let (Some(lhs_value), Some(rhs_value)) =
                    (lhs.evaluate_to_field_element(&kind), rhs.evaluate_to_field_element(&kind))
                {
                    if let Some(result) = op.function(lhs_value, rhs_value, &kind) {
                        return Type::Constant(result, kind);
                    }
                }

                dbg!("canonicalize_helper: not constant");

                // TODO?
                let lhs = lhs.canonicalize_helper(found_txm, run_simplifications);
                let rhs = rhs.canonicalize_helper(found_txm, run_simplifications);

                if !run_simplifications {
                    return Type::InfixExpr(Box::new(lhs), op, Box::new(rhs));
                }

                if let Some(result) = Self::try_simplify_non_constants_in_lhs(&lhs, op, &rhs) {
                    // TODO
                    dbg!("try_simplify_non_constants_in_lhs");

                    let found_txm_inner = true;
                    let run_simplifications = true;
                    let result = result.canonicalize_helper(found_txm_inner, run_simplifications);

                    // TODO
                    let found_txm = true;
                    if found_txm {
                        return result;
                    } else {
                        return Type::Txm(Box::new(result), Box::new(self.clone()));
                    }
                }

                if let Some(result) = Self::try_simplify_non_constants_in_rhs(&lhs, op, &rhs) {
                    // TODO
                    dbg!("try_simplify_non_constants_in_rhs");

                    let found_txm_inner = true;
                    let run_simplifications = true;
                    let result = result.canonicalize_helper(found_txm_inner, run_simplifications);

                    // TODO
                    let found_txm = true;
                    if found_txm {
                        return result;
                    } else {
                        return Type::Txm(Box::new(result), Box::new(self.clone()));
                    }
                }

                // Try to simplify partially constant expressions in the form `(N op1 C1) op2 C2`
                // where C1 and C2 are constants that can be combined (e.g. N + 5 - 3 = N + 2)
                if let Some(result) = Self::try_simplify_partial_constants(&lhs, op, &rhs) {
                    // TODO
                    dbg!("try_simplify_partial_constants");

                    // TODO parameter always true
                    let found_txm_inner = true;
                    let run_simplifications = true;
                    let result = result.canonicalize_helper(found_txm_inner, run_simplifications);

                    // TODO
                    let found_txm = true;
                    if found_txm {
                        return result;
                    } else {
                        return Type::Txm(Box::new(result), Box::new(self.clone()));
                    }
                }

                if op.is_commutative() {
                    // TODO
                    dbg!("sort_commutative");

                    let result = Self::sort_commutative(&lhs, op, &rhs);

                    // TODO: re-enable
                    let found_txm = true;
                    if found_txm {
                        return result;
                    } else {
                        return Type::Txm(Box::new(result), Box::new(self.clone()));
                    }
                }

                // TODO
                dbg!("canonicalize_helper: output", (&lhs, &rhs));
                // Type::InfixExpr(lhs, op, rhs)
                Type::InfixExpr(Box::new(lhs), op, Box::new(rhs))
            }
            Type::Txm(to, from) => {
                // TODO
                dbg!("canonicalize_helper: Txm {:?}", (&to, &from));

                let to_found_txm = true;
                let run_simplifications = true;
                let to = to.canonicalize_helper(to_found_txm, run_simplifications);

                if found_txm {
                    return to;
                }

                let skip_simplifications = false;
                let from = from.canonicalize_helper(to_found_txm, skip_simplifications);

                Type::Txm(Box::new(to), Box::new(from))
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
            // TODO dbg
            dbg!("sort_commutative:pop", &item);
            let found_txm_inner = true;
            let run_simplifications = true;
            match item.canonicalize_helper(found_txm_inner, run_simplifications) {
                Type::InfixExpr(lhs_inner, new_op, rhs_inner) if new_op == op => {
                    // TODO dbg
                    dbg!("sort_commutative: InfixExpr", &lhs_inner, &rhs_inner);

                    queue.push(*lhs_inner);
                    queue.push(*rhs_inner);
                }
                Type::Constant(new_constant, new_constant_kind) => {
                    if let Some(result) = op.function(constant, new_constant, &new_constant_kind) {
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

        // TODO dbg
        dbg!("end sort_commutative");

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
            // TODO: run before try_simplify_non_constants_in_lhs, right?
            // || l_rhs.canonicalize_helper(found_txm, run_simplifications) != *rhs
            || *l_rhs != *rhs
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
        if r_op.inverse() != Some(op)
            // TODO: run before try_simplify_non_constants_in_rhs, right?
            // || *lhs != r_rhs.canonicalize_helper(found_txm, run_simplifications)
            || *lhs != *r_rhs
        {
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
        let rhs = rhs.evaluate_to_field_element(&kind)?;

        let Type::InfixExpr(l_type, l_op, l_rhs) = lhs.follow_bindings() else {
            return None;
        };

        let l_rhs = l_rhs.evaluate_to_field_element(&kind)?;
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
                let result = op.function(l_const, r_const, &lhs.infix_kind(rhs))?;
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
                    let result = op.function(l_const, r_const, &lhs.infix_kind(rhs))?;
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
                if let Some(rhs_a_value) = rhs_a.evaluate_to_field_element(&kind) {
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
                if let Some(rhs_b_value) = rhs_b.evaluate_to_field_element(&kind) {
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
