//! Implementation for [foreign calls][acir::brillig::Opcode::ForeignCall]
use acir::{
    AcirField,
    brillig::{
        BitSize, ForeignCallParam, HeapArray, HeapValueType, HeapVector, IntegerBitSize,
        MemoryAddress, ValueOrArray,
    },
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::{MemoryValue, VM, VMStatus};

impl<F: AcirField, B: BlackBoxFunctionSolver<F>> VM<'_, F, B> {
    /// Handles the execution of a single [ForeignCall opcode][acir::brillig::Opcode::ForeignCall].
    ///
    /// This method performs the following steps:
    /// 1. Checks if the foreign call results are already available. If not, it resolves the input
    ///    values from memory and pauses execution by returning `VMStatus::ForeignCallWait`.
    ///    For vectors, the preceding `u32` length field is used to truncate the slice input to its semantic length.
    /// 2. If results are available, it writes them to memory, ensuring that the returned data
    ///    matches the expected types and sizes. Nested arrays are reconstructed from flat
    ///    outputs when necessary. Nested vectors are an unsupported return type and will trigger an error.
    /// 3. Increments the foreign call counter and advances the program counter.
    ///
    /// # Parameters
    /// The borrowed fields of a [ForeignCall opcode][acir::brillig::Opcode::ForeignCall].
    /// They are listed again below:
    /// - `function`: Name of the foreign function being called.
    /// - `destinations`: Pointers or heap structures where the return values will be written.
    /// - `destination_value_types`: Expected type layout for each destination.
    /// - `inputs`: Pointers or heap structures representing the inputs for the foreign call.
    /// - `input_value_types`: Expected type layout for each input.
    ///
    /// # Returns
    /// - [VMStatus] indicating the next state of the VM:
    ///   - [VMStatus::ForeignCallWait] if the results are not yet available.
    ///   - [VMStatus::Finished] or [VMStatus::Failure] depending on whether writing the results succeeded.
    ///
    /// # Panics
    /// - If `inputs` and `input_value_types` lengths do not match.
    /// - If `destinations` and `destination_value_types` lengths do not match.
    pub(super) fn process_foreign_call(
        &mut self,
        function: &str,
        destinations: &[ValueOrArray],
        destination_value_types: &[HeapValueType],
        inputs: &[ValueOrArray],
        input_value_types: &[HeapValueType],
    ) -> VMStatus<F> {
        assert_eq!(inputs.len(), input_value_types.len());
        assert_eq!(destinations.len(), destination_value_types.len());

        if self.foreign_call_counter >= self.foreign_call_results.len() {
            // When this opcode is called, it is possible that the results of a foreign call are
            // not yet known (not enough entries in `foreign_call_results`).
            // If that is the case, just resolve the inputs and pause the VM with a status
            // (VMStatus::ForeignCallWait) that communicates the foreign function name and
            // resolved inputs back to the caller. Once the caller pushes to `foreign_call_results`,
            // they can then make another call to the VM that starts at this opcode
            // but has the necessary results to proceed with execution.

            // With slices we might have more items in the HeapVector than the semantic length
            // indicated by the field preceding the pointer to the vector in the inputs.
            // This happens when SSA merges slices of different length, which can result in
            // a vector that has room for the longer of the two, partially filled with items
            // from the shorter. There are ways to deal with this on the receiver side,
            // but it is cumbersome, and the cleanest solution is not to send the extra empty
            // items at all. To do this, however, we need infer which input is the vector length.
            let mut vector_length: Option<usize> = None;

            let resolved_inputs = inputs
                .iter()
                .zip(input_value_types)
                .map(|(input, input_type)| {
                    let mut input = self.get_memory_values(*input, input_type);
                    // Truncate slices to their semantic length, which we remember from the preceding field.
                    match input_type {
                        HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32)) => {
                            // If we have a single u32 we may have a slice representation, so store this input.
                            // On the next iteration, if we have a vector then we know we have the dynamic length
                            // for that slice.
                            let ForeignCallParam::Single(length) = input else {
                                unreachable!("expected u32; got {input:?}");
                            };
                            vector_length = Some(length.to_u128() as usize);
                        }
                        HeapValueType::Vector { value_types } => {
                            if let Some(length) = vector_length {
                                let type_size = vector_element_size(value_types);
                                let mut fields = input.fields();
                                fields.truncate(length * type_size);
                                input = ForeignCallParam::Array(fields);
                            }
                            vector_length = None;
                        }
                        _ => {
                            // Otherwise we are not dealing with a u32 followed by a vector.
                            vector_length = None;
                        }
                    }
                    input
                })
                .collect::<Vec<_>>();

            return self.wait_for_foreign_call(function.to_owned(), resolved_inputs);
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

    /// Get input data from memory to pass to foreign calls.
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

            // We want to send vectors to foreign functions truncated to their semantic length.
            let mut vector_length: Option<usize> = None;

            (0..size)
                .zip(value_types.iter().cycle())
                .map(|(i, value_type)| {
                    let value_address = start.offset(i);
                    let values = match value_type {
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
                    };
                    (value_type, values)
                })
                .flat_map(|(value_type, mut values)| {
                    match value_type {
                        HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32)) => {
                            vector_length = Some(values[0].to_usize());
                        }
                        HeapValueType::Vector { value_types } => {
                            if let Some(length) = vector_length {
                                let type_size = vector_element_size(value_types);
                                values.truncate(length * type_size);
                            }
                            vector_length = None;
                        }
                        _ => {
                            // Otherwise we are not dealing with a u32 followed by a vector.
                            vector_length = None;
                        }
                    }
                    values
                })
                .collect::<Vec<_>>()
        }
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

    /// Write a foreign call's results to the VM memory.
    ///
    /// We match the expected types with the actual results.
    /// However foreign call results do not support nested structures:
    /// They are either a single integer value or a vector of integer values (field elements).
    /// Therefore, nested arrays returned from foreign call results are flattened.
    /// If the expected array sizes do not match the actual size, we reconstruct the nested
    /// structure from the flat output array.
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

        debug_assert_eq!(
            destinations.len(),
            destination_value_types.len(),
            "Number of destinations must match number of value types",
        );
        debug_assert_eq!(
            destinations.len(),
            values.len(),
            "Number of foreign call return values must match number of destinations",
        );
        for ((destination, value_type), output) in
            destinations.iter().zip(destination_value_types).zip(&values)
        {
            match (destination, value_type) {
                (ValueOrArray::MemoryAddress(value_index), HeapValueType::Simple(bit_size)) => {
                    let output_fields = output.fields();
                    if value_type
                        .flattened_size()
                        .is_some_and(|flattened_size| output_fields.len() != flattened_size)
                    {
                        return Err(format!(
                            "Foreign call return value does not match expected size. Expected {} but got {}",
                            value_type.flattened_size().unwrap(),
                            output_fields.len(),
                        ));
                    }

                    match output {
                        ForeignCallParam::Single(value) => {
                            self.write_value_to_memory(*value_index, value, *bit_size)?;
                        }
                        _ => {
                            return Err(format!(
                                "Function result size does not match brillig bytecode. Expected 1 result but got {output:?}"
                            ));
                        }
                    }
                }
                (
                    ValueOrArray::HeapArray(HeapArray { pointer: pointer_index, size }),
                    HeapValueType::Array { value_types, size: type_size },
                ) if size == type_size => {
                    let output_fields = output.fields();
                    if value_type
                        .flattened_size()
                        .is_some_and(|flattened_size| output_fields.len() != flattened_size)
                    {
                        return Err(format!(
                            "Foreign call return value does not match expected size. Expected {} but got {}",
                            value_type.flattened_size().unwrap(),
                            output_fields.len(),
                        ));
                    }

                    if HeapValueType::all_simple(value_types) {
                        match output {
                            ForeignCallParam::Array(values) => {
                                if values.len() != *size {
                                    // foreign call returning flattened values into a nested type, so the sizes do not match
                                    let destination = self.memory.read_ref(*pointer_index);

                                    let mut flatten_values_idx = 0; //index of values read from flatten_values
                                    self.write_slice_of_values_to_memory(
                                        destination,
                                        &output_fields,
                                        &mut flatten_values_idx,
                                        value_type,
                                    )?;
                                    // Should be caught earlier but we want to be explicit.
                                    debug_assert_eq!(
                                        flatten_values_idx,
                                        output_fields.len(),
                                        "Not all values were written to memory"
                                    );
                                } else {
                                    self.write_values_to_memory_slice(
                                        *pointer_index,
                                        values,
                                        value_types,
                                    )?;
                                }
                            }
                            _ => {
                                return Err(
                                    "Function result size does not match brillig bytecode size"
                                        .to_string(),
                                );
                            }
                        }
                    } else {
                        // foreign call returning flattened values into a nested type, so the sizes do not match
                        let destination = self.memory.read_ref(*pointer_index);
                        let return_type = value_type;
                        let mut flatten_values_idx = 0; //index of values read from flatten_values
                        self.write_slice_of_values_to_memory(
                            destination,
                            &output_fields,
                            &mut flatten_values_idx,
                            return_type,
                        )?;
                        debug_assert_eq!(
                            flatten_values_idx,
                            output_fields.len(),
                            "Not all values were written to memory"
                        );
                    }
                }
                (
                    ValueOrArray::HeapVector(HeapVector {
                        pointer: pointer_index,
                        size: size_index,
                    }),
                    HeapValueType::Vector { value_types },
                ) => {
                    if HeapValueType::all_simple(value_types) {
                        match output {
                            ForeignCallParam::Array(values) => {
                                if values.len() % value_types.len() != 0 {
                                    return Err("Returned data does not match vector element size"
                                        .to_string());
                                }
                                // Set our size in the size address
                                self.memory.write(*size_index, values.len().into());
                                self.write_values_to_memory_slice(
                                    *pointer_index,
                                    values,
                                    value_types,
                                )?;
                            }
                            _ => {
                                return Err(
                                    "Function result size does not match brillig bytecode size"
                                        .to_string(),
                                );
                            }
                        }
                    } else {
                        unimplemented!("deflattening heap vectors from foreign calls");
                    }
                }
                _ => {
                    return Err(format!(
                        "Unexpected value type {value_type:?} for destination {destination:?}"
                    ));
                }
            }
        }

        self.foreign_call_results[foreign_call_index].values = values;

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
                "Foreign call result value {value} does not fit in bit size {value_bit_size:?}"
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
                "Foreign call result values {values:?} do not match expected bit sizes",
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
                                    "Unsupported returned type in foreign calls {typ:?}"
                                ));
                            }
                        }
                    }
                }
                Ok(())
            }
            HeapValueType::Vector { .. } => {
                Err(format!("Unsupported returned type in foreign calls {value_type:?}"))
            }
        }
    }
}

/// Returns the total number of field elements required to represent the elements in the vector in memory.
///
/// Panics if the vector contains nested vectors. Such types are not supported and are rejected by the frontend.
fn vector_element_size(value_types: &[HeapValueType]) -> usize {
    value_types
        .iter()
        .map(|typ| {
            typ.flattened_size()
                .unwrap_or_else(|| panic!("unexpected nested dynamic element type: {typ:?}"))
        })
        .sum()
}
