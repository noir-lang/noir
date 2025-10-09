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
pub use memory::{MEMORY_ADDRESSING_BIT_SIZE, Memory, MemoryValue};

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
        revert_data_offset: usize,
        /// Size of the revert data.
        revert_data_size: usize,
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
        return_data_offset: usize,
        /// Size of the return data.
        return_data_size: usize,
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
        /// Interpreted by simulator context
        function: String,
        /// Input values
        /// Each input is a list of values as an input can be either a single value or a memory pointer
        inputs: Vec<ForeignCallParam<F>>,
    },
}

impl<F> VMStatus<F> {
    pub fn is_finished(&self) -> bool {
        matches!(self, VMStatus::Finished { .. })
    }
}

/// All samples for each opcode that was executed
pub type BrilligProfilingSamples = Vec<BrilligProfilingSample>;

/// The position of an opcode that is currently being executed in the bytecode
pub type OpcodePosition = usize;

/// The position of the next opcode that will be executed in the bytecode or an id of a specific state produced by the opcode
pub type NextOpcodePositionOrState = usize;

/// A sample for an executed opcode
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BrilligProfilingSample {
    /// The call stack when processing a given opcode.
    pub call_stack: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// VM encapsulates the state of the Brillig VM during execution.
pub struct VM<'a, F, B: BlackBoxFunctionSolver<F>> {
    /// Calldata to the brillig function
    calldata: Vec<F>,
    /// Instruction pointer
    program_counter: usize,
    /// A counter maintained throughout a Brillig process that determines
    /// whether the caller has resolved the results of a [foreign call][Opcode::ForeignCall].
    foreign_call_counter: usize,
    /// Represents the outputs of all foreign calls during a Brillig process
    /// List is appended onto by the caller upon reaching a [VMStatus::ForeignCallWait]
    foreign_call_results: Vec<ForeignCallResult<F>>,
    /// Executable opcodes
    bytecode: &'a [Opcode<F>],
    /// Status of the VM
    status: VMStatus<F>,
    /// Memory of the VM
    memory: Memory<'a, F>,
    /// Call stack
    call_stack: Vec<usize>,
    /// The solver for blackbox functions
    black_box_solver: &'a B,
    // Flag that determines whether we want to profile VM.
    profiling_active: bool,
    // Samples for profiling the VM execution.
    profiling_samples: BrilligProfilingSamples,

    /// Fuzzing trace structure
    /// If the field is `None` then fuzzing is inactive
    fuzzing_trace: Option<FuzzingTrace>,
}

impl<'a, F: AcirField, B: BlackBoxFunctionSolver<F>> VM<'a, F, B> {
    /// Constructs a new VM instance
    pub fn new(
        calldata: Vec<F>,
        bytecode: &'a [Opcode<F>],
        global_memory: &'a [MemoryValue<F>],
        black_box_solver: &'a B,
        profiling_active: bool,
        with_branch_to_feature_map: Option<&BranchToFeatureMap>,
    ) -> Self {
        let fuzzing_trace = with_branch_to_feature_map.map(|map| FuzzingTrace::new(map.clone()));

        Self {
            calldata,
            program_counter: 0,
            foreign_call_counter: 0,
            foreign_call_results: Vec::new(),
            bytecode,
            status: VMStatus::InProgress,
            memory: Memory::with_globals(global_memory),
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
    fn status(&mut self, status: VMStatus<F>) -> VMStatus<F> {
        self.status = status.clone();
        status
    }

    pub fn get_status(&self) -> VMStatus<F> {
        self.status.clone()
    }

    /// Sets the current status of the VM to Finished (completed execution).
    fn finish(&mut self, return_data_offset: usize, return_data_size: usize) -> VMStatus<F> {
        self.status(VMStatus::Finished { return_data_offset, return_data_size })
    }

    /// Provide the results of a Foreign Call to the VM
    /// and resume execution of the VM.
    pub fn resolve_foreign_call(&mut self, foreign_call_result: ForeignCallResult<F>) {
        if self.foreign_call_counter < self.foreign_call_results.len() {
            panic!("No unresolved foreign calls");
        }
        self.foreign_call_results.push(foreign_call_result);
        self.status(VMStatus::InProgress);
    }

    fn get_error_stack(&self) -> Vec<usize> {
        let mut error_stack: Vec<_> = self.call_stack.clone();
        error_stack.push(self.program_counter);
        error_stack
    }

    /// Sets the current status of the VM to `fail`.
    /// Indicating that the VM encountered a `Trap` Opcode
    /// or an invalid state.
    fn trap(&mut self, revert_data_offset: usize, revert_data_size: usize) -> VMStatus<F> {
        self.status(VMStatus::Failure {
            call_stack: self.get_error_stack(),
            reason: FailureReason::Trap { revert_data_offset, revert_data_size },
        });
        self.status.clone()
    }

    fn fail(&mut self, message: String) -> VMStatus<F> {
        self.status(VMStatus::Failure {
            call_stack: self.get_error_stack(),
            reason: FailureReason::RuntimeError { message },
        });
        self.status.clone()
    }

    /// Loop over the bytecode and update the program counter
    pub fn process_opcodes(&mut self) -> VMStatus<F> {
        while !matches!(
            self.process_opcode(),
            VMStatus::Finished { .. } | VMStatus::Failure { .. } | VMStatus::ForeignCallWait { .. }
        ) {}
        self.status.clone()
    }

    pub fn get_memory(&self) -> &[MemoryValue<F>] {
        self.memory.values()
    }

    pub fn take_memory(mut self) -> Memory<'a, F> {
        std::mem::take(&mut self.memory)
    }

    pub fn foreign_call_counter(&self) -> usize {
        self.foreign_call_counter
    }

    pub fn write_memory_at(&mut self, ptr: usize, value: MemoryValue<F>) {
        self.memory.write(MemoryAddress::direct(ptr), value);
    }

    /// Returns the VM's current call stack, including the actual program
    /// counter in the last position of the returned vector.
    pub fn get_call_stack(&self) -> Vec<usize> {
        self.call_stack.iter().copied().chain(std::iter::once(self.program_counter)).collect()
    }

    /// Process a single opcode and modify the program counter.
    pub fn process_opcode(&mut self) -> VMStatus<F> {
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
    ///    - For instance a binary 'result = lhs+rhs' opcode will read the VM memory at the lhs and rhs addresses,
    ///      compute the sum and write it to the 'result' memory address.
    /// 3. Update the program counter, usually by incrementing it.
    ///
    /// - Control flow opcodes jump around the bytecode by setting the program counter.
    /// - Foreign call opcodes pause the VM until the foreign call results are available
    /// - Function call opcodes backup the current program counter into the call stack and jump to the function entry point.
    ///   The stack frame for function calls is handled during codegen.
    fn process_opcode_internal(&mut self) -> VMStatus<F> {
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
            Opcode::Cast { destination: destination_address, source: source_address, bit_size } => {
                let source_value = self.memory.read(*source_address);
                let casted_value = cast::cast(source_value, *bit_size);
                self.memory.write(*destination_address, casted_value);
                self.increment_program_counter()
            }
            Opcode::Jump { location: destination } => self.set_program_counter(*destination),
            Opcode::JumpIf { condition, location: destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true
                let condition_value = self.memory.read(*condition);
                if condition_value.expect_u1().expect("condition value is not a boolean") {
                    self.fuzzing_trace_branching(*destination);
                    return self.set_program_counter(*destination);
                }
                self.fuzzing_trace_branching(self.program_counter + 1);
                self.increment_program_counter()
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

                let condition_value_bool =
                    condition_value.expect_u1().expect("condition value is not a boolean");
                if condition_value_bool {
                    self.memory.write(*destination, self.memory.read(*source_a));
                } else {
                    self.memory.write(*destination, self.memory.read(*source_b));
                }
                self.fuzzing_trace_conditional_mov(condition_value_bool);
                self.increment_program_counter()
            }
            Opcode::Trap { revert_data } => {
                let revert_data_size = self.memory.read(revert_data.size).to_usize();
                if revert_data_size > 0 {
                    self.trap(
                        self.memory.read_ref(revert_data.pointer).unwrap_direct(),
                        revert_data_size,
                    )
                } else {
                    self.trap(0, 0)
                }
            }
            Opcode::Stop { return_data } => {
                let return_data_size = self.memory.read(return_data.size).to_usize();
                if return_data_size > 0 {
                    self.finish(
                        self.memory.read_ref(return_data.pointer).unwrap_direct(),
                        return_data_size,
                    )
                } else {
                    self.finish(0, 0)
                }
            }
            Opcode::Load { destination: destination_address, source_pointer } => {
                // Convert our source_pointer to an address
                let source = self.memory.read_ref(*source_pointer);
                // Use our usize source index to lookup the value in memory
                let value = self.memory.read(source);
                self.memory.write(*destination_address, value);
                self.increment_program_counter()
            }
            Opcode::Store { destination_pointer, source: source_address } => {
                // Convert our destination_pointer to an address
                let destination = self.memory.read_ref(*destination_pointer);
                // Use our usize destination index to set the value in memory
                let value = self.memory.read(*source_address);
                self.memory.write(destination, value);
                self.increment_program_counter()
            }
            Opcode::Call { location } => {
                // Push a return location
                self.call_stack.push(self.program_counter);
                self.set_program_counter(*location)
            }
            Opcode::Const { destination, value, bit_size } => {
                // Consts are not checked in runtime to fit in the bit size, since they can safely be checked statically.
                self.memory.write(*destination, MemoryValue::new_from_field(*value, *bit_size));
                self.increment_program_counter()
            }
            Opcode::IndirectConst { destination_pointer, bit_size, value } => {
                // Convert our destination_pointer to an address
                let destination = self.memory.read_ref(*destination_pointer);
                // Use our usize destination index to set the value in memory
                self.memory.write(destination, MemoryValue::new_from_field(*value, *bit_size));
                self.increment_program_counter()
            }
            Opcode::BlackBox(black_box_op) => {
                match evaluate_black_box(black_box_op, self.black_box_solver, &mut self.memory) {
                    Ok(()) => self.increment_program_counter(),
                    Err(e) => self.fail(e.to_string()),
                }
            }
        }
    }

    /// Returns the current value of the program counter.
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    /// Increments the program counter by 1.
    fn increment_program_counter(&mut self) -> VMStatus<F> {
        self.set_program_counter(self.program_counter + 1)
    }

    /// Increments the program counter by `value`.
    /// If the program counter no longer points to an opcode
    /// in the bytecode, then the VMStatus reports halted.
    fn set_program_counter(&mut self, value: usize) -> VMStatus<F> {
        assert!(self.program_counter < self.bytecode.len());
        self.program_counter = value;
        if self.program_counter >= self.bytecode.len() {
            self.status = VMStatus::Finished { return_data_offset: 0, return_data_size: 0 };
        }
        self.status.clone()
    }

    /// Process a binary operation.
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

    /// Process a binary operation.
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

#[cfg(test)]
mod tests {
    use crate::memory::MEMORY_ADDRESSING_BIT_SIZE;
    use acir::{
        AcirField, FieldElement,
        brillig::{BitSize, HeapVector},
    };
    use acvm_blackbox_solver::StubbedBlackBoxSolver;

    use super::*;

    #[test]
    fn add_single_step_smoke() {
        let calldata = vec![];

        let opcodes = [Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(27u128),
        }];

        // Start VM
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, &opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // The address at index `2` should have the value of 3 since we had an
        // add opcode
        let VM { memory, .. } = vm;
        let output_value = memory.read(MemoryAddress::direct(0));

        assert_eq!(output_value.to_field(), FieldElement::from(27u128));
    }

    #[test]
    fn jmpif_opcode() {
        let mut calldata: Vec<FieldElement> = vec![];

        let lhs = {
            calldata.push(2u128.into());
            MemoryAddress::direct(calldata.len() - 1)
        };

        let rhs = {
            calldata.push(2u128.into());
            MemoryAddress::direct(calldata.len() - 1)
        };

        let destination = MemoryAddress::direct(calldata.len());

        let opcodes = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(2u64),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
            Opcode::BinaryFieldOp { destination, op: BinaryFieldOp::Equals, lhs, rhs },
            Opcode::Jump { location: 5 },
            Opcode::JumpIf { condition: destination, location: 6 },
        ];

        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, &opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.memory.read(destination);
        assert_eq!(output_cmp_value.to_field(), true.into());

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });
    }

    #[test]
    fn cast_opcode() {
        let calldata: Vec<FieldElement> = vec![((2_u128.pow(32)) - 1).into()];

        let value_address = MemoryAddress::direct(1);
        let one_usize = MemoryAddress::direct(2);
        let zero_usize = MemoryAddress::direct(3);

        let opcodes = &[
            Opcode::Const {
                destination: one_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(1u64),
            },
            Opcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: value_address,
                size_address: one_usize,
                offset_address: zero_usize,
            },
            Opcode::Cast {
                destination: value_address,
                source: value_address,
                bit_size: BitSize::Integer(IntegerBitSize::U8),
            },
            Opcode::Stop {
                return_data: HeapVector {
                    pointer: one_usize, // Since value_address is direct(1)
                    size: one_usize,
                },
            },
        ];
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 1, return_data_size: 1 });

        let VM { memory, .. } = vm;

        let casted_value = memory.read(MemoryAddress::direct(1));
        assert_eq!(casted_value.to_field(), (2_u128.pow(8) - 1).into());
    }

    #[test]
    fn not_opcode() {
        let calldata: Vec<FieldElement> = vec![(1_usize).into()];

        let value_address = MemoryAddress::direct(1);
        let one_usize = MemoryAddress::direct(2);
        let zero_usize = MemoryAddress::direct(3);

        let opcodes = &[
            Opcode::Const {
                destination: one_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(1u64),
            },
            Opcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: value_address,
                size_address: one_usize,
                offset_address: zero_usize,
            },
            Opcode::Cast {
                destination: value_address,
                source: value_address,
                bit_size: BitSize::Integer(IntegerBitSize::U128),
            },
            Opcode::Not {
                destination: value_address,
                source: value_address,
                bit_size: IntegerBitSize::U128,
            },
            Opcode::Stop {
                return_data: HeapVector {
                    pointer: one_usize, // Since value_address is direct(1)
                    size: one_usize,
                },
            },
        ];
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 1, return_data_size: 1 });

        let VM { memory, .. } = vm;

        let MemoryValue::U128(negated_value) = memory.read(MemoryAddress::direct(1)) else {
            panic!("Expected integer as the output of Not");
        };
        assert_eq!(negated_value, !1_u128);
    }

    #[test]
    fn mov_opcode() {
        let calldata: Vec<FieldElement> = vec![(1u128).into(), (2u128).into(), (3u128).into()];

        let opcodes = &[
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3u64),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
            Opcode::Mov { destination: MemoryAddress::direct(2), source: MemoryAddress::direct(0) },
        ];
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();

        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let VM { memory, .. } = vm;

        let destination_value = memory.read(MemoryAddress::direct(2));
        assert_eq!(destination_value.to_field(), (1u128).into());

        let source_value = memory.read(MemoryAddress::direct(0));
        assert_eq!(source_value.to_field(), (1u128).into());
    }

    #[test]
    fn cmov_opcode() {
        let calldata: Vec<FieldElement> =
            vec![(0u128).into(), (1u128).into(), (2u128).into(), (3u128).into()];

        let opcodes = &[
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(4u64),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
            Opcode::Cast {
                destination: MemoryAddress::direct(0),
                source: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U1),
            },
            Opcode::Cast {
                destination: MemoryAddress::direct(1),
                source: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U1),
            },
            Opcode::ConditionalMov {
                destination: MemoryAddress::direct(4), // Sets 3_u128 to memory address 4
                source_a: MemoryAddress::direct(2),
                source_b: MemoryAddress::direct(3),
                condition: MemoryAddress::direct(0),
            },
            Opcode::ConditionalMov {
                destination: MemoryAddress::direct(5), // Sets 2_u128 to memory address 5
                source_a: MemoryAddress::direct(2),
                source_b: MemoryAddress::direct(3),
                condition: MemoryAddress::direct(1),
            },
        ];
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let VM { memory, .. } = vm;

        let destination_value = memory.read(MemoryAddress::direct(4));
        assert_eq!(destination_value.to_field(), (3_u128).into());

        let source_value = memory.read(MemoryAddress::direct(5));
        assert_eq!(source_value.to_field(), (2_u128).into());
    }

    #[test]
    fn cmp_binary_ops() {
        let bit_size = MEMORY_ADDRESSING_BIT_SIZE;
        let calldata: Vec<FieldElement> =
            vec![(2u128).into(), (2u128).into(), (0u128).into(), (5u128).into(), (6u128).into()];
        let calldata_size = calldata.len();

        let calldata_copy_opcodes = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(5u64),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
        ];

        let cast_opcodes: Vec<_> = (0..calldata_size)
            .map(|index| Opcode::Cast {
                destination: MemoryAddress::direct(index),
                source: MemoryAddress::direct(index),
                bit_size: BitSize::Integer(bit_size),
            })
            .collect();

        let equal_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::Equals,
            lhs: MemoryAddress::direct(0),
            rhs: MemoryAddress::direct(1),
            destination: MemoryAddress::direct(2),
        };

        let not_equal_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::Equals,
            lhs: MemoryAddress::direct(0),
            rhs: MemoryAddress::direct(3),
            destination: MemoryAddress::direct(2),
        };

        let less_than_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::LessThan,
            lhs: MemoryAddress::direct(3),
            rhs: MemoryAddress::direct(4),
            destination: MemoryAddress::direct(2),
        };

        let less_than_equal_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::LessThanEquals,
            lhs: MemoryAddress::direct(3),
            rhs: MemoryAddress::direct(4),
            destination: MemoryAddress::direct(2),
        };

        let opcodes: Vec<_> = calldata_copy_opcodes
            .into_iter()
            .chain(cast_opcodes)
            .chain([equal_opcode, not_equal_opcode, less_than_opcode, less_than_equal_opcode])
            .collect();
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, &opcodes, &[], &solver, false, None);

        // Calldata copy
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        for _ in 0..calldata_size {
            let status = vm.process_opcode();
            assert_eq!(status, VMStatus::InProgress);
        }

        // Equals
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_eq_value = vm.memory.read(MemoryAddress::direct(2));
        assert_eq!(output_eq_value, true.into());

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_neq_value = vm.memory.read(MemoryAddress::direct(2));
        assert_eq!(output_neq_value, false.into());

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let lt_value = vm.memory.read(MemoryAddress::direct(2));
        assert_eq!(lt_value, true.into());

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let lte_value = vm.memory.read(MemoryAddress::direct(2));
        assert_eq!(lte_value, true.into());
    }

    #[test]
    fn store_opcode() {
        /// Brillig code for the following:
        ///     let mut i = 0;
        ///     let len = memory.len();
        ///     while i < len {
        ///         memory[i] = i as Value;
        ///         i += 1;
        ///     }
        fn brillig_write_memory(item_count: usize) -> Vec<MemoryValue<FieldElement>> {
            let integer_bit_size = MEMORY_ADDRESSING_BIT_SIZE;
            let bit_size = BitSize::Integer(integer_bit_size);
            let r_i = MemoryAddress::direct(0);
            let r_len = MemoryAddress::direct(1);
            let r_tmp = MemoryAddress::direct(2);
            let r_pointer = MemoryAddress::direct(3);

            let start: [Opcode<FieldElement>; 3] = [
                // i = 0
                Opcode::Const { destination: r_i, value: 0u128.into(), bit_size },
                // len = memory.len() (approximation)
                Opcode::Const { destination: r_len, value: item_count.into(), bit_size },
                // pointer = free_memory_ptr
                Opcode::Const { destination: r_pointer, value: 4u128.into(), bit_size },
            ];
            let loop_body = [
                // *i = i
                Opcode::Store { destination_pointer: r_pointer, source: r_i },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: integer_bit_size,
                },
                // pointer = pointer + 1
                Opcode::BinaryIntOp {
                    destination: r_pointer,
                    lhs: r_pointer,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: integer_bit_size,
                },
                // tmp = i < len
                Opcode::BinaryIntOp {
                    destination: r_tmp,
                    lhs: r_i,
                    op: BinaryIntOp::LessThan,
                    rhs: r_len,
                    bit_size: integer_bit_size,
                },
                // if tmp != 0 goto loop_body
                Opcode::JumpIf { condition: r_tmp, location: start.len() },
            ];

            let opcodes = [&start[..], &loop_body[..]].concat();
            let solver = StubbedBlackBoxSolver::default();
            let vm = brillig_execute_and_get_vm(vec![], &opcodes, &solver);
            vm.get_memory()[4..].to_vec()
        }

        let memory = brillig_write_memory(5);
        let expected =
            vec![(0u32).into(), (1u32).into(), (2u32).into(), (3u32).into(), (4u32).into()];
        assert_eq!(memory, expected);

        let memory = brillig_write_memory(1024);
        let expected: Vec<_> = (0..1024).map(|i: u32| i.into()).collect();
        assert_eq!(memory, expected);
    }

    #[test]
    fn iconst_opcode() {
        let opcodes = &[
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
                value: FieldElement::from(8_usize),
            },
            Opcode::IndirectConst {
                destination_pointer: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
                value: FieldElement::from(27_usize),
            },
        ];
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(vec![], opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let VM { memory, .. } = vm;

        let destination_value = memory.read(MemoryAddress::direct(8));
        assert_eq!(destination_value.to_field(), (27_usize).into());
    }

    #[test]
    fn load_opcode() {
        /// Brillig code for the following:
        ///     let mut sum = 0;
        ///     let mut i = 0;
        ///     let len = memory.len();
        ///     while i < len {
        ///         sum += memory[i];
        ///         i += 1;
        ///     }
        fn brillig_sum_memory(memory: Vec<FieldElement>) -> FieldElement {
            let bit_size = IntegerBitSize::U32;
            let r_i = MemoryAddress::direct(0);
            let r_len = MemoryAddress::direct(1);
            let r_sum = MemoryAddress::direct(2);
            let r_tmp = MemoryAddress::direct(3);
            let r_pointer = MemoryAddress::direct(4);

            let start = [
                // sum = 0
                Opcode::Const { destination: r_sum, value: 0u128.into(), bit_size: BitSize::Field },
                // i = 0
                Opcode::Const {
                    destination: r_i,
                    value: 0u128.into(),
                    bit_size: BitSize::Integer(bit_size),
                },
                // len = array.len() (approximation)
                Opcode::Const {
                    destination: r_len,
                    value: memory.len().into(),
                    bit_size: BitSize::Integer(bit_size),
                },
                // pointer = array_ptr
                Opcode::Const {
                    destination: r_pointer,
                    value: 5u128.into(),
                    bit_size: BitSize::Integer(bit_size),
                },
                Opcode::Const {
                    destination: MemoryAddress::direct(100),
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(memory.len() as u32),
                },
                Opcode::Const {
                    destination: MemoryAddress::direct(101),
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(0u64),
                },
                Opcode::CalldataCopy {
                    destination_address: MemoryAddress::direct(5),
                    size_address: MemoryAddress::direct(100),
                    offset_address: MemoryAddress::direct(101),
                },
            ];
            let loop_body = [
                // tmp = *i
                Opcode::Load { destination: r_tmp, source_pointer: r_pointer },
                // sum = sum + tmp
                Opcode::BinaryFieldOp {
                    destination: r_sum,
                    lhs: r_sum,
                    op: BinaryFieldOp::Add,
                    rhs: r_tmp,
                },
                // tmp = 1
                Opcode::Const {
                    destination: r_tmp,
                    value: 1u128.into(),
                    bit_size: BitSize::Integer(bit_size),
                },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size,
                },
                // pointer = pointer + 1
                Opcode::BinaryIntOp {
                    destination: r_pointer,
                    lhs: r_pointer,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size,
                },
                // tmp = i < len
                Opcode::BinaryIntOp {
                    destination: r_tmp,
                    lhs: r_i,
                    op: BinaryIntOp::LessThan,
                    rhs: r_len,
                    bit_size,
                },
                // if tmp != 0 goto loop_body
                Opcode::JumpIf { condition: r_tmp, location: start.len() },
            ];

            let opcodes = [&start[..], &loop_body[..]].concat();
            let solver = StubbedBlackBoxSolver::default();
            let vm = brillig_execute_and_get_vm(memory, &opcodes, &solver);
            vm.memory.read(r_sum).to_field()
        }

        assert_eq!(
            brillig_sum_memory(vec![
                (1u128).into(),
                (2u128).into(),
                (3u128).into(),
                (4u128).into(),
                (5u128).into(),
            ]),
            (15u128).into()
        );
        assert_eq!(brillig_sum_memory(vec![(1u128).into(); 1024]), (1024u128).into());
    }

    #[test]
    fn call_and_return_opcodes() {
        /// Brillig code for the following recursive function:
        ///     fn recursive_write(i: u128, len: u128) {
        ///         if len <= i {
        ///             return;
        ///         }
        ///         memory[i as usize] = i as Value;
        ///         recursive_write(memory, i + 1, len);
        ///     }
        /// Note we represent a 100% in-stack optimized form in brillig
        fn brillig_recursive_write_memory<F: AcirField>(size: usize) -> Vec<MemoryValue<F>> {
            let integer_bit_size = MEMORY_ADDRESSING_BIT_SIZE;
            let bit_size = BitSize::Integer(integer_bit_size);
            let r_i = MemoryAddress::direct(0);
            let r_len = MemoryAddress::direct(1);
            let r_tmp = MemoryAddress::direct(2);
            let r_pointer = MemoryAddress::direct(3);

            let start: [Opcode<F>; 5] = [
                // i = 0
                Opcode::Const { destination: r_i, value: 0u128.into(), bit_size },
                // len = size
                Opcode::Const { destination: r_len, value: (size as u128).into(), bit_size },
                // pointer = free_memory_ptr
                Opcode::Const { destination: r_pointer, value: 4u128.into(), bit_size },
                // call recursive_fn
                Opcode::Call {
                            location: 5, // Call after 'start'
                        },
                // end program by jumping to end
                Opcode::Jump { location: 100 },
            ];

            let recursive_fn = [
                // tmp = len <= i
                Opcode::BinaryIntOp {
                    destination: r_tmp,
                    lhs: r_len,
                    op: BinaryIntOp::LessThanEquals,
                    rhs: r_i,
                    bit_size: integer_bit_size,
                },
                // if !tmp, goto end
                Opcode::JumpIf {
                    condition: r_tmp,
                    location: start.len() + 7, // 8 ops in recursive_fn, go to 'Return'
                },
                // *i = i
                Opcode::Store { destination_pointer: r_pointer, source: r_i },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: integer_bit_size,
                },
                // pointer = pointer + 1
                Opcode::BinaryIntOp {
                    destination: r_pointer,
                    lhs: r_pointer,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: integer_bit_size,
                },
                // call recursive_fn
                Opcode::Call { location: start.len() },
                Opcode::Return {},
            ];

            let opcodes = [&start[..], &recursive_fn[..]].concat();
            let solver = StubbedBlackBoxSolver::default();
            let vm = brillig_execute_and_get_vm(vec![], &opcodes, &solver);
            vm.get_memory()[4..].to_vec()
        }

        let memory = brillig_recursive_write_memory::<FieldElement>(5);
        let expected =
            vec![(0u32).into(), (1u32).into(), (2u32).into(), (3u32).into(), (4u32).into()];
        assert_eq!(memory, expected);

        let memory = brillig_recursive_write_memory::<FieldElement>(1024);
        let expected: Vec<_> = (0..1024).map(|i: u32| i.into()).collect();
        assert_eq!(memory, expected);
    }

    /// Helper to execute brillig code
    fn brillig_execute_and_get_vm<'a, F: AcirField>(
        calldata: Vec<F>,
        opcodes: &'a [Opcode<F>],
        solver: &'a StubbedBlackBoxSolver,
    ) -> VM<'a, F, StubbedBlackBoxSolver> {
        let mut vm = VM::new(calldata, opcodes, &[], solver, false, None);
        brillig_execute(&mut vm);
        assert!(vm.call_stack.is_empty());
        vm
    }

    fn brillig_execute<F: AcirField>(vm: &mut VM<F, StubbedBlackBoxSolver>) {
        loop {
            let status = vm.process_opcode();
            if matches!(status, VMStatus::Finished { .. } | VMStatus::ForeignCallWait { .. }) {
                break;
            }
            assert_eq!(status, VMStatus::InProgress);
        }
    }

    #[test]
    fn relative_addressing() {
        let calldata = vec![];
        let bit_size = BitSize::Integer(IntegerBitSize::U32);
        let value = FieldElement::from(3u128);

        let opcodes = [
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size,
                value: FieldElement::from(27u128),
            },
            Opcode::Const {
                destination: MemoryAddress::relative(1), // Resolved address 28 value 3
                bit_size,
                value,
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1), // Address 1 value 3
                bit_size,
                value,
            },
            Opcode::BinaryIntOp {
                destination: MemoryAddress::direct(1),
                op: BinaryIntOp::Equals,
                bit_size: IntegerBitSize::U32,
                lhs: MemoryAddress::direct(1),
                rhs: MemoryAddress::direct(28),
            },
        ];

        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, &opcodes, &[], &solver, false, None);

        vm.process_opcode();
        vm.process_opcode();
        vm.process_opcode();
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let VM { memory, .. } = vm;
        let output_value = memory.read(MemoryAddress::direct(1));

        assert_eq!(output_value.to_field(), FieldElement::from(1u128));
    }

    #[test]
    fn field_zero_division_regression() {
        let calldata: Vec<FieldElement> = vec![];

        let opcodes = &[
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Field,
                value: FieldElement::from(1u64),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Field,
                value: FieldElement::from(0u64),
            },
            Opcode::BinaryFieldOp {
                destination: MemoryAddress::direct(2),
                op: BinaryFieldOp::Div,
                lhs: MemoryAddress::direct(0),
                rhs: MemoryAddress::direct(1),
            },
        ];
        let solver = StubbedBlackBoxSolver::default();
        let mut vm = VM::new(calldata, opcodes, &[], &solver, false, None);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);
        let status = vm.process_opcode();
        assert_eq!(
            status,
            VMStatus::Failure {
                reason: FailureReason::RuntimeError {
                    message: "Attempted to divide by zero".into()
                },
                call_stack: vec![2]
            }
        );
    }
}
