use acvm::{
    AcirField,
    acir::{
        brillig::{
            HeapVector, MemoryAddress,
            lengths::{ElementTypesLength, SemanticLength, SemiFlattenedLength},
        },
        circuit::ErrorSelector,
    },
};

use crate::{
    brillig::{assert_u32, assert_usize, brillig_ir::registers::Allocated},
    ssa::ir::instruction::ErrorType,
};

use super::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
    artifact::BrilligParameter,
    brillig_variable::{BrilligVariable, SingleAddrVariable},
    debug_show::DebugToString,
    registers::RegisterAllocator,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn codegen_generic_iteration<T>(
        &mut self,
        make_iterator: impl FnOnce(&mut BrilligContext<F, Registers>) -> T,
        update_iterator: impl FnOnce(&mut BrilligContext<F, Registers>, &T),
        make_finish_condition: impl FnOnce(
            &mut BrilligContext<F, Registers>,
            &T,
        ) -> Allocated<SingleAddrVariable, Registers>,
        on_iteration: impl FnOnce(&mut BrilligContext<F, Registers>, &T),
        clean_iterator: impl FnOnce(&mut BrilligContext<F, Registers>, T),
    ) {
        let iterator = make_iterator(self);

        let (loop_section, loop_label) = self.reserve_next_section_label();
        self.enter_section(loop_section);

        // Loop body
        let should_end = make_finish_condition(self, &iterator);

        let (exit_loop_section, exit_loop_label) = self.reserve_next_section_label();

        self.jump_if_instruction(should_end.address, exit_loop_label);

        // Call the on iteration function
        on_iteration(self, &iterator);

        // Update iterator
        update_iterator(self, &iterator);
        self.jump_instruction(loop_label);

        // Exit the loop
        self.enter_section(exit_loop_section);

        // Free any resources held by the iterator.
        clean_iterator(self, iterator);
    }

    /// This codegen will issue a loop for (let iterator_register = loop_start; i < loop_bound; i += step)
    /// The body of the loop should be issued by the caller in the on_iteration closure.
    ///
    /// # Safety
    /// Iterator increment uses wrapping 32-bit arithmetic. Callers must ensure the iterator
    /// reaches the bound before wrapping. For `loop_start=0, step=1`, this requires
    /// `loop_bound <= u32::MAX`. For pointer iteration, VM allocation checks ensure safety.
    pub(crate) fn codegen_for_loop(
        &mut self,
        loop_start: Option<MemoryAddress>,
        loop_bound: MemoryAddress,
        step: Option<MemoryAddress>,
        on_iteration: impl FnOnce(&mut BrilligContext<F, Registers>, SingleAddrVariable),
    ) {
        let step_register = step.unwrap_or(ReservedRegisters::usize_one());

        self.codegen_generic_iteration(
            // 1. make_iterator: Initialize counter
            |ctx| {
                if let Some(loop_start) = loop_start {
                    let iterator = ctx.allocate_single_addr_usize();
                    ctx.mov_instruction(iterator.address, loop_start);
                    iterator
                } else {
                    ctx.make_usize_constant_instruction(0_usize.into())
                }
            },
            // 2. update_iterator: Add step
            |ctx, iterator| {
                ctx.memory_op_instruction(
                    iterator.address,
                    step_register,
                    iterator.address,
                    BrilligBinaryOp::Add,
                );
            },
            // 3. make_finish_condition: Check bound <= iterator
            |ctx, iterator| {
                let reached_bound = ctx.allocate_single_addr_bool();
                ctx.memory_op_instruction(
                    loop_bound,
                    iterator.address,
                    reached_bound.address,
                    BrilligBinaryOp::LessThanEquals,
                );
                reached_bound
            },
            // 4. on_iteration: Call user's closure
            |ctx, iterator| {
                on_iteration(ctx, **iterator);
            },
            // 5. clean_iterator: Nothing to clean (auto-deallocated)
            |_, _| {},
        );
    }

    /// This codegen will issue a loop that will iterate from 0 to iteration_count.
    /// The body of the loop should be issued by the caller in the on_iteration closure.
    ///
    /// # Safety
    /// `iteration_count` value must not exceed u32::MAX for correct behavior.
    /// See [BrilligContext::codegen_for_loop] for more information.     
    pub(crate) fn codegen_loop(
        &mut self,
        iteration_count: MemoryAddress,
        on_iteration: impl FnOnce(&mut BrilligContext<F, Registers>, SingleAddrVariable),
    ) {
        self.codegen_for_loop(None, iteration_count, None, on_iteration);
    }

    /// This codegen will issue an if-then branch that will check if the condition is true
    /// and if so, perform the instructions given in `f(self, true)` and otherwise perform the
    /// instructions given in `f(self, false)`. A boolean is passed instead of two separate
    /// functions to allow the given function to mutably alias its environment.
    pub(crate) fn codegen_branch(
        &mut self,
        condition: MemoryAddress,
        mut f: impl FnMut(&mut BrilligContext<F, Registers>, bool),
    ) {
        // Reserve 3 sections
        let (then_section, then_label) = self.reserve_next_section_label();
        let (otherwise_section, otherwise_label) = self.reserve_next_section_label();
        let (end_section, end_label) = self.reserve_next_section_label();

        self.jump_if_instruction(condition, then_label);
        self.jump_instruction(otherwise_label);

        self.enter_section(then_section);
        f(self, true);
        self.jump_instruction(end_label);

        self.enter_section(otherwise_section);
        f(self, false);
        // Fall through into `end_section`

        self.enter_section(end_section);
    }

    /// This codegen issues a branch that jumps over the code generated by the given function if the condition is false
    #[expect(unused)]
    pub(crate) fn codegen_if(
        &mut self,
        condition: MemoryAddress,
        f: impl FnOnce(&mut BrilligContext<F, Registers>),
    ) {
        let (end_section, end_label) = self.reserve_next_section_label();
        let (then_section, then_label) = self.reserve_next_section_label();

        self.jump_if_instruction(condition, then_label);
        self.jump_instruction(end_label);

        self.enter_section(then_section);
        f(self);

        self.enter_section(end_section);
    }

    /// This codegen issues a branch that jumps over the code generated by the given function if the condition is truthy
    pub(crate) fn codegen_if_not(
        &mut self,
        condition: MemoryAddress,
        f: impl FnOnce(&mut BrilligContext<F, Registers>),
    ) {
        let (end_section, end_label) = self.reserve_next_section_label();

        self.jump_if_instruction(condition, end_label);

        f(self);

        self.enter_section(end_section);
    }

    /// Jump to a trap condition if `condition` is false.
    /// The trap will include the given message as error data.
    ///
    /// If `error_selector` is None, an empty error data is generated (no error information).
    pub(crate) fn codegen_constrain_with_error_data(
        &mut self,
        condition: SingleAddrVariable,
        error_data_items: Vec<BrilligVariable>,
        error_data_types: Vec<BrilligParameter>,
        error_selector: Option<ErrorSelector>,
    ) {
        assert!(condition.bit_size == 1);

        self.codegen_if_not(condition.address, |ctx| {
            let data_size = Self::flattened_tuple_size(&error_data_types);

            // Special case: No error selector means completely empty error data
            let Some(error_selector) = error_selector else {
                let error_data =
                    ctx.make_usize_constant_instruction(0_usize.into()).map(|size| HeapVector {
                        pointer: ReservedRegisters::free_memory_pointer(),
                        size: size.address,
                    });
                ctx.trap_instruction(*error_data);
                return;
            };

            // Shortcuts: empty data does not need without allocation, and can even use the procedure.
            if data_size == 0 {
                // use the procedure call for better code reuse
                if ctx.can_call_procedures {
                    if let Some(ErrorType::String(message)) =
                        ctx.obj.error_types.get(&error_selector)
                    {
                        ctx.call_error_with_string_procedure(message.clone());
                        return;
                    }
                }

                // Fast path: Just write selector to free memory pointer, no allocation needed
                ctx.indirect_const_instruction(
                    ReservedRegisters::free_memory_pointer(),
                    64,
                    u128::from(error_selector.as_u64()).into(),
                );
                ctx.trap_instruction(HeapVector {
                    pointer: ReservedRegisters::free_memory_pointer(),
                    size: ReservedRegisters::usize_one(),
                });
                return;
            }

            // Allocate buffer and serialize data
            // + 1 due to the error data id being the first item returned
            let error_data_size = data_size + 1;
            let error_data_size_var = ctx.make_usize_constant_instruction(error_data_size.into());
            let error_data = ctx
                .allocate_register()
                .map(|pointer| HeapVector { pointer, size: error_data_size_var.address });
            ctx.codegen_allocate_immediate_mem(error_data.pointer, error_data_size);

            let current_error_data_pointer = ctx.allocate_register();
            ctx.mov_instruction(*current_error_data_pointer, error_data.pointer);
            ctx.indirect_const_instruction(
                *current_error_data_pointer,
                64,
                u128::from(error_selector.as_u64()).into(),
            );

            ctx.codegen_usize_op_in_place(*current_error_data_pointer, BrilligBinaryOp::Add, 1);
            for (error_variable, error_param) in
                error_data_items.into_iter().zip(error_data_types.into_iter())
            {
                let flattened_size = Self::flattened_size(&error_param);
                match error_param {
                    BrilligParameter::SingleAddr(_) => {
                        ctx.store_instruction(
                            *current_error_data_pointer,
                            error_variable.extract_single_addr().address,
                        );
                    }
                    BrilligParameter::Array(item_type, item_count) => {
                        let deflattened_items_pointer =
                            ctx.codegen_make_array_items_pointer(error_variable.extract_array());

                        ctx.flatten_array(
                            &item_type,
                            item_count,
                            *current_error_data_pointer,
                            *deflattened_items_pointer,
                        );
                    }
                    BrilligParameter::Vector(_, _) => {
                        unimplemented!("Vectors are not supported as error data")
                    }
                }
                ctx.codegen_usize_op_in_place(
                    *current_error_data_pointer,
                    BrilligBinaryOp::Add,
                    flattened_size,
                );
            }
            ctx.trap_instruction(*error_data);
        });
    }

    /// Jump to a trap condition if `condition` is false,
    /// with any assertion message written to the error data.
    pub(crate) fn codegen_constrain(
        &mut self,
        condition: SingleAddrVariable,
        assert_message: Option<String>,
    ) {
        debug_assert!(condition.bit_size == 1);

        // Compute error selector if we have a message
        let error_selector = assert_message.map(|message| {
            let error_type = ErrorType::String(message);
            let error_selector = error_type.selector();
            self.obj.error_types.insert(error_selector, error_type);
            error_selector
        });

        self.codegen_constrain_with_error_data(
            condition,
            vec![], // No runtime data items
            vec![], // No runtime data types
            error_selector,
        );
    }

    /// Computes the size of a parameter if it was flattened
    pub(super) fn flattened_size(param: &BrilligParameter) -> usize {
        match param {
            BrilligParameter::SingleAddr(_) => 1,
            BrilligParameter::Array(item_types, item_count)
            | BrilligParameter::Vector(item_types, item_count) => {
                let item_size: usize = item_types.iter().map(Self::flattened_size).sum();
                assert_usize(item_count.0) * item_size
            }
        }
    }

    /// Computes the size of a parameter if it was flattened
    pub(super) fn flattened_tuple_size(tuple: &[BrilligParameter]) -> usize {
        tuple.iter().map(Self::flattened_size).sum()
    }

    /// Computes the size of a parameter if it was flattened
    pub(crate) fn has_nested_arrays(tuple: &[BrilligParameter]) -> bool {
        tuple.iter().any(|param| !matches!(param, BrilligParameter::SingleAddr(_)))
    }

    // Flattens an array by recursively copying nested arrays and regular items.
    pub(super) fn flatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: SemanticLength,
        flattened_array_pointer: MemoryAddress,
        deflattened_items_pointer: MemoryAddress,
    ) {
        if Self::has_nested_arrays(item_type) {
            let movement_register = self.allocate_register();

            let source_item_size = item_type.len();
            let target_item_size = Self::flattened_tuple_size(item_type);

            for item_index in 0..item_count.0 {
                let source_item_base_index = assert_usize(item_index) * source_item_size;
                let target_item_base_index = assert_usize(item_index) * target_item_size;

                let mut target_offset = 0;

                for (subitem_index, subitem) in item_type.iter().enumerate() {
                    let source_index = self.make_usize_constant_instruction(
                        (source_item_base_index + subitem_index).into(),
                    );
                    let target_index = self.make_usize_constant_instruction(
                        (target_item_base_index + target_offset).into(),
                    );

                    match subitem {
                        BrilligParameter::SingleAddr(_) => {
                            self.codegen_load_with_offset(
                                deflattened_items_pointer,
                                *source_index,
                                *movement_register,
                            );
                            self.codegen_store_with_offset(
                                flattened_array_pointer,
                                *target_index,
                                *movement_register,
                            );
                        }
                        BrilligParameter::Array(
                            nested_array_item_type,
                            nested_array_item_count,
                        ) => {
                            // Usually we need to pass a semi-flattened length to `allocate_brillig_array`.
                            // However, since we are deflattening arrays, we need to allocate as many elements
                            // as there are in this particular nested array, which is its semantic length.
                            let deflattened_nested_array = self.allocate_brillig_array(
                                SemiFlattenedLength(nested_array_item_count.0),
                            );

                            self.codegen_load_with_offset(
                                deflattened_items_pointer,
                                *source_index,
                                deflattened_nested_array.pointer,
                            );
                            let deflattened_nested_array_items =
                                self.codegen_make_array_items_pointer(*deflattened_nested_array);

                            let flattened_nested_array_pointer = self.allocate_register();
                            self.memory_op_instruction(
                                flattened_array_pointer,
                                target_index.address,
                                *flattened_nested_array_pointer,
                                BrilligBinaryOp::Add,
                            );

                            self.flatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                *flattened_nested_array_pointer,
                                *deflattened_nested_array_items,
                            );
                        }
                        BrilligParameter::Vector(..) => unreachable!("ICE: Cannot flatten vectors"),
                    }

                    target_offset += Self::flattened_size(subitem);
                }
            }
        } else {
            let size: SemiFlattenedLength =
                item_count * ElementTypesLength(assert_u32(item_type.len()));
            let item_count = self.make_usize_constant_instruction(size.0.into());
            self.codegen_mem_copy(deflattened_items_pointer, flattened_array_pointer, *item_count);
        }
    }
}
