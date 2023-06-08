use acvm::acir::brillig_vm::ForeignCallResult;
use acvm::acir::circuit::Opcode;
use acvm::pwg::{solve, PartialWitnessGeneratorStatus, UnresolvedBrilligCall};
use acvm::PartialWitnessGenerator;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap, pwg::block::Blocks};

use crate::NargoError;

pub fn execute_circuit(
    backend: &impl PartialWitnessGenerator,
    circuit: Circuit,
    mut initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut blocks = Blocks::default();
    let solver_status = solve(backend, &mut initial_witness, &mut blocks, circuit.opcodes)?;

    // TODO(#1615): Nargo only supports "oracle_print_impl" functions that print a singular value and nothing else
    // expand this in a general logging refactor
    if let PartialWitnessGeneratorStatus::RequiresOracleData {
        unresolved_brillig_calls,
        required_oracle_data,
        unsolved_opcodes,
    } = solver_status
    {
        if !required_oracle_data.is_empty() {
            unreachable!("oracles are not supported by nargo execute")
        }
        for unresolved_brillig_call in unresolved_brillig_calls {
            let UnresolvedBrilligCall { foreign_call_wait_info, mut brillig } =
                unresolved_brillig_call;
            let value = foreign_call_wait_info.inputs[0];

            // Execute foreign call "oracle_print_impl"
            println!("{:?}", value.to_field().to_hex());

            // TODO(#1615): "oracle_print_impl" is just an identity func
            brillig.foreign_call_results.push(ForeignCallResult { values: vec![value] });

            let mut next_opcodes_for_solving = vec![Opcode::Brillig(brillig)];
            next_opcodes_for_solving.extend_from_slice(&unsolved_opcodes[..]);

            let solver_status =
                solve(backend, &mut initial_witness, &mut blocks, next_opcodes_for_solving)?;
            if matches!(solver_status, PartialWitnessGeneratorStatus::RequiresOracleData { .. }) {
                todo!("Add multiple foreign call support to nargo execute")
                // TODO 1557
            }
        }
    }

    Ok(initial_witness)
}
