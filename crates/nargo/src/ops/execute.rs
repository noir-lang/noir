use acvm::{acir::circuit::Circuit, pwg::block::Blocks};
use acvm::{PartialWitnessGenerator, PartialWitnessGeneratorStatus};
use noirc_abi::WitnessMap;

use crate::NargoError;

pub fn execute_circuit(
    backend: &impl PartialWitnessGenerator,
    circuit: Circuit,
    mut initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut blocks = Blocks::default();
    let solver_status = backend.solve(&mut initial_witness, &mut blocks, circuit.opcodes)?;
    if matches!(solver_status, PartialWitnessGeneratorStatus::RequiresOracleData { .. }) {
        todo!("Add oracle support to nargo execute")
    }

    Ok(initial_witness)
}
