use acir::{
    brillig::{ForeignCallParam, ForeignCallResult, RegisterIndex, Value},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        OpcodeLocation,
    },
    native_types::WitnessMap,
    FieldElement,
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;
use brillig_vm::{Registers, VMStatus, VM};

use crate::{pwg::OpcodeNotSolvable, OpcodeResolutionError};

use super::{get_value, insert_value};

pub(super) enum BrilligSolverStatus {
    Finished,
    InProgress,
    ForeignCallWait(ForeignCallWaitInfo),
}

pub(super) struct BrilligSolver<'b, B: BlackBoxFunctionSolver> {
    vm: VM<'b, B>,
    acir_index: usize,
}

impl<'b, B: BlackBoxFunctionSolver> BrilligSolver<'b, B> {
    /// Evaluates if the Brillig block should be skipped entirely
    pub(super) fn should_skip(
        witness: &WitnessMap,
        brillig: &Brillig,
    ) -> Result<bool, OpcodeResolutionError> {
        // If the predicate is `None`, the block should never be skipped
        // If the predicate is `Some` but we cannot find a value, then we return stalled
        match &brillig.predicate {
            Some(pred) => Ok(get_value(pred, witness)?.is_zero()),
            None => Ok(false),
        }
    }

    /// Assigns the zero value to all outputs of the given [`Brillig`] bytecode.
    pub(super) fn zero_out_brillig_outputs(
        initial_witness: &mut WitnessMap,
        brillig: &Brillig,
    ) -> Result<(), OpcodeResolutionError> {
        for output in &brillig.outputs {
            match output {
                BrilligOutputs::Simple(witness) => {
                    insert_value(witness, FieldElement::zero(), initial_witness)?;
                }
                BrilligOutputs::Array(witness_arr) => {
                    for witness in witness_arr {
                        insert_value(witness, FieldElement::zero(), initial_witness)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Constructs a solver for a Brillig block given the bytecode and initial
    /// witness.
    pub(super) fn new(
        initial_witness: &mut WitnessMap,
        brillig: &'b Brillig,
        bb_solver: &'b B,
        acir_index: usize,
    ) -> Result<Self, OpcodeResolutionError> {
        // Set input values
        let mut input_register_values: Vec<Value> = Vec::new();
        let mut input_memory: Vec<Value> = Vec::new();
        // Each input represents an expression or array of expressions to evaluate.
        // Iterate over each input and evaluate the expression(s) associated with it.
        // Push the results into registers and/or memory.
        // If a certain expression is not solvable, we stall the ACVM and do not proceed with Brillig VM execution.
        for input in &brillig.inputs {
            match input {
                BrilligInputs::Single(expr) => match get_value(expr, initial_witness) {
                    Ok(value) => input_register_values.push(value.into()),
                    Err(_) => {
                        return Err(OpcodeResolutionError::OpcodeNotSolvable(
                            OpcodeNotSolvable::ExpressionHasTooManyUnknowns(expr.clone()),
                        ))
                    }
                },
                BrilligInputs::Array(expr_arr) => {
                    // Attempt to fetch all array input values
                    let memory_pointer = input_memory.len();
                    for expr in expr_arr.iter() {
                        match get_value(expr, initial_witness) {
                            Ok(value) => input_memory.push(value.into()),
                            Err(_) => {
                                return Err(OpcodeResolutionError::OpcodeNotSolvable(
                                    OpcodeNotSolvable::ExpressionHasTooManyUnknowns(expr.clone()),
                                ))
                            }
                        }
                    }

                    // Push value of the array pointer as a register
                    input_register_values.push(Value::from(memory_pointer));
                }
            }
        }

        // Instantiate a Brillig VM given the solved input registers and memory
        // along with the Brillig bytecode.
        let input_registers = Registers::load(input_register_values);
        let vm = VM::new(input_registers, input_memory, &brillig.bytecode, vec![], bb_solver);
        Ok(Self { vm, acir_index })
    }

    pub(super) fn solve(&mut self) -> Result<BrilligSolverStatus, OpcodeResolutionError> {
        let status = self.vm.process_opcodes();
        self.handle_vm_status(status)
    }

    fn handle_vm_status(
        &self,
        vm_status: VMStatus,
    ) -> Result<BrilligSolverStatus, OpcodeResolutionError> {
        // Check the status of the Brillig VM and return a resolution.
        // It may be finished, in-progress, failed, or may be waiting for results of a foreign call.
        // Return the "resolution" to the caller who may choose to make subsequent calls
        // (when it gets foreign call results for example).
        match vm_status {
            VMStatus::Finished => Ok(BrilligSolverStatus::Finished),
            VMStatus::InProgress => Ok(BrilligSolverStatus::InProgress),
            VMStatus::Failure { message, call_stack } => {
                Err(OpcodeResolutionError::BrilligFunctionFailed {
                    message,
                    call_stack: call_stack
                        .iter()
                        .map(|brillig_index| OpcodeLocation::Brillig {
                            acir_index: self.acir_index,
                            brillig_index: *brillig_index,
                        })
                        .collect(),
                })
            }
            VMStatus::ForeignCallWait { function, inputs } => {
                Ok(BrilligSolverStatus::ForeignCallWait(ForeignCallWaitInfo { function, inputs }))
            }
        }
    }

    pub(super) fn finalize(
        self,
        witness: &mut WitnessMap,
        brillig: &Brillig,
    ) -> Result<(), OpcodeResolutionError> {
        // Finish the Brillig execution by writing the outputs to the witness map
        let vm_status = self.vm.get_status();
        match vm_status {
            VMStatus::Finished => {
                self.write_brillig_outputs(witness, brillig)?;
                Ok(())
            }
            _ => panic!("Brillig VM has not completed execution"),
        }
    }

    fn write_brillig_outputs(
        &self,
        witness_map: &mut WitnessMap,
        brillig: &Brillig,
    ) -> Result<(), OpcodeResolutionError> {
        // Write VM execution results into the witness map
        for (i, output) in brillig.outputs.iter().enumerate() {
            let register_value = self.vm.get_registers().get(RegisterIndex::from(i));
            match output {
                BrilligOutputs::Simple(witness) => {
                    insert_value(witness, register_value.to_field(), witness_map)?;
                }
                BrilligOutputs::Array(witness_arr) => {
                    // Treat the register value as a pointer to memory
                    for (i, witness) in witness_arr.iter().enumerate() {
                        let value = &self.vm.get_memory()[register_value.to_usize() + i];
                        insert_value(witness, value.to_field(), witness_map)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn resolve_pending_foreign_call(&mut self, foreign_call_result: ForeignCallResult) {
        match self.vm.get_status() {
            VMStatus::ForeignCallWait { .. } => self.vm.resolve_foreign_call(foreign_call_result),
            _ => unreachable!("Brillig VM is not waiting for a foreign call"),
        }
    }
}

/// Encapsulates a request from a Brillig VM process that encounters a [foreign call opcode][acir::brillig_vm::Opcode::ForeignCall]
/// where the result of the foreign call has not yet been provided.
///
/// The caller must resolve this opcode externally based upon the information in the request.
#[derive(Debug, PartialEq, Clone)]
pub struct ForeignCallWaitInfo {
    /// An identifier interpreted by the caller process
    pub function: String,
    /// Resolved inputs to a foreign call computed in the previous steps of a Brillig VM process
    pub inputs: Vec<ForeignCallParam>,
}
