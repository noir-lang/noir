use noirc_driver::{CompiledContract, CompiledProgram};

/// Stub - ACVM transformations are not available in Sensei
pub fn transform_program(
    compiled_program: CompiledProgram,
    _expression_width: String,
) -> CompiledProgram {
    // Without ACVM, we return the program unchanged
    compiled_program
}

/// Stub - ACVM transformations are not available in Sensei  
pub fn transform_contract(
    contract: CompiledContract,
    _expression_width: String,
) -> CompiledContract {
    // Without ACVM, we return the contract unchanged
    contract
}