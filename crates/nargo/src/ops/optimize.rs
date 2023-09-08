use acvm::{
    acir::circuit::{Circuit, Opcode},
    compiler::AcirTransformationMap,
    Language,
};
use iter_extended::try_vecmap;
use noirc_driver::CompiledContract;

use crate::NargoError;

pub fn optimize_circuit(
    circuit: Circuit,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<(Circuit, AcirTransformationMap), NargoError> {
    acvm::compiler::compile(circuit, np_language, &is_opcode_supported).map_err(NargoError::from)
}

pub fn optimize_contract(
    contract: CompiledContract,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<CompiledContract, NargoError> {
    let functions = try_vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) =
            acvm::compiler::compile(func.bytecode, np_language, &is_opcode_supported)?;
        func.bytecode = optimized_bytecode;
        func.debug.update_acir(location_map);
        Ok::<_, NargoError>(func)
    })?;

    Ok(CompiledContract { functions, ..contract })
}
