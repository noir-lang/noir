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

pub(crate) struct VectorMetaData<Registers: RegisterAllocator> {
    pub(crate) rc: Allocated<SingleAddrVariable, Registers>,
    pub(crate) size: Allocated<SingleAddrVariable, Registers>,
    pub(crate) capacity: Allocated<SingleAddrVariable, Registers>,
    pub(crate) items_pointer: Allocated<SingleAddrVariable, Registers>,
}

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
            // Temporary register for copying source to target, ie. between two heap addresses through the stack.
            let value_register = self.allocate_register();

            // End pointer for finish condition.
            let end_source_pointer = self.allocate_register();
            self.memory_op_instruction(
                source_pointer,
                num_elements_variable.address,
                *end_source_pointer,
                BrilligBinaryOp::Add,
            );

            self.codegen_generic_iteration(
                |brillig_context| {
                    let source_iterator = brillig_context.allocate_register();
                    let target_iterator = brillig_context.allocate_register();

                    brillig_context.mov_instruction(*source_iterator, source_pointer);
                    brillig_context.mov_instruction(*target_iterator, destination_pointer);

                    (source_iterator, target_iterator)
                },
                |brillig_context, (source_iterator, target_iterator)| {
                    brillig_context.memory_op_inc_by_usize_one(**source_iterator);
                    brillig_context.memory_op_inc_by_usize_one(**target_iterator);
                },
                |brillig_context, (source_iterator, _)| {
                    // We have finished when the source iterator reaches the end pointer.
                    let finish_condition = brillig_context.allocate_single_addr_bool();
                    brillig_context.memory_op_instruction(
                        **source_iterator,
                        *end_source_pointer,
                        finish_condition.address,
                        BrilligBinaryOp::Equals,
                    );
                    finish_condition
                },
                |brillig_context, (source_iterator, target_iterator)| {
                    brillig_context.load_instruction(*value_register, **source_iterator);
                    brillig_context.store_instruction(**target_iterator, *value_register);
                },
                |_, _| {},
            );
        }
    }

    /// Copies `num_elements_variable` number of elements from the `source_start` pointer to the `target_start` pointer,
    /// starting from the end, moving backwards.
    ///
    /// By moving back-to-front, it can shift items backwards, modifying a vector in-place to make room in the front.
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
                    *num_items_minus_one,
                    BrilligBinaryOp::Sub,
                    1,
                );
                // target = &target_items[num_elements - 1]
                let target_pointer = brillig_context.allocate_register();
                brillig_context.memory_op_instruction(
                    target_start,
                    *num_items_minus_one,
                    *target_pointer,
                    BrilligBinaryOp::Add,
                );
                // source = &source_items[num_elements - 1]
                let source_pointer = brillig_context.allocate_register();
                brillig_context.memory_op_instruction(
                    source_start,
                    *num_items_minus_one,
                    *source_pointer,
                    BrilligBinaryOp::Add,
                );
                (source_pointer, target_pointer)
            },
            |brillig_context, (source_pointer, target_pointer)| {
                // source -= 1
                brillig_context.codegen_usize_op_in_place(
                    **source_pointer,
                    BrilligBinaryOp::Sub,
                    1,
                );
                // target -= 1
                brillig_context.codegen_usize_op_in_place(
                    **target_pointer,
                    BrilligBinaryOp::Sub,
                    1,
                );
            },
            |brillig_context, (source_pointer, _)| {
                // We have finished when the source/target pointer is less than the source/target start
                let finish_condition = brillig_context.allocate_single_addr_bool();
                brillig_context.memory_op_instruction(
                    **source_pointer,
                    source_start,
                    finish_condition.address,
                    BrilligBinaryOp::LessThan,
                );
                finish_condition
            },
            |brillig_context, (source_pointer, target_pointer)| {
                let value_register = brillig_context.allocate_register();
                brillig_context.load_instruction(*value_register, **source_pointer);
                brillig_context.store_instruction(**target_pointer, *value_register);
            },
            |_, _| {},
        );
    }

    /// Emit opcodes to reverse the order of the `size` elements pointed by `pointer`.
    ///
    /// It is the responsibility of the caller to ensure that the ref-count of the array is 1.
    pub(crate) fn codegen_array_reverse(
        &mut self,
        items_pointer: MemoryAddress,
        size: MemoryAddress,
    ) {
        if self.can_call_procedures {
            self.call_array_reverse_procedure(items_pointer, size);
            return;
        }

        // for i in 0..size/2 { swap(items[i], items[size-1-i]); }
        let iteration_count = self.allocate_register();
        self.codegen_usize_op(size, *iteration_count, BrilligBinaryOp::UnsignedDiv, 2);

        let start_value_register = self.allocate_register();
        let end_value_register = self.allocate_register();
        let index_at_end = self.allocate_register();

        // The index going from back to front.
        self.mov_instruction(*index_at_end, size);

        self.codegen_loop(*iteration_count, |ctx, iterator_register| {
            // The index at the end of the array is size - 1 - iterator
            ctx.codegen_usize_op_in_place(*index_at_end, BrilligBinaryOp::Sub, 1);
            let index_at_end_var = SingleAddrVariable::new_usize(*index_at_end);

            // Load both values
            ctx.codegen_load_with_offset(items_pointer, iterator_register, *start_value_register);
            ctx.codegen_load_with_offset(items_pointer, index_at_end_var, *end_value_register);

            // Write both values
            ctx.codegen_store_with_offset(items_pointer, iterator_register, *end_value_register);
            ctx.codegen_store_with_offset(items_pointer, index_at_end_var, *start_value_register);
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

    /// Returns a variable holding the ref-count of a given array or vector.
    pub(crate) fn codegen_read_rc(
        &mut self,
        pointer: MemoryAddress,
    ) -> Allocated<SingleAddrVariable, Registers> {
        let result = self.allocate_single_addr_usize();
        self.load_instruction(result.address, pointer);
        result
    }

    /// Returns a variable holding the ref-count of a given vector.
    pub(crate) fn codegen_read_vector_rc(
        &mut self,
        vector: BrilligVector,
    ) -> Allocated<SingleAddrVariable, Registers> {
        self.codegen_read_rc(vector.pointer)
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

    /// Reads the metadata of a vector into individual registers and returns them as [VectorMetaData].
    ///
    /// If the `semantic_length_and_item_size` is given, then instead of reading the size from the
    /// vector data structure, it is calculated as a multiplication of length and item size.
    pub(crate) fn codegen_read_vector_metadata(
        &mut self,
        vector: BrilligVector,
        semantic_length_and_item_size: Option<(MemoryAddress, MemoryAddress)>,
    ) -> VectorMetaData<Registers> {
        let rc = self.allocate_single_addr_usize();
        let size = self.allocate_single_addr_usize();
        let capacity = self.allocate_single_addr_usize();
        let items_pointer = self.allocate_single_addr_usize();

        // Vector layout: [ref_count, size, capacity, items...]
        self.load_instruction(rc.address, vector.pointer);
        let read_pointer = self.allocate_register();
        self.codegen_usize_op(
            vector.pointer,
            *read_pointer,
            BrilligBinaryOp::Add,
            offsets::VECTOR_SIZE,
        );
        if let Some((length, item_size)) = semantic_length_and_item_size {
            self.codegen_vector_flattened_size(size.address, length, item_size);
        } else {
            self.load_instruction(size.address, *read_pointer);
        }
        self.codegen_usize_op_in_place(
            *read_pointer,
            BrilligBinaryOp::Add,
            offsets::VECTOR_CAPACITY - offsets::VECTOR_SIZE,
        );
        self.load_instruction(capacity.address, *read_pointer);
        self.codegen_usize_op(
            *read_pointer,
            items_pointer.address,
            BrilligBinaryOp::Add,
            offsets::VECTOR_ITEMS - offsets::VECTOR_CAPACITY,
        );

        VectorMetaData { rc, size, capacity, items_pointer }
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
        // Allocate memory for the ref counter and `size` items.
        self.codegen_allocate_immediate_mem(array.pointer, array.size + offsets::ARRAY_META_COUNT);
        self.codegen_initialize_rc(array.pointer, 1);
    }

    /// Initialize the reference counter for an array or vector.
    /// This should only be used internally in the array and vector initialization methods.
    ///
    /// The RC is in the 0th slot of the memory allocated for the array or vector.
    pub(crate) fn codegen_initialize_rc(&mut self, pointer: MemoryAddress, rc: usize) {
        self.indirect_const_instruction(pointer, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, rc.into());
    }

    /// Decrement the ref-count by 1.
    ///
    /// The inputs are:
    /// * the `pointer` to the array/vector
    /// * the `rc` address of the vector where we have the current RC loaded already
    pub(crate) fn codegen_decrement_rc(&mut self, _pointer: MemoryAddress, _rc: MemoryAddress) {
        // In benchmarks having this on didn't have a noticeable performance benefit,
        // but it does have a small increase in byte code size and the number of executed opcodes.
        // When we disabled `dec_rc` in SSA, the performance improved, so for now we disabled this,
        // in order to not deviate conceptually from SSA. The method is left in place as a reference
        // to where we could re-enable them.

        // // Modify the RC (it's on the stack, or scratch space).
        // self.codegen_usize_op_in_place(rc, BrilligBinaryOp::Sub, 1);
        // // Write it back onto the heap.
        // self.store_instruction(pointer, rc);
    }

    /// Increment the ref-count by 1.
    ///
    /// The inputs are:
    /// * the `pointer` to the array/vector
    /// * the `rc` address of the vector where we have the current RC loaded already
    /// * the `by` is a constant by which to increment the RC, typically 1
    pub(crate) fn codegen_increment_rc(&mut self, pointer: MemoryAddress, rc: MemoryAddress) {
        // Modify the RC (it's on the stack, or scratch space).
        self.codegen_usize_op_in_place(rc, BrilligBinaryOp::Add, 1);
        // Write it back onto the heap.
        self.store_instruction(pointer, rc);
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
    ///
    /// Sets the reference count to 1.
    pub(super) fn codegen_initialize_vector_metadata(
        &mut self,
        vector: BrilligVector,
        size: SingleAddrVariable,
        capacity: SingleAddrVariable,
    ) {
        // Write RC
        self.codegen_initialize_rc(vector.pointer, 1);

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

    /// Initialize the [BrilligVector] after the data returned by a foreign call has been written to the heap.
    ///
    /// We don't know the length of a vector returned externally before the call,
    /// so we write the size and the data to the _free memory pointer_.
    ///
    /// Here we are adjusting the rest of the meta-data required by the vector structure: basically the RC and the capacity.
    ///
    /// The VM is expected to adjust the _free memory pointer_ to point beyond where the data was written,
    /// so we don't have to generate bytecode to increase it here.
    ///
    /// Returns the size variable, which we can use to set the semantic length.
    pub(crate) fn codegen_initialize_externally_returned_vector(
        &mut self,
        vector: BrilligVector,
    ) -> Allocated<SingleAddrVariable, Registers> {
        // Read the address of the size on the heap based on the vector pointer on the stack.
        let size_var = self.codegen_read_vector_size(vector);
        // For externally returned vectors, capacity equals size.
        self.codegen_initialize_vector_metadata(vector, *size_var, *size_var);
        size_var
    }
}
