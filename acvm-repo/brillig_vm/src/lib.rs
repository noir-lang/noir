#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The Brillig VM is a specialized VM which allows the [ACVM][acvm] to perform custom non-determinism.
//!
//! Brillig bytecode is distinct from regular [ACIR][acir] in that it does not generate constraints.
//! This is a generalization over the fixed directives that exists within in the ACVM.
//!
//! [acir]: https://crates.io/crates/acir
//! [acvm]: https://crates.io/crates/acvm

use acir::brillig::{
    BinaryFieldOp, BinaryIntOp, ForeignCallParam, ForeignCallResult, HeapArray, HeapValueType,
    HeapVector, MemoryAddress, Opcode, Value, ValueOrArray,
};
use acir::FieldElement;
// Re-export `brillig`.
pub use acir::brillig;

mod arithmetic;
mod black_box;
mod memory;

use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use arithmetic::{evaluate_binary_bigint_op, evaluate_binary_field_op};
use black_box::evaluate_black_box;

pub use memory::Memory;
use num_bigint::BigUint;

/// The error call stack contains the opcode indexes of the call stack at the time of failure, plus the index of the opcode that failed.
pub type ErrorCallStack = Vec<usize>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VMStatus {
    Finished {
        return_data_offset: usize,
        return_data_size: usize,
    },
    InProgress,
    Failure {
        message: String,
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
        inputs: Vec<ForeignCallParam>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// VM encapsulates the state of the Brillig VM during execution.
pub struct VM<'a, B: BlackBoxFunctionSolver> {
    /// Calldata to the brillig function
    calldata: Vec<Value>,
    /// Instruction pointer
    program_counter: usize,
    /// A counter maintained throughout a Brillig process that determines
    /// whether the caller has resolved the results of a [foreign call][Opcode::ForeignCall].
    foreign_call_counter: usize,
    /// Represents the outputs of all foreign calls during a Brillig process
    /// List is appended onto by the caller upon reaching a [VMStatus::ForeignCallWait]
    foreign_call_results: Vec<ForeignCallResult>,
    /// Executable opcodes
    bytecode: &'a [Opcode],
    /// Status of the VM
    status: VMStatus,
    /// Memory of the VM
    memory: Memory,
    /// Call stack
    call_stack: Vec<Value>,
    /// The solver for blackbox functions
    black_box_solver: &'a B,
}

impl<'a, B: BlackBoxFunctionSolver> VM<'a, B> {
    /// Constructs a new VM instance
    pub fn new(
        calldata: Vec<Value>,
        bytecode: &'a [Opcode],
        foreign_call_results: Vec<ForeignCallResult>,
        black_box_solver: &'a B,
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
        }
    }

    /// Updates the current status of the VM.
    /// Returns the given status.
    fn status(&mut self, status: VMStatus) -> VMStatus {
        self.status = status.clone();
        status
    }

    pub fn get_status(&self) -> VMStatus {
        self.status.clone()
    }

    /// Sets the current status of the VM to Finished (completed execution).
    fn finish(&mut self, return_data_offset: usize, return_data_size: usize) -> VMStatus {
        self.status(VMStatus::Finished { return_data_offset, return_data_size })
    }

    /// Sets the status of the VM to `ForeignCallWait`.
    /// Indicating that the VM is now waiting for a foreign call to be resolved.
    fn wait_for_foreign_call(
        &mut self,
        function: String,
        inputs: Vec<ForeignCallParam>,
    ) -> VMStatus {
        self.status(VMStatus::ForeignCallWait { function, inputs })
    }

    pub fn resolve_foreign_call(&mut self, foreign_call_result: ForeignCallResult) {
        if self.foreign_call_counter < self.foreign_call_results.len() {
            panic!("No unresolved foreign calls");
        }
        self.foreign_call_results.push(foreign_call_result);
        self.status(VMStatus::InProgress);
    }

    /// Sets the current status of the VM to `fail`.
    /// Indicating that the VM encountered a `Trap` Opcode
    /// or an invalid state.
    fn fail(&mut self, message: String) -> VMStatus {
        let mut error_stack: Vec<_> =
            self.call_stack.iter().map(|value| value.to_usize()).collect();
        error_stack.push(self.program_counter);
        self.status(VMStatus::Failure { call_stack: error_stack, message });
        self.status.clone()
    }

    /// Loop over the bytecode and update the program counter
    pub fn process_opcodes(&mut self) -> VMStatus {
        while !matches!(
            self.process_opcode(),
            VMStatus::Finished { .. } | VMStatus::Failure { .. } | VMStatus::ForeignCallWait { .. }
        ) {}
        self.status.clone()
    }

    pub fn get_memory(&self) -> &[Value] {
        self.memory.values()
    }

    pub fn write_memory_at(&mut self, ptr: usize, value: Value) {
        self.memory.write(MemoryAddress(ptr), value);
    }

    /// Returns the VM's current call stack, including the actual program
    /// counter in the last position of the returned vector.
    pub fn get_call_stack(&self) -> Vec<usize> {
        self.call_stack
            .iter()
            .map(|program_counter| program_counter.to_usize())
            .chain(std::iter::once(self.program_counter))
            .collect()
    }

    /// Process a single opcode and modify the program counter.
    pub fn process_opcode(&mut self) -> VMStatus {
        let opcode = &self.bytecode[self.program_counter];
        match opcode {
            Opcode::BinaryFieldOp { op, lhs, rhs, destination: result } => {
                self.process_binary_field_op(*op, *lhs, *rhs, *result);
                self.increment_program_counter()
            }
            Opcode::BinaryIntOp { op, bit_size, lhs, rhs, destination: result } => {
                if let Err(error) = self.process_binary_int_op(*op, *bit_size, *lhs, *rhs, *result)
                {
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
                if !condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::JumpIfNot { condition, location: destination } => {
                let condition_value = self.memory.read(*condition);
                if condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::CalldataCopy { destination_address, size, offset } => {
                let values = &self.calldata[*offset..(*offset + size)];
                self.memory.write_slice(*destination_address, values);
                self.increment_program_counter()
            }
            Opcode::Return => {
                if let Some(return_location) = self.call_stack.pop() {
                    self.set_program_counter(return_location.to_usize() + 1)
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

                let values = &self.foreign_call_results[self.foreign_call_counter].values;

                let mut invalid_foreign_call_result = false;
                for ((destination, value_type), output) in
                    destinations.iter().zip(destination_value_types).zip(values)
                {
                    match (destination, value_type) {
                        (ValueOrArray::MemoryAddress(value_index), HeapValueType::Simple) => {
                            match output {
                                ForeignCallParam::Single(value) => {
                                    self.memory.write(*value_index, *value);
                                }
                                _ => unreachable!(
                                    "Function result size does not match brillig bytecode. Expected 1 result but got {output:?}"
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
                                            invalid_foreign_call_result = true;
                                            break;
                                        }
                                        // Convert the destination pointer to a usize
                                        let destination = self.memory.read_ref(*pointer_index);
                                        // Write to our destination memory
                                        self.memory.write_slice(destination, values);
                                    }
                                    _ => {
                                        unreachable!("Function result size does not match brillig bytecode size")
                                    }
                                }
                            } else {
                                unimplemented!("deflattening heap arrays from foreign calls");
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
                                        self.memory.write(*size_index, Value::from(values.len()));
                                        // Convert the destination pointer to a usize
                                        let destination = self.memory.read_ref(*pointer_index);
                                        // Write to our destination memory
                                        self.memory.write_slice(destination, values);
                                    }
                                    _ => {
                                        unreachable!("Function result size does not match brillig bytecode size")
                                    }
                                }
                            } else {
                                unimplemented!("deflattening heap vectors from foreign calls");
                            }
                        }
                        _ => {
                            unreachable!("Unexpected value type {value_type:?} for destination {destination:?}");
                        }
                    }
                }

                // These checks must come after resolving the foreign call outputs as `fail` uses a mutable reference
                if destinations.len() != values.len() {
                    self.fail(format!("{} output values were provided as a foreign call result for {} destination slots", values.len(), destinations.len()));
                }
                if invalid_foreign_call_result {
                    self.fail("Function result size does not match brillig bytecode".to_owned());
                }

                self.foreign_call_counter += 1;
                self.increment_program_counter()
            }
            Opcode::Mov { destination: destination_address, source: source_address } => {
                let source_value = self.memory.read(*source_address);
                self.memory.write(*destination_address, source_value);
                self.increment_program_counter()
            }
            Opcode::Trap => self.fail("explicit trap hit in brillig".to_string()),
            Opcode::Stop { return_data_offset, return_data_size } => {
                self.finish(*return_data_offset, *return_data_size)
            }
            Opcode::Load { destination: destination_address, source_pointer } => {
                // Convert our source_pointer to an address
                let source = self.memory.read_ref(*source_pointer);
                // Use our usize source index to lookup the value in memory
                let value = &self.memory.read(source);
                self.memory.write(*destination_address, *value);
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
                self.call_stack.push(Value::from(self.program_counter));
                self.set_program_counter(*location)
            }
            Opcode::Const { destination, value, bit_size: _ } => {
                self.memory.write(*destination, *value);
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
    fn increment_program_counter(&mut self) -> VMStatus {
        self.set_program_counter(self.program_counter + 1)
    }

    /// Increments the program counter by `value`.
    /// If the program counter no longer points to an opcode
    /// in the bytecode, then the VMStatus reports halted.
    fn set_program_counter(&mut self, value: usize) -> VMStatus {
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
    ) -> ForeignCallParam {
        match (input, value_type) {
            (ValueOrArray::MemoryAddress(value_index), HeapValueType::Simple) => {
                self.memory.read(value_index).into()
            }
            (
                ValueOrArray::HeapArray(HeapArray { pointer: pointer_index, size }),
                HeapValueType::Array { value_types, size: type_size },
            ) if *type_size == size => {
                let start = self.memory.read_ref(pointer_index);
                self.read_slice_of_values_from_memory(start, size, value_types).into()
            }
            (
                ValueOrArray::HeapVector(HeapVector { pointer: pointer_index, size: size_index }),
                HeapValueType::Vector { value_types },
            ) => {
                let start = self.memory.read_ref(pointer_index);
                let size = self.memory.read(size_index).to_usize();
                self.read_slice_of_values_from_memory(start, size, value_types).into()
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
    ) -> Vec<Value> {
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
                    let value_address: MemoryAddress = (start.to_usize() + i).into();
                    match value_type {
                        HeapValueType::Simple => {
                            let value = self.memory.read(value_address);
                            vec![value]
                        }
                        HeapValueType::Array { value_types, size } => {
                            let array_address = self.memory.read_ref(value_address);
                            let array_start = self.memory.read_ref(array_address);
                            self.read_slice_of_values_from_memory(array_start, *size, value_types)
                        }
                        HeapValueType::Vector { value_types } => {
                            let vector_address = self.memory.read_ref(value_address);
                            let vector_start = self.memory.read_ref(vector_address);
                            let size_address: MemoryAddress =
                                (vector_address.to_usize() + 1).into();
                            let vector_size = self.memory.read(size_address).to_usize();
                            self.read_slice_of_values_from_memory(
                                vector_start,
                                vector_size,
                                value_types,
                            )
                        }
                    }
                })
                .collect::<Vec<_>>()
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
    ) {
        let lhs_value = self.memory.read(lhs);
        let rhs_value = self.memory.read(rhs);

        let result_value =
            evaluate_binary_field_op(&op, lhs_value.to_field(), rhs_value.to_field());

        self.memory.write(result, result_value.into());
    }

    /// Process a binary operation.
    /// This method will not modify the program counter.
    fn process_binary_int_op(
        &mut self,
        op: BinaryIntOp,
        bit_size: u32,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
    ) -> Result<(), String> {
        let lhs_value = self.memory.read(lhs);
        let rhs_value = self.memory.read(rhs);

        // Convert to big integers
        let lhs_big = BigUint::from_bytes_be(&lhs_value.to_field().to_be_bytes());
        let rhs_big = BigUint::from_bytes_be(&rhs_value.to_field().to_be_bytes());
        let result_value = evaluate_binary_bigint_op(&op, lhs_big, rhs_big, bit_size)?;
        // Convert back to field element
        self.memory
            .write(result, FieldElement::from_be_bytes_reduce(&result_value.to_bytes_be()).into());
        Ok(())
    }

    /// Casts a value to a different bit size.
    fn cast(&self, bit_size: u32, value: Value) -> Value {
        let lhs_big = BigUint::from_bytes_be(&value.to_field().to_be_bytes());
        let mask = BigUint::from(2_u32).pow(bit_size) - 1_u32;
        FieldElement::from_be_bytes_reduce(&(lhs_big & mask).to_bytes_be()).into()
    }
}

pub(crate) struct DummyBlackBoxSolver;

impl BlackBoxFunctionSolver for DummyBlackBoxSolver {
    fn schnorr_verify(
        &self,
        _public_key_x: &FieldElement,
        _public_key_y: &FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError> {
        Ok(true)
    }
    fn pedersen_commitment(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Ok((2_u128.into(), 3_u128.into()))
    }
    fn pedersen_hash(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<FieldElement, BlackBoxResolutionError> {
        Ok(6_u128.into())
    }
    fn fixed_base_scalar_mul(
        &self,
        _low: &FieldElement,
        _high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Ok((4_u128.into(), 5_u128.into()))
    }
    fn ec_add(
        &self,
        _input1_x: &FieldElement,
        _input1_y: &FieldElement,
        _input2_x: &FieldElement,
        _input2_y: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Ok((5_u128.into(), 6_u128.into()))
    }
    fn poseidon2_permutation(
        &self,
        _input: &[FieldElement],
        len: u32,
    ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
        Ok(vec![0_u128.into(); len as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_single_step_smoke() {
        let calldata = vec![Value::from(27u128)];

        // Add opcode to add the value in address `0` and `1`
        // and place the output in address `2`
        let calldata_copy = Opcode::CalldataCopy {
            destination_address: MemoryAddress::from(0),
            size: 1,
            offset: 0,
        };

        // Start VM
        let opcodes = [calldata_copy];
        let mut vm = VM::new(calldata, &opcodes, vec![], &DummyBlackBoxSolver);

        // Process a single VM opcode
        //
        // After processing a single opcode, we should have
        // the vm status as finished since there is only one opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // The address at index `2` should have the value of 3 since we had an
        // add opcode
        let VM { memory, .. } = vm;
        let output_value = memory.read(MemoryAddress::from(0));

        assert_eq!(output_value, Value::from(27u128));
    }

    #[test]
    fn jmpif_opcode() {
        let mut calldata = vec![];
        let mut opcodes = vec![];

        let lhs = {
            calldata.push(Value::from(2u128));
            MemoryAddress::from(calldata.len() - 1)
        };

        let rhs = {
            calldata.push(Value::from(2u128));
            MemoryAddress::from(calldata.len() - 1)
        };

        let destination = MemoryAddress::from(calldata.len());

        opcodes.push(Opcode::CalldataCopy {
            destination_address: MemoryAddress::from(0),
            size: 2,
            offset: 0,
        });
        let equal_cmp_opcode =
            Opcode::BinaryIntOp { op: BinaryIntOp::Equals, bit_size: 1, lhs, rhs, destination };
        opcodes.push(equal_cmp_opcode);
        opcodes.push(Opcode::Jump { location: 3 });
        opcodes.push(Opcode::JumpIf { condition: MemoryAddress::from(2), location: 4 });

        let mut vm = VM::new(calldata, &opcodes, vec![], &DummyBlackBoxSolver);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.memory.read(MemoryAddress::from(2));
        assert_eq!(output_cmp_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });
    }

    #[test]
    fn jmpifnot_opcode() {
        let calldata = vec![Value::from(1u128), Value::from(2u128)];

        let calldata_copy = Opcode::CalldataCopy {
            destination_address: MemoryAddress::from(0),
            size: 2,
            offset: 0,
        };

        let jump_opcode = Opcode::Jump { location: 3 };

        let trap_opcode = Opcode::Trap;

        let not_equal_cmp_opcode = Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Equals,
            lhs: MemoryAddress::from(0),
            rhs: MemoryAddress::from(1),
            destination: MemoryAddress::from(2),
        };

        let jump_if_not_opcode =
            Opcode::JumpIfNot { condition: MemoryAddress::from(2), location: 2 };

        let add_opcode = Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Add,
            lhs: MemoryAddress::from(0),
            rhs: MemoryAddress::from(1),
            destination: MemoryAddress::from(2),
        };

        let opcodes = [
            calldata_copy,
            jump_opcode,
            trap_opcode,
            not_equal_cmp_opcode,
            jump_if_not_opcode,
            add_opcode,
        ];
        let mut vm = VM::new(calldata, &opcodes, vec![], &DummyBlackBoxSolver);
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.memory.read(MemoryAddress::from(2));
        assert_eq!(output_cmp_value, Value::from(false));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(
            status,
            VMStatus::Failure {
                message: "explicit trap hit in brillig".to_string(),
                call_stack: vec![2]
            }
        );

        // The address at index `2` should have not changed as we jumped over the add opcode
        let VM { memory, .. } = vm;
        let output_value = memory.read(MemoryAddress::from(2));
        assert_eq!(output_value, Value::from(false));
    }

    #[test]
    fn cast_opcode() {
        let calldata = vec![Value::from((2_u128.pow(32)) - 1)];

        let opcodes = &[
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::from(0),
                size: 1,
                offset: 0,
            },
            Opcode::Cast {
                destination: MemoryAddress::from(1),
                source: MemoryAddress::from(0),
                bit_size: 8,
            },
            Opcode::Stop { return_data_offset: 1, return_data_size: 1 },
        ];
        let mut vm = VM::new(calldata, opcodes, vec![], &DummyBlackBoxSolver);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 1, return_data_size: 1 });

        let VM { memory, .. } = vm;

        let casted_value = memory.read(MemoryAddress::from(1));
        assert_eq!(casted_value, Value::from(2_u128.pow(8) - 1));
    }

    #[test]
    fn mov_opcode() {
        let calldata = vec![Value::from(1u128), Value::from(2u128), Value::from(3u128)];

        let calldata_copy = Opcode::CalldataCopy {
            destination_address: MemoryAddress::from(0),
            size: 3,
            offset: 0,
        };

        let mov_opcode =
            Opcode::Mov { destination: MemoryAddress::from(2), source: MemoryAddress::from(0) };

        let opcodes = &[calldata_copy, mov_opcode];
        let mut vm = VM::new(calldata, opcodes, vec![], &DummyBlackBoxSolver);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let VM { memory, .. } = vm;

        let destination_value = memory.read(MemoryAddress::from(2));
        assert_eq!(destination_value, Value::from(1u128));

        let source_value = memory.read(MemoryAddress::from(0));
        assert_eq!(source_value, Value::from(1u128));
    }

    #[test]
    fn cmp_binary_ops() {
        let bit_size = 32;
        let calldata = vec![
            Value::from(2u128),
            Value::from(2u128),
            Value::from(0u128),
            Value::from(5u128),
            Value::from(6u128),
        ];

        let calldata_copy = Opcode::CalldataCopy {
            destination_address: MemoryAddress::from(0),
            size: 5,
            offset: 0,
        };

        let equal_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::Equals,
            lhs: MemoryAddress::from(0),
            rhs: MemoryAddress::from(1),
            destination: MemoryAddress::from(2),
        };

        let not_equal_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::Equals,
            lhs: MemoryAddress::from(0),
            rhs: MemoryAddress::from(3),
            destination: MemoryAddress::from(2),
        };

        let less_than_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::LessThan,
            lhs: MemoryAddress::from(3),
            rhs: MemoryAddress::from(4),
            destination: MemoryAddress::from(2),
        };

        let less_than_equal_opcode = Opcode::BinaryIntOp {
            bit_size,
            op: BinaryIntOp::LessThanEquals,
            lhs: MemoryAddress::from(3),
            rhs: MemoryAddress::from(4),
            destination: MemoryAddress::from(2),
        };

        let opcodes = [
            calldata_copy,
            equal_opcode,
            not_equal_opcode,
            less_than_opcode,
            less_than_equal_opcode,
        ];
        let mut vm = VM::new(calldata, &opcodes, vec![], &DummyBlackBoxSolver);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_eq_value = vm.memory.read(MemoryAddress::from(2));
        assert_eq!(output_eq_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_neq_value = vm.memory.read(MemoryAddress::from(2));
        assert_eq!(output_neq_value, Value::from(false));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let lt_value = vm.memory.read(MemoryAddress::from(2));
        assert_eq!(lt_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        let lte_value = vm.memory.read(MemoryAddress::from(2));
        assert_eq!(lte_value, Value::from(true));
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
        fn brillig_write_memory(item_count: usize) -> Vec<Value> {
            let bit_size = 32;
            let r_i = MemoryAddress::from(0);
            let r_len = MemoryAddress::from(1);
            let r_tmp = MemoryAddress::from(2);
            let r_pointer = MemoryAddress::from(3);

            let start = [
                // i = 0
                Opcode::Const { destination: r_i, value: 0u128.into(), bit_size: 32 },
                // len = memory.len() (approximation)
                Opcode::Const {
                    destination: r_len,
                    value: Value::from(item_count as u128),
                    bit_size: 32,
                },
                // pointer = free_memory_ptr
                Opcode::Const { destination: r_pointer, value: 4u128.into(), bit_size: 32 },
            ];
            let loop_body = [
                // *i = i
                Opcode::Store { destination_pointer: r_pointer, source: r_i },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size: 32 },
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
            let vm = brillig_execute_and_get_vm(vec![], &opcodes);
            vm.get_memory()[4..].to_vec()
        }

        let memory = brillig_write_memory(5);
        let expected = vec![
            Value::from(0u128),
            Value::from(1u128),
            Value::from(2u128),
            Value::from(3u128),
            Value::from(4u128),
        ];
        assert_eq!(memory, expected);

        let memory = brillig_write_memory(1024);
        let expected: Vec<Value> = (0..1024).map(|i| Value::from(i as u128)).collect();
        assert_eq!(memory, expected);
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
        fn brillig_sum_memory(memory: Vec<Value>) -> Value {
            let bit_size = 32;
            let r_i = MemoryAddress::from(0);
            let r_len = MemoryAddress::from(1);
            let r_sum = MemoryAddress::from(2);
            let r_tmp = MemoryAddress::from(3);
            let r_pointer = MemoryAddress::from(4);

            let start = [
                // sum = 0
                Opcode::Const { destination: r_sum, value: 0u128.into(), bit_size: 32 },
                // i = 0
                Opcode::Const { destination: r_i, value: 0u128.into(), bit_size: 32 },
                // len = array.len() (approximation)
                Opcode::Const {
                    destination: r_len,
                    value: Value::from(memory.len() as u128),
                    bit_size: 32,
                },
                // pointer = array_ptr
                Opcode::Const { destination: r_pointer, value: 5u128.into(), bit_size: 32 },
                Opcode::CalldataCopy {
                    destination_address: MemoryAddress(5),
                    size: memory.len(),
                    offset: 0,
                },
            ];
            let loop_body = [
                // tmp = *i
                Opcode::Load { destination: r_tmp, source_pointer: r_pointer },
                // sum = sum + tmp
                Opcode::BinaryIntOp {
                    destination: r_sum,
                    lhs: r_sum,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size,
                },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size: 32 },
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
            vm.memory.read(r_sum)
        }

        assert_eq!(
            brillig_sum_memory(vec![
                Value::from(1u128),
                Value::from(2u128),
                Value::from(3u128),
                Value::from(4u128),
                Value::from(5u128),
            ]),
            Value::from(15u128)
        );
        assert_eq!(brillig_sum_memory(vec![Value::from(1u128); 1024]), Value::from(1024u128));
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
        fn brillig_recursive_write_memory(size: usize) -> Vec<Value> {
            let bit_size = 32;
            let r_i = MemoryAddress::from(0);
            let r_len = MemoryAddress::from(1);
            let r_tmp = MemoryAddress::from(2);
            let r_pointer = MemoryAddress::from(3);

            let start = [
                // i = 0
                Opcode::Const { destination: r_i, value: 0u128.into(), bit_size: 32 },
                // len = size
                Opcode::Const { destination: r_len, value: size.into(), bit_size: 32 },
                // pointer = free_memory_ptr
                Opcode::Const { destination: r_pointer, value: 4u128.into(), bit_size: 32 },
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
                    bit_size,
                },
                // if !tmp, goto end
                Opcode::JumpIf {
                    condition: r_tmp,
                    location: start.len() + 7, // 8 ops in recursive_fn, go to 'Return'
                },
                // *i = i
                Opcode::Store { destination_pointer: r_pointer, source: r_i },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size: 32 },
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
                // call recursive_fn
                Opcode::Call { location: start.len() },
                Opcode::Return {},
            ];

            let opcodes = [&start[..], &recursive_fn[..]].concat();
            let vm = brillig_execute_and_get_vm(vec![], &opcodes);
            vm.get_memory()[4..].to_vec()
        }

        let memory = brillig_recursive_write_memory(5);
        let expected = vec![
            Value::from(0u128),
            Value::from(1u128),
            Value::from(2u128),
            Value::from(3u128),
            Value::from(4u128),
        ];
        assert_eq!(memory, expected);

        let memory = brillig_recursive_write_memory(1024);
        let expected: Vec<Value> = (0..1024).map(|i| Value::from(i as u128)).collect();
        assert_eq!(memory, expected);
    }

    /// Helper to execute brillig code
    fn brillig_execute_and_get_vm(
        calldata: Vec<Value>,
        opcodes: &[Opcode],
    ) -> VM<'_, DummyBlackBoxSolver> {
        let mut vm = VM::new(calldata, opcodes, vec![], &DummyBlackBoxSolver);
        brillig_execute(&mut vm);
        assert_eq!(vm.call_stack, vec![]);
        vm
    }

    fn brillig_execute(vm: &mut VM<DummyBlackBoxSolver>) {
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
        let r_input = MemoryAddress::from(0);
        let r_result = MemoryAddress::from(1);

        let double_program = vec![
            // Load input address with value 5
            Opcode::Const { destination: r_input, value: Value::from(5u128), bit_size: 32 },
            // Call foreign function "double" with the input address
            Opcode::ForeignCall {
                function: "double".into(),
                destinations: vec![ValueOrArray::MemoryAddress(r_result)],
                destination_value_types: vec![HeapValueType::Simple],
                inputs: vec![ValueOrArray::MemoryAddress(r_input)],
                input_value_types: vec![HeapValueType::Simple],
            },
        ];

        let mut vm = brillig_execute_and_get_vm(vec![], &double_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "double".into(),
                inputs: vec![Value::from(5u128).into()]
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(
            Value::from(10u128).into(), // Result of doubling 5u128
        );

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result address
        let result_value = vm.memory.read(r_result);
        assert_eq!(result_value, Value::from(10u128));

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_memory_result() {
        let r_input = MemoryAddress::from(0);
        let r_output = MemoryAddress::from(1);

        // Define a simple 2x2 matrix in memory
        let initial_matrix =
            vec![Value::from(1u128), Value::from(2u128), Value::from(3u128), Value::from(4u128)];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result =
            vec![Value::from(1u128), Value::from(3u128), Value::from(2u128), Value::from(4u128)];

        let invert_program = vec![
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::from(2),
                size: initial_matrix.len(),
                offset: 0,
            },
            // input = 0
            Opcode::Const { destination: r_input, value: 2_usize.into(), bit_size: 32 },
            // output = 0
            Opcode::Const { destination: r_output, value: 2_usize.into(), bit_size: 32 },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destinations: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_output,
                    size: initial_matrix.len(),
                })],
                destination_value_types: vec![HeapValueType::Array {
                    size: initial_matrix.len(),
                    value_types: vec![HeapValueType::Simple],
                }],
                inputs: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_input,
                    size: initial_matrix.len(),
                })],
                input_value_types: vec![HeapValueType::Array {
                    value_types: vec![HeapValueType::Simple],
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
        let result_values = vm.memory.read_slice(MemoryAddress(2), 4).to_vec();
        assert_eq!(result_values, expected_result);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    /// Calling a simple foreign call function that takes any string input, concatenates it with itself, and reverses the concatenation
    #[test]
    fn foreign_call_opcode_vector_input_and_output() {
        let r_input_pointer = MemoryAddress::from(0);
        let r_input_size = MemoryAddress::from(1);
        // We need to pass a location of appropriate size
        let r_output_pointer = MemoryAddress::from(2);
        let r_output_size = MemoryAddress::from(3);

        // Our first string to use the identity function with
        let input_string =
            vec![Value::from(1u128), Value::from(2u128), Value::from(3u128), Value::from(4u128)];
        // Double the string (concatenate it with itself)
        let mut output_string: Vec<Value> =
            input_string.iter().cloned().chain(input_string.clone()).collect();
        // Reverse the concatenated string
        output_string.reverse();

        // First call:
        let string_double_program = vec![
            Opcode::CalldataCopy {
                destination_address: MemoryAddress(4),
                size: input_string.len(),
                offset: 0,
            },
            // input_pointer = 4
            Opcode::Const { destination: r_input_pointer, value: Value::from(4u128), bit_size: 32 },
            // input_size = input_string.len() (constant here)
            Opcode::Const {
                destination: r_input_size,
                value: Value::from(input_string.len()),
                bit_size: 32,
            },
            // output_pointer = 4 + input_size
            Opcode::Const {
                destination: r_output_pointer,
                value: Value::from(4 + input_string.len()),
                bit_size: 32,
            },
            // output_size = input_size * 2
            Opcode::Const {
                destination: r_output_size,
                value: Value::from(input_string.len() * 2),
                bit_size: 32,
            },
            // output_pointer[0..output_size] = string_double(input_pointer[0...input_size])
            Opcode::ForeignCall {
                function: "string_double".into(),
                destinations: vec![ValueOrArray::HeapVector(HeapVector {
                    pointer: r_output_pointer,
                    size: r_output_size,
                })],
                destination_value_types: vec![HeapValueType::Vector {
                    value_types: vec![HeapValueType::Simple],
                }],
                inputs: vec![ValueOrArray::HeapVector(HeapVector {
                    pointer: r_input_pointer,
                    size: r_input_size,
                })],
                input_value_types: vec![HeapValueType::Vector {
                    value_types: vec![HeapValueType::Simple],
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
        let result_values = vm
            .memory
            .read_slice(MemoryAddress(4 + input_string.len()), output_string.len())
            .to_vec();
        assert_eq!(result_values, output_string);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_memory_alloc_result() {
        let r_input = MemoryAddress::from(0);
        let r_output = MemoryAddress::from(1);

        // Define a simple 2x2 matrix in memory
        let initial_matrix =
            vec![Value::from(1u128), Value::from(2u128), Value::from(3u128), Value::from(4u128)];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result =
            vec![Value::from(1u128), Value::from(3u128), Value::from(2u128), Value::from(4u128)];

        let invert_program = vec![
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::from(2),
                size: initial_matrix.len(),
                offset: 0,
            },
            // input = 0
            Opcode::Const { destination: r_input, value: Value::from(2u128), bit_size: 32 },
            // output = 0
            Opcode::Const { destination: r_output, value: Value::from(6u128), bit_size: 32 },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destinations: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_output,
                    size: initial_matrix.len(),
                })],
                destination_value_types: vec![HeapValueType::Array {
                    size: initial_matrix.len(),
                    value_types: vec![HeapValueType::Simple],
                }],
                inputs: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_input,
                    size: initial_matrix.len(),
                })],
                input_value_types: vec![HeapValueType::Array {
                    size: initial_matrix.len(),
                    value_types: vec![HeapValueType::Simple],
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
        let initial_values = vm.memory.read_slice(MemoryAddress(2), 4).to_vec();
        assert_eq!(initial_values, initial_matrix);

        // Check result in memory
        let result_values = vm.memory.read_slice(MemoryAddress(6), 4).to_vec();
        assert_eq!(result_values, expected_result);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_multiple_array_inputs_result() {
        let r_input_a = MemoryAddress::from(0);
        let r_input_b = MemoryAddress::from(1);
        let r_output = MemoryAddress::from(2);

        // Define a simple 2x2 matrix in memory
        let matrix_a =
            vec![Value::from(1u128), Value::from(2u128), Value::from(3u128), Value::from(4u128)];

        let matrix_b = vec![
            Value::from(10u128),
            Value::from(11u128),
            Value::from(12u128),
            Value::from(13u128),
        ];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result = vec![
            Value::from(34u128),
            Value::from(37u128),
            Value::from(78u128),
            Value::from(85u128),
        ];

        let matrix_mul_program = vec![
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::from(3),
                size: matrix_a.len() + matrix_b.len(),
                offset: 0,
            },
            // input = 3
            Opcode::Const { destination: r_input_a, value: Value::from(3u128), bit_size: 32 },
            // input = 7
            Opcode::Const { destination: r_input_b, value: Value::from(7u128), bit_size: 32 },
            // output = 0
            Opcode::Const { destination: r_output, value: Value::from(0u128), bit_size: 32 },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destinations: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_output,
                    size: matrix_a.len(),
                })],
                destination_value_types: vec![HeapValueType::Array {
                    size: matrix_a.len(),
                    value_types: vec![HeapValueType::Simple],
                }],
                inputs: vec![
                    ValueOrArray::HeapArray(HeapArray { pointer: r_input_a, size: matrix_a.len() }),
                    ValueOrArray::HeapArray(HeapArray { pointer: r_input_b, size: matrix_b.len() }),
                ],
                input_value_types: vec![
                    HeapValueType::Array {
                        size: matrix_a.len(),
                        value_types: vec![HeapValueType::Simple],
                    },
                    HeapValueType::Array {
                        size: matrix_b.len(),
                        value_types: vec![HeapValueType::Simple],
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
        let result_values = vm.memory.read_slice(MemoryAddress(0), 4).to_vec();
        assert_eq!(result_values, expected_result);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }

    #[test]
    fn foreign_call_opcode_nested_arrays_and_slices_input() {
        // [(1, <2,3>, [4]), (5, <6,7,8>, [9])]

        let v2 = vec![Value::from(2u128), Value::from(3u128)];
        let a4 = vec![Value::from(4u128)];
        let v6 = vec![Value::from(6u128), Value::from(7u128), Value::from(8u128)];
        let a9 = vec![Value::from(9u128)];

        // construct memory by declaring all inner arrays/vectors first
        let v2_ptr = 0u128;
        let mut memory = v2.clone();
        let v2_start = memory.len();
        memory.extend(vec![Value::from(v2_ptr), Value::from(v2.len()), Value::from(1u128)]);
        let a4_ptr = memory.len();
        memory.extend(a4.clone());
        let a4_start = memory.len();
        memory.extend(vec![Value::from(a4_ptr), Value::from(1u128)]);
        let v6_ptr = memory.len();
        memory.extend(v6.clone());
        let v6_start = memory.len();
        memory.extend(vec![Value::from(v6_ptr), Value::from(v6.len()), Value::from(1u128)]);
        let a9_ptr = memory.len();
        memory.extend(a9.clone());
        let a9_start = memory.len();
        memory.extend(vec![Value::from(a9_ptr), Value::from(1u128)]);
        // finally we add the contents of the outer array
        let outer_ptr = memory.len();
        let outer_array = vec![
            Value::from(1u128),
            Value::from(v2.len()),
            Value::from(v2_start),
            Value::from(a4_start),
            Value::from(5u128),
            Value::from(v6.len()),
            Value::from(v6_start),
            Value::from(a9_start),
        ];
        memory.extend(outer_array.clone());

        let input_array_value_types = vec![
            HeapValueType::Simple,
            HeapValueType::Simple, // size of following vector
            HeapValueType::Vector { value_types: vec![HeapValueType::Simple] },
            HeapValueType::Array { value_types: vec![HeapValueType::Simple], size: 1 },
        ];

        // memory address of the end of the above data structures
        let r_ptr = memory.len();

        let r_input = MemoryAddress::from(r_ptr);
        let r_output = MemoryAddress::from(r_ptr + 1);

        let program = vec![
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::from(0),
                size: memory.len(),
                offset: 0,
            },
            // input = 0
            Opcode::Const { destination: r_input, value: Value::from(outer_ptr), bit_size: 32 },
            // some_function(input)
            Opcode::ForeignCall {
                function: "flat_sum".into(),
                destinations: vec![ValueOrArray::MemoryAddress(r_output)],
                destination_value_types: vec![HeapValueType::Simple],
                inputs: vec![ValueOrArray::HeapArray(HeapArray {
                    pointer: r_input,
                    size: outer_array.len(),
                })],
                input_value_types: vec![HeapValueType::Array {
                    value_types: input_array_value_types,
                    size: outer_array.len(),
                }],
            },
        ];

        let mut vm = brillig_execute_and_get_vm(memory, &program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "flat_sum".into(),
                inputs: vec![ForeignCallParam::Array(vec![
                    Value::from(1u128),
                    Value::from(2u128), // size of following vector
                    Value::from(2u128),
                    Value::from(3u128),
                    Value::from(4u128),
                    Value::from(5u128),
                    Value::from(3u128), // size of following vector
                    Value::from(6u128),
                    Value::from(7u128),
                    Value::from(8u128),
                    Value::from(9u128),
                ])],
            }
        );

        // Push result we're waiting for
        vm.resolve_foreign_call(Value::from(45u128).into());

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

        // Check result
        let result_value = vm.memory.read(r_output);
        assert_eq!(result_value, Value::from(45u128));

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }
}
