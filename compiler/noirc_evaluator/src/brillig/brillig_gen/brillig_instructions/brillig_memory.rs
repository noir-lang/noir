use std::sync::Arc;

use acvm::acir::brillig::MemoryAddress;
use acvm::{AcirField, FieldElement};
use im::Vector;

use crate::brillig::brillig_gen::brillig_block::BrilligBlock;
use crate::brillig::brillig_ir::brillig_variable::{BrilligVariable, SingleAddrVariable};
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::brillig::brillig_ir::{
    BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, BrilligBinaryOp, BrilligContext, ReservedRegisters,
};
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::types::Type;
use crate::ssa::ir::{dfg::DataFlowGraph, value::ValueId};

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// Initializes a constant array in Brillig memory.
    ///
    /// This method is responsible for writing a constant array's contents into memory, starting
    /// from the given `pointer`. It chooses between compile-time or runtime initialization
    /// depending on the data pattern and size.
    ///
    /// If the array is large (`>10` items), its elements are all numeric, and all items are identical,
    /// a **runtime loop** is generated to perform the initialization more efficiently.
    ///
    /// Otherwise, the method falls back to a straightforward **compile-time** initialization, where
    /// each array element is emitted explicitly.
    ///
    /// This optimization helps reduce Brillig bytecode size and runtime cost when initializing large,
    /// uniform arrays.
    ///
    /// # Example
    /// For an array like [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5], a runtime loop will be used
    /// For an array like [1, 2, 3, 4], each element will be set explicitly
    fn initialize_constant_array(
        &mut self,
        data: &Vector<ValueId>,
        typ: &Type,
        dfg: &DataFlowGraph,
        pointer: MemoryAddress,
    ) {
        if data.is_empty() {
            return;
        }
        let item_types = typ.clone().element_types();

        // Find out if we are repeating the same item over and over
        let first_item = data.iter().take(item_types.len()).copied().collect();
        let mut is_repeating = true;

        for item_index in (item_types.len()..data.len()).step_by(item_types.len()) {
            let item: Vec<_> = (0..item_types.len()).map(|i| data[item_index + i]).collect();
            if first_item != item {
                is_repeating = false;
                break;
            }
        }

        // If all the items are single address, and all have the same initial value, we can initialize the array in a runtime loop.
        // Since the cost in instructions for a runtime loop is in the order of magnitude of 10, we only do this if the item_count is bigger than that.
        let item_count = data.len() / item_types.len();

        if item_count > 10
            && is_repeating
            && item_types.iter().all(|typ| matches!(typ, Type::Numeric(_)))
        {
            self.initialize_constant_array_runtime(
                item_types, first_item, item_count, pointer, dfg,
            );
        } else {
            self.initialize_constant_array_comptime(data, dfg, pointer);
        }
    }

    /// Codegens Brillig instructions to initialize a large, constant array using a runtime loop.
    ///
    /// This method assumes the array consists of identical items repeated multiple times.
    /// It generates a Brillig loop that writes the repeated item into memory efficiently,
    /// reducing bytecode size and instruction count compared to unrolling each element.
    ///
    /// For complex types (e.g., tuples), multiple memory writes happen per loop iteration.
    /// For primitive type (e.g., u32, Field), a single memory write happens per loop iteration.
    fn initialize_constant_array_runtime(
        &mut self,
        item_types: Arc<Vec<Type>>,
        item_to_repeat: Vec<ValueId>,
        item_count: usize,
        pointer: MemoryAddress,
        dfg: &DataFlowGraph,
    ) {
        let mut subitem_to_repeat_variables = Vec::with_capacity(item_types.len());
        for subitem_id in item_to_repeat.into_iter() {
            subitem_to_repeat_variables.push(self.convert_ssa_value(subitem_id, dfg));
        }

        // Initialize loop bound with the array length
        let end_pointer_variable = self
            .brillig_context
            .make_usize_constant_instruction((item_count * item_types.len()).into());

        // Add the pointer to the array length
        self.brillig_context.memory_op_instruction(
            end_pointer_variable.address,
            pointer,
            end_pointer_variable.address,
            BrilligBinaryOp::Add,
        );

        // If this is an array with complex subitems, we need a custom step in the loop to write all the subitems while iterating.
        if item_types.len() > 1 {
            let step_variable =
                self.brillig_context.make_usize_constant_instruction(item_types.len().into());

            let subitem_pointer =
                SingleAddrVariable::new_usize(self.brillig_context.allocate_register());

            // Initializes a single subitem
            let initializer_fn =
                |ctx: &mut BrilligContext<_, _>, subitem_start_pointer: SingleAddrVariable| {
                    ctx.mov_instruction(subitem_pointer.address, subitem_start_pointer.address);
                    for (subitem_index, subitem) in
                        subitem_to_repeat_variables.into_iter().enumerate()
                    {
                        ctx.store_instruction(subitem_pointer.address, subitem.extract_register());
                        if subitem_index != item_types.len() - 1 {
                            ctx.memory_op_instruction(
                                subitem_pointer.address,
                                ReservedRegisters::usize_one(),
                                subitem_pointer.address,
                                BrilligBinaryOp::Add,
                            );
                        }
                    }
                };

            // for (let subitem_start_pointer = pointer; subitem_start_pointer < pointer + data_length; subitem_start_pointer += step) { initializer_fn(iterator) }
            self.brillig_context.codegen_for_loop(
                Some(pointer),
                end_pointer_variable.address,
                Some(step_variable.address),
                initializer_fn,
            );

            self.brillig_context.deallocate_single_addr(step_variable);
            self.brillig_context.deallocate_single_addr(subitem_pointer);
        } else {
            let subitem = subitem_to_repeat_variables.into_iter().next().unwrap();

            let initializer_fn =
                |ctx: &mut BrilligContext<_, _>, item_pointer: SingleAddrVariable| {
                    ctx.store_instruction(item_pointer.address, subitem.extract_register());
                };

            // for (let item_pointer = pointer; item_pointer < pointer + data_length; item_pointer += 1) { initializer_fn(iterator) }
            self.brillig_context.codegen_for_loop(
                Some(pointer),
                end_pointer_variable.address,
                None,
                initializer_fn,
            );
        }
        self.brillig_context.deallocate_single_addr(end_pointer_variable);
    }

    /// Codegens Brillig instructions to initialize a constant array at compile time.
    ///
    /// This method generates one `store` instruction per array element, writing each
    /// value from the SSA into consecutive memory addresses starting at `pointer`.
    ///
    /// Unlike [initialize_constant_array_runtime][Self::initialize_constant_array_runtime], this
    /// does not use loops and emits one instruction per write, which can increase bytecode size
    /// but provides fine-grained control.
    fn initialize_constant_array_comptime(
        &mut self,
        data: &Vector<ValueId>,
        dfg: &DataFlowGraph,
        pointer: MemoryAddress,
    ) {
        // Allocate a register for the iterator
        let write_pointer_register = self.brillig_context.allocate_register();

        self.brillig_context.mov_instruction(write_pointer_register, pointer);

        for (element_idx, element_id) in data.iter().enumerate() {
            let element_variable = self.convert_ssa_value(*element_id, dfg);
            // Store the item in memory
            self.brillig_context
                .store_instruction(write_pointer_register, element_variable.extract_register());

            if element_idx != data.len() - 1 {
                // Increment the write_pointer_register
                self.brillig_context.memory_op_instruction(
                    write_pointer_register,
                    ReservedRegisters::usize_one(),
                    write_pointer_register,
                    BrilligBinaryOp::Add,
                );
            }
        }

        self.brillig_context.deallocate_register(write_pointer_register);
    }

    /// Load from an array variable at a specific index into a specified destination
    ///
    /// # Panics
    /// - The array variable is not a [BrilligVariable::BrilligArray] or [BrilligVariable::BrilligVector] when `has_offset` is false
    fn convert_ssa_array_get(
        &mut self,
        array_variable: BrilligVariable,
        index_variable: SingleAddrVariable,
        destination_variable: BrilligVariable,
        has_offset: bool,
    ) {
        let items_pointer = if has_offset {
            array_variable.extract_register()
        } else {
            self.brillig_context.codegen_make_array_or_vector_items_pointer(array_variable)
        };

        self.brillig_context.codegen_load_with_offset(
            items_pointer,
            index_variable,
            destination_variable.extract_register(),
        );

        if !has_offset {
            self.brillig_context.deallocate_register(items_pointer);
        }
    }

    /// Array set operation in SSA returns a new array or vector that is a copy of the parameter array or vector
    /// with a specific value changed.
    ///
    /// Whether an actual copy other the array occurs or we write into the same source array is determined by the
    /// [call into the array copy procedure][BrilligContext::call_array_copy_procedure].
    /// If the reference count of an array pointer is one, we write directly to the array.
    /// Look at the [procedure compilation][crate::brillig::brillig_ir::procedures::compile_procedure] for the exact procedure's codegen.
    fn convert_ssa_array_set(
        &mut self,
        source_variable: BrilligVariable,
        destination_variable: BrilligVariable,
        index_register: SingleAddrVariable,
        value_variable: BrilligVariable,
        mutable: bool,
        has_offset: bool,
    ) {
        assert!(index_register.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        match (source_variable, destination_variable) {
            (
                BrilligVariable::BrilligArray(source_array),
                BrilligVariable::BrilligArray(destination_array),
            ) => {
                if !mutable {
                    self.brillig_context.call_array_copy_procedure(source_array, destination_array);
                }
            }
            (
                BrilligVariable::BrilligVector(source_vector),
                BrilligVariable::BrilligVector(destination_vector),
            ) => {
                if !mutable {
                    self.brillig_context
                        .call_vector_copy_procedure(source_vector, destination_vector);
                }
            }
            _ => unreachable!("ICE: array set on non-array"),
        }

        let destination_for_store = if mutable { source_variable } else { destination_variable };

        // Then set the value in the newly created array
        let items_pointer = if has_offset {
            destination_for_store.extract_register()
        } else {
            self.brillig_context.codegen_make_array_or_vector_items_pointer(destination_for_store)
        };

        self.brillig_context.codegen_store_with_offset(
            items_pointer,
            index_register,
            value_variable.extract_register(),
        );

        // If we mutated the source array we want instructions that use the destination array to point to the source array
        if mutable {
            self.brillig_context.mov_instruction(
                destination_variable.extract_register(),
                source_variable.extract_register(),
            );
        }

        if !has_offset {
            self.brillig_context.deallocate_register(items_pointer);
        }
    }

    /// Debug utility method to determine whether an array's reference count (RC) is zero.
    /// If RC's have drifted down to zero it means the RC increment/decrement instructions
    /// have been written incorrectly.
    ///
    /// Should only be called if [BrilligContext::enable_debug_assertions] returns true.
    fn assert_rc_neq_zero(&mut self, rc_register: MemoryAddress) {
        let zero = SingleAddrVariable::new(self.brillig_context.allocate_register(), 32);

        self.brillig_context.const_instruction(zero, FieldElement::zero());

        let condition = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);

        self.brillig_context.memory_op_instruction(
            zero.address,
            rc_register,
            condition.address,
            BrilligBinaryOp::Equals,
        );
        self.brillig_context.not_instruction(condition, condition);
        self.brillig_context
            .codegen_constrain(condition, Some("array ref-count underflow detected".to_owned()));
        self.brillig_context.deallocate_single_addr(condition);
    }

    pub(crate) fn codegen_allocate(&mut self, instruction_id: InstructionId, dfg: &DataFlowGraph) {
        let [result_value] = dfg.instruction_result(instruction_id);
        let pointer = self.variables.define_single_addr_variable(
            self.function_context,
            self.brillig_context,
            result_value,
            dfg,
        );
        self.brillig_context.codegen_allocate_immediate_mem(pointer.address, 1);
    }
    pub(crate) fn codegen_store(&mut self, address: ValueId, value: ValueId, dfg: &DataFlowGraph) {
        let address_var = self.convert_ssa_single_addr_value(address, dfg);
        let source_variable = self.convert_ssa_value(value, dfg);

        self.brillig_context
            .store_instruction(address_var.address, source_variable.extract_register());
    }
    pub(crate) fn codegen_load(
        &mut self,
        instruction_id: InstructionId,
        address: ValueId,
        dfg: &DataFlowGraph,
    ) {
        let [result_value] = dfg.instruction_result(instruction_id);

        let target_variable = self.variables.define_variable(
            self.function_context,
            self.brillig_context,
            result_value,
            dfg,
        );

        let address_variable = self.convert_ssa_single_addr_value(address, dfg);

        self.brillig_context
            .load_instruction(target_variable.extract_register(), address_variable.address);
    }

    pub(crate) fn array_get(
        &mut self,
        instruction_id: InstructionId,
        array: ValueId,
        index: ValueId,
        dfg: &DataFlowGraph,
    ) {
        let [result_id] = dfg.instruction_result(instruction_id);
        let destination_variable = self.variables.define_variable(
            self.function_context,
            self.brillig_context,
            result_id,
            dfg,
        );

        let array_variable = self.convert_ssa_value(array, dfg);
        let index_variable = self.convert_ssa_single_addr_value(index, dfg);

        // Constants are assumed to have been offset just before Brillig gen.
        let has_offset = dfg.is_constant(index);

        self.convert_ssa_array_get(
            array_variable,
            index_variable,
            destination_variable,
            has_offset,
        );
    }

    pub(crate) fn array_set(
        &mut self,
        instruction_id: InstructionId,
        array: ValueId,
        index: ValueId,
        value: ValueId,
        mutable: bool,
        dfg: &DataFlowGraph,
    ) {
        let source_variable = self.convert_ssa_value(array, dfg);
        let index_register = self.convert_ssa_single_addr_value(index, dfg);
        let value_variable = self.convert_ssa_value(value, dfg);

        let result_ids = dfg.instruction_results(instruction_id);
        let destination_variable = self.variables.define_variable(
            self.function_context,
            self.brillig_context,
            result_ids[0],
            dfg,
        );

        // Constants are assumed to have been offset just before Brillig gen.
        let has_offset = dfg.is_constant(index);

        self.convert_ssa_array_set(
            source_variable,
            destination_variable,
            index_register,
            value_variable,
            mutable,
            has_offset,
        );
    }

    pub(crate) fn make_array(
        &mut self,
        instruction_id: InstructionId,
        array: &Vector<ValueId>,
        typ: &Type,
        dfg: &DataFlowGraph,
    ) {
        let value_id = dfg.instruction_results(instruction_id)[0];
        if !self.variables.is_allocated(&value_id) {
            let new_variable = self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                value_id,
                dfg,
            );

            // Initialize the variable
            match new_variable {
                BrilligVariable::BrilligArray(brillig_array) => {
                    self.brillig_context.codegen_initialize_array(brillig_array);
                }
                BrilligVariable::BrilligVector(vector) => {
                    let size =
                        self.brillig_context.make_usize_constant_instruction(array.len().into());
                    self.brillig_context.codegen_initialize_vector(vector, size, None);
                    self.brillig_context.deallocate_single_addr(size);
                }
                _ => unreachable!("ICE: Cannot initialize array value created as {new_variable:?}"),
            };

            // Write the items
            let items_pointer =
                self.brillig_context.codegen_make_array_or_vector_items_pointer(new_variable);

            self.initialize_constant_array(array, typ, dfg, items_pointer);

            self.brillig_context.deallocate_register(items_pointer);
        }
    }

    pub(crate) fn increment_rc(&mut self, value: ValueId, dfg: &DataFlowGraph) {
        let array_or_vector = self.convert_ssa_value(value, dfg);
        let rc_register = self.brillig_context.allocate_register();

        // RC is always directly pointed by the array/vector pointer
        self.brillig_context.load_instruction(rc_register, array_or_vector.extract_register());

        // Ensure we're not incrementing from 0 back to 1
        if self.brillig_context.enable_debug_assertions() {
            self.assert_rc_neq_zero(rc_register);
        }

        self.brillig_context.codegen_usize_op_in_place(rc_register, BrilligBinaryOp::Add, 1);
        self.brillig_context.store_instruction(array_or_vector.extract_register(), rc_register);
        self.brillig_context.deallocate_register(rc_register);
    }
    pub(crate) fn decrement_rc(&mut self, value: ValueId, dfg: &DataFlowGraph) {
        let array_or_vector = self.convert_ssa_value(value, dfg);
        let array_register = array_or_vector.extract_register();

        let rc_register = self.brillig_context.allocate_register();
        self.brillig_context.load_instruction(rc_register, array_register);

        // Check that the refcount isn't already 0 before we decrement. If we allow it to underflow
        // and become usize::MAX, and then return to 1, then it will indicate
        // an array as mutable when it probably shouldn't be.
        if self.brillig_context.enable_debug_assertions() {
            self.assert_rc_neq_zero(rc_register);
        }

        self.brillig_context.codegen_usize_op_in_place(rc_register, BrilligBinaryOp::Sub, 1);
        self.brillig_context.store_instruction(array_register, rc_register);
        self.brillig_context.deallocate_register(rc_register);
    }
}
