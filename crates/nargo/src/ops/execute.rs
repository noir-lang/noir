use acvm::acir::circuit::Opcode;
use acvm::pwg::{solve, Blocks, PartialWitnessGeneratorStatus, UnresolvedBrilligCall};
use acvm::PartialWitnessGenerator;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::NargoError;

pub fn execute_circuit(
    backend: &impl PartialWitnessGenerator,
    circuit: Circuit,
    mut initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut blocks = Blocks::default();
    let solver_status = solve(backend, &mut initial_witness, &mut blocks, circuit.opcodes)?;

    // TODO(#1615): Nargo only supports "oracle_print_**_impl" functions  that print a singular value or an array and nothing else
    // This should be expanded in a general logging refactor
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

            // Execute foreign calls
            // TODO(#1615): "oracle_print_impl" and "oracle_print_array_impl" are just identity funcs
            if foreign_call_wait_info.function == "oracle_print_impl" {
                let values = &foreign_call_wait_info.inputs[0];
                println!("{:?}", values[0].to_field().to_hex());
                brillig.foreign_call_results.push(foreign_call_wait_info.inputs[0][0].into());
            } else if foreign_call_wait_info.function == "oracle_print_array_impl" {
                let mut outputs_hex = Vec::new();
                for values in foreign_call_wait_info.inputs.clone() {
                    for value in values {
                        outputs_hex.push(value.to_field().to_hex());
                    }
                }
                // Join all of the hex strings using a comma
                let comma_separated_elements = outputs_hex.join(", ");
                let output_witnesses_string = "[".to_owned() + &comma_separated_elements + "]";
                println!("{output_witnesses_string}");
                brillig.foreign_call_results.push(foreign_call_wait_info.inputs[0][0].into());
            }

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
