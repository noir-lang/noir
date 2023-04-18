use acvm::{acir::circuit::Circuit, pwg::block::Blocks};
use acvm::{PartialWitnessGenerator, UnresolvedData};
use noirc_abi::WitnessMap;

use crate::NargoError;

pub fn execute_circuit(
    backend: &impl PartialWitnessGenerator,
    circuit: Circuit,
    mut initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut blocks = Blocks::default();
    let UnresolvedData { unresolved_opcodes, unresolved_oracles, unresolved_brilligs } =
        backend.solve(&mut initial_witness, &mut blocks, circuit.opcodes.clone())?;
    if !unresolved_opcodes.is_empty()
        || !unresolved_oracles.is_empty()
        || !unresolved_brilligs.is_empty()
    {
        todo!("Add oracle support to nargo execute")
    }

    Ok(initial_witness)
}
