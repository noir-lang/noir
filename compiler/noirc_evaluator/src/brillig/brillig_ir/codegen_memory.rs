use acvm::{
    AcirField,
    acir::brillig::{HeapArray, HeapVector, MemoryAddress, ValueOrArray},
    brillig_vm::offsets,
};

use crate::brillig::brillig_ir::{BrilligBinaryOp, registers::Allocated};

use super::{
    BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, BrilligContext, ReservedRegisters,
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::RegisterAllocator,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Loads the current _free memory pointer_ into `pointer_register` and
    /// increases the _free memory pointer_ by `size`, thus allocating the
    /// required amount of memory starting at the pointer.
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

    /// Loads the current _free memory pointer_ into `pointer_register` and
    /// increases the _free memory pointer_ by the value stored at `size_register`,
    /// thus allocating the required amount of memory starting at the pointer.
    pub(crate) fn codegen_allocate_mem(
        &mut self,
        pointer_register: MemoryAddress,
        size_register: MemoryAddress,
    ) {
        self.load_free_memory_pointer_instruction(pointer_register);
        self.increase_free_memory_pointer_instruction(size_register);
    }

    /// Gets the value stored at `base_ptr` + `index` and stores it in `result`.
    pub(crate) fn codegen_load_with_offset(
        &mut self,
        base_ptr: MemoryAddress,
        index: SingleAddrVariable,
        result: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        let final_index = self.allocate_register();
        self.memory_op_instruction(base_ptr, index.address, *final_index, BrilligBinaryOp::Add);
        self.load_instruction(result, *final_index);
    }

    /// Stores value at `base_ptr` + `index`.
    pub(crate) fn codegen_store_with_offset(
        &mut self,
        base_ptr: MemoryAddress,
        index: SingleAddrVariable,
        value: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        let final_index = self.allocate_register();
        self.memory_op_instruction(base_ptr, index.address, *final_index, BrilligBinaryOp::Add);
        self.store_instruction(*final_index, value);
    }

    /// Copies the values of memory pointed by `source_pointer` with length stored in `num_elements_variable`
    /// after the address pointed by `destination_pointer`.
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
            // Early exit if count is 0
            let count_is_zero = self.allocate_single_addr_bool();
            self.codegen_usize_op(
                num_elements_variable.address,
                count_is_zero.address,
                BrilligBinaryOp::Equals,
                0,
            );

            self.codegen_if_not(count_is_zero.address, |ctx| {
                // Setup: Compute end pointer and initialize iterators
                let end_source_pointer = ctx.allocate_register();
                ctx.memory_op_instruction(
                    source_pointer,
                    num_elements_variable.address,
                    *end_source_pointer,
                    BrilligBinaryOp::Add,
                );

                let source_iterator = ctx.allocate_register();
                let target_iterator = ctx.allocate_register();
                ctx.mov_instruction(*source_iterator, source_pointer);
                ctx.mov_instruction(*target_iterator, destination_pointer);

                let (loop_section, loop_label) = ctx.reserve_next_section_label();
                ctx.enter_section(loop_section);

                // Copy the current element
                let value_register = ctx.allocate_register();
                ctx.load_instruction(*value_register, *source_iterator);
                ctx.store_instruction(*target_iterator, *value_register);

                // Increment both pointers
                ctx.memory_op_inc_by_usize_one(*source_iterator);
                ctx.memory_op_inc_by_usize_one(*target_iterator);

                // Check if we should continue: source_iterator < end_source_pointer?
                // This is equivalent to checking != with one less opcode than using Equals + NOT
                let should_continue = ctx.allocate_single_addr_bool();
                ctx.memory_op_instruction(
                    *source_iterator,
                    *end_source_pointer,
                    should_continue.address,
                    BrilligBinaryOp::LessThan,
                );

                // If should continue, jump back to loop; otherwise fall through and exit
                ctx.jump_if_instruction(should_continue.address, loop_label);
            });
        }
    }

    /// Copies num_elements_variable from the source pointer to the target pointer, starting from the end
    pub(crate) fn codegen_mem_copy_from_the_end(
        &mut self,
        source_start: MemoryAddress,
        target_start: MemoryAddress,
        num_elements_variable: SingleAddrVariable,
    ) {
        // Early exit if count is 0
        let count_is_zero = self.allocate_single_addr_bool();
        self.codegen_usize_op(
            num_elements_variable.address,
            count_is_zero.address,
            BrilligBinaryOp::Equals,
            0,
        );

        self.codegen_if_not(count_is_zero.address, |ctx| {
            // Setup: Initialize source and target pointers to ONE PAST the last element
            // This allows us to decrement first, avoiding the need to compute (count - 1)
            let source_pointer = ctx.allocate_register();
            ctx.memory_op_instruction(
                source_start,
                num_elements_variable.address,
                *source_pointer,
                BrilligBinaryOp::Add,
            );

            let target_pointer = ctx.allocate_register();
            ctx.memory_op_instruction(
                target_start,
                num_elements_variable.address,
                *target_pointer,
                BrilligBinaryOp::Add,
            );

            let (loop_section, loop_label) = ctx.reserve_next_section_label();
            ctx.enter_section(loop_section);

            // Decrement both pointers to point to the current element
            ctx.codegen_usize_op_in_place(*source_pointer, BrilligBinaryOp::Sub, 1);
            ctx.codegen_usize_op_in_place(*target_pointer, BrilligBinaryOp::Sub, 1);

            // Copy the current element
            let value_register = ctx.allocate_register();
            ctx.load_instruction(*value_register, *source_pointer);
            ctx.store_instruction(*target_pointer, *value_register);

            // Check if we should continue: source_start < source_pointer?
            // If yes, we haven't finished yet, so loop back
            let should_continue = ctx.allocate_single_addr_bool();
            ctx.memory_op_instruction(
                source_start,
                *source_pointer,
                should_continue.address,
                BrilligBinaryOp::LessThan,
            );

            // If should continue, jump back to loop; otherwise fall through and exit
            ctx.jump_if_instruction(should_continue.address, loop_label);
        });
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
        self.codegen_usize_op(size, *iteration_count, BrilligBinaryOp::UnsignedDiv, 2);

        let start_value_register = self.allocate_register();
        let end_value_register = self.allocate_register();
        let index_at_end_of_array = self.allocate_register();

        self.mov_instruction(*index_at_end_of_array, size);

        self.codegen_loop(*iteration_count, |ctx, iterator_register| {
            // The index at the end of array is size - 1 - iterator
            ctx.codegen_usize_op_in_place(*index_at_end_of_array, BrilligBinaryOp::Sub, 1);
            let index_at_end_of_array_var = SingleAddrVariable::new_usize(*index_at_end_of_array);

            // Load both values
            ctx.codegen_load_with_offset(items_pointer, iterator_register, *start_value_register);
            ctx.codegen_load_with_offset(
                items_pointer,
                index_at_end_of_array_var,
                *end_value_register,
            );

            // Write both values
            ctx.codegen_store_with_offset(items_pointer, iterator_register, *end_value_register);
            ctx.codegen_store_with_offset(
                items_pointer,
                index_at_end_of_array_var,
                *start_value_register,
            );
        });
    }

    /// Converts a [BrilligArray] (pointer to `[RC, ...items]`) to a [HeapArray] (pointer to `[...items]`).
    pub(crate) fn codegen_brillig_array_to_heap_array(
        &mut self,
        array: BrilligArray,
    ) -> Allocated<HeapArray, Registers> {
        let heap_array = self.allocate_heap_array(array.size);
        self.codegen_usize_op(
            array.pointer,
            heap_array.pointer,
            BrilligBinaryOp::Add,
            offsets::ARRAY_ITEMS,
        );
        heap_array
    }

    /// Converts a [BrilligVector] (pointer to `[RC, size, capacity, ...items]`) to a [HeapVector] (two pointers to `[...items]` and `size`).
    pub(crate) fn codegen_brillig_vector_to_heap_vector(
        &mut self,
        vector: BrilligVector,
    ) -> Allocated<HeapVector, Registers> {
        let heap_vector = self.allocate_heap_vector();

        // Read the size using the dedicated helper function
        let size_variable = self.codegen_read_vector_size(vector);
        self.mov_instruction(heap_vector.size, size_variable.address);

        // Get the pointer to the items using the dedicated helper function
        self.codegen_vector_items_pointer(vector, heap_vector.pointer);

        heap_vector
    }

    /// Converts a [BrilligVariable] to [ValueOrArray].
    ///
    /// This can involve allocating new registers and loading values into them
    /// from the input data structures.
    pub(crate) fn variable_to_value_or_array(
        &mut self,
        variable: BrilligVariable,
    ) -> Allocated<ValueOrArray, Registers> {
        match variable {
            BrilligVariable::SingleAddr(SingleAddrVariable { address, .. }) => {
                Allocated::pure(ValueOrArray::MemoryAddress(address))
            }
            BrilligVariable::BrilligArray(array) => {
                self.codegen_brillig_array_to_heap_array(array).map(ValueOrArray::HeapArray)
            }
            BrilligVariable::BrilligVector(vector) => {
                self.codegen_brillig_vector_to_heap_vector(vector).map(ValueOrArray::HeapVector)
            }
        }
    }

    /// Returns a variable holding the size of a given vector.
    pub(crate) fn codegen_read_vector_size(
        &mut self,
        vector: BrilligVector,
    ) -> Allocated<SingleAddrVariable, Registers> {
        let result = self.allocate_single_addr_usize();
        self.codegen_usize_op(
            vector.pointer,
            result.address,
            BrilligBinaryOp::Add,
            offsets::VECTOR_SIZE,
        );
        self.load_instruction(result.address, result.address);
        result
    }

    /// Writes the value of new size to the size pointer of the vector.
    pub(crate) fn codegen_update_vector_size(
        &mut self,
        vector: BrilligVector,
        new_size: SingleAddrVariable,
    ) {
        let write_pointer = self.allocate_register();
        self.codegen_usize_op(
            vector.pointer,
            *write_pointer,
            BrilligBinaryOp::Add,
            offsets::VECTOR_SIZE,
        );
        self.store_instruction(*write_pointer, new_size.address);
    }

    /// Returns a variable holding the capacity of a given vector.
    pub(crate) fn codegen_read_vector_capacity(
        &mut self,
        vector: BrilligVector,
    ) -> Allocated<SingleAddrVariable, Registers> {
        let result = self.allocate_single_addr_usize();
        self.codegen_usize_op(
            vector.pointer,
            result.address,
            BrilligBinaryOp::Add,
            offsets::VECTOR_CAPACITY,
        );
        self.load_instruction(result.address, result.address);
        result
    }

    /// Writes the pointer to the items of a given vector to the `result`.
    pub(crate) fn codegen_vector_items_pointer(
        &mut self,
        vector: BrilligVector,
        result: MemoryAddress,
    ) {
        self.codegen_usize_op(vector.pointer, result, BrilligBinaryOp::Add, offsets::VECTOR_ITEMS);
    }

    /// Returns a pointer to the items of a given vector.
    pub(crate) fn codegen_make_vector_items_pointer(
        &mut self,
        vector: BrilligVector,
    ) -> Allocated<MemoryAddress, Registers> {
        let result = self.allocate_register();
        self.codegen_vector_items_pointer(vector, *result);
        result
    }

    /// Reads the metadata of a vector and stores it in the given variables
    ///
    /// If the `semantic_length_and_item_size` is given, then instead of reading the size from the
    /// vector data structure, it is calculated as a multiplication of length and item size.
    pub(crate) fn codegen_read_vector_metadata(
        &mut self,
        vector: BrilligVector,
        rc: SingleAddrVariable,
        size: SingleAddrVariable,
        capacity: SingleAddrVariable,
        items_pointer: SingleAddrVariable,
        semantic_length_and_item_size: Option<(MemoryAddress, MemoryAddress)>,
    ) {
        assert!(rc.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        assert!(size.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        assert!(capacity.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        assert!(items_pointer.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        // Vector layout: [ref_count, size, capacity, items...]
        self.load_instruction(rc.address, vector.pointer);
        let read_pointer = self.allocate_register();
        self.codegen_usize_op(vector.pointer, *read_pointer, BrilligBinaryOp::Add, 1);
        if let Some((length, item_size)) = semantic_length_and_item_size {
            self.codegen_vector_flattened_size(size.address, length, item_size);
        } else {
            self.load_instruction(size.address, *read_pointer);
        }
        self.codegen_usize_op_in_place(*read_pointer, BrilligBinaryOp::Add, 1);
        self.load_instruction(capacity.address, *read_pointer);
        self.codegen_usize_op(*read_pointer, items_pointer.address, BrilligBinaryOp::Add, 1);
    }

    /// Generate code to calculate the flattened vector size from its semantic length and the item size.
    ///
    /// For example a `[(u32, bool)]` would have a flattened item size of 2, because each item consists of 2 values.
    /// Such a vector with a semantic length of 3 would have a flattened size of 6.
    pub(crate) fn codegen_vector_flattened_size(
        &mut self,
        destination: MemoryAddress,
        length: MemoryAddress,
        item_size: MemoryAddress,
    ) {
        self.memory_op_instruction(length, item_size, destination, BrilligBinaryOp::Mul);
    }

    /// Returns a pointer to the items of a given array.
    pub(crate) fn codegen_make_array_items_pointer(
        &mut self,
        array: BrilligArray,
    ) -> Allocated<MemoryAddress, Registers> {
        let result = self.allocate_register();
        self.codegen_usize_op(array.pointer, *result, BrilligBinaryOp::Add, offsets::ARRAY_ITEMS);
        result
    }

    /// Returns a pointer to the items of an array or vector.
    pub(crate) fn codegen_make_array_or_vector_items_pointer(
        &mut self,
        variable: BrilligVariable,
    ) -> Allocated<MemoryAddress, Registers> {
        match variable {
            BrilligVariable::BrilligArray(array) => self.codegen_make_array_items_pointer(array),
            BrilligVariable::BrilligVector(vector) => {
                self.codegen_make_vector_items_pointer(vector)
            }
            _ => unreachable!("ICE: Expected array or vector, got {variable:?}"),
        }
    }

    /// Initializes an array, allocating memory on the heap to store its representation and initializing the reference counter to 1.
    pub(crate) fn codegen_initialize_array(&mut self, array: BrilligArray) {
        // Allocate memory for the 1 ref counter and `size` items.
        self.codegen_allocate_immediate_mem(array.pointer, array.size + offsets::ARRAY_META_COUNT);
        self.initialize_rc(array.pointer, 1);
    }

    /// Initialize the reference counter for an array or vector.
    /// This should only be used internally in the array and vector initialization methods.
    ///
    /// The RC is in the 0th slot of the memory allocated for the array or vector.
    fn initialize_rc(&mut self, pointer: MemoryAddress, rc_value: usize) {
        self.indirect_const_instruction(
            pointer,
            BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            rc_value.into(),
        );
    }

    /// Initializes a vector, allocating memory on the heap to store its representation and initializing the reference counter, size and capacity.
    pub(crate) fn codegen_initialize_vector(
        &mut self,
        // Pointer which will hold the address of where the vector got allocated.
        vector: BrilligVector,
        // Address holding the size which depends on the number and type of items the vector is created with,
        // or it could be the result of adding or subtracting a value during push/pop.
        size: SingleAddrVariable,
        // Address of any explicit extra capacity we want to allocate the vector with, the allow room for future growth for example.
        // Defaults to `size` if empty.
        capacity: Option<SingleAddrVariable>,
    ) {
        let capacity = capacity.unwrap_or(size);

        // Allocation size = capacity + 3 (rc, size, capacity)
        let allocation_size = self.allocate_register();
        self.codegen_usize_op(
            capacity.address,
            *allocation_size,
            BrilligBinaryOp::Add,
            offsets::VECTOR_META_COUNT,
        );
        self.codegen_allocate_mem(vector.pointer, *allocation_size);

        // Deallocate to match unit test expectations about slot reuse.
        allocation_size.deallocate();

        self.codegen_initialize_vector_metadata(vector, size, capacity);
    }

    /// Writes vector metadata (reference count, size, and capacity) into the allocated memory.
    pub(super) fn codegen_initialize_vector_metadata(
        &mut self,
        vector: BrilligVector,
        size: SingleAddrVariable,
        capacity: SingleAddrVariable,
    ) {
        // Write RC
        self.initialize_rc(vector.pointer, 1);

        // Write size
        let write_pointer = self.allocate_register();
        self.codegen_usize_op(
            vector.pointer,
            *write_pointer,
            BrilligBinaryOp::Add,
            offsets::VECTOR_SIZE,
        );
        self.store_instruction(*write_pointer, size.address);

        // Write capacity
        self.codegen_usize_op_in_place(
            *write_pointer,
            BrilligBinaryOp::Add,
            offsets::VECTOR_CAPACITY - offsets::VECTOR_SIZE,
        );
        self.store_instruction(*write_pointer, capacity.address);
    }

    /// Initialize the [BrilligVector] from a [HeapVector] returned by a foreign call.
    ///
    /// We don't know the length of a vector returned externally before the call,
    /// so we pass the free memory pointer and then use this function to allocate
    /// after the fact when we know the length.
    ///
    /// This method assumes nothing else has been allocated into the space tentatively
    /// reserved for the vector, that is, that the _free memory pointer_ is where it was
    /// before the foreign call.
    pub(crate) fn codegen_initialize_externally_returned_vector(
        &mut self,
        vector: BrilligVector,
        resulting_heap_vector: HeapVector,
    ) {
        // The size in the heap vector only represents the items.
        // Figure out how much memory we need to allocate to hold it, accounting for the metadata.
        let total_size = self.allocate_register();
        self.codegen_usize_op(
            resulting_heap_vector.size,
            *total_size,
            BrilligBinaryOp::Add,
            offsets::VECTOR_META_COUNT,
        );

        // Increase the free memory pointer to make sure the vector is not going to be allocated to something else.
        self.increase_free_memory_pointer_instruction(*total_size);

        // Initialize metadata (RC, size, capacity) using the shared helper
        // For externally returned vectors, capacity equals size
        let size_var = SingleAddrVariable::new_usize(resulting_heap_vector.size);
        self.codegen_initialize_vector_metadata(vector, size_var, size_var);
    }
}
