use std::collections::HashMap;

use acir::{
    brillig::{ForeignCallParam, ForeignCallResult, Opcode as BrilligOpcode},
    circuit::{
        brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::BlockId,
        ErrorSelector, OpcodeLocation, RawAssertionPayload, ResolvedAssertionPayload,
    },
    native_types::WitnessMap,
    AcirField,
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;
use brillig_vm::{BrilligProfilingSamples, FailureReason, MemoryValue, VMStatus, VM};
use serde::{Deserialize, Serialize};

use crate::{pwg::OpcodeNotSolvable, OpcodeResolutionError};

use super::{get_value, insert_value, memory_op::MemoryOpSolver};

#[derive(Debug)]
pub enum BrilligSolverStatus<F> {
    Finished,
    InProgress,
    ForeignCallWait(ForeignCallWaitInfo<F>),
}

pub struct BrilligSolver<'b, F, B: BlackBoxFunctionSolver<F>> {
    vm: VM<'b, F, B>,
    acir_index: usize,
    /// This id references which Brillig function within the main ACIR program we are solving.
    /// This is used for appropriately resolving errors as the ACIR program artifacts
    /// set up their Brillig debug metadata by function id.
    pub function_id: BrilligFunctionId,
}

impl<'b, B: BlackBoxFunctionSolver<F>, F: AcirField> BrilligSolver<'b, F, B> {
    /// Assigns the zero value to all outputs of the given [`Brillig`] bytecode.
    pub(super) fn zero_out_brillig_outputs(
        initial_witness: &mut WitnessMap<F>,
        outputs: &[BrilligOutputs],
    ) -> Result<(), OpcodeResolutionError<F>> {
        for output in outputs {
            match output {
                BrilligOutputs::Simple(witness) => {
                    insert_value(witness, F::zero(), initial_witness)?;
                }
                BrilligOutputs::Array(witness_arr) => {
                    for witness in witness_arr {
                        insert_value(witness, F::zero(), initial_witness)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Constructs a solver for a Brillig block given the bytecode and initial
    /// witness.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_call(
        initial_witness: &WitnessMap<F>,
        memory: &HashMap<BlockId, MemoryOpSolver<F>>,
        inputs: &'b [BrilligInputs<F>],
        brillig_bytecode: &'b [BrilligOpcode<F>],
        bb_solver: &'b B,
        acir_index: usize,
        brillig_function_id: BrilligFunctionId,
        profiling_active: bool,
    ) -> Result<Self, OpcodeResolutionError<F>> {
        let vm = Self::setup_brillig_vm(
            initial_witness,
            memory,
            inputs,
            brillig_bytecode,
            bb_solver,
            profiling_active,
        )?;
        Ok(Self { vm, acir_index, function_id: brillig_function_id })
    }

    fn setup_brillig_vm(
        initial_witness: &WitnessMap<F>,
        memory: &HashMap<BlockId, MemoryOpSolver<F>>,
        inputs: &[BrilligInputs<F>],
        brillig_bytecode: &'b [BrilligOpcode<F>],
        bb_solver: &'b B,
        profiling_active: bool,
    ) -> Result<VM<'b, F, B>, OpcodeResolutionError<F>> {
        // Set input values
        let mut calldata: Vec<F> = Vec::new();
        // Each input represents an expression or array of expressions to evaluate.
        // Iterate over each input and evaluate the expression(s) associated with it.
        // Push the results into memory.
        // If a certain expression is not solvable, we stall the ACVM and do not proceed with Brillig VM execution.
        for input in inputs {
            match input {
                BrilligInputs::Single(expr) => match get_value(expr, initial_witness) {
                    Ok(value) => calldata.push(value),
                    Err(_) => {
                        return Err(OpcodeResolutionError::OpcodeNotSolvable(
                            OpcodeNotSolvable::ExpressionHasTooManyUnknowns(expr.clone()),
                        ))
                    }
                },
                BrilligInputs::Array(expr_arr) => {
                    // Attempt to fetch all array input values
                    for expr in expr_arr.iter() {
                        match get_value(expr, initial_witness) {
                            Ok(value) => calldata.push(value),
                            Err(_) => {
                                return Err(OpcodeResolutionError::OpcodeNotSolvable(
                                    OpcodeNotSolvable::ExpressionHasTooManyUnknowns(expr.clone()),
                                ))
                            }
                        }
                    }
                }
                BrilligInputs::MemoryArray(block_id) => {
                    let memory_block = memory
                        .get(block_id)
                        .ok_or(OpcodeNotSolvable::MissingMemoryBlock(block_id.0))?;
                    for memory_index in 0..memory_block.block_len {
                        let memory_value = memory_block
                            .block_value
                            .get(&memory_index)
                            .expect("All memory is initialized on creation");
                        calldata.push(*memory_value);
                    }
                }
            }
        }

        // Instantiate a Brillig VM given the solved calldata
        // along with the Brillig bytecode.
        let vm = VM::new(calldata, brillig_bytecode, vec![], bb_solver, profiling_active);
        Ok(vm)
    }

    pub fn get_memory(&self) -> &[MemoryValue<F>] {
        self.vm.get_memory()
    }

    pub fn write_memory_at(&mut self, ptr: usize, value: MemoryValue<F>) {
        self.vm.write_memory_at(ptr, value);
    }

    pub fn get_call_stack(&self) -> Vec<usize> {
        self.vm.get_call_stack()
    }

    pub(crate) fn solve(&mut self) -> Result<BrilligSolverStatus<F>, OpcodeResolutionError<F>> {
        let status = self.vm.process_opcodes();
        self.handle_vm_status(status)
    }

    pub fn step(&mut self) -> Result<BrilligSolverStatus<F>, OpcodeResolutionError<F>> {
        let status = self.vm.process_opcode();
        self.handle_vm_status(status)
    }

    pub fn program_counter(&self) -> usize {
        self.vm.program_counter()
    }

    fn handle_vm_status(
        &self,
        vm_status: VMStatus<F>,
    ) -> Result<BrilligSolverStatus<F>, OpcodeResolutionError<F>> {
        // Check the status of the Brillig VM and return a resolution.
        // It may be finished, in-progress, failed, or may be waiting for results of a foreign call.
        // Return the "resolution" to the caller who may choose to make subsequent calls
        // (when it gets foreign call results for example).
        match vm_status {
            VMStatus::Finished { .. } => Ok(BrilligSolverStatus::Finished),
            VMStatus::InProgress => Ok(BrilligSolverStatus::InProgress),
            VMStatus::Failure { reason, call_stack } => {
                let call_stack = call_stack
                    .iter()
                    .map(|brillig_index| OpcodeLocation::Brillig {
                        acir_index: self.acir_index,
                        brillig_index: *brillig_index,
                    })
                    .collect();
                let payload = match reason {
                    FailureReason::RuntimeError { message } => {
                        Some(ResolvedAssertionPayload::String(message))
                    }
                    FailureReason::Trap { revert_data_offset, revert_data_size } => {
                        extract_failure_payload_from_memory(
                            self.vm.get_memory(),
                            revert_data_offset,
                            revert_data_size,
                        )
                    }
                };

                Err(OpcodeResolutionError::BrilligFunctionFailed {
                    function_id: self.function_id,
                    payload,
                    call_stack,
                })
            }
            VMStatus::ForeignCallWait { function, inputs } => {
                Ok(BrilligSolverStatus::ForeignCallWait(ForeignCallWaitInfo { function, inputs }))
            }
        }
    }

    pub(crate) fn finalize(
        self,
        witness: &mut WitnessMap<F>,
        outputs: &[BrilligOutputs],
    ) -> Result<(), OpcodeResolutionError<F>> {
        assert!(!self.vm.is_profiling_active(), "Expected VM profiling to not be active");
        self.finalize_inner(witness, outputs)
    }

    pub(crate) fn finalize_with_profiling(
        mut self,
        witness: &mut WitnessMap<F>,
        outputs: &[BrilligOutputs],
    ) -> Result<BrilligProfilingSamples, OpcodeResolutionError<F>> {
        assert!(self.vm.is_profiling_active(), "Expected VM profiling to be active");
        self.finalize_inner(witness, outputs)?;
        Ok(self.vm.take_profiling_samples())
    }

    fn finalize_inner(
        &self,
        witness: &mut WitnessMap<F>,
        outputs: &[BrilligOutputs],
    ) -> Result<(), OpcodeResolutionError<F>> {
        // Finish the Brillig execution by writing the outputs to the witness map
        let vm_status = self.vm.get_status();
        match vm_status {
            VMStatus::Finished { return_data_offset, return_data_size } => {
                self.write_brillig_outputs(witness, return_data_offset, return_data_size, outputs)?;
                Ok(())
            }
            _ => panic!("Brillig VM has not completed execution"),
        }
    }

    fn write_brillig_outputs(
        &self,
        witness_map: &mut WitnessMap<F>,
        return_data_offset: usize,
        return_data_size: usize,
        outputs: &[BrilligOutputs],
    ) -> Result<(), OpcodeResolutionError<F>> {
        // Write VM execution results into the witness map
        let memory = self.vm.get_memory();
        let mut current_ret_data_idx = return_data_offset;
        for output in outputs.iter() {
            match output {
                BrilligOutputs::Simple(witness) => {
                    insert_value(witness, memory[current_ret_data_idx].to_field(), witness_map)?;
                    current_ret_data_idx += 1;
                }
                BrilligOutputs::Array(witness_arr) => {
                    for witness in witness_arr.iter() {
                        let value = &memory[current_ret_data_idx];
                        insert_value(witness, value.to_field(), witness_map)?;
                        current_ret_data_idx += 1;
                    }
                }
            }
        }

        assert!(
            current_ret_data_idx == return_data_offset + return_data_size,
            "Brillig VM did not write the expected number of return values"
        );
        Ok(())
    }

    pub fn resolve_pending_foreign_call(&mut self, foreign_call_result: ForeignCallResult<F>) {
        match self.vm.get_status() {
            VMStatus::ForeignCallWait { .. } => self.vm.resolve_foreign_call(foreign_call_result),
            _ => unreachable!("Brillig VM is not waiting for a foreign call"),
        }
    }
}

/// Extracts a `ResolvedAssertionPayload` from a block of memory of a Brillig VM instance.
///
/// Returns `None` if the amount of memory requested is zero.
fn extract_failure_payload_from_memory<F: AcirField>(
    memory: &[MemoryValue<F>],
    revert_data_offset: usize,
    revert_data_size: usize,
) -> Option<ResolvedAssertionPayload<F>> {
    // Since noir can only revert with strings currently, we can parse return data as a string
    if revert_data_size == 0 {
        None
    } else {
        let mut revert_values_iter =
            memory[revert_data_offset..(revert_data_offset + revert_data_size)].iter();
        let error_selector = ErrorSelector::new(
            revert_values_iter
                .next()
                .copied()
                .expect("Incorrect revert data size")
                .try_into()
                .expect("Error selector is not u64"),
        );

        Some(ResolvedAssertionPayload::Raw(RawAssertionPayload {
            selector: error_selector,
            data: revert_values_iter.map(|value| value.to_field()).collect(),
        }))
    }
}

/// Encapsulates a request from a Brillig VM process that encounters a [foreign call opcode][acir::brillig_vm::Opcode::ForeignCall]
/// where the result of the foreign call has not yet been provided.
///
/// The caller must resolve this opcode externally based upon the information in the request.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ForeignCallWaitInfo<F> {
    /// An identifier interpreted by the caller process
    pub function: String,
    /// Resolved inputs to a foreign call computed in the previous steps of a Brillig VM process
    pub inputs: Vec<ForeignCallParam<F>>,
}
