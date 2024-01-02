use acir::{
    native_types::{Expression, Witness},
    FieldElement,
};
use indexmap::IndexMap;

/// The `GeneralOptimizer` processes all [`Expression`]s to:
/// - remove any zero-coefficient terms.
/// - merge any quadratic terms containing the same two witnesses.
pub(crate) struct GeneralOptimizer;

impl GeneralOptimizer {
    pub(crate) fn optimize(opcode: Expression) -> Expression {
        // XXX: Perhaps this optimization can be done on the fly
        let opcode = remove_zero_coefficients(opcode);
        let opcode = simplify_mul_terms(opcode);
        simplify_linear_terms(opcode)
    }
}

// Remove all terms with zero as a coefficient
fn remove_zero_coefficients(mut opcode: Expression) -> Expression {
    // Check the mul terms
    opcode.mul_terms.retain(|(scale, _, _)| !scale.is_zero());
    // Check the linear combination terms
    opcode.linear_combinations.retain(|(scale, _)| !scale.is_zero());
    opcode
}

// Simplifies all mul terms with the same bi-variate variables
fn simplify_mul_terms(mut gate: Expression) -> Expression {
    let mut hash_map: IndexMap<(Witness, Witness), FieldElement> = IndexMap::new();

    // Canonicalize the ordering of the multiplication, lets just order by variable name
    for (scale, w_l, w_r) in gate.mul_terms.into_iter() {
        let mut pair = [w_l, w_r];
        // Sort using rust sort algorithm
        pair.sort();

        *hash_map.entry((pair[0], pair[1])).or_insert_with(FieldElement::zero) += scale;
    }

    gate.mul_terms = hash_map.into_iter().map(|((w_l, w_r), scale)| (scale, w_l, w_r)).collect();
    gate
}

// Simplifies all mul terms with the same bi-variate variables
fn simplify_linear_terms(mut gate: Expression) -> Expression {
    let mut hash_map: IndexMap<Witness, FieldElement> = IndexMap::new();

    // Canonicalize the ordering of the multiplication, lets just order by variable name
    for (scale, witness) in gate.linear_combinations.into_iter() {
        *hash_map.entry(witness).or_insert_with(FieldElement::zero) += scale;
    }

    gate.linear_combinations = hash_map
        .into_iter()
        .filter(|(_, scale)| scale != &FieldElement::zero())
        .map(|(witness, scale)| (scale, witness))
        .collect();
    gate
}
