//! HashToField128Security fallback function.
use super::{
    blake2s::create_blake2s_constraint,
    utils::{byte_decomposition, round_to_nearest_byte},
    UInt32,
};
use crate::helpers::VariableStore;
use acir::{
    brillig::{self, RegisterIndex},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

pub fn hash_to_field(
    inputs: Vec<(Expression, u32)>,
    outputs: Witness,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_opcodes = Vec::new();
    let mut new_inputs = Vec::new();

    // Decompose the input field elements into bytes and collect the resulting witnesses.
    for (witness, num_bits) in inputs {
        let num_bytes = round_to_nearest_byte(num_bits);
        let (extra_opcodes, extra_inputs, updated_witness_counter) =
            byte_decomposition(witness, num_bytes, num_witness);
        new_opcodes.extend(extra_opcodes);
        new_inputs.extend(extra_inputs);
        num_witness = updated_witness_counter;
    }

    let (result, num_witness, extra_opcodes) = create_blake2s_constraint(new_inputs, num_witness);
    new_opcodes.extend(extra_opcodes);

    // transform bytes to a single field
    let (result, extra_opcodes, num_witness) = field_from_be_bytes(&result, num_witness);
    new_opcodes.extend(extra_opcodes);

    // constrain the outputs to be the same as the result of the circuit
    let mut expr = Expression::from(outputs);
    expr.push_addition_term(-FieldElement::one(), result);
    new_opcodes.push(Opcode::Arithmetic(expr));
    (num_witness, new_opcodes)
}

/// Convert bytes represented by [Witness]es to a single [FieldElement]
fn field_from_be_bytes(result: &[Witness], num_witness: u32) -> (Witness, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();

    // Load `0` and `256` using the load constant function from UInt32
    let (new_witness, extra_opcodes, num_witness) = UInt32::load_constant(0, num_witness);
    let mut new_witness = new_witness.inner;
    new_opcodes.extend(extra_opcodes);
    let (const_256, extra_opcodes, mut num_witness) = UInt32::load_constant(256, num_witness);
    let const_256 = const_256.inner;
    new_opcodes.extend(extra_opcodes);

    // add byte and multiply 256 each round
    for r in result.iter().take(result.len() - 1) {
        let (updated_witness, extra_opcodes, updated_witness_counter) =
            field_addition(&new_witness, r, num_witness);
        new_opcodes.extend(extra_opcodes);
        let (updated_witness, extra_opcodes, updated_witness_counter) =
            field_mul(&updated_witness, &const_256, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        new_witness = updated_witness;
        num_witness = updated_witness_counter;
    }

    let (new_witness, extra_opcodes, num_witness) =
        field_addition(&new_witness, &result[result.len() - 1], num_witness);
    new_opcodes.extend(extra_opcodes);

    (new_witness, new_opcodes, num_witness)
}

/// Caculate and constrain `self` + `rhs` as field
fn field_addition(
    lhs: &Witness,
    rhs: &Witness,
    mut num_witness: u32,
) -> (Witness, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);
    let new_witness = variables.new_variable();

    // calculate `self` + `rhs` as field
    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), *lhs)],
                q_c: FieldElement::zero(),
            }),
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), *rhs)],
                q_c: FieldElement::zero(),
            }),
        ],
        outputs: vec![BrilligOutputs::Simple(new_witness)],
        foreign_call_results: vec![],
        bytecode: vec![brillig::Opcode::BinaryFieldOp {
            op: brillig::BinaryFieldOp::Add,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(0),
        }],
        predicate: None,
    });
    new_opcodes.push(brillig_opcode);
    let num_witness = variables.finalize();

    // constrain addition
    let mut add_expr = Expression::from(new_witness);
    add_expr.push_addition_term(-FieldElement::one(), *lhs);
    add_expr.push_addition_term(-FieldElement::one(), *rhs);
    new_opcodes.push(Opcode::Arithmetic(add_expr));

    (new_witness, new_opcodes, num_witness)
}

/// Calculate and constrain `self` * `rhs` as field
pub(crate) fn field_mul(
    lhs: &Witness,
    rhs: &Witness,
    mut num_witness: u32,
) -> (Witness, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);
    let new_witness = variables.new_variable();

    // calulate `self` * `rhs` with overflow
    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), *lhs)],
                q_c: FieldElement::zero(),
            }),
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), *rhs)],
                q_c: FieldElement::zero(),
            }),
        ],
        outputs: vec![BrilligOutputs::Simple(new_witness)],
        foreign_call_results: vec![],
        bytecode: vec![brillig::Opcode::BinaryFieldOp {
            op: brillig::BinaryFieldOp::Mul,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(0),
        }],
        predicate: None,
    });
    new_opcodes.push(brillig_opcode);
    let num_witness = variables.finalize();

    // constrain mul
    let mut mul_constraint = Expression::from(new_witness);
    mul_constraint.push_multiplication_term(-FieldElement::one(), *lhs, *rhs);
    new_opcodes.push(Opcode::Arithmetic(mul_constraint));

    (new_witness, new_opcodes, num_witness)
}
