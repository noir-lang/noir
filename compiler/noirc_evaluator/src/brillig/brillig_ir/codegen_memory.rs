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

            self.codegen_loop(num_elements_variable.address, |ctx, iterator| {
                ctx.codegen_load_with_offset(source_pointer, iterator, value_register);
                ctx.codegen_store_with_offset(destination_pointer, iterator, value_register);
            });

            self.deallocate_register(value_register);
        }
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
        self.codegen_usize_op(current_pointer, heap_vector.pointer, BrilligBinaryOp::Add, 1);

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

    /// Returns a pointer to the items of a given vector
    pub(crate) fn codegen_make_vector_items_pointer(
        &mut self,
        vector: BrilligVector,
    ) -> MemoryAddress {
        let result = self.allocate_register();
        self.codegen_usize_op(vector.pointer, result, BrilligBinaryOp::Add, 2);
        result
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

    /// Initializes a vector, allocating memory to store its representation and initializing the reference counter and size.
    pub(crate) fn codegen_initialize_vector(
        &mut self,
        vector: BrilligVector,
        size: SingleAddrVariable,
    ) {
        let allocation_size = self.allocate_register();
        self.codegen_usize_op(size.address, allocation_size, BrilligBinaryOp::Add, 2);
        self.codegen_allocate_mem(vector.pointer, allocation_size);
        self.deallocate_register(allocation_size);

        // Write RC
        self.indirect_const_instruction(
            vector.pointer,
            BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            1_usize.into(),
        );

        // Write size
        let len_write_pointer = self.allocate_register();
        self.codegen_usize_op(vector.pointer, len_write_pointer, BrilligBinaryOp::Add, 1);
        self.store_instruction(len_write_pointer, size.address);
        self.deallocate_register(len_write_pointer);
    }
}
