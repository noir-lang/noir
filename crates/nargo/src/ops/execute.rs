use acvm::acir::brillig::{ForeignCallResult, Value};
use acvm::pwg::{ACVMStatus, ForeignCallWaitInfo, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use iter_extended::vecmap;

use crate::NargoError;

pub fn execute_circuit<B: BlackBoxFunctionSolver + Default>(
    _backend: &B,
    circuit: Circuit,
    initial_witness: WitnessMap,
) -> Result<WitnessMap, NargoError> {
    let mut acvm = ACVM::new(B::default(), circuit.opcodes, initial_witness);

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => return Err(error.into()),
            ACVMStatus::RequiresForeignCall => {
                while let Some(foreign_call) = acvm.get_pending_foreign_call() {
                    let foreign_call_result = execute_foreign_call(foreign_call);
                    acvm.resolve_pending_foreign_call(foreign_call_result);
                }
            }
        }
    }

    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}

fn execute_foreign_call(foreign_call: &ForeignCallWaitInfo) -> ForeignCallResult {
    // TODO(#1615): Nargo only supports "oracle_print_**_impl" functions  that print a singular value or an array and nothing else
    // This should be expanded in a general logging refactor
    match foreign_call.function.as_str() {
        // TODO(#1910): Move to an enum and don't match directly on these strings
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
        "get_number_sequence" => {
            let sequence_length: u128 = foreign_call.inputs[0][0].to_field().to_u128();

            vecmap(0..sequence_length, Value::from).into()
        }
        "get_reverse_number_sequence" => {
            let sequence_length: u128 = foreign_call.inputs[0][0].to_field().to_u128();

            vecmap((0..sequence_length).rev(), Value::from).into()
        }
        _ => panic!("unexpected foreign call type"),
    }
}
