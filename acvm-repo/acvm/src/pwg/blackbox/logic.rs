use crate::OpcodeResolutionError;
use crate::pwg::{check_bit_size, input_to_value, insert_value};
use acir::circuit::opcodes::FunctionInput;
use acir::{
    AcirField,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::{bit_and, bit_xor};

/// Solves a [`BlackBoxFunc::And`][acir::circuit::black_box_functions::BlackBoxFunc::AND] opcode and inserts
/// the result into the supplied witness map
pub(super) fn and<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    lhs: &FunctionInput<F>,
    rhs: &FunctionInput<F>,
    num_bits: u32,
    output: &Witness,
    pedantic_solving: bool,
) -> Result<(), OpcodeResolutionError<F>> {
    solve_logic_opcode(
        initial_witness,
        lhs,
        rhs,
        num_bits,
        *output,
        pedantic_solving,
        |left, right| bit_and(left, right, num_bits),
    )
}

/// Solves a [`BlackBoxFunc::XOR`][acir::circuit::black_box_functions::BlackBoxFunc::XOR] opcode and inserts
/// the result into the supplied witness map
pub(super) fn xor<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    lhs: &FunctionInput<F>,
    rhs: &FunctionInput<F>,
    num_bits: u32,
    output: &Witness,
    pedantic_solving: bool,
) -> Result<(), OpcodeResolutionError<F>> {
    solve_logic_opcode(
        initial_witness,
        lhs,
        rhs,
        num_bits,
        *output,
        pedantic_solving,
        |left, right| bit_xor(left, right, num_bits),
    )
}

/// Derives the rest of the witness based on the initial low level variables
fn solve_logic_opcode<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    a: &FunctionInput<F>,
    b: &FunctionInput<F>,
    num_bits: u32,
    result: Witness,
    pedantic_solving: bool,
    logic_op: impl Fn(F, F) -> F,
) -> Result<(), OpcodeResolutionError<F>> {
    // TODO(https://github.com/noir-lang/noir/issues/5985): re-enable these by
    // default once we figure out how to combine these with existing
    // noirc_frontend/noirc_evaluator overflow error messages
    let skip_bitsize_checks = !pedantic_solving;
    let w_l_value = input_to_value(initial_witness, *a)?;
    let w_r_value = input_to_value(initial_witness, *b)?;
    let assignment = logic_op(w_l_value, w_r_value);
    if !skip_bitsize_checks {
        check_bit_size(w_l_value, num_bits)?;
        check_bit_size(w_r_value, num_bits)?;
    }
    insert_value(&result, assignment, initial_witness)
}

#[cfg(test)]
mod tests {
    use crate::pwg::blackbox::{and, xor};
    use acir::{
        FieldElement,
        circuit::opcodes::FunctionInput,
        native_types::{Witness, WitnessMap},
    };
    use std::collections::BTreeMap;

    #[test]
    fn test_and() {
        let lhs = FunctionInput::Witness(Witness(1));
        let rhs = FunctionInput::Witness(Witness(2));

        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(5u128)),
            (Witness(2), FieldElement::from(8u128)),
        ]));
        and(&mut initial_witness, &lhs, &rhs, 8, &Witness(3), false).unwrap();
        assert_eq!(initial_witness[&Witness(3)], FieldElement::from(0u128));

        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(26u128)),
            (Witness(2), FieldElement::from(34u128)),
        ]));
        and(&mut initial_witness, &lhs, &rhs, 8, &Witness(3), false).unwrap();
        assert_eq!(initial_witness[&Witness(3)], FieldElement::from(2u128));
    }

    #[test]
    fn test_xor() {
        let lhs = FunctionInput::Witness(Witness(1));
        let rhs = FunctionInput::Witness(Witness(2));

        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(5u128)),
            (Witness(2), FieldElement::from(8u128)),
        ]));
        xor(&mut initial_witness, &lhs, &rhs, 8, &Witness(3), false).unwrap();
        assert_eq!(initial_witness[&Witness(3)], FieldElement::from(13u128));

        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(26u128)),
            (Witness(2), FieldElement::from(34u128)),
        ]));
        xor(&mut initial_witness, &lhs, &rhs, 8, &Witness(3), false).unwrap();
        assert_eq!(initial_witness[&Witness(3)], FieldElement::from(56u128));
    }
}
