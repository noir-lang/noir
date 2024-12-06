use acvm::{
    acir::{
        brillig::{HeapVector, MemoryAddress},
        circuit::ErrorSelector,
    },
    AcirField,
};

use crate::ssa::ir::instruction::ErrorType;

use super::{
    artifact::BrilligParameter,
    brillig_variable::{BrilligArray, BrilligVariable, SingleAddrVariable},
    debug_show::DebugToString,
    registers::RegisterAllocator,
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn codegen_generic_iteration<T>(
        &mut self,
        make_iterator: impl FnOnce(&mut BrilligContext<F, Registers>) -> T,
        update_iterator: impl FnOnce(&mut BrilligContext<F, Registers>, &T),
        make_finish_condition: impl FnOnce(&mut BrilligContext<F, Registers>, &T) -> SingleAddrVariable,
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

        // Deallocate our temporary registers
        self.deallocate_single_addr(should_end);
        clean_iterator(self, iterator);
    }

    /// This codegen will issue a loop for (let iterator_register = loop_start; i < loop_bound; i += step)
    /// The body of the loop should be issued by the caller in the on_iteration closure.
    pub(crate) fn codegen_for_loop(
        &mut self,
        loop_start: Option<MemoryAddress>, // Defaults to zero
        loop_bound: MemoryAddress,
        step: Option<MemoryAddress>, // Defaults to 1
        on_iteration: impl FnOnce(&mut BrilligContext<F, Registers>, SingleAddrVariable),
    ) {
        let iterator_register = if let Some(loop_start) = loop_start {
            let iterator_register = SingleAddrVariable::new_usize(self.allocate_register());
            self.mov_instruction(iterator_register.address, loop_start);
            iterator_register
        } else {
            self.make_usize_constant_instruction(0_usize.into())
        };

        let step_register = step.unwrap_or(ReservedRegisters::usize_one());

        let (loop_section, loop_label) = self.reserve_next_section_label();
        self.enter_section(loop_section);

        // Loop body

        // Check if iterator < loop_bound
        let iterator_less_than_iterations =
            SingleAddrVariable { address: self.allocate_register(), bit_size: 1 };

        self.memory_op_instruction(
            iterator_register.address,
            loop_bound,
            iterator_less_than_iterations.address,
            BrilligBinaryOp::LessThan,
        );

        let (exit_loop_section, exit_loop_label) = self.reserve_next_section_label();

        self.not_instruction(iterator_less_than_iterations, iterator_less_than_iterations);

        self.jump_if_instruction(iterator_less_than_iterations.address, exit_loop_label);

        // Call the on iteration function
        on_iteration(self, iterator_register);

        // Add step to the iterator register
        self.memory_op_instruction(
            iterator_register.address,
            step_register,
            iterator_register.address,
            BrilligBinaryOp::Add,
        );

        self.jump_instruction(loop_label);

        // Exit the loop
        self.enter_section(exit_loop_section);

        // Deallocate our temporary registers
        self.deallocate_single_addr(iterator_less_than_iterations);
        self.deallocate_single_addr(iterator_register);
    }

    /// This codegen will issue a loop that will iterate from 0 to iteration_count
    /// The body of the loop should be issued by the caller in the on_iteration closure.
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
        self.jump_instruction(end_label.clone());

        self.enter_section(otherwise_section);
        f(self, false);
        self.jump_instruction(end_label);

        self.enter_section(end_section);
    }

    /// This codegen issues a branch that jumps over the code generated by the given function if the condition is false
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

    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false. The trap will include the given message as revert data.
    pub(crate) fn codegen_constrain_with_revert_data(
        &mut self,
        condition: SingleAddrVariable,
        revert_data_items: Vec<BrilligVariable>,
        revert_data_types: Vec<BrilligParameter>,
        error_selector: ErrorSelector,
    ) {
        assert!(condition.bit_size == 1);

        self.codegen_if_not(condition.address, |ctx| {
            // + 1 due to the revert data id being the first item returned
            let revert_data_size = Self::flattened_tuple_size(&revert_data_types) + 1;
            let revert_data_size_var = ctx.make_usize_constant_instruction(revert_data_size.into());
            let revert_data =
                HeapVector { pointer: ctx.allocate_register(), size: revert_data_size_var.address };
            ctx.codegen_allocate_immediate_mem(revert_data.pointer, revert_data_size);

            let current_revert_data_pointer = ctx.allocate_register();
            ctx.mov_instruction(current_revert_data_pointer, revert_data.pointer);
            ctx.indirect_const_instruction(
                current_revert_data_pointer,
                64,
                (error_selector.as_u64() as u128).into(),
            );

            ctx.codegen_usize_op_in_place(current_revert_data_pointer, BrilligBinaryOp::Add, 1);
            for (revert_variable, revert_param) in
                revert_data_items.into_iter().zip(revert_data_types.into_iter())
            {
                let flattened_size = Self::flattened_size(&revert_param);
                match revert_param {
                    BrilligParameter::SingleAddr(_) => {
                        ctx.store_instruction(
                            current_revert_data_pointer,
                            revert_variable.extract_single_addr().address,
                        );
                    }
                    BrilligParameter::Array(item_type, item_count) => {
                        let deflattened_items_pointer =
                            ctx.codegen_make_array_items_pointer(revert_variable.extract_array());

                        ctx.flatten_array(
                            &item_type,
                            item_count,
                            current_revert_data_pointer,
                            deflattened_items_pointer,
                        );

                        ctx.deallocate_register(deflattened_items_pointer);
                    }
                    BrilligParameter::Slice(_, _) => {
                        unimplemented!("Slices are not supported as revert data")
                    }
                }
                ctx.codegen_usize_op_in_place(
                    current_revert_data_pointer,
                    BrilligBinaryOp::Add,
                    flattened_size,
                );
            }
            ctx.trap_instruction(revert_data);
            ctx.deallocate_single_addr(revert_data_size_var);
            ctx.deallocate_register(revert_data.pointer);
            ctx.deallocate_register(current_revert_data_pointer);
        });
    }

    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn codegen_constrain(
        &mut self,
        condition: SingleAddrVariable,
        assert_message: Option<String>,
    ) {
        assert!(condition.bit_size == 1);

        self.codegen_if_not(condition.address, |ctx| {
            if let Some(assert_message) = assert_message {
                ctx.revert_with_string(assert_message);
            } else {
                let revert_data = HeapVector {
                    pointer: ReservedRegisters::free_memory_pointer(),
                    size: ctx.make_usize_constant_instruction(0_usize.into()).address,
                };
                ctx.trap_instruction(revert_data);
                ctx.deallocate_register(revert_data.size);
            };
        });
    }

    pub(super) fn revert_with_string(&mut self, revert_string: String) {
        if self.can_call_procedures {
            self.call_revert_with_string_procedure(revert_string);
        } else {
            let error_type = ErrorType::String(revert_string);
            let error_selector = error_type.selector();
            self.obj.error_types.insert(error_selector, error_type);
            self.indirect_const_instruction(
                ReservedRegisters::free_memory_pointer(),
                64,
                (error_selector.as_u64() as u128).into(),
            );
            self.trap_instruction(HeapVector {
                pointer: ReservedRegisters::free_memory_pointer(),
                size: ReservedRegisters::usize_one(),
            });
        }
    }

    /// Computes the size of a parameter if it was flattened
    pub(super) fn flattened_size(param: &BrilligParameter) -> usize {
        match param {
            BrilligParameter::SingleAddr(_) => 1,
            BrilligParameter::Array(item_types, item_count)
            | BrilligParameter::Slice(item_types, item_count) => {
                let item_size: usize = item_types.iter().map(Self::flattened_size).sum();
                item_count * item_size
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
        item_count: usize,
        flattened_array_pointer: MemoryAddress,
        deflattened_items_pointer: MemoryAddress,
    ) {
        if Self::has_nested_arrays(item_type) {
            let movement_register = self.allocate_register();

            let source_item_size = item_type.len();
            let target_item_size: usize = item_type.iter().map(Self::flattened_size).sum();

            for item_index in 0..item_count {
                let source_item_base_index = item_index * source_item_size;
                let target_item_base_index = item_index * target_item_size;

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
                                source_index,
                                movement_register,
                            );
                            self.codegen_store_with_offset(
                                flattened_array_pointer,
                                target_index,
                                movement_register,
                            );
                            target_offset += 1;
                        }
                        BrilligParameter::Array(
                            nested_array_item_type,
                            nested_array_item_count,
                        ) => {
                            let deflattened_nested_array = BrilligArray {
                                pointer: self.allocate_register(),
                                size: *nested_array_item_count,
                            };

                            self.codegen_load_with_offset(
                                deflattened_items_pointer,
                                source_index,
                                deflattened_nested_array.pointer,
                            );
                            let deflattened_nested_array_items =
                                self.codegen_make_array_items_pointer(deflattened_nested_array);

                            let flattened_nested_array_pointer = self.allocate_register();
                            self.memory_op_instruction(
                                flattened_array_pointer,
                                target_index.address,
                                flattened_nested_array_pointer,
                                BrilligBinaryOp::Add,
                            );

                            self.flatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                flattened_nested_array_pointer,
                                deflattened_nested_array_items,
                            );

                            self.deallocate_register(deflattened_nested_array.pointer);
                            self.deallocate_register(deflattened_nested_array_items);
                            self.deallocate_register(flattened_nested_array_pointer);

                            target_offset += Self::flattened_size(subitem);
                        }
                        BrilligParameter::Slice(..) => unreachable!("ICE: Cannot flatten slices"),
                    }

                    self.deallocate_single_addr(source_index);
                    self.deallocate_single_addr(target_index);
                }
            }

            self.deallocate_register(movement_register);
        } else {
            let item_count =
                self.make_usize_constant_instruction((item_count * item_type.len()).into());
            self.codegen_mem_copy(deflattened_items_pointer, flattened_array_pointer, item_count);
            self.deallocate_single_addr(item_count);
        }
    }
}
