use acvm::PartialWitnessGenerator;
use acvm::{acir::circuit::Circuit, pwg::block::Blocks};
use noirc_abi::WitnessMap;

use crate::NargoError;

pub fn execute_circuit(
    backend: &impl PartialWitnessGenerator,
    circuit: Circuit,
    mut initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut blocks = Blocks::default();
    let (unresolved_opcodes, oracles) =
        backend.solve(&mut initial_witness, &mut blocks, circuit.opcodes)?;
    if !unresolved_opcodes.is_empty() || !oracles.is_empty() {
        todo!("Add oracle support to nargo execute")
    }

    Ok(initial_witness)
}
