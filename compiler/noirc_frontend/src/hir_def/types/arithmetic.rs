use std::collections::BTreeSet;

use crate::{BinaryTypeOperator, Type};

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
            Type::InfixExpr(lhs, op, rhs) => {
                // evaluate_to_u32 also calls canonicalize so if we just called
                // `self.evaluate_to_u32()` we'd get infinite recursion.
                if let (Some(lhs), Some(rhs)) = (lhs.evaluate_to_u32(), rhs.evaluate_to_u32()) {
                    if let Some(result) = op.function(lhs, rhs) {
                        return Type::Constant(result);
                    }
                }

                let lhs = lhs.canonicalize();
                let rhs = rhs.canonicalize();
                if let Some(result) = Self::try_simplify_non_constants_in_lhs(&lhs, op, &rhs) {
                    return result.canonicalize();
                }

                if let Some(result) = Self::try_simplify_non_constants_in_rhs(&lhs, op, &rhs) {
                    return result.canonicalize();
                }

                // Try to simplify partially constant expressions in the form `(N op1 C1) op2 C2`
                // where C1 and C2 are constants that can be combined (e.g. N + 5 - 3 = N + 2)
                if let Some(result) = Self::try_simplify_partial_constants(&lhs, op, &rhs) {
                    return result.canonicalize();
                }

                if op.is_commutative() {
                    return Self::sort_commutative(&lhs, op, &rhs);
                }

                Type::InfixExpr(Box::new(lhs), op, Box::new(rhs))
            }
            other => other,
        }
    }

    fn sort_commutative(lhs: &Type, op: BinaryTypeOperator, rhs: &Type) -> Type {
        let mut queue = vec![lhs.clone(), rhs.clone()];

        let mut sorted = BTreeSet::new();

        let zero_value = if op == BinaryTypeOperator::Addition { 0 } else { 1 };
        let mut constant = zero_value;

        // Push each non-constant term to `sorted` to sort them. Recur on InfixExprs with the same operator.
        while let Some(item) = queue.pop() {
            match item.canonicalize() {
                Type::InfixExpr(lhs, new_op, rhs) if new_op == op => {
                    queue.push(*lhs);
                    queue.push(*rhs);
                }
                Type::Constant(new_constant) => {
                    if let Some(result) = op.function(constant, new_constant) {
                        constant = result;
                    } else {
                        sorted.insert(Type::Constant(new_constant));
                    }
                }
                other => {
                    sorted.insert(other);
                }
            }
        }

        if let Some(first) = sorted.pop_first() {
            let mut typ = first.clone();

            for rhs in sorted {
                typ = Type::InfixExpr(Box::new(typ), op, Box::new(rhs.clone()));
            }

            if constant != zero_value {
                typ = Type::InfixExpr(Box::new(typ), op, Box::new(Type::Constant(constant)));
            }

            typ
        } else {
            // Every type must have been a constant
            Type::Constant(constant)
        }
    }

    /// Try to simplify non-constant expressions in the form `(N op1 M) op2 M`
    /// where the two `M` terms are expected to cancel out.
    /// Precondition: `lhs & rhs are in canonical form`
    ///
    /// - Simplifies `(N +/- M) -/+ M` to `N`
    /// - Simplifies `(N */÷ M) ÷/* M` to `N`
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
        if l_op.inverse() != Some(op) || l_rhs.canonicalize() != *rhs {
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
        if r_op.inverse() != Some(op) || *lhs != r_rhs.canonicalize() {
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
    ) -> Option<(Box<Type>, BinaryTypeOperator, u32, u32)> {
        let rhs = rhs.evaluate_to_u32()?;

        let Type::InfixExpr(l_type, l_op, l_rhs) = lhs.follow_bindings() else {
            return None;
        };

        let l_rhs = l_rhs.evaluate_to_u32()?;
        Some((l_type, l_op, l_rhs, rhs))
    }

    /// Try to simplify partially constant expressions in the form `(N op1 C1) op2 C2`
    /// where C1 and C2 are constants that can be combined (e.g. N + 5 - 3 = N + 2)
    /// Precondition: `lhs & rhs are in canonical form`
    ///
    /// - Simplifies `(N +/- C1) +/- C2` to `N +/- (C1 +/- C2)` if C1 and C2 are constants.
    /// - Simplifies `(N */÷ C1) */÷ C2` to `N */÷ (C1 */÷ C2)` if C1 and C2 are constants.
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
                let result = op.function(l_const, r_const)?;
                Some(Type::InfixExpr(l_type, l_op, Box::new(Type::Constant(result))))
            }
            (Multiplication | Division, Multiplication | Division) => {
                // If l_op is a division we want to inverse the rhs operator.
                if l_op == Division {
                    op = op.inverse()?;
                }
                // If op is a division we need to ensure it divides evenly
                if op == Division && (r_const == 0 || l_const % r_const != 0) {
                    None
                } else {
                    let result = op.function(l_const, r_const)?;
                    Some(Type::InfixExpr(l_type, l_op, Box::new(Type::Constant(result))))
                }
            }
            _ => None,
        }
    }
}
