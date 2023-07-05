use acvm::acir::brillig_vm::ForeignCallResult;
use acvm::pwg::{ForeignCallWaitInfo, PartialWitnessGeneratorStatus, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::NargoError;

pub fn execute_circuit<B: BlackBoxFunctionSolver + Default>(
    _backend: &B,
    circuit: Circuit,
    initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut acvm = ACVM::new(B::default(), circuit.opcodes, initial_witness);

    // TODO(#1615): Nargo only supports "oracle_print_**_impl" functions  that print a singular value or an array and nothing else
    // This should be expanded in a general logging refactor
    loop {
        let solver_status = acvm.solve()?;

        match solver_status {
            PartialWitnessGeneratorStatus::Solved => break,
            PartialWitnessGeneratorStatus::RequiresForeignCall => {
                let foreign_call =
                    acvm.get_pending_foreign_call().expect("Should be waiting on a foreign call");

                let foreign_call_result = execute_foreign_call(foreign_call);
                acvm.resolve_pending_foreign_call(foreign_call_result);
            }
        }
    }

    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}

fn execute_foreign_call(foreign_call: &ForeignCallWaitInfo) -> ForeignCallResult {
    // TODO(#1615): "oracle_print_impl" and "oracle_print_array_impl" are just identity funcs
    match foreign_call.function.as_str() {
        "oracle_print_impl" => {
            let values = &foreign_call.inputs[0];
            println!("{:?}", values[0].to_field().to_hex());
            values[0].into()
        }
        "oracle_print_array_impl" => {
            let mut outputs_hex = Vec::new();
            for values in &foreign_call.inputs {
                for value in values {
                    outputs_hex.push(value.to_field().to_hex());
                }
            }
            // Join all of the hex strings using a comma
            let comma_separated_elements = outputs_hex.join(", ");
            let output_witnesses_string = "[".to_owned() + &comma_separated_elements + "]";
            println!("{output_witnesses_string}");

            foreign_call.inputs[0][0].into()
        }
        _ => panic!("unexpected foreign call type"),
    }
}
