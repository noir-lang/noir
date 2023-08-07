use acvm::acir::circuit::OpcodeLabel;
use acvm::pwg::{ACVMStatus, OpcodeResolutionError, ACVM};
use acvm::Backend;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use noirc_abi::input_parser::InputValue;
use noirc_abi::{Abi, InputMap};
use noirc_errors::debug_info::DebugInfo;

use crate::errors::ForeignCallError;
use crate::NargoError;

use super::foreign_calls::ForeignCall;
use super::optimize::OptimizedFunction;

#[derive(Debug)]
pub struct SolvedFunction {
    pub name: String,

    pub abi: Abi,

    pub bytecode: Circuit,

    pub debug: DebugInfo,

    pub witness: WitnessMap,

    pub return_value: Option<InputValue>,
}

pub fn execute_function<B: Backend>(
    _backend: &B,
    func: OptimizedFunction,
    inputs_map: InputMap,
) -> Result<SolvedFunction, NargoError> {
    let OptimizedFunction { name, bytecode, abi, debug, .. } = func;

    let initial_witness = abi.encode(&inputs_map, None).map_err(ForeignCallError::from)?;
    // TODO: ACVM needs to be fixed to not consume this
    let mut acvm = ACVM::new(B::default(), bytecode.opcodes.clone(), initial_witness);

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(err) => match err {
                OpcodeResolutionError::UnsatisfiedConstrain {
                    opcode_label: OpcodeLabel::Resolved(opcode_index),
                } => return Err(NargoError::UnsatisfiedConstraint(opcode_index as usize)),
                _ => return Err(err.into()),
            },
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result = ForeignCall::execute(&foreign_call, true)?;
                acvm.resolve_pending_foreign_call(foreign_call_result);
            }
        }
    }

    let solved_witness = acvm.finalize();
    let public_abi = abi.public_abi();
    let (_, return_value) = public_abi.decode(&solved_witness).map_err(ForeignCallError::from)?;

    let solved_function = SolvedFunction {
        name,
        // TODO: Is this wrong?
        abi: public_abi,
        bytecode,
        debug,
        witness: solved_witness,
        return_value,
    };

    Ok(solved_function)
}
