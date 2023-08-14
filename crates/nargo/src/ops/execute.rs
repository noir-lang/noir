use acvm::pwg::{ACVMStatus, ACVM};
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::NargoError;

use super::foreign_calls::ForeignCall;

pub fn execute_circuit(
    circuit: Circuit,
    initial_witness: WitnessMap,
    show_output: bool,
) -> Result<WitnessMap, NargoError> {
    #[allow(deprecated)]
    let black_box_function_solver = acvm::blackbox_solver::BarretenbergSolver::new();
    let mut acvm = ACVM::new(black_box_function_solver, circuit.opcodes, initial_witness);

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => return Err(error.into()),
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result = ForeignCall::execute(&foreign_call, show_output)?;
                acvm.resolve_pending_foreign_call(foreign_call_result);
            }
        }
    }

    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}
