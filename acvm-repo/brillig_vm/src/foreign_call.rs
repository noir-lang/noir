//! Implementation for [foreign calls][acir::brillig::Opcode::ForeignCall]
use acir::{
    AcirField,
    brillig::{
        BitSize, ForeignCallParam, HeapArray, HeapValueType, HeapVector, IntegerBitSize,
        MemoryAddress, ValueOrArray,
    },
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::{
    FREE_MEMORY_POINTER_ADDRESS, MemoryValue, VM, VMStatus,
    memory::{ArrayAddress, VectorAddress},
    offsets,
};

impl<F: AcirField, B: BlackBoxFunctionSolver<F>> VM<'_, F, B> {
    /// Handles the execution of a single [ForeignCall opcode][acir::brillig::Opcode::ForeignCall].
    ///
    /// This method performs the following steps:
    /// 1. Checks if the foreign call results are already available. If not, it resolves the input
    ///    values from memory and pauses execution by returning `VMStatus::ForeignCallWait`.
    ///    For vectors, the preceding `u32` length field is used to truncate the slice input to its semantic length.
    /// 2. If results are available, it writes them to memory, ensuring that the returned data
    ///    matches the expected types and sizes:
    ///     * Nested arrays are reconstructed from flat outputs when necessary.
    ///     * Nested vectors are an unsupported return type and will trigger an error.
    ///     * Vectors are written to the heap starting at the free memory pointer, and their address gets stored in the destination.
    ///     * Update free memory pointer based on how much data (if any) was written to it.
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
    ) -> &VMStatus<F> {
        assert_eq!(inputs.len(), input_value_types.len());
        assert_eq!(destinations.len(), destination_value_types.len());

        if !self.has_unprocessed_foreign_call_result() {
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

        // Mark the foreign call result as processed.
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
            (ValueOrArray::MemoryAddress(value_addr), HeapValueType::Simple(_)) => {
                ForeignCallParam::Single(self.memory.read(value_addr).to_field())
            }
            (
                ValueOrArray::HeapArray(HeapArray { pointer, size }),
                HeapValueType::Array { value_types, size: type_size },
            ) if *type_size == size => {
                let start = self.memory.read_ref(pointer);
                self.read_slice_of_values_from_memory(start, size, value_types)
                    .into_iter()
                    .map(|mem_value| mem_value.to_field())
                    .collect::<Vec<_>>()
                    .into()
            }
            (
                ValueOrArray::HeapVector(HeapVector { pointer, size: size_addr }),
                HeapValueType::Vector { value_types },
            ) => {
                let start = self.memory.read_ref(pointer);
                let size = self.memory.read(size_addr).to_usize();
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
        assert!(start.is_direct(), "read_slice_of_values_from_memory requires direct addresses");
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
                            let array_address =
                                ArrayAddress::from(self.memory.read_ref(value_address));

                            self.read_slice_of_values_from_memory(
                                array_address.items_start(),
                                *size,
                                value_types,
                            )
                        }
                        HeapValueType::Vector { value_types } => {
                            let vector_address =
                                VectorAddress::from(self.memory.read_ref(value_address));

                            let side_addr = vector_address.size_addr();
                            let items_start = vector_address.items_start();
                            let vector_size = self.memory.read(side_addr).to_usize();
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
    ) -> &VMStatus<F> {
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
        // Take ownership of values to allow calling mutating methods on self.
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

        for ((destination, value_type), output) in
            destinations.iter().zip(destination_value_types).zip(&values)
        {
            match (destination, value_type) {
                (ValueOrArray::MemoryAddress(value_addr), HeapValueType::Simple(bit_size)) => {
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
                            self.write_value_to_memory(*value_addr, value, *bit_size)?;
                        }
                        _ => {
                            return Err(format!(
                                "Function result size does not match brillig bytecode. Expected 1 result but got {output:?}"
                            ));
                        }
                    }
                }
                (
                    ValueOrArray::HeapArray(HeapArray { pointer, size }),
                    HeapValueType::Array { value_types, size: type_size },
                ) => {
                    if size != type_size {
                        return Err(format!(
                            "Destination array size of {size} does not match the type size of {type_size}"
                        ));
                    }
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
                        let ForeignCallParam::Array(values) = output else {
                            return Err("Foreign call returned a single value for an array type"
                                .to_string());
                        };
                        if values.len() != *size {
                            // foreign call returning flattened values into a nested type, so the sizes do not match
                            let destination = self.memory.read_ref(*pointer);

                            let mut flatten_values_idx = 0; //index of values read from flatten_values
                            self.write_flattened_values_to_memory(
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
                            self.write_values_to_memory(*pointer, true, values, value_types)?;
                        }
                    } else {
                        // foreign call returning flattened values into a nested type, so the sizes do not match
                        let destination = self.memory.read_ref(*pointer);
                        let mut flatten_values_idx = 0; //index of values read from flatten_values
                        self.write_flattened_values_to_memory(
                            destination,
                            &output_fields,
                            &mut flatten_values_idx,
                            value_type,
                        )?;
                        debug_assert_eq!(
                            flatten_values_idx,
                            output_fields.len(),
                            "Not all values were written to memory"
                        );
                    }
                }
                (
                    ValueOrArray::MemoryAddress(vector_pointer),
                    HeapValueType::Vector { value_types },
                ) if self.version().enable_foreign_call_multi_vector_output() => {
                    if HeapValueType::all_simple(value_types) {
                        let ForeignCallParam::Array(values) = output else {
                            return Err("Foreign call returned a single value for an vector type"
                                .to_string());
                        };
                        if values.len() % value_types.len() != 0 {
                            return Err(
                                "Returned data does not match vector element size".to_string()
                            );
                        }

                        // We write the data to the current free memory pointer.
                        let free_memory_addr = self.memory.read_ref(FREE_MEMORY_POINTER_ADDRESS);

                        // Store the address itself back in the destination.
                        self.memory.write_ref(*vector_pointer, free_memory_addr);

                        // Set the size in the size address and write the data.
                        // The RC and the capacity will be initialized in codegen after the call.
                        let vector_address = VectorAddress::from(free_memory_addr);
                        self.memory.write(vector_address.size_addr(), values.len().into());
                        self.write_values_to_memory(
                            vector_address.items_start(),
                            false,
                            values,
                            value_types,
                        )?;

                        // Increase the free memory pointer by the amount of space taken by the vector, including metadata,
                        // so the next vector can go to after where this was written.
                        let total_size = offsets::VECTOR_META_COUNT + values.len();
                        self.memory.write_ref(
                            FREE_MEMORY_POINTER_ADDRESS,
                            free_memory_addr.offset(total_size),
                        );
                    } else {
                        unimplemented!("deflattening heap vectors from foreign calls");
                    }
                }
                // Legacy way of handling output vectors, that doesn't require the VM to know about the Free Memory Pointer.
                (
                    ValueOrArray::HeapVector(HeapVector { pointer, size: size_addr }),
                    HeapValueType::Vector { value_types },
                ) if !self.version().enable_foreign_call_multi_vector_output() => {
                    if HeapValueType::all_simple(value_types) {
                        let ForeignCallParam::Array(values) = output else {
                            return Err("Foreign call returned a single value for an vector type"
                                .to_string());
                        };
                        if values.len() % value_types.len() != 0 {
                            return Err(
                                "Returned data does not match vector element size".to_string()
                            );
                        }
                        // Set the size in the size address
                        self.memory.write(*size_addr, values.len().into());
                        self.write_values_to_memory(*pointer, true, values, value_types)?;
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

    /// Write a single numeric value to the destination address, ensuring that the bit size matches the expectation.
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

    /// Write an array or slice either directly to an address, or indirectly to a destination pointed at by that address.
    fn write_values_to_memory(
        &mut self,
        address: MemoryAddress,
        is_pointer: bool,
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

        // Convert the destination pointer to an address.
        let destination = if is_pointer { self.memory.read_ref(address) } else { address };

        // Write to the destination memory.
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

    /// Writes flattened values to memory, using the provided type.
    ///
    /// The method calls itself recursively in order to work with recursive types (nested arrays).
    /// `values_idx` is the current index in the `values` vector and is incremented every time
    /// a value is written to memory.
    fn write_flattened_values_to_memory(
        &mut self,
        destination: MemoryAddress,
        values: &Vec<F>,
        values_idx: &mut usize,
        value_type: &HeapValueType,
    ) -> Result<(), String> {
        assert!(
            destination.is_direct(),
            "write_flattened_values_to_memory requires direct addresses"
        );
        match value_type {
            HeapValueType::Simple(bit_size) => {
                self.write_value_to_memory(destination, &values[*values_idx], *bit_size)?;
                *values_idx += 1;
                Ok(())
            }
            HeapValueType::Array { value_types, size } => {
                let mut current_pointer = destination;
                for _ in 0..*size {
                    for typ in value_types {
                        match typ {
                            HeapValueType::Simple(bit_size) => {
                                self.write_value_to_memory(
                                    current_pointer,
                                    &values[*values_idx],
                                    *bit_size,
                                )?;
                                *values_idx += 1;
                                current_pointer = current_pointer.offset(1);
                            }
                            HeapValueType::Array { .. } => {
                                // The next memory destination is an array, somewhere else in memory where the pointer points to.
                                let destination =
                                    ArrayAddress::from(self.memory.read_ref(current_pointer));

                                self.write_flattened_values_to_memory(
                                    destination.items_start(),
                                    values,
                                    values_idx,
                                    typ,
                                )?;

                                // Move on to the next slot in *this* array.
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
