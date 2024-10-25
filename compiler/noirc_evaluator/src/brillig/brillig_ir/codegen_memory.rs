use acvm::{
    acir::brillig::{HeapArray, HeapVector, MemoryAddress, ValueOrArray},
    AcirField,
};

use crate::brillig::brillig_ir::BrilligBinaryOp;

use super::{
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::RegisterAllocator,
    BrilligContext, ReservedRegisters, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn codegen_allocate_immediate_mem(
        &mut self,
        pointer_register: MemoryAddress,
        size: usize,
    ) {
        self.load_free_memory_pointer_instruction(pointer_register);
        self.codegen_usize_op_in_place(
            ReservedRegisters::free_memory_pointer(),
            BrilligBinaryOp::Add,
            size,
        );
    }

    /// Allocates an array of size contained in size_register and stores the
    /// pointer to the array in `pointer_register`
    pub(crate) fn codegen_allocate_mem(
        &mut self,
        pointer_register: MemoryAddress,
        size_register: MemoryAddress,
    ) {
        self.load_free_memory_pointer_instruction(pointer_register);
        self.increase_free_memory_pointer_instruction(size_register);
    }

    /// Gets the value stored at base_ptr + index and stores it in result
    pub(crate) fn codegen_load_with_offset(
        &mut self,
        base_ptr: MemoryAddress,
        index: SingleAddrVariable,
        result: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        let final_index = self.allocate_register();
        self.memory_op_instruction(base_ptr, index.address, final_index, BrilligBinaryOp::Add);
        self.load_instruction(result, final_index);
        // Free up temporary register
        self.deallocate_register(final_index);
    }

    /// Stores value at base_ptr + index
    pub(crate) fn codegen_store_with_offset(
        &mut self,
        base_ptr: MemoryAddress,
        index: SingleAddrVariable,
        value: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        let final_index = self.allocate_register();
        self.binary_instruction(
            SingleAddrVariable::new_usize(base_ptr),
            index,
            SingleAddrVariable::new_usize(final_index),
            BrilligBinaryOp::Add,
        );

        self.store_instruction(final_index, value);
        // Free up temporary register
        self.deallocate_register(final_index);
    }

    /// Copies the values of memory pointed by source with length stored in `num_elements_register`
    /// After the address pointed by destination
    pub(crate) fn codegen_mem_copy(
        &mut self,
        source_pointer: MemoryAddress,
        destination_pointer: MemoryAddress,
        num_elements_variable: SingleAddrVariable,
    ) {
        assert!(num_elements_variable.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);

        if self.can_call_procedures {
            self.call_mem_copy_procedure(
                source_pointer,
                destination_pointer,
                num_elements_variable.address,
            );
        } else {
            let value_register = self.allocate_register();
            let end_source_pointer = self.allocate_register();
            self.memory_op_instruction(
                source_pointer,
                num_elements_variable.address,
                end_source_pointer,
                BrilligBinaryOp::Add,
            );

            self.codegen_generic_iteration(
                |brillig_context| {
                    let source_iterator = brillig_context.allocate_register();
                    let target_iterator = brillig_context.allocate_register();

                    brillig_context.mov_instruction(source_iterator, source_pointer);
                    brillig_context.mov_instruction(target_iterator, destination_pointer);

                    (source_iterator, target_iterator)
                },
                |brillig_context, &(source_iterator, target_iterator)| {
                    brillig_context.codegen_usize_op_in_place(
                        source_iterator,
                        BrilligBinaryOp::Add,
                        1,
                    );
                    brillig_context.codegen_usize_op_in_place(
                        target_iterator,
                        BrilligBinaryOp::Add,
                        1,
                    );
                },
                |brillig_context, &(source_iterator, _)| {
                    // We have finished when the source/target pointer is less than the source/target start
                    let finish_condition =
                        SingleAddrVariable::new(brillig_context.allocate_register(), 1);
                    brillig_context.memory_op_instruction(
                        source_iterator,
                        end_source_pointer,
                        finish_condition.address,
                        BrilligBinaryOp::Equals,
                    );
                    finish_condition
                },
                |brillig_context, &(source_iterator, target_iterator)| {
                    brillig_context.load_instruction(value_register, source_iterator);
                    brillig_context.store_instruction(target_iterator, value_register);
                },
                |brillig_context, (source_iterator, target_iterator)| {
                    brillig_context.deallocate_register(source_iterator);
                    brillig_context.deallocate_register(target_iterator);
                },
            );
            self.deallocate_register(value_register);
            self.deallocate_register(end_source_pointer);
        }
    }

    /// Copies num_elements_variable from the source pointer to the target pointer, starting from the end
    pub(crate) fn codegen_mem_copy_from_the_end(
        &mut self,
        source_start: MemoryAddress,
        target_start: MemoryAddress,
        num_elements_variable: SingleAddrVariable,
    ) {
        self.codegen_generic_iteration(
            |brillig_context| {
                // Create the pointer to the last item for both source and target
                let num_items_minus_one = brillig_context.allocate_register();
                brillig_context.codegen_usize_op(
                    num_elements_variable.address,
                    num_items_minus_one,
                    BrilligBinaryOp::Sub,
                    1,
                );
                let target_pointer = brillig_context.allocate_register();
                brillig_context.memory_op_instruction(
                    target_start,
                    num_items_minus_one,
                    target_pointer,
                    BrilligBinaryOp::Add,
                );
                let source_pointer = brillig_context.allocate_register();
                brillig_context.memory_op_instruction(
                    source_start,
                    num_items_minus_one,
                    source_pointer,
                    BrilligBinaryOp::Add,
                );
                brillig_context.deallocate_register(num_items_minus_one);
                (source_pointer, target_pointer)
            },
            |brillig_context, &(source_pointer, target_pointer)| {
                brillig_context.codegen_usize_op_in_place(source_pointer, BrilligBinaryOp::Sub, 1);
                brillig_context.codegen_usize_op_in_place(target_pointer, BrilligBinaryOp::Sub, 1);
            },
            |brillig_context, &(source_pointer, _)| {
                // We have finished when the source/target pointer is less than the source/target start
                let finish_condition =
                    SingleAddrVariable::new(brillig_context.allocate_register(), 1);
                brillig_context.memory_op_instruction(
                    source_pointer,
                    source_start,
                    finish_condition.address,
                    BrilligBinaryOp::LessThan,
                );
                finish_condition
            },
            |brillig_context, &(source_pointer, target_pointer)| {
                let value_register = brillig_context.allocate_register();
                brillig_context.load_instruction(value_register, source_pointer);
                brillig_context.store_instruction(target_pointer, value_register);
                brillig_context.deallocate_register(value_register);
            },
            |brillig_context, (source_pointer, target_pointer)| {
                brillig_context.deallocate_register(source_pointer);
                brillig_context.deallocate_register(target_pointer);
            },
        );
    }

    /// This instruction will reverse the order of the `size` elements pointed by `pointer`.
    pub(crate) fn codegen_array_reverse(
        &mut self,
        items_pointer: MemoryAddress,
        size: MemoryAddress,
    ) {
        if self.can_call_procedures {
            self.call_array_reverse_procedure(items_pointer, size);
            return;
        }

        let iteration_count = self.allocate_register();
        self.codegen_usize_op(size, iteration_count, BrilligBinaryOp::UnsignedDiv, 2);

        let start_value_register = self.allocate_register();
        let end_value_register = self.allocate_register();
        let index_at_end_of_array = self.allocate_register();

        self.mov_instruction(index_at_end_of_array, size);

        self.codegen_loop(iteration_count, |ctx, iterator_register| {
            // The index at the end of array is size - 1 - iterator
            ctx.codegen_usize_op_in_place(index_at_end_of_array, BrilligBinaryOp::Sub, 1);
            let index_at_end_of_array_var = SingleAddrVariable::new_usize(index_at_end_of_array);

            // Load both values
            ctx.codegen_load_with_offset(items_pointer, iterator_register, start_value_register);
            ctx.codegen_load_with_offset(
                items_pointer,
                index_at_end_of_array_var,
                end_value_register,
            );

            // Write both values
            ctx.codegen_store_with_offset(items_pointer, iterator_register, end_value_register);
            ctx.codegen_store_with_offset(
                items_pointer,
                index_at_end_of_array_var,
                start_value_register,
            );
        });

        self.deallocate_register(iteration_count);
        self.deallocate_register(start_value_register);
        self.deallocate_register(end_value_register);
        self.deallocate_register(index_at_end_of_array);
    }

    /// Converts a BrilligArray (pointer to [RC, ...items]) to a HeapArray (pointer to [items])
    pub(crate) fn codegen_brillig_array_to_heap_array(&mut self, array: BrilligArray) -> HeapArray {
        let heap_array = HeapArray { pointer: self.allocate_register(), size: array.size };
        self.codegen_usize_op(array.pointer, heap_array.pointer, BrilligBinaryOp::Add, 1);
        heap_array
    }

    pub(crate) fn codegen_brillig_vector_to_heap_vector(
        &mut self,
        vector: BrilligVector,
    ) -> HeapVector {
        let heap_vector =
            HeapVector { pointer: self.allocate_register(), size: self.allocate_register() };
        let current_pointer = self.allocate_register();

        // Prepare a pointer to the size
        self.codegen_usize_op(vector.pointer, current_pointer, BrilligBinaryOp::Add, 1);
        self.load_instruction(heap_vector.size, current_pointer);
        // Now prepare the pointer to the items
        self.codegen_usize_op(current_pointer, heap_vector.pointer, BrilligBinaryOp::Add, 2);

        self.deallocate_register(current_pointer);
        heap_vector
    }

    pub(crate) fn variable_to_value_or_array(&mut self, variable: BrilligVariable) -> ValueOrArray {
        match variable {
            BrilligVariable::SingleAddr(SingleAddrVariable { address, .. }) => {
                ValueOrArray::MemoryAddress(address)
            }
            BrilligVariable::BrilligArray(array) => {
                ValueOrArray::HeapArray(self.codegen_brillig_array_to_heap_array(array))
            }
            BrilligVariable::BrilligVector(vector) => {
                ValueOrArray::HeapVector(self.codegen_brillig_vector_to_heap_vector(vector))
            }
        }
    }

    /// Returns a variable holding the length of a given vector
    pub(crate) fn codegen_make_vector_length(
        &mut self,
        vector: BrilligVector,
    ) -> SingleAddrVariable {
        let result = SingleAddrVariable::new_usize(self.allocate_register());
        self.codegen_usize_op(vector.pointer, result.address, BrilligBinaryOp::Add, 1);
        self.load_instruction(result.address, result.address);
        result
    }

    pub(crate) fn codegen_update_vector_length(
        &mut self,
        vector: BrilligVector,
        new_length: SingleAddrVariable,
    ) {
        let write_pointer = self.allocate_register();
        self.codegen_usize_op(vector.pointer, write_pointer, BrilligBinaryOp::Add, 1);
        self.store_instruction(write_pointer, new_length.address);
        self.deallocate_register(write_pointer);
    }

    /// Returns a variable holding the capacity of a given vector
    pub(crate) fn codegen_make_vector_capacity(
        &mut self,
        vector: BrilligVector,
    ) -> SingleAddrVariable {
        let result = SingleAddrVariable::new_usize(self.allocate_register());
        self.codegen_usize_op(vector.pointer, result.address, BrilligBinaryOp::Add, 2);
        self.load_instruction(result.address, result.address);
        result
    }

    /// Writes a pointer to the items of a given vector
    pub(crate) fn codegen_vector_items_pointer(
        &mut self,
        vector: BrilligVector,
        result: MemoryAddress,
    ) {
        self.codegen_usize_op(vector.pointer, result, BrilligBinaryOp::Add, 3);
    }

    /// Returns a pointer to the items of a given vector
    pub(crate) fn codegen_make_vector_items_pointer(
        &mut self,
        vector: BrilligVector,
    ) -> MemoryAddress {
        let result = self.allocate_register();
        self.codegen_vector_items_pointer(vector, result);
        result
    }

    /// Reads the metadata of a vector and stores it in the given variables
    pub(crate) fn codegen_read_vector_metadata(
        &mut self,
        vector: BrilligVector,
        rc: SingleAddrVariable,
        size: SingleAddrVariable,
        capacity: SingleAddrVariable,
        items_pointer: SingleAddrVariable,
    ) {
        assert!(rc.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        assert!(size.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        assert!(capacity.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        assert!(items_pointer.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);

        self.load_instruction(rc.address, vector.pointer);

        let read_pointer = self.allocate_register();
        self.codegen_usize_op(vector.pointer, read_pointer, BrilligBinaryOp::Add, 1);
        self.load_instruction(size.address, read_pointer);
        self.codegen_usize_op_in_place(read_pointer, BrilligBinaryOp::Add, 1);
        self.load_instruction(capacity.address, read_pointer);
        self.codegen_usize_op(read_pointer, items_pointer.address, BrilligBinaryOp::Add, 1);

        self.deallocate_register(read_pointer);
    }

    /// Returns a variable holding the length of a given array
    pub(crate) fn codegen_make_array_length(&mut self, array: BrilligArray) -> SingleAddrVariable {
        let result = SingleAddrVariable::new_usize(self.allocate_register());
        self.usize_const_instruction(result.address, array.size.into());
        result
    }

    /// Returns a pointer to the items of a given array
    pub(crate) fn codegen_make_array_items_pointer(
        &mut self,
        array: BrilligArray,
    ) -> MemoryAddress {
        let result = self.allocate_register();
        self.codegen_usize_op(array.pointer, result, BrilligBinaryOp::Add, 1);
        result
    }

    pub(crate) fn codegen_make_array_or_vector_length(
        &mut self,
        variable: BrilligVariable,
    ) -> SingleAddrVariable {
        match variable {
            BrilligVariable::BrilligArray(array) => self.codegen_make_array_length(array),
            BrilligVariable::BrilligVector(vector) => self.codegen_make_vector_length(vector),
            _ => unreachable!("ICE: Expected array or vector, got {variable:?}"),
        }
    }

    pub(crate) fn codegen_make_array_or_vector_items_pointer(
        &mut self,
        variable: BrilligVariable,
    ) -> MemoryAddress {
        match variable {
            BrilligVariable::BrilligArray(array) => self.codegen_make_array_items_pointer(array),
            BrilligVariable::BrilligVector(vector) => {
                self.codegen_make_vector_items_pointer(vector)
            }
            _ => unreachable!("ICE: Expected array or vector, got {variable:?}"),
        }
    }

    /// Initializes an array, allocating memory to store its representation and initializing the reference counter.
    pub(crate) fn codegen_initialize_array(&mut self, array: BrilligArray) {
        self.codegen_allocate_immediate_mem(array.pointer, array.size + 1);
        self.indirect_const_instruction(
            array.pointer,
            BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            1_usize.into(),
        );
    }

    pub(crate) fn codegen_initialize_vector_metadata(
        &mut self,
        vector: BrilligVector,
        size: SingleAddrVariable,
        capacity: Option<SingleAddrVariable>,
    ) {
        // Write RC
        self.indirect_const_instruction(
            vector.pointer,
            BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            1_usize.into(),
        );

        // Write size
        let write_pointer = self.allocate_register();
        self.codegen_usize_op(vector.pointer, write_pointer, BrilligBinaryOp::Add, 1);
        self.store_instruction(write_pointer, size.address);

        // Write capacity
        self.codegen_usize_op_in_place(write_pointer, BrilligBinaryOp::Add, 1);
        self.store_instruction(write_pointer, capacity.unwrap_or(size).address);

        self.deallocate_register(write_pointer);
    }

    /// Initializes a vector, allocating memory to store its representation and initializing the reference counter, size and capacity
    pub(crate) fn codegen_initialize_vector(
        &mut self,
        vector: BrilligVector,
        size: SingleAddrVariable,
        capacity: Option<SingleAddrVariable>, // Defaults to size if None
    ) {
        let allocation_size = self.allocate_register();
        // Allocation size = capacity + 3 (rc, size, capacity)
        self.codegen_usize_op(
            capacity.unwrap_or(size).address,
            allocation_size,
            BrilligBinaryOp::Add,
            3,
        );
        self.codegen_allocate_mem(vector.pointer, allocation_size);
        self.deallocate_register(allocation_size);

        self.codegen_initialize_vector_metadata(vector, size, capacity);
    }

    /// We don't know the length of a vector returned externally before the call
    /// so we pass the free memory pointer and then use this function to allocate
    /// after the fact when we know the length.
    pub(crate) fn initialize_externally_returned_vector(
        &mut self,
        vector: BrilligVector,
        resulting_heap_vector: HeapVector,
    ) {
        let total_size = self.allocate_register();
        self.codegen_usize_op(
            resulting_heap_vector.size,
            total_size,
            BrilligBinaryOp::Add,
            3, // Rc, length and capacity
        );

        self.increase_free_memory_pointer_instruction(total_size);
        let write_pointer = self.allocate_register();

        // Vectors are [RC, Size, Capacity, ...items]
        // Initialize RC
        self.indirect_const_instruction(
            vector.pointer,
            BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            1_usize.into(),
        );

        // Initialize size
        self.codegen_usize_op(vector.pointer, write_pointer, BrilligBinaryOp::Add, 1_usize);
        self.store_instruction(write_pointer, resulting_heap_vector.size);

        // Initialize capacity
        self.codegen_usize_op_in_place(write_pointer, BrilligBinaryOp::Add, 1_usize);
        self.store_instruction(write_pointer, resulting_heap_vector.size);

        self.deallocate_register(write_pointer);
        self.deallocate_register(total_size);
    }
}
