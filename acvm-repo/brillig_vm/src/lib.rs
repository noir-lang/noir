#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The Brillig VM is a specialized VM which allows the [ACVM][acvm] to perform custom non-determinism.
//!
//! Brillig bytecode is distinct from regular [ACIR][acir] in that it does not generate constraints.
//!
//! [acir]: https://crates.io/crates/acir
//! [acvm]: https://crates.io/crates/acvm

use acir::brillig::{
    BinaryFieldOp, BinaryIntOp, BitSize, ForeignCallParam, ForeignCallResult, HeapArray,
    HeapValueType, HeapVector, IntegerBitSize, MemoryAddress, Opcode, ValueOrArray,
};
use acir::AcirField;
use acvm_blackbox_solver::BlackBoxFunctionSolver;
use arithmetic::{evaluate_binary_field_op, evaluate_binary_int_op, BrilligArithmeticError};
use black_box::{evaluate_black_box, BrilligBigIntSolver};

// Re-export `brillig`.
pub use acir::brillig;
pub use memory::{Memory, MemoryValue, MEMORY_ADDRESSING_BIT_SIZE};

mod arithmetic;
mod black_box;
mod memory;

/// The error call stack contains the opcode indexes of the call stack at the time of failure, plus the index of the opcode that failed.
pub type ErrorCallStack = Vec<usize>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FailureReason {
    Trap { revert_data_offset: usize, revert_data_size: usize },
    RuntimeError { message: String },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VMStatus<F> {
    Finished {
        return_data_offset: usize,
        return_data_size: usize,
    },
    InProgress,
    Failure {
        reason: FailureReason,
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

// A sample for each opcode that was executed.
pub type BrilligProfilingSamples = Vec<BrilligProfilingSample>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BrilligProfilingSample {
    // The call stack when processing a given opcode.
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
    memory: Memory<F>,
    /// Call stack
    call_stack: Vec<usize>,
    /// The solver for blackbox functions
    black_box_solver: &'a B,
    // The solver for big integers
    bigint_solver: BrilligBigIntSolver,
    // Flag that determines whether we want to profile VM.
    profiling_active: bool,
    // Samples for profiling the VM execution.
    profiling_samples: BrilligProfilingSamples,
}

impl<'a, F: AcirField, B: BlackBoxFunctionSolver<F>> VM<'a, F, B> {
    /// Constructs a new VM instance
    pub fn new(
        calldata: Vec<F>,
        bytecode: &'a [Opcode<F>],
        foreign_call_results: Vec<ForeignCallResult<F>>,
        black_box_solver: &'a B,
        profiling_active: bool,
    ) -> Self {
        Self {
            calldata,
            program_counter: 0,
            foreign_call_counter: 0,
            foreign_call_results,
            bytecode,
            status: VMStatus::InProgress,
            memory: Memory::default(),
            call_stack: Vec::new(),
            black_box_solver,
            bigint_solver: Default::default(),
            profiling_active,
            profiling_samples: Vec::with_capacity(bytecode.len()),
        }
    }

    pub fn is_profiling_active(&self) -> bool {
        self.profiling_active
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

    /// Sets the status of the VM to `ForeignCallWait`.
    /// Indicating that the VM is now waiting for a foreign call to be resolved.
    fn wait_for_foreign_call(
        &mut self,
        function: String,
        inputs: Vec<ForeignCallParam<F>>,
    ) -> VMStatus<F> {
        self.status(VMStatus::ForeignCallWait { function, inputs })
    }

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
                    self.fail(error)
                } else {
                    self.increment_program_counter()
                }
            }
            Opcode::Cast { destination: destination_address, source: source_address, bit_size } => {
                let source_value = self.memory.read(*source_address);
                let casted_value = self.cast(*bit_size, source_value);
                self.memory.write(*destination_address, casted_value);
                self.increment_program_counter()
            }
            Opcode::Jump { location: destination } => self.set_program_counter(*destination),
            Opcode::JumpIf { condition, location: destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true
                let condition_value = self.memory.read(*condition);
                if condition_value.try_into().expect("condition value is not a boolean") {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::JumpIfNot { condition, location: destination } => {
                let condition_value = self.memory.read(*condition);
                if condition_value.try_into().expect("condition value is not a boolean") {
                    return self.increment_program_counter();
                }
                self.set_program_counter(*destination)
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
            } => {
                assert!(inputs.len() == input_value_types.len());
                assert!(destinations.len() == destination_value_types.len());

                if self.foreign_call_counter >= self.foreign_call_results.len() {
                    // When this opcode is called, it is possible that the results of a foreign call are
                    // not yet known (not enough entries in `foreign_call_results`).
                    // If that is the case, just resolve the inputs and pause the VM with a status
                    // (VMStatus::ForeignCallWait) that communicates the foreign function name and
                    // resolved inputs back to the caller. Once the caller pushes to `foreign_call_results`,
                    // they can then make another call to the VM that starts at this opcode
                    // but has the necessary results to proceed with execution.
                    let resolved_inputs = inputs
                        .iter()
                        .zip(input_value_types)
                        .map(|(input, input_type)| self.get_memory_values(*input, input_type))
                        .collect::<Vec<_>>();
                    return self.wait_for_foreign_call(function.clone(), resolved_inputs);
                }

                let write_result = self.write_foreign_call_result(
                    destinations,
                    destination_value_types,
                    self.foreign_call_counter,
                );

                if let Err(e) = write_result {
                    return self.fail(e);
                }

                self.foreign_call_counter += 1;
                self.increment_program_counter()
            }
            Opcode::Mov { destination: destination_address, source: source_address } => {
                let source_value = self.memory.read(*source_address);
                self.memory.write(*destination_address, source_value);
                self.increment_program_counter()
            }
            Opcode::ConditionalMov { destination, source_a, source_b, condition } => {
                let condition_value = self.memory.read(*condition);
                if condition_value.try_into().expect("condition value is not a boolean") {
                    self.memory.write(*destination, self.memory.read(*source_a));
                } else {
                    self.memory.write(*destination, self.memory.read(*source_b));
                }
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
                self.memory.write(destination, self.memory.read(*source_address));
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
                match evaluate_black_box(
                    black_box_op,
                    self.black_box_solver,
                    &mut self.memory,
                    &mut self.bigint_solver,
                ) {
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

    fn get_memory_values(
        &self,
        input: ValueOrArray,
        value_type: &HeapValueType,
    ) -> ForeignCallParam<F> {
        match (input, value_type) {
            (ValueOrArray::MemoryAddress(value_index), HeapValueType::Simple(_)) => {
                ForeignCallParam::Single(self.memory.read(value_index).to_field())
            }
            (
                ValueOrArray::HeapArray(HeapArray { pointer: pointer_index, size }),
                HeapValueType::Array { value_types, size: type_size },
            ) if *type_size == size => {
                let start = self.memory.read_ref(pointer_index);
                self.read_slice_of_values_from_memory(start, size, value_types)
                    .into_iter()
                    .map(|mem_value| mem_value.to_field())
                    .collect::<Vec<_>>()
                    .into()
            }
            (
                ValueOrArray::HeapVector(HeapVector { pointer: pointer_index, size: size_index }),
                HeapValueType::Vector { value_types },
            ) => {
                let start = self.memory.read_ref(pointer_index);
                let size = self.memory.read(size_index).to_usize();
                self.read_slice_of_values_from_memory(start, size, value_types)
                    .into_iter()
                    .map(|mem_value| mem_value.to_field())
                    .collect::<Vec<_>>()
                    .into()
            }
            _ => {
                unreachable!("Unexpected value type {value_type:?} for input {input:?}");
            }
        }
    }

    /// Reads an array/vector from memory but recursively reads pointers to
    /// nested arrays/vectors according to the sequence of value types.
    fn read_slice_of_values_from_memory(
        &self,
        start: MemoryAddress,
        size: usize,
        value_types: &[HeapValueType],
    ) -> Vec<MemoryValue<F>> {
        assert!(!start.is_relative(), "read_slice_of_values_from_memory requires direct addresses");
        if HeapValueType::all_simple(value_types) {
            self.memory.read_slice(start, size).to_vec()
        } else {
            // Check that the sequence of value types fit an integer number of
            // times inside the given size.
            assert!(
                0 == size % value_types.len(),
                "array/vector does not contain a whole number of elements"
            );
            (0..size)
                .zip(value_types.iter().cycle())
                .flat_map(|(i, value_type)| {
                    let value_address = start.offset(i);
                    match value_type {
                        HeapValueType::Simple(_) => {
                            vec![self.memory.read(value_address)]
                        }
                        HeapValueType::Array { value_types, size } => {
                            let array_address = self.memory.read_ref(value_address);

                            self.read_slice_of_values_from_memory(
                                array_address.offset(1),
                                *size,
                                value_types,
                            )
                        }
                        HeapValueType::Vector { value_types } => {
                            let vector_address = self.memory.read_ref(value_address);
                            let size_address =
                                MemoryAddress::direct(vector_address.unwrap_direct() + 1);
                            let items_start = vector_address.offset(2);
                            let vector_size = self.memory.read(size_address).to_usize();
                            self.read_slice_of_values_from_memory(
                                items_start,
                                vector_size,
                                value_types,
                            )
                        }
                    }
                })
                .collect::<Vec<_>>()
        }
    }

    fn write_foreign_call_result(
        &mut self,
        destinations: &[ValueOrArray],
        destination_value_types: &[HeapValueType],
        foreign_call_index: usize,
    ) -> Result<(), String> {
        let values = std::mem::take(&mut self.foreign_call_results[foreign_call_index].values);

        if destinations.len() != values.len() {
            return Err(format!(
                "{} output values were provided as a foreign call result for {} destination slots",
                values.len(),
                destinations.len()
            ));
        }

        for ((destination, value_type), output) in
            destinations.iter().zip(destination_value_types).zip(&values)
        {
            match (destination, value_type) {
            (ValueOrArray::MemoryAddress(value_index), HeapValueType::Simple(bit_size)) => {
                match output {
                    ForeignCallParam::Single(value) => {
                        self.write_value_to_memory(*value_index, value, *bit_size)?;
                    }
                    _ => return Err(format!(
                        "Function result size does not match brillig bytecode. Expected 1 result but got {output:?}")
                    ),
                }
            }
            (
                ValueOrArray::HeapArray(HeapArray { pointer: pointer_index, size }),
                HeapValueType::Array { value_types, size: type_size },
            ) if size == type_size => {
                if HeapValueType::all_simple(value_types) {
                    match output {
                        ForeignCallParam::Array(values) => {
                            if values.len() != *size {
                                // foreign call returning flattened values into a nested type, so the sizes do not match
                               let destination = self.memory.read_ref(*pointer_index);
                               let return_type = value_type;
                               let mut flatten_values_idx = 0; //index of values read from flatten_values
                               self.write_slice_of_values_to_memory(destination, &output.fields(), &mut flatten_values_idx, return_type)?;
                            } else {
                                self.write_values_to_memory_slice(*pointer_index, values, value_types)?;
                            }
                        }
                        _ => {
                            return Err("Function result size does not match brillig bytecode size".to_string());
                        }
                    }
                } else {
                    // foreign call returning flattened values into a nested type, so the sizes do not match
                    let destination = self.memory.read_ref(*pointer_index);
                    let return_type = value_type;
                    let mut flatten_values_idx = 0; //index of values read from flatten_values
                    self.write_slice_of_values_to_memory(destination, &output.fields(), &mut flatten_values_idx, return_type)?;
            }
        }
            (
                ValueOrArray::HeapVector(HeapVector {pointer: pointer_index, size: size_index }),
                HeapValueType::Vector { value_types },
            ) => {
                if HeapValueType::all_simple(value_types) {
                    match output {
                        ForeignCallParam::Array(values) => {
                            // Set our size in the size address
                            self.memory.write(*size_index, values.len().into());
                            self.write_values_to_memory_slice(*pointer_index, values, value_types)?;

                        }
                        _ => {
                            return Err("Function result size does not match brillig bytecode size".to_string());
                        }
                    }
                } else {
                    unimplemented!("deflattening heap vectors from foreign calls");
                }
            }
            _ => {
                return Err(format!("Unexpected value type {value_type:?} for destination {destination:?}"));
            }
        }
        }

        let _ =
            std::mem::replace(&mut self.foreign_call_results[foreign_call_index].values, values);

        Ok(())
    }

    fn write_value_to_memory(
        &mut self,
        destination: MemoryAddress,
        value: &F,
        value_bit_size: BitSize,
    ) -> Result<(), String> {
        let memory_value = MemoryValue::new_checked(*value, value_bit_size);

        if let Some(memory_value) = memory_value {
            self.memory.write(destination, memory_value);
        } else {
            return Err(format!(
                "Foreign call result value {} does not fit in bit size {:?}",
                value, value_bit_size
            ));
        }
        Ok(())
    }

    fn write_values_to_memory_slice(
        &mut self,
        pointer_index: MemoryAddress,
        values: &[F],
        value_types: &[HeapValueType],
    ) -> Result<(), String> {
        let bit_sizes_iterator = value_types
            .iter()
            .map(|typ| match typ {
                HeapValueType::Simple(bit_size) => *bit_size,
                _ => unreachable!("Expected simple value type"),
            })
            .cycle();

        // Convert the destination pointer to a usize
        let destination = self.memory.read_ref(pointer_index);
        // Write to our destination memory
        let memory_values: Option<Vec<_>> = values
            .iter()
            .zip(bit_sizes_iterator)
            .map(|(value, bit_size)| MemoryValue::new_checked(*value, bit_size))
            .collect();
        if let Some(memory_values) = memory_values {
            self.memory.write_slice(destination, &memory_values);
        } else {
            return Err(format!(
                "Foreign call result values {:?} do not match expected bit sizes",
                values,
            ));
        }
        Ok(())
    }

    /// Writes flattened values to memory, using the provided type
    /// Function calls itself recursively in order to work with recursive types (nested arrays)
    /// values_idx is the current index in the values vector and is incremented every time
    /// a value is written to memory
    /// The function returns the address of the next value to be written
    fn write_slice_of_values_to_memory(
        &mut self,
        destination: MemoryAddress,
        values: &Vec<F>,
        values_idx: &mut usize,
        value_type: &HeapValueType,
    ) -> Result<(), String> {
        assert!(
            !destination.is_relative(),
            "write_slice_of_values_to_memory requires direct addresses"
        );
        let mut current_pointer = destination;
        match value_type {
            HeapValueType::Simple(bit_size) => {
                self.write_value_to_memory(destination, &values[*values_idx], *bit_size)?;
                *values_idx += 1;
                Ok(())
            }
            HeapValueType::Array { value_types, size } => {
                for _ in 0..*size {
                    for typ in value_types {
                        match typ {
                            HeapValueType::Simple(len) => {
                                self.write_value_to_memory(
                                    current_pointer,
                                    &values[*values_idx],
                                    *len,
                                )?;
                                *values_idx += 1;
                                current_pointer = current_pointer.offset(1);
                            }
                            HeapValueType::Array { .. } => {
                                let destination = self.memory.read_ref(current_pointer).offset(1);
                                self.write_slice_of_values_to_memory(
                                    destination,
                                    values,
                                    values_idx,
                                    typ,
                                )?;
                                current_pointer = current_pointer.offset(1);
                            }
                            HeapValueType::Vector { .. } => {
                                return Err(format!(
                                    "Unsupported returned type in foreign calls {:?}",
                                    typ
                                ));
                            }
                        }
                    }
                }
                Ok(())
            }
            HeapValueType::Vector { .. } => {
                Err(format!("Unsupported returned type in foreign calls {:?}", value_type))
            }
        }
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
        Ok(())
    }

    fn process_not(
        &mut self,
        source: MemoryAddress,
        destination: MemoryAddress,
        op_bit_size: IntegerBitSize,
    ) -> Result<(), String> {
        let (value, bit_size) = self
            .memory
            .read(source)
            .extract_integer()
            .ok_or("Not opcode source is not an integer")?;

        if bit_size != op_bit_size {
            return Err(format!(
                "Not opcode source bit size {} does not match expected bit size {}",
                bit_size, op_bit_size
            ));
        }

        let negated_value = if let IntegerBitSize::U128 = bit_size {
            !value
        } else {
            let bit_size: u32 = bit_size.into();
            let mask = (1_u128 << bit_size as u128) - 1;
            (!value) & mask
        };
        self.memory.write(destination, MemoryValue::new_integer(negated_value, bit_size));
        Ok(())
    }

    /// Casts a value to a different bit size.
    fn cast(&self, target_bit_size: BitSize, source_value: MemoryValue<F>) -> MemoryValue<F> {
        match (source_value, target_bit_size) {
            // Field to field, no op
            (MemoryValue::Field(_), BitSize::Field) => source_value,
            // Field downcast to u128
            (MemoryValue::Field(field), BitSize::Integer(IntegerBitSize::U128)) => {
                MemoryValue::Integer(field.to_u128(), IntegerBitSize::U128)
            }
            // Field downcast to arbitrary bit size
            (MemoryValue::Field(field), BitSize::Integer(target_bit_size)) => {
                let as_u128 = field.to_u128();
                let target_bit_size_u32: u32 = target_bit_size.into();
                let mask = (1_u128 << target_bit_size_u32) - 1;
                MemoryValue::Integer(as_u128 & mask, target_bit_size)
            }
            // Integer upcast to field
            (MemoryValue::Integer(integer, _), BitSize::Field) => {
                MemoryValue::new_field(integer.into())
            }
            // Integer upcast to integer
            (MemoryValue::Integer(integer, source_bit_size), BitSize::Integer(target_bit_size))
                if source_bit_size <= target_bit_size =>
            {
                MemoryValue::Integer(integer, target_bit_size)
            }
            // Integer downcast
            (MemoryValue::Integer(integer, _), BitSize::Integer(target_bit_size)) => {
                let target_bit_size_u32: u32 = target_bit_size.into();
                let mask = (1_u128 << target_bit_size_u32) - 1;
                MemoryValue::Integer(integer & mask, target_bit_size)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::MEMORY_ADDRESSING_BIT_SIZE;
    use acir::{AcirField, FieldElement};
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
        let mut vm = VM::new(calldata, &opcodes, vec![], &StubbedBlackBoxSolver, false);

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

        let mut vm = VM::new(calldata, &opcodes, vec![], &StubbedBlackBoxSolver, false);

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
    fn jmpifnot_opcode() {
        let calldata: Vec<FieldElement> = vec![1u128.into(), 2u128.into()];

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
            Opcode::Jump { location: 6 },
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::Trap {
                revert_data: HeapVector {
                    pointer: MemoryAddress::direct(0),
                    size: MemoryAddress::direct(0),
                },
            },
            Opcode::BinaryFieldOp {
                op: BinaryFieldOp::Equals,
                lhs: MemoryAddress::direct(0),
                rhs: MemoryAddress::direct(1),
                destination: MemoryAddress::direct(2),
            },
            Opcode::JumpIfNot { condition: MemoryAddress::direct(2), location: 4 },
            Opcode::BinaryFieldOp {
                op: BinaryFieldOp::Add,
                lhs: MemoryAddress::direct(0),
                rhs: MemoryAddress::direct(1),
                destination: MemoryAddress::direct(2),
            },
        ];

        let mut vm = VM::new(calldata, &opcodes, vec![], &StubbedBlackBoxSolver, false);

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

        let output_cmp_value = vm.memory.read(MemoryAddress::direct(2));
        assert_eq!(output_cmp_value.to_field(), false.into());

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(
            status,
            VMStatus::Failure {
                reason: FailureReason::Trap { revert_data_offset: 0, revert_data_size: 0 },
                call_stack: vec![5]
            }
        );

        // The address at index `2` should have not changed as we jumped over the add opcode
        let VM { memory, .. } = vm;
        let output_value = memory.read(MemoryAddress::direct(2));
        assert_eq!(output_value.to_field(), false.into());
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
        let mut vm = VM::new(calldata, opcodes, vec![], &StubbedBlackBoxSolver, false);

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
        let mut vm = VM::new(calldata, opcodes, vec![], &StubbedBlackBoxSolver, false);

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

        let (negated_value, _) = memory
            .read(MemoryAddress::direct(1))
            .extract_integer()
            .expect("Expected integer as the output of Not");
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
        let mut vm = VM::new(calldata, opcodes, vec![], &StubbedBlackBoxSolver, false);

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
        let mut vm = VM::new(calldata, opcodes, vec![], &StubbedBlackBoxSolver, false);

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
        let mut vm = VM::new(calldata, &opcodes, vec![], &StubbedBlackBoxSolver, false);

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
            let vm = brillig_execute_and_get_vm(vec![], &opcodes);
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
        let mut vm = VM::new(vec![], opcodes, vec![], &StubbedBlackBoxSolver, false);

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
            let vm = brillig_execute_and_get_vm(memory, &opcodes);
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
            let vm = brillig_execute_and_get_vm(vec![], &opcodes);
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
    fn brillig_execute_and_get_vm<F: AcirField>(
        calldata: Vec<F>,
        opcodes: &[Opcode<F>],
    ) -> VM<'_, F, StubbedBlackBoxSolver> {
        let mut vm = VM::new(calldata, opcodes, vec![], &StubbedBlackBoxSolver, false);
        brillig_execute(&mut vm);
        assert_eq!(vm.call_stack, vec![]);
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
    fn foreign_call_opcode_simple_result() {
        let r_input = MemoryAddress::direct(0);
        let r_result = MemoryAddress::direct(1);

        let double_program = vec![
            // Load input address with value 5
            Opcode::Const {
                destination: r_input,
                value: (5u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // Call foreign function "double" with the input address
            Opcode::ForeignCall {
                function: "double".into(),
                destinations: vec![ValueOrArray::MemoryAddress(r_result)],
                destination_value_types: vec![HeapValueType::Simple(BitSize::Integer(
                    MEMORY_ADDRESSING_BIT_SIZE,
                ))],
                inputs: vec![ValueOrArray::MemoryAddress(r_input)],
                input_value_types: vec![HeapValueType::Simple(BitSize::Integer(
                    MEMORY_ADDRESSING_BIT_SIZE,
                ))],
            },
        ];

        let mut vm = brillig_execute_and_get_vm(vec![], &double_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "double".into(),
                inputs: vec![FieldElement::from(5usize).into()]
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(
            FieldElement::from(10u128).into(), // Result of doubling 5u128
        );

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result address
        let result_value = vm.memory.read(r_result);
        assert_eq!(result_value, (10u32).into());

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_memory_result() {
        let r_input = MemoryAddress::direct(0);
        let r_output = MemoryAddress::direct(1);

        // Define a simple 2x2 matrix in memory
        let initial_matrix: Vec<FieldElement> =
            vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result: Vec<FieldElement> =
            vec![(1u128).into(), (3u128).into(), (2u128).into(), (4u128).into()];

        let invert_program = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(initial_matrix.len() as u32),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(2),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
            // input = 0
            Opcode::Const {
                destination: r_input,
                value: 2_usize.into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // output = 0
            Opcode::Const {
                destination: r_output,
                value: 2_usize.into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destinations: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_output,
                    size: initial_matrix.len(),
                })],
                destination_value_types: vec![HeapValueType::Array {
                    size: initial_matrix.len(),
                    value_types: vec![HeapValueType::field()],
                }],
                inputs: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_input,
                    size: initial_matrix.len(),
                })],
                input_value_types: vec![HeapValueType::Array {
                    value_types: vec![HeapValueType::field()],
                    size: initial_matrix.len(),
                }],
            },
        ];

        let mut vm = brillig_execute_and_get_vm(initial_matrix.clone(), &invert_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "matrix_2x2_transpose".into(),
                inputs: vec![initial_matrix.into()]
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(expected_result.clone().into());

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result in memory
        let result_values = vm.memory.read_slice(MemoryAddress::direct(2), 4).to_vec();
        assert_eq!(
            result_values.into_iter().map(|mem_value| mem_value.to_field()).collect::<Vec<_>>(),
            expected_result
        );

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    /// Calling a simple foreign call function that takes any string input, concatenates it with itself, and reverses the concatenation
    #[test]
    fn foreign_call_opcode_vector_input_and_output() {
        let r_input_pointer = MemoryAddress::direct(0);
        let r_input_size = MemoryAddress::direct(1);
        // We need to pass a location of appropriate size
        let r_output_pointer = MemoryAddress::direct(2);
        let r_output_size = MemoryAddress::direct(3);

        // Our first string to use the identity function with
        let input_string: Vec<FieldElement> =
            vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];
        // Double the string (concatenate it with itself)
        let mut output_string: Vec<_> =
            input_string.iter().cloned().chain(input_string.clone()).collect();
        // Reverse the concatenated string
        output_string.reverse();

        // First call:
        let string_double_program = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(100),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(input_string.len() as u32),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(101),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(4),
                size_address: MemoryAddress::direct(100),
                offset_address: MemoryAddress::direct(101),
            },
            // input_pointer = 4
            Opcode::Const {
                destination: r_input_pointer,
                value: (4u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // input_size = input_string.len() (constant here)
            Opcode::Const {
                destination: r_input_size,
                value: input_string.len().into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // output_pointer = 4 + input_size
            Opcode::Const {
                destination: r_output_pointer,
                value: (4 + input_string.len()).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // output_size = input_size * 2
            Opcode::Const {
                destination: r_output_size,
                value: (input_string.len() * 2).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // output_pointer[0..output_size] = string_double(input_pointer[0...input_size])
            Opcode::ForeignCall {
                function: "string_double".into(),
                destinations: vec![ValueOrArray::HeapVector(HeapVector {
                    pointer: r_output_pointer,
                    size: r_output_size,
                })],
                destination_value_types: vec![HeapValueType::Vector {
                    value_types: vec![HeapValueType::field()],
                }],
                inputs: vec![ValueOrArray::HeapVector(HeapVector {
                    pointer: r_input_pointer,
                    size: r_input_size,
                })],
                input_value_types: vec![HeapValueType::Vector {
                    value_types: vec![HeapValueType::field()],
                }],
            },
        ];

        let mut vm = brillig_execute_and_get_vm(input_string.clone(), &string_double_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "string_double".into(),
                inputs: vec![input_string.clone().into()]
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(ForeignCallResult {
            values: vec![ForeignCallParam::Array(output_string.clone())],
        });

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result in memory
        let result_values: Vec<_> = vm
            .memory
            .read_slice(MemoryAddress::direct(4 + input_string.len()), output_string.len())
            .iter()
            .map(|mem_val| mem_val.clone().to_field())
            .collect();
        assert_eq!(result_values, output_string);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_memory_alloc_result() {
        let r_input = MemoryAddress::direct(0);
        let r_output = MemoryAddress::direct(1);

        // Define a simple 2x2 matrix in memory
        let initial_matrix: Vec<FieldElement> =
            vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result: Vec<FieldElement> =
            vec![(1u128).into(), (3u128).into(), (2u128).into(), (4u128).into()];

        let invert_program = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(100),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(initial_matrix.len() as u32),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(101),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(2),
                size_address: MemoryAddress::direct(100),
                offset_address: MemoryAddress::direct(101),
            },
            // input = 0
            Opcode::Const {
                destination: r_input,
                value: (2u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // output = 0
            Opcode::Const {
                destination: r_output,
                value: (6u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destinations: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_output,
                    size: initial_matrix.len(),
                })],
                destination_value_types: vec![HeapValueType::Array {
                    size: initial_matrix.len(),
                    value_types: vec![HeapValueType::field()],
                }],
                inputs: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_input,
                    size: initial_matrix.len(),
                })],
                input_value_types: vec![HeapValueType::Array {
                    size: initial_matrix.len(),
                    value_types: vec![HeapValueType::field()],
                }],
            },
        ];

        let mut vm = brillig_execute_and_get_vm(initial_matrix.clone(), &invert_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "matrix_2x2_transpose".into(),
                inputs: vec![initial_matrix.clone().into()]
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(expected_result.clone().into());

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check initial memory still in place
        let initial_values: Vec<_> = vm
            .memory
            .read_slice(MemoryAddress::direct(2), 4)
            .iter()
            .map(|mem_val| mem_val.clone().to_field())
            .collect();
        assert_eq!(initial_values, initial_matrix);

        // Check result in memory
        let result_values: Vec<_> = vm
            .memory
            .read_slice(MemoryAddress::direct(6), 4)
            .iter()
            .map(|mem_val| mem_val.clone().to_field())
            .collect();
        assert_eq!(result_values, expected_result);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_multiple_array_inputs_result() {
        let r_input_a = MemoryAddress::direct(0);
        let r_input_b = MemoryAddress::direct(1);
        let r_output = MemoryAddress::direct(2);

        // Define a simple 2x2 matrix in memory
        let matrix_a: Vec<FieldElement> =
            vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];

        let matrix_b: Vec<FieldElement> =
            vec![(10u128).into(), (11u128).into(), (12u128).into(), (13u128).into()];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result: Vec<FieldElement> =
            vec![(34u128).into(), (37u128).into(), (78u128).into(), (85u128).into()];

        let matrix_mul_program = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(100),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(matrix_a.len() + matrix_b.len()),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(101),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(3),
                size_address: MemoryAddress::direct(100),
                offset_address: MemoryAddress::direct(101),
            },
            // input = 3
            Opcode::Const {
                destination: r_input_a,
                value: (3u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // input = 7
            Opcode::Const {
                destination: r_input_b,
                value: (7u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // output = 0
            Opcode::Const {
                destination: r_output,
                value: (0u128).into(),
                bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destinations: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_output,
                    size: matrix_a.len(),
                })],
                destination_value_types: vec![HeapValueType::Array {
                    size: matrix_a.len(),
                    value_types: vec![HeapValueType::field()],
                }],
                inputs: vec![
                    ValueOrArray::HeapArray(HeapArray { pointer: r_input_a, size: matrix_a.len() }),
                    ValueOrArray::HeapArray(HeapArray { pointer: r_input_b, size: matrix_b.len() }),
                ],
                input_value_types: vec![
                    HeapValueType::Array {
                        size: matrix_a.len(),
                        value_types: vec![HeapValueType::field()],
                    },
                    HeapValueType::Array {
                        size: matrix_b.len(),
                        value_types: vec![HeapValueType::field()],
                    },
                ],
            },
        ];
        let mut initial_memory = matrix_a.clone();
        initial_memory.extend(matrix_b.clone());
        let mut vm = brillig_execute_and_get_vm(initial_memory, &matrix_mul_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "matrix_2x2_transpose".into(),
                inputs: vec![matrix_a.into(), matrix_b.into()]
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(expected_result.clone().into());

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result in memory
        let result_values: Vec<_> = vm
            .memory
            .read_slice(MemoryAddress::direct(0), 4)
            .iter()
            .map(|mem_val| mem_val.clone().to_field())
            .collect();
        assert_eq!(result_values, expected_result);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_nested_arrays_and_slices_input() {
        // [(1, <2,3>, [4]), (5, <6,7,8>, [9])]

        let v2: Vec<MemoryValue<FieldElement>> = vec![
            MemoryValue::new_field(FieldElement::from(2u128)),
            MemoryValue::new_field(FieldElement::from(3u128)),
        ];
        let a4: Vec<MemoryValue<FieldElement>> =
            vec![MemoryValue::new_field(FieldElement::from(4u128))];
        let v6: Vec<MemoryValue<FieldElement>> = vec![
            MemoryValue::new_field(FieldElement::from(6u128)),
            MemoryValue::new_field(FieldElement::from(7u128)),
            MemoryValue::new_field(FieldElement::from(8u128)),
        ];
        let a9: Vec<MemoryValue<FieldElement>> =
            vec![MemoryValue::new_field(FieldElement::from(9u128))];

        // construct memory by declaring all inner arrays/vectors first
        // Declare v2
        let v2_ptr: usize = 0usize;
        let mut memory = vec![MemoryValue::from(1_u32), v2.len().into()];
        memory.extend(v2.clone());
        let a4_ptr = memory.len();
        memory.extend(vec![MemoryValue::from(1_u32)]);
        memory.extend(a4.clone());
        let v6_ptr = memory.len();
        memory.extend(vec![MemoryValue::from(1_u32), v6.len().into()]);
        memory.extend(v6.clone());
        let a9_ptr = memory.len();
        memory.extend(vec![MemoryValue::from(1_u32)]);
        memory.extend(a9.clone());
        // finally we add the contents of the outer array
        memory.extend(vec![MemoryValue::from(1_u32)]);
        let outer_start = memory.len();
        let outer_array = vec![
            MemoryValue::new_field(FieldElement::from(1u128)),
            MemoryValue::from(v2.len() as u32),
            MemoryValue::from(v2_ptr),
            MemoryValue::from(a4_ptr),
            MemoryValue::new_field(FieldElement::from(5u128)),
            MemoryValue::from(v6.len() as u32),
            MemoryValue::from(v6_ptr),
            MemoryValue::from(a9_ptr),
        ];
        memory.extend(outer_array.clone());

        let input_array_value_types: Vec<HeapValueType> = vec![
            HeapValueType::field(),
            HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U64)), // size of following vector
            HeapValueType::Vector { value_types: vec![HeapValueType::field()] },
            HeapValueType::Array { value_types: vec![HeapValueType::field()], size: 1 },
        ];

        // memory address of the end of the above data structures
        let r_ptr = memory.len();

        let r_input = MemoryAddress::direct(r_ptr);
        let r_output = MemoryAddress::direct(r_ptr + 1);

        let program: Vec<_> = vec![
            Opcode::Const {
                destination: MemoryAddress::direct(100),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(memory.len()),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(101),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(100),
                offset_address: MemoryAddress::direct(101),
            },
        ]
        .into_iter()
        .chain(memory.iter().enumerate().map(|(index, mem_value)| Opcode::Cast {
            destination: MemoryAddress::direct(index),
            source: MemoryAddress::direct(index),
            bit_size: mem_value.bit_size(),
        }))
        .chain(vec![
            // input = 0
            Opcode::Const {
                destination: r_input,
                value: (outer_start).into(),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
            },
            // some_function(input)
            Opcode::ForeignCall {
                function: "flat_sum".into(),
                destinations: vec![ValueOrArray::MemoryAddress(r_output)],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_input,
                    size: outer_array.len(),
                })],
                input_value_types: vec![HeapValueType::Array {
                    value_types: input_array_value_types,
                    size: outer_array.len(),
                }],
            },
        ])
        .collect();

        let mut vm = brillig_execute_and_get_vm(
            memory.into_iter().map(|mem_value| mem_value.to_field()).collect(),
            &program,
        );

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "flat_sum".into(),
                inputs: vec![ForeignCallParam::Array(vec![
                    (1u128).into(),
                    (2u128).into(), // size of following vector
                    (2u128).into(),
                    (3u128).into(),
                    (4u128).into(),
                    (5u128).into(),
                    (3u128).into(), // size of following vector
                    (6u128).into(),
                    (7u128).into(),
                    (8u128).into(),
                    (9u128).into(),
                ])],
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(FieldElement::from(45u128).into());

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result
        let result_value = vm.memory.read(r_output);
        assert_eq!(result_value, MemoryValue::new_field(FieldElement::from(45u128)));

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
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

        let mut vm = VM::new(calldata, &opcodes, vec![], &StubbedBlackBoxSolver, false);

        vm.process_opcode();
        vm.process_opcode();
        vm.process_opcode();
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let VM { memory, .. } = vm;
        let output_value = memory.read(MemoryAddress::direct(1));

        assert_eq!(output_value.to_field(), FieldElement::from(1u128));
    }
}
