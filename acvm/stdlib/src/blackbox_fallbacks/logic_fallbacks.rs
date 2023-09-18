use crate::{blackbox_fallbacks::utils::mul_with_witness, helpers::VariableStore};

use super::utils::{bit_decomposition, boolean_expr};
use acir::{
    acir_field::FieldElement,
    circuit::Opcode,
    native_types::{Expression, Witness},
};

// Range constraint
pub fn range(opcode: Expression, bit_size: u32, mut num_witness: u32) -> (u32, Vec<Opcode>) {
    if bit_size == 1 {
        let mut variables = VariableStore::new(&mut num_witness);
        let bit_constraint = Opcode::Arithmetic(boolean_expr(&opcode, &mut variables));
        return (variables.finalize(), vec![bit_constraint]);
    }

    let (new_opcodes, _, updated_witness_counter) =
        bit_decomposition(opcode, bit_size, num_witness);
    (updated_witness_counter, new_opcodes)
}

/// Returns a set of opcodes which constrain `a & b == result`
///
/// `a` and `b` are assumed to be constrained to fit within `bit_size` externally.
pub fn and(
    a: Expression,
    b: Expression,
    result: Witness,
    bit_size: u32,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    if bit_size == 1 {
        let mut variables = VariableStore::new(&mut num_witness);

        let mut and_expr = mul_with_witness(&a, &b, &mut variables);
        and_expr.push_addition_term(-FieldElement::one(), result);

        return (variables.finalize(), vec![Opcode::Arithmetic(and_expr)]);
    }
    // Decompose the operands into bits
    //
    let (extra_opcodes_a, a_bits, updated_witness_counter) =
        bit_decomposition(a, bit_size, num_witness);

    let (extra_opcodes_b, b_bits, updated_witness_counter) =
        bit_decomposition(b, bit_size, updated_witness_counter);

    assert_eq!(a_bits.len(), b_bits.len());
    assert_eq!(a_bits.len(), bit_size as usize);

    let mut two_pow = FieldElement::one();
    let two = FieldElement::from(2_i128);

    // Build an expression that Multiplies each bit element-wise
    // This gives the same truth table as the AND operation
    // Additionally, we multiply by a power of 2 to build up the
    // expected output; ie result = \sum 2^i x_i * y_i
    let mut and_expr = Expression::default();
    for (a_bit, b_bit) in a_bits.into_iter().zip(b_bits) {
        and_expr.push_multiplication_term(two_pow, a_bit, b_bit);
        two_pow = two * two_pow;
    }
    and_expr.push_addition_term(-FieldElement::one(), result);

    and_expr.sort();

    let mut new_opcodes = Vec::new();
    new_opcodes.extend(extra_opcodes_a);
    new_opcodes.extend(extra_opcodes_b);
    new_opcodes.push(Opcode::Arithmetic(and_expr));

    (updated_witness_counter, new_opcodes)
}

/// Returns a set of opcodes which constrain `a ^ b == result`
///
/// `a` and `b` are assumed to be constrained to fit within `bit_size` externally.
pub fn xor(
    a: Expression,
    b: Expression,
    result: Witness,
    bit_size: u32,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    if bit_size == 1 {
        let mut variables = VariableStore::new(&mut num_witness);

        let product = mul_with_witness(&a, &b, &mut variables);
        let mut xor_expr = &(&a + &b) - &product;
        xor_expr.push_addition_term(-FieldElement::one(), result);

        return (variables.finalize(), vec![Opcode::Arithmetic(xor_expr)]);
    }

    // Decompose the operands into bits
    //
    let (extra_opcodes_a, a_bits, updated_witness_counter) =
        bit_decomposition(a, bit_size, num_witness);
    let (extra_opcodes_b, b_bits, updated_witness_counter) =
        bit_decomposition(b, bit_size, updated_witness_counter);

    assert_eq!(a_bits.len(), b_bits.len());
    assert_eq!(a_bits.len(), bit_size as usize);

    let mut two_pow = FieldElement::one();
    let two = FieldElement::from(2_i128);

    // Build an xor expression
    // TODO: check this is the correct arithmetization
    let mut xor_expr = Expression::default();
    for (a_bit, b_bit) in a_bits.into_iter().zip(b_bits) {
        xor_expr.push_addition_term(two_pow, a_bit);
        xor_expr.push_addition_term(two_pow, b_bit);
        two_pow = two * two_pow;
        xor_expr.push_multiplication_term(-two_pow, a_bit, b_bit);
    }
    xor_expr.push_addition_term(-FieldElement::one(), result);

    xor_expr.sort();
    let mut new_opcodes = Vec::new();
    new_opcodes.extend(extra_opcodes_a);
    new_opcodes.extend(extra_opcodes_b);
    new_opcodes.push(Opcode::Arithmetic(xor_expr));

    (updated_witness_counter, new_opcodes)
}
