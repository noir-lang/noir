use acir::{
    AcirField,
    native_types::{Expression, Witness},
};
use indexmap::IndexMap;

/// The `GeneralOptimizer` processes all [`Expression`]s to:
/// - remove any zero-coefficient terms.
/// - merge any quadratic terms containing the same two witnesses.
///
/// This pass does not depend on any other pass and should be the first one in a set of optimizing passes.
pub(crate) struct GeneralOptimizer;

impl GeneralOptimizer {
    pub(crate) fn optimize<F: AcirField>(opcode: Expression<F>) -> Expression<F> {
        // TODO(https://github.com/noir-lang/noir/issues/10109): Perhaps this optimization can be done on the fly
        let opcode = simplify_mul_terms(opcode);
        simplify_linear_terms(opcode)
    }
}

/// Simplifies all mul terms of the form `scale*w1*w2` with the same bi-variate variables
/// while also removing terms that end up with a zero coefficient.
///
/// For instance, mul terms `0*w1*w1 + 2*w2*w1 - w2*w1 - w1*w2` will return an
/// empty vector, because: w1*w2 and w2*w1 are the same bi-variate variable
/// and the resulting scale is `2-1-1 = 0`
fn simplify_mul_terms<F: AcirField>(mut gate: Expression<F>) -> Expression<F> {
    let mut hash_map: IndexMap<(Witness, Witness), F> = IndexMap::new();

    // Canonicalize the ordering of the multiplication, lets just order by variable name
    for (scale, w_l, w_r) in gate.mul_terms.into_iter() {
        let mut pair = [w_l, w_r];
        // Sort using rust sort algorithm
        pair.sort();

        *hash_map.entry((pair[0], pair[1])).or_insert_with(F::zero) += scale;
    }

    gate.mul_terms = hash_map
        .into_iter()
        .filter(|(_, scale)| !scale.is_zero())
        .map(|((w_l, w_r), scale)| (scale, w_l, w_r))
        .collect();
    gate
}

// Simplifies all linear terms with the same variables while also removing
// terms that end up with a zero coefficient.
fn simplify_linear_terms<F: AcirField>(mut gate: Expression<F>) -> Expression<F> {
    let mut hash_map: IndexMap<Witness, F> = IndexMap::new();

    // Canonicalize the ordering of the terms, let's just order by variable name
    for (scale, witness) in gate.linear_combinations.into_iter() {
        *hash_map.entry(witness).or_insert_with(F::zero) += scale;
    }

    gate.linear_combinations = hash_map
        .into_iter()
        .filter(|(_, scale)| !scale.is_zero())
        .map(|(witness, scale)| (scale, witness))
        .collect();
    gate
}

#[cfg(test)]
mod tests {
    use acir::{
        FieldElement,
        circuit::{Circuit, Opcode},
    };

    use crate::{assert_circuit_snapshot, compiler::optimizers::GeneralOptimizer};

    fn optimize(circuit: Circuit<FieldElement>) -> Circuit<FieldElement> {
        let opcodes = circuit
            .clone()
            .opcodes
            .into_iter()
            .map(|opcode| {
                if let Opcode::AssertZero(arith_expr) = opcode {
                    Opcode::AssertZero(GeneralOptimizer::optimize(arith_expr))
                } else {
                    opcode
                }
            })
            .collect();
        let mut optimized_circuit = circuit;
        optimized_circuit.opcodes = opcodes;
        optimized_circuit
    }

    #[test]
    fn removes_zero_coefficients_from_mul_terms() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []

        // The first multiplication should be removed
        ASSERT 0*w0*w1 + w0*w1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0 = w0*w1
        ");
    }

    #[test]
    fn removes_zero_coefficients_from_linear_terms() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []

        // The first linear combination should be removed
        ASSERT 0*w0 + w1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT w1 = 0
        ");
    }

    #[test]
    fn simplifies_mul_terms() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []

        // There are all mul terms with the same variables so we should end up with just one
        // that is the sum of all the coefficients
        ASSERT 2*w0*w1 + 3*w1*w0 + 4*w0*w1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0 = 9*w0*w1
        ");
    }

    #[test]
    fn removes_zero_coefficients_after_simplifying_mul_terms() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 2*w0*w1 + 3*w1*w0 - 5*w0*w1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0 = 0
        ");
    }

    #[test]
    fn simplifies_linear_terms() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []

        // These are all linear terms with the same variable so we should end up with just one
        // that is the sum of all the coefficients
        ASSERT w0 + 2*w0 + 3*w0 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0 = 6*w0
        ");
    }

    #[test]
    fn removes_zero_coefficients_after_simplifying_linear_terms() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT w0 + 2*w0 - 3*w0 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0 = 0
        ");
    }

    #[test]
    fn simplify_mul_terms_example() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0*w1*w1 + 2*w2*w1 - w2*w1 - w1*w2 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = optimize(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT 0 = 0
        ");
    }
}
