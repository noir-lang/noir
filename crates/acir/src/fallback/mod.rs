use noir_field::FieldElement;

use crate::{
    circuit::{gate::Directive, Gate},
    native_types::{Expression, Witness},
};

// Perform bit decomposition on the provided expression
pub fn split(
    gate: Expression,
    bit_size: u32,
    mut num_witness: u32,
    new_gates: &mut Vec<Gate>,
) -> Vec<Witness> {
    let mut bit_vector = Vec::new();
    let mut g = gate.clone();
    let mut two_pow = FieldElement::one();
    let mut intermediate_gates = Vec::new();
    for _i in 0..bit_size {
        let w = Witness(num_witness);
        num_witness += 1;
        bit_vector.push(w);
        let w_bin = Expression {
            mul_terms: vec![(FieldElement::one(), w, w)],
            linear_combinations: vec![(-FieldElement::one(), w)],
            q_c: FieldElement::zero(),
        };
        intermediate_gates.push(Gate::Arithmetic(w_bin));
        g.linear_combinations.push((-two_pow, w));
        two_pow = FieldElement::from(2_i128) * two_pow;
    }
    new_gates.push(Gate::Directive(Directive::ToRadix {
        a: gate,
        b: bit_vector.clone(),
        radix: 2,
    }));
    new_gates.extend(intermediate_gates);
    g.sort();
    new_gates.push(Gate::Arithmetic(g));
    bit_vector
}

// Range constraint
pub fn range(gate: Expression, bit_size: u32, num_witness: u32, new_gates: &mut Vec<Gate>) -> u32 {
    let bits = split(gate, bit_size, num_witness, new_gates);
    num_witness + bits.len() as u32
}

pub fn and(
    a: Expression,
    b: Expression,
    result: Witness,
    bit_size: u32,
    mut num_witness: u32,
    new_gates: &mut Vec<Gate>,
) -> u32 {
    let a_bits = split(a, bit_size, num_witness, new_gates);
    num_witness += a_bits.len() as u32;
    let b_bits = split(b, bit_size, num_witness, new_gates);
    num_witness += b_bits.len() as u32;
    let mut g = Expression::default();
    let mut two_pow = FieldElement::one();
    for i in 0..bit_size {
        g.mul_terms.push((two_pow, a_bits[i as usize], b_bits[i as usize]));
        two_pow = FieldElement::from(2_i128) * two_pow;
    }
    g.linear_combinations = vec![(-FieldElement::one(), result)];
    g.sort();
    new_gates.push(Gate::Arithmetic(g));
    num_witness
}

pub fn xor(
    a: Expression,
    b: Expression,
    result: Witness,
    bit_size: u32,
    mut num_witness: u32,
    new_gates: &mut Vec<Gate>,
) -> u32 {
    let a_bits = split(a, bit_size, num_witness, new_gates);
    num_witness += a_bits.len() as u32;
    let b_bits = split(b, bit_size, num_witness, new_gates);
    num_witness += b_bits.len() as u32;
    let mut g = Expression::default();
    let mut two_pow = FieldElement::one();
    for i in 0..bit_size {
        g.linear_combinations.push((two_pow, a_bits[i as usize]));
        g.linear_combinations.push((two_pow, b_bits[i as usize]));
        two_pow = FieldElement::from(2_i128) * two_pow;
        g.mul_terms.push((-two_pow, a_bits[i as usize], b_bits[i as usize]));
    }
    g.linear_combinations.push((-FieldElement::one(), result));
    g.sort();
    new_gates.push(Gate::Arithmetic(g));
    num_witness
}
