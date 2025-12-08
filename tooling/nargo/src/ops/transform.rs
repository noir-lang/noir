use acvm::acir::circuit::ExpressionWidth;
use noirc_driver::{CompiledContract, CompiledProgram};

/// Apply ACVM optimizations on the circuit.
pub fn transform_program(
    compiled_program: CompiledProgram,
    _expression_width: ExpressionWidth,
) -> CompiledProgram {
    super::optimize_program(compiled_program)
}

/// Apply the optimizing transformation on each function in the contract.
pub fn transform_contract(
    contract: CompiledContract,
    _expression_width: ExpressionWidth,
) -> CompiledContract {
    super::optimize_contract(contract)
}
