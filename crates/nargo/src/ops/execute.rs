use acvm::pwg::{ACVMStatus, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use crate::NargoError;

use super::foreign_calls::ForeignCall;

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
                    let foreign_call_result = ForeignCall::execute(foreign_call)?;
                    acvm.resolve_pending_foreign_call(foreign_call_result);
                }
            }
        }
    }

    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}
