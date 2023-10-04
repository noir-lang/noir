use acir::{
    brillig::{ForeignCallParam, RegisterIndex, Value},
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
    witness: &'b mut WitnessMap,
    brillig: &'b Brillig,
    acir_index: usize,
    vm: VM<'b, B>,
}

impl<'b, B: BlackBoxFunctionSolver> BrilligSolver<'b, B> {
    pub(super) fn build_or_skip(
        initial_witness: &'b mut WitnessMap,
        brillig: &'b Brillig,
        bb_solver: &'b B,
        acir_index: usize,
    ) -> Result<Option<Self>, OpcodeResolutionError> {
        if Self::should_skip(initial_witness, brillig)? {
            Self::zero_out_brillig_outputs(initial_witness, brillig)?;
            return Ok(None);
        }

        let vm = Self::setup_vm(initial_witness, brillig, bb_solver)?;
        Ok(Some(
            Self {
                witness: initial_witness,
                brillig,
                acir_index,
                vm,
            }
        ))
    }

    fn should_skip(witness: &mut WitnessMap, brillig: &Brillig) -> Result<bool, OpcodeResolutionError> {
        // If the predicate is `None`, then we simply return the value 1
        // If the predicate is `Some` but we cannot find a value, then we return stalled
        let pred_value = match &brillig.predicate {
            Some(pred) => get_value(pred, witness),
            None => Ok(FieldElement::one()),
        }?;

        // A zero predicate indicates the oracle should be skipped, and its outputs zeroed.
        Ok(pred_value.is_zero())
    }

    /// Assigns the zero value to all outputs of the given [`Brillig`] bytecode.
    fn zero_out_brillig_outputs(
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

    fn setup_vm(
        witness: &mut WitnessMap,
        brillig: &Brillig,
        bb_solver: &'b B,
    ) -> Result<VM<'b, B>, OpcodeResolutionError> {
        // Set input values
        let mut input_register_values: Vec<Value> = Vec::new();
        let mut input_memory: Vec<Value> = Vec::new();
        // Each input represents an expression or array of expressions to evaluate.
        // Iterate over each input and evaluate the expression(s) associated with it.
        // Push the results into registers and/or memory.
        // If a certain expression is not solvable, we stall the ACVM and do not proceed with Brillig VM execution.
        for input in &brillig.inputs {
            match input {
                BrilligInputs::Single(expr) => match get_value(expr, witness) {
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
                        match get_value(expr, witness) {
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
        // along with the Brillig bytecode, and any present foreign call results.
        let input_registers = Registers::load(input_register_values);
        Ok(VM::new(
            input_registers,
            input_memory,
            brillig.bytecode.clone(),
            brillig.foreign_call_results.clone(),
            bb_solver,
        ))
    }

    pub(super) fn solve(&mut self) -> Result<BrilligSolverStatus, OpcodeResolutionError> {
        // Run the Brillig VM on these inputs, bytecode, etc!
        while matches!(self.vm.process_opcode(), VMStatus::InProgress) {}

        self.finish_execution()
    }

    pub(super) fn finish_execution(&mut self) -> Result<BrilligSolverStatus, OpcodeResolutionError> {
        // Check the status of the Brillig VM.
        // It may be finished, in-progress, failed, or may be waiting for results of a foreign call.
        // Return the "resolution" to the caller who may choose to make subsequent calls
        // (when it gets foreign call results for example).
        let vm_status = self.vm.get_status();
        match vm_status {
            VMStatus::Finished => {
                self.write_brillig_outputs()?;
                Ok(BrilligSolverStatus::Finished)
            }
            VMStatus::InProgress => unreachable!("Brillig VM has not completed execution"),
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

    fn write_brillig_outputs(&mut self) -> Result<(), OpcodeResolutionError> {
        // Write VM execution results into the witness map
        for (i, output) in self.brillig.outputs.iter().enumerate() {
            let register_value = self.vm.get_registers().get(RegisterIndex::from(i));
            match output {
                BrilligOutputs::Simple(witness) => {
                    insert_value(witness, register_value.to_field(), self.witness)?;
                }
                BrilligOutputs::Array(witness_arr) => {
                    // Treat the register value as a pointer to memory
                    for (i, witness) in witness_arr.iter().enumerate() {
                        let value = &self.vm.get_memory()[register_value.to_usize() + i];
                        insert_value(witness, value.to_field(), self.witness)?;
                    }
                }
            }
        }
        Ok(())
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
