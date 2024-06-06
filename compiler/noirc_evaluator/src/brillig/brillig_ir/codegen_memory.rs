use acvm::acir::brillig::MemoryAddress;

use crate::brillig::brillig_ir::BrilligBinaryOp;

use super::{
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    BrilligContext, ReservedRegisters, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};

impl BrilligContext {
    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn codegen_allocate_fixed_length_array(
        &mut self,
        pointer_register: MemoryAddress,
        size: usize,
    ) {
        let size_register = self.make_usize_constant_instruction(size.into());
        self.codegen_allocate_array(pointer_register, size_register.address);
        self.deallocate_single_addr(size_register);
    }

    /// Allocates an array of size contained in size_register and stores the
    /// pointer to the array in `pointer_register`
    pub(crate) fn codegen_allocate_array(
        &mut self,
        pointer_register: MemoryAddress,
        size_register: MemoryAddress,
    ) {
        self.load_free_memory_pointer_instruction(pointer_register);
        self.increase_free_memory_pointer_instruction(size_register);
    }

    /// Allocates a variable in memory and stores the
    /// pointer to the array in `pointer_register`
    fn codegen_allocate_variable_reference(
        &mut self,
        pointer_register: MemoryAddress,
        size: usize,
    ) {
        // A variable can be stored in up to three values, so we reserve three values for that.
        let size_register = self.make_usize_constant_instruction(size.into());
        self.mov_instruction(pointer_register, ReservedRegisters::free_memory_pointer());
        self.memory_op_instruction(
            ReservedRegisters::free_memory_pointer(),
            size_register.address,
            ReservedRegisters::free_memory_pointer(),
            BrilligBinaryOp::Add,
        );
        self.deallocate_single_addr(size_register);
    }

    pub(crate) fn codegen_allocate_single_addr_reference(
        &mut self,
        pointer_register: MemoryAddress,
    ) {
        self.codegen_allocate_variable_reference(pointer_register, 1);
    }

    pub(crate) fn codegen_allocate_array_reference(&mut self, pointer_register: MemoryAddress) {
        self.codegen_allocate_variable_reference(pointer_register, BrilligArray::registers_count());
    }

    pub(crate) fn codegen_allocate_vector_reference(&mut self, pointer_register: MemoryAddress) {
        self.codegen_allocate_variable_reference(
            pointer_register,
            BrilligVector::registers_count(),
        );
    }

    /// Gets the value in the array at index `index` and stores it in `result`
    pub(crate) fn codegen_array_get(
        &mut self,
        array_ptr: MemoryAddress,
        index: SingleAddrVariable,
        result: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.memory_op_instruction(
            array_ptr,
            index.address,
            index_of_element_in_memory,
            BrilligBinaryOp::Add,
        );
        self.load_instruction(result, index_of_element_in_memory);
        // Free up temporary register
        self.deallocate_register(index_of_element_in_memory);
    }

    /// Sets the item in the array at index `index` to `value`
    pub(crate) fn codegen_array_set(
        &mut self,
        array_ptr: MemoryAddress,
        index: SingleAddrVariable,
        value: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.binary_instruction(
            SingleAddrVariable::new_usize(array_ptr),
            index,
            SingleAddrVariable::new_usize(index_of_element_in_memory),
            BrilligBinaryOp::Add,
        );

        self.store_instruction(index_of_element_in_memory, value);
        // Free up temporary register
        self.deallocate_register(index_of_element_in_memory);
    }

    /// Copies the values of an array pointed by source with length stored in `num_elements_register`
    /// Into the array pointed by destination
    pub(crate) fn codegen_copy_array(
        &mut self,
        source_pointer: MemoryAddress,
        destination_pointer: MemoryAddress,
        num_elements_variable: SingleAddrVariable,
    ) {
        assert!(num_elements_variable.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);

        let value_register = self.allocate_register();

        self.codegen_loop(num_elements_variable.address, |ctx, iterator| {
            ctx.codegen_array_get(source_pointer, iterator, value_register);
            ctx.codegen_array_set(destination_pointer, iterator, value_register);
        });

        self.deallocate_register(value_register);
    }

    /// Loads a variable stored previously
    pub(crate) fn codegen_load_variable(
        &mut self,
        destination: BrilligVariable,
        variable_pointer: MemoryAddress,
    ) {
        match destination {
            BrilligVariable::SingleAddr(single_addr) => {
                self.load_instruction(single_addr.address, variable_pointer);
            }
            BrilligVariable::BrilligArray(BrilligArray { pointer, size: _, rc }) => {
                self.load_instruction(pointer, variable_pointer);

                let rc_pointer = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.codegen_usize_op_in_place(rc_pointer, BrilligBinaryOp::Add, 1_usize);

                self.load_instruction(rc, rc_pointer);
                self.deallocate_register(rc_pointer);
            }
            BrilligVariable::BrilligVector(BrilligVector { pointer, size, rc }) => {
                self.load_instruction(pointer, variable_pointer);

                let size_pointer = self.allocate_register();
                self.mov_instruction(size_pointer, variable_pointer);
                self.codegen_usize_op_in_place(size_pointer, BrilligBinaryOp::Add, 1_usize);

                self.load_instruction(size, size_pointer);
                self.deallocate_register(size_pointer);

                let rc_pointer = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.codegen_usize_op_in_place(rc_pointer, BrilligBinaryOp::Add, 2_usize);

                self.load_instruction(rc, rc_pointer);
                self.deallocate_register(rc_pointer);
            }
        }
    }

    /// Stores a variable by saving its registers to memory
    pub(crate) fn codegen_store_variable(
        &mut self,
        variable_pointer: MemoryAddress,
        source: BrilligVariable,
    ) {
        match source {
            BrilligVariable::SingleAddr(single_addr) => {
                self.store_instruction(variable_pointer, single_addr.address);
            }
            BrilligVariable::BrilligArray(BrilligArray { pointer, size: _, rc }) => {
                self.store_instruction(variable_pointer, pointer);

                let rc_pointer: MemoryAddress = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.codegen_usize_op_in_place(rc_pointer, BrilligBinaryOp::Add, 1_usize);
                self.store_instruction(rc_pointer, rc);
                self.deallocate_register(rc_pointer);
            }
            BrilligVariable::BrilligVector(BrilligVector { pointer, size, rc }) => {
                self.store_instruction(variable_pointer, pointer);

                let size_pointer = self.allocate_register();
                self.mov_instruction(size_pointer, variable_pointer);
                self.codegen_usize_op_in_place(size_pointer, BrilligBinaryOp::Add, 1_usize);
                self.store_instruction(size_pointer, size);

                let rc_pointer: MemoryAddress = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.codegen_usize_op_in_place(rc_pointer, BrilligBinaryOp::Add, 2_usize);
                self.store_instruction(rc_pointer, rc);

                self.deallocate_register(size_pointer);
                self.deallocate_register(rc_pointer);
            }
        }
    }

    /// This instruction will reverse the order of the elements in a vector.
    pub(crate) fn codegen_reverse_vector_in_place(&mut self, vector: BrilligVector) {
        let iteration_count = self.allocate_register();
        self.codegen_usize_op(vector.size, iteration_count, BrilligBinaryOp::UnsignedDiv, 2);

        let start_value_register = self.allocate_register();
        let index_at_end_of_array = self.allocate_register();
        let end_value_register = self.allocate_register();

        self.codegen_loop(iteration_count, |ctx, iterator_register| {
            // Load both values
            ctx.codegen_array_get(vector.pointer, iterator_register, start_value_register);

            // The index at the end of array is size - 1 - iterator
            ctx.mov_instruction(index_at_end_of_array, vector.size);
            ctx.codegen_usize_op_in_place(index_at_end_of_array, BrilligBinaryOp::Sub, 1);
            ctx.memory_op_instruction(
                index_at_end_of_array,
                iterator_register.address,
                index_at_end_of_array,
                BrilligBinaryOp::Sub,
            );

            ctx.codegen_array_get(
                vector.pointer,
                SingleAddrVariable::new_usize(index_at_end_of_array),
                end_value_register,
            );

            // Write both values
            ctx.codegen_array_set(vector.pointer, iterator_register, end_value_register);
            ctx.codegen_array_set(
                vector.pointer,
                SingleAddrVariable::new_usize(index_at_end_of_array),
                start_value_register,
            );
        });

        self.deallocate_register(iteration_count);
        self.deallocate_register(start_value_register);
        self.deallocate_register(end_value_register);
        self.deallocate_register(index_at_end_of_array);
    }
}
