#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The Brillig VM is a specialized VM which allows the [ACVM][acvm] to perform custom non-determinism.
//!
//! Brillig bytecode is distinct from regular [ACIR][acir] in that it does not generate constraints.
//!
//! [acir]: https://crates.io/crates/acir
//! [acvm]: https://crates.io/crates/acvm

use acir::AcirField;
use acir::brillig::{
    BinaryFieldOp, BinaryIntOp, ForeignCallParam, ForeignCallResult, IntegerBitSize, MemoryAddress,
    Opcode,
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;
use arithmetic::{BrilligArithmeticError, evaluate_binary_field_op, evaluate_binary_int_op};
use black_box::evaluate_black_box;

// Re-export `brillig`.
pub use acir::brillig;
use memory::MemoryTypeError;
pub use memory::{
    FREE_MEMORY_POINTER_ADDRESS, MEMORY_ADDRESSING_BIT_SIZE, Memory, MemoryValue,
    STACK_POINTER_ADDRESS, offsets,
};

pub use crate::fuzzing::BranchToFeatureMap;
use crate::fuzzing::FuzzingTrace;

mod arithmetic;
mod black_box;
mod cast;
mod foreign_call;
pub mod fuzzing;
mod memory;

/// The error call stack contains the opcode indexes of the call stack at the time of failure, plus the index of the opcode that failed.
pub type ErrorCallStack = Vec<usize>;

/// Represents the reason why the Brillig VM failed during execution.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FailureReason {
    /// A trap was encountered, which indicates an explicit failure from within the VM program.
    ///
    /// A trap is triggered explicitly by the [trap opcode][Opcode::Trap].
    /// The revert data is referenced by the offset and size in the VM memory.
    Trap {
        /// Offset in memory where the revert data begins.
        revert_data_offset: u32,
        /// Size of the revert data.
        revert_data_size: u32,
    },
    /// A runtime failure during execution.
    /// This error is triggered by all opcodes aside the [trap opcode][Opcode::Trap].
    /// For example, a [binary operation][Opcode::BinaryIntOp] can trigger a division by zero error.
    RuntimeError { message: String },
}

/// Represents the current execution status of the Brillig VM.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VMStatus<F> {
    /// The VM has completed execution successfully.
    /// The output of the program is stored in the VM memory and can be accessed via the provided offset and size.
    Finished {
        /// Offset in memory where the return data begins.
        return_data_offset: u32,
        /// Size of the return data.
        return_data_size: u32,
    },
    /// The VM is still in progress and has not yet completed execution.
    /// This is used when simulating execution.
    InProgress,
    /// The VM encountered a failure and halted execution.
    Failure {
        /// The reason for the failure.
        reason: FailureReason,
        /// The call stack at the time the failure occurred, useful for debugging nested calls.
        call_stack: ErrorCallStack,
    },
    /// The VM process is not solvable as a [foreign call][Opcode::ForeignCall] has been
    /// reached where the outputs are yet to be resolved.
    ///
    /// The caller should interpret the information returned to compute a [ForeignCallResult]
    /// and update the Brillig process. The VM can then be restarted to fully solve the previously
    /// unresolved foreign call as well as the remaining Brillig opcodes.
    ForeignCallWait {
        /// Interpreted by simulator context.
        function: String,
        /// Input values.
        /// Each input can be either a single value or an array of values read from a memory pointer.
        inputs: Vec<ForeignCallParam<F>>,
    },
}

/// The position of an opcode that is currently being executed in the bytecode.
pub type OpcodePosition = usize;

/// The position of the next opcode that will be executed in the bytecode,
/// or an id of a specific state produced by the opcode.
pub type NextOpcodePositionOrState = usize;

/// A sample for an executed opcode.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BrilligProfilingSample {
    /// The call stack when processing a given opcode.
    pub call_stack: Vec<usize>,
}

/// All samples for each opcode that was executed.
pub type BrilligProfilingSamples = Vec<BrilligProfilingSample>;

#[derive(Debug, PartialEq, Eq, Clone)]
/// VM encapsulates the state of the Brillig VM during execution.
pub struct VM<'a, F, B: BlackBoxFunctionSolver<F>> {
    /// Calldata to the brillig function.
    calldata: Vec<F>,
    /// Instruction pointer.
    program_counter: usize,
    /// A counter maintained throughout a Brillig process that determines
    /// whether the caller has resolved the results of a [foreign call][Opcode::ForeignCall].
    ///
    /// Incremented when the results of a foreign call have been processed and the output
    /// values were written to memory.
    ///
    /// * When the counter is less than the length of the results, it indicates that we have
    ///   unprocessed responses returned from the external foreign call handler.
    foreign_call_counter: usize,
    /// Accumulates the outputs of all foreign calls during a Brillig process.
    /// The list is appended onto by the caller upon reaching a [VMStatus::ForeignCallWait].
    foreign_call_results: Vec<ForeignCallResult<F>>,
    /// Executable opcodes.
    bytecode: &'a [Opcode<F>],
    /// Status of the VM.
    status: VMStatus<F>,
    /// Memory of the VM.
    memory: Memory<F>,
    /// Call stack.
    call_stack: Vec<usize>,
    /// The solver for blackbox functions.
    black_box_solver: &'a B,
    // Flag that determines whether we want to profile VM.
    profiling_active: bool,
    // Samples for profiling the VM execution.
    profiling_samples: BrilligProfilingSamples,

    /// Fuzzing trace structure.
    /// If the field is `None` then fuzzing is inactive.
    fuzzing_trace: Option<FuzzingTrace>,
}

impl<'a, F: AcirField, B: BlackBoxFunctionSolver<F>> VM<'a, F, B> {
    /// Constructs a new VM instance.
    pub fn new(
        calldata: Vec<F>,
        bytecode: &'a [Opcode<F>],
        black_box_solver: &'a B,
        profiling_active: bool,
        with_branch_to_feature_map: Option<&BranchToFeatureMap>,
    ) -> Self {
        let fuzzing_trace = with_branch_to_feature_map.cloned().map(FuzzingTrace::new);

        Self {
            calldata,
            program_counter: 0,
            foreign_call_counter: 0,
            foreign_call_results: Vec::new(),
            bytecode,
            status: VMStatus::InProgress,
            memory: Memory::default(),
            call_stack: Vec::new(),
            black_box_solver,
            profiling_active,
            profiling_samples: Vec::with_capacity(bytecode.len()),
            fuzzing_trace,
        }
    }

    pub fn is_profiling_active(&self) -> bool {
        self.profiling_active
    }

    pub fn is_fuzzing_active(&self) -> bool {
        self.fuzzing_trace.is_some()
    }

    pub fn take_profiling_samples(&mut self) -> BrilligProfilingSamples {
        std::mem::take(&mut self.profiling_samples)
    }

    /// Updates the current status of the VM.
    /// Returns the given status.
    fn status(&mut self, status: VMStatus<F>) -> &VMStatus<F> {
        self.status = status.clone();
        &self.status
    }

    pub fn get_status(&self) -> VMStatus<F> {
        self.status.clone()
    }

    /// Sets the current status of the VM to Finished (completed execution).
    fn finish(&mut self, return_data_offset: u32, return_data_size: u32) -> &VMStatus<F> {
        self.status(VMStatus::Finished { return_data_offset, return_data_size })
    }

    /// Check whether the latest foreign call result is available yet.
    fn has_unprocessed_foreign_call_result(&self) -> bool {
        self.foreign_call_counter < self.foreign_call_results.len()
    }

    /// Provide the results of a Foreign Call to the VM
    /// and resume execution of the VM.
    pub fn resolve_foreign_call(&mut self, foreign_call_result: ForeignCallResult<F>) {
        if self.has_unprocessed_foreign_call_result() {
            panic!("No unresolved foreign calls; the previous results haven't been processed yet");
        }
        self.foreign_call_results.push(foreign_call_result);
        self.status(VMStatus::InProgress);
    }

    /// Sets the current status of the VM to `Failure`,
    /// indicating that the VM encountered a `Trap` Opcode.
    fn trap(&mut self, revert_data_offset: u32, revert_data_size: u32) -> &VMStatus<F> {
        self.status(VMStatus::Failure {
            call_stack: self.get_call_stack(),
            reason: FailureReason::Trap { revert_data_offset, revert_data_size },
        })
    }

    /// Sets the current status of the VM to `Failure`,
    /// indicating that the VM encountered an invalid state.
    fn fail(&mut self, message: String) -> &VMStatus<F> {
        self.status(VMStatus::Failure {
            call_stack: self.get_call_stack(),
            reason: FailureReason::RuntimeError { message },
        })
    }

    /// Process opcodes in a loop until a status of `Finished`,
    /// `Failure` or `ForeignCallWait` is encountered.
    pub fn process_opcodes(&mut self) -> VMStatus<F> {
        while !matches!(
            self.process_opcode(),
            VMStatus::Finished { .. } | VMStatus::Failure { .. } | VMStatus::ForeignCallWait { .. }
        ) {}
        self.status.clone()
    }

    /// Read memory slots.
    ///
    /// Used by the debugger to inspect the contents of the memory.
    pub fn get_memory(&self) -> &[MemoryValue<F>] {
        self.memory.values()
    }

    /// Take all the contents of the memory, leaving it empty.
    ///
    /// Used only for testing purposes.
    pub fn take_memory(mut self) -> Memory<F> {
        std::mem::take(&mut self.memory)
    }

    pub fn foreign_call_counter(&self) -> usize {
        self.foreign_call_counter
    }

    /// Write a numeric value to direct memory slot.
    ///
    /// Used by the debugger to alter memory.
    pub fn write_memory_at(&mut self, ptr: usize, value: MemoryValue<F>) {
        self.memory.write(MemoryAddress::direct(ptr), value);
    }

    /// Returns the VM's current call stack, including the actual program
    /// counter in the last position of the returned vector.
    pub fn get_call_stack(&self) -> Vec<usize> {
        let mut call_stack = self.get_call_stack_no_current_counter();
        call_stack.push(self.program_counter);
        call_stack
    }

    /// Returns the VM's call stack, but unlike [Self::get_call_stack] without the attaching
    /// the program counter in the last position of the returned vector.
    /// This is meant only for fetching the call stack after execution has completed.
    pub fn get_call_stack_no_current_counter(&self) -> Vec<usize> {
        self.call_stack.clone()
    }

    /// Process a single opcode and modify the program counter.
    pub fn process_opcode(&mut self) -> &VMStatus<F> {
        if self.profiling_active {
            let call_stack: Vec<usize> = self.get_call_stack();
            self.profiling_samples.push(BrilligProfilingSample { call_stack });
        }

        self.process_opcode_internal()
    }

    pub fn get_fuzzing_trace(&self) -> Vec<u32> {
        self.fuzzing_trace.as_ref().map(|trace| trace.get_trace()).unwrap_or_default()
    }

    /// Execute a single opcode:
    /// 1. Retrieve the current opcode using the program counter
    /// 2. Execute the opcode.
    ///    - For instance a binary 'result = lhs+rhs' opcode will read the VM memory at the 'lhs' and 'rhs' addresses,
    ///      compute the sum and write it to the 'result' memory address.
    /// 3. Update the program counter, usually by incrementing it.
    ///
    /// - Control flow opcodes jump around the bytecode by setting the program counter.
    /// - Foreign call opcodes pause the VM until the foreign call results are available.
    /// - Function call opcodes backup the current program counter into the call stack and jump to the function entry point.
    ///   The stack frame for function calls is handled during codegen.
    fn process_opcode_internal(&mut self) -> &VMStatus<F> {
        let opcode = &self.bytecode[self.program_counter];
        match opcode {
            Opcode::BinaryFieldOp { op, lhs, rhs, destination: result } => {
                if let Err(error) = self.process_binary_field_op(*op, *lhs, *rhs, *result) {
                    self.fail(error.to_string())
                } else {
                    self.increment_program_counter()
                }
            }
            Opcode::BinaryIntOp { op, bit_size, lhs, rhs, destination: result } => {
                match self.process_free_memory_op(*op, *bit_size, *lhs, *rhs, *result) {
                    Err(error) => return self.fail(error),
                    Ok(true) => return self.increment_program_counter(),
                    Ok(false) => {
                        // Not a free memory op, carry on as a regular binary operation.
                    }
                };
                if let Err(error) = self.process_binary_int_op(*op, *bit_size, *lhs, *rhs, *result)
                {
                    self.fail(error.to_string())
                } else {
                    self.increment_program_counter()
                }
            }
            Opcode::Not { destination, source, bit_size } => {
                if let Err(error) = self.process_not(*source, *destination, *bit_size) {
                    self.fail(error.to_string())
                } else {
                    self.increment_program_counter()
                }
            }
            Opcode::Cast { destination, source, bit_size } => {
                let source_value = self.memory.read(*source);
                let casted_value = cast::cast(source_value, *bit_size);
                self.memory.write(*destination, casted_value);
                self.increment_program_counter()
            }
            Opcode::Jump { location: destination } => self.set_program_counter(*destination),
            Opcode::JumpIf { condition, location: destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true
                let condition_value = self.memory.read(*condition);
                let condition_value = match condition_value.expect_u1() {
                    Err(error) => {
                        return self.fail(format!("condition value is not a boolean: {error}"));
                    }
                    Ok(cond) => cond,
                };
                if condition_value {
                    self.fuzzing_trace_branching(*destination);
                    self.set_program_counter(*destination)
                } else {
                    self.fuzzing_trace_branching(self.program_counter + 1);
                    self.increment_program_counter()
                }
            }
            Opcode::CalldataCopy { destination_address, size_address, offset_address } => {
                let size = self.memory.read(*size_address).to_usize();
                let offset = self.memory.read(*offset_address).to_usize();
                let values: Vec<_> = self.calldata[offset..(offset + size)]
                    .iter()
                    .map(|value| MemoryValue::new_field(*value))
                    .collect();
                self.memory.write_slice(*destination_address, &values);
                self.increment_program_counter()
            }
            Opcode::Return => {
                if let Some(return_location) = self.call_stack.pop() {
                    self.set_program_counter(return_location + 1)
                } else {
                    self.fail("return opcode hit, but callstack already empty".to_string())
                }
            }
            Opcode::ForeignCall {
                function,
                destinations,
                destination_value_types,
                inputs,
                input_value_types,
            } => self.process_foreign_call(
                function,
                destinations,
                destination_value_types,
                inputs,
                input_value_types,
            ),
            Opcode::Mov { destination: destination_address, source: source_address } => {
                let source_value = self.memory.read(*source_address);
                self.memory.write(*destination_address, source_value);
                self.increment_program_counter()
            }
            Opcode::ConditionalMov { destination, source_a, source_b, condition } => {
                let condition_value = self.memory.read(*condition);

                let condition_value = match condition_value.expect_u1() {
                    Err(error) => {
                        return self.fail(format!("condition value is not a boolean: {error}"));
                    }
                    Ok(cond) => cond,
                };
                if condition_value {
                    self.memory.write(*destination, self.memory.read(*source_a));
                } else {
                    self.memory.write(*destination, self.memory.read(*source_b));
                }
                self.fuzzing_trace_conditional_mov(condition_value);
                self.increment_program_counter()
            }
            Opcode::Trap { revert_data } => {
                let revert_data_size = self.memory.read(revert_data.size).to_usize();
                if revert_data_size > 0 {
                    self.trap(
                        self.memory
                            .read_ref(revert_data.pointer)
                            .unwrap_direct()
                            .try_into()
                            .expect("Failed conversion from usize to u32"),
                        revert_data_size.try_into().expect("Failed conversion from usize to u32"),
                    )
                } else {
                    self.trap(0, 0)
                }
            }
            Opcode::Stop { return_data } => {
                let return_data_size = self.memory.read(return_data.size).to_usize();
                if return_data_size > 0 {
                    self.finish(
                        self.memory
                            .read_ref(return_data.pointer)
                            .unwrap_direct()
                            .try_into()
                            .expect("Failed conversion from usize to u32"),
                        return_data_size.try_into().expect("Failed conversion from usize to u32"),
                    )
                } else {
                    self.finish(0, 0)
                }
            }
            Opcode::Load { destination, source_pointer } => {
                // Convert the source_pointer to an address
                let source = self.memory.read_ref(*source_pointer);
                // Use the source address to lookup the value in memory
                let value = self.memory.read(source);
                self.memory.write(*destination, value);
                self.increment_program_counter()
            }
            Opcode::Store { destination_pointer, source: source_address } => {
                // Convert the destination_pointer to an address
                let destination = self.memory.read_ref(*destination_pointer);
                // Read the value at the source address
                let value = self.memory.read(*source_address);
                // Use the destination address to set the value in memory
                self.memory.write(destination, value);
                self.increment_program_counter()
            }
            Opcode::Call { location } => {
                // Push the return location to the call stack.
                self.call_stack.push(self.program_counter);
                self.set_program_counter(*location)
            }
            Opcode::Const { destination, value, bit_size } => {
                // Consts are not checked in runtime to fit in the bit size, since they can safely be checked statically.
                self.memory.write(*destination, MemoryValue::new_from_field(*value, *bit_size));
                self.increment_program_counter()
            }
            Opcode::IndirectConst { destination_pointer, bit_size, value } => {
                // Convert the destination_pointer to an address
                let destination = self.memory.read_ref(*destination_pointer);
                // Use the destination address to set the value in memory
                self.memory.write(destination, MemoryValue::new_from_field(*value, *bit_size));
                self.increment_program_counter()
            }
            Opcode::BlackBox(black_box_op) => {
                if let Err(e) =
                    evaluate_black_box(black_box_op, self.black_box_solver, &mut self.memory)
                {
                    self.fail(e.to_string())
                } else {
                    self.increment_program_counter()
                }
            }
        }
    }

    /// Returns the current value of the program counter.
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    /// Increments the program counter by 1.
    fn increment_program_counter(&mut self) -> &VMStatus<F> {
        self.set_program_counter(self.program_counter + 1)
    }

    /// Sets the program counter to `value`.
    /// If the program counter no longer points to an opcode
    /// in the bytecode, then the VMStatus reports `Finished`.
    fn set_program_counter(&mut self, value: usize) -> &VMStatus<F> {
        assert!(self.program_counter < self.bytecode.len());
        self.program_counter = value;
        if self.program_counter >= self.bytecode.len() {
            self.status = VMStatus::Finished { return_data_offset: 0, return_data_size: 0 };
        }
        &self.status
    }

    /// Process a binary field operation.
    /// This method will not modify the program counter.
    fn process_binary_field_op(
        &mut self,
        op: BinaryFieldOp,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
    ) -> Result<(), BrilligArithmeticError> {
        let lhs_value = self.memory.read(lhs);
        let rhs_value = self.memory.read(rhs);

        let result_value = evaluate_binary_field_op(&op, lhs_value, rhs_value)?;
        self.memory.write(result, result_value);
        self.fuzzing_trace_binary_field_op_comparison(&op, lhs_value, rhs_value, result_value);
        Ok(())
    }

    /// Process a binary integer operation.
    /// This method will not modify the program counter.
    fn process_binary_int_op(
        &mut self,
        op: BinaryIntOp,
        bit_size: IntegerBitSize,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
    ) -> Result<(), BrilligArithmeticError> {
        let lhs_value = self.memory.read(lhs);
        let rhs_value = self.memory.read(rhs);

        let result_value = evaluate_binary_int_op(&op, lhs_value, rhs_value, bit_size)?;
        self.memory.write(result, result_value);
        self.fuzzing_trace_binary_int_op_comparison(&op, lhs_value, rhs_value, result_value);
        Ok(())
    }

    /// Special handling for the increment of the _free memory pointer_.
    ///
    /// Binary operations in Brillig wrap around on overflow,
    /// but there are usually other instruction in the SSA itself
    /// to make sure the circuit fails when overflows occur.
    ///
    /// This is not the case for the _free memory pointer_ itself, however,
    /// which only exists in Brillig, and points at the first free memory
    /// slot on the heap where nothing has been allocated yet. If we allowed
    /// it to wrap around during an overflowing increment, then we could end
    /// up overwriting parts of the memory reserved for globals, the stack,
    /// or other values on the heap.
    ///
    /// This special handling is not adopted by the AVM. Still, if it fails
    /// here, at least we have a way to detect this unlikely edge case,
    /// rather than go into undefined behavior by corrupting memory.
    /// Detecting overflows with additional bytecode would be an overkill.
    /// Perhaps in the future the AVM will offer checked operations instead.
    ///
    /// Returns:
    /// * `Ok(false)` if it's not a _free memory pointer_ increase
    /// * `Ok(true)` if the operation was handled
    /// * `Err(RuntimeError("Out of memory"))` if there was an overflow
    fn process_free_memory_op(
        &mut self,
        op: BinaryIntOp,
        bit_size: IntegerBitSize,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
    ) -> Result<bool, String> {
        if result != FREE_MEMORY_POINTER_ADDRESS
            || op != BinaryIntOp::Add
            || bit_size != MEMORY_ADDRESSING_BIT_SIZE
        {
            return Ok(false);
        }

        let lhs_value = self.memory.read(lhs);
        let rhs_value = self.memory.read(rhs);

        let MemoryValue::U32(lhs_value) = lhs_value else {
            return Ok(false);
        };
        let MemoryValue::U32(rhs_value) = rhs_value else {
            return Ok(false);
        };
        let Some(result_value) = lhs_value.checked_add(rhs_value) else {
            return Err("Out of memory".to_string());
        };

        self.memory.write(result, result_value.into());

        Ok(true)
    }

    /// Process a unary negation operation.
    ///
    /// It returns `MemoryTypeError` if the value type does not match the type
    /// indicated by `op_bit_size`.
    fn process_not(
        &mut self,
        source: MemoryAddress,
        destination: MemoryAddress,
        op_bit_size: IntegerBitSize,
    ) -> Result<(), MemoryTypeError> {
        let value = self.memory.read(source);

        let negated_value = match op_bit_size {
            IntegerBitSize::U1 => MemoryValue::U1(!value.expect_u1()?),
            IntegerBitSize::U8 => MemoryValue::U8(!value.expect_u8()?),
            IntegerBitSize::U16 => MemoryValue::U16(!value.expect_u16()?),
            IntegerBitSize::U32 => MemoryValue::U32(!value.expect_u32()?),
            IntegerBitSize::U64 => MemoryValue::U64(!value.expect_u64()?),
            IntegerBitSize::U128 => MemoryValue::U128(!value.expect_u128()?),
        };
        self.memory.write(destination, negated_value);
        Ok(())
    }
}
