use crate::pwg::{input_to_value, insert_value};
use crate::OpcodeResolutionError;
use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    AcirField,
};
use acvm_blackbox_solver::{bit_and, bit_xor};

/// Solves a [`BlackBoxFunc::And`][acir::circuit::black_box_functions::BlackBoxFunc::AND] opcode and inserts
/// the result into the supplied witness map
pub(super) fn and<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    lhs: &FunctionInput<F>,
    rhs: &FunctionInput<F>,
    output: &Witness,
) -> Result<(), OpcodeResolutionError<F>> {
    assert_eq!(
        lhs.num_bits(),
        rhs.num_bits(),
        "number of bits specified for each input must be the same"
    );
    solve_logic_opcode(initial_witness, lhs, rhs, *output, |left, right| {
        bit_and(left, right, lhs.num_bits())
    })
}

/// Solves a [`BlackBoxFunc::XOR`][acir::circuit::black_box_functions::BlackBoxFunc::XOR] opcode and inserts
/// the result into the supplied witness map
pub(super) fn xor<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    lhs: &FunctionInput<F>,
    rhs: &FunctionInput<F>,
    output: &Witness,
) -> Result<(), OpcodeResolutionError<F>> {
    assert_eq!(
        lhs.num_bits(),
        rhs.num_bits(),
        "number of bits specified for each input must be the same"
    );
    solve_logic_opcode(initial_witness, lhs, rhs, *output, |left, right| {
        bit_xor(left, right, lhs.num_bits())
    })
}

/// Derives the rest of the witness based on the initial low level variables
fn solve_logic_opcode<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    a: &FunctionInput<F>,
    b: &FunctionInput<F>,
    result: Witness,
    logic_op: impl Fn(F, F) -> F,
) -> Result<(), OpcodeResolutionError<F>> {
    // TODO(https://github.com/noir-lang/noir/issues/5985): re-enable these once we figure out how to combine these with existing
    // noirc_frontend/noirc_evaluator overflow error messages
    let skip_bitsize_checks = true;
    let w_l_value = input_to_value(initial_witness, *a, skip_bitsize_checks)?;
    let w_r_value = input_to_value(initial_witness, *b, skip_bitsize_checks)?;
    let assignment = logic_op(w_l_value, w_r_value);

    insert_value(&result, assignment, initial_witness)
}
