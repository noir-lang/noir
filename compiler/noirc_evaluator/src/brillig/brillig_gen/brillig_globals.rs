use std::sync::Arc;

use acvm::{acir::brillig::MemoryAddress, FieldElement};
use fxhash::FxHashMap as HashMap;

use super::{BrilligVariable, ValueId};
use crate::brillig::{
    allocate_value_with_type, brillig_gen::BrilligBlock, brillig_ir::BrilligContext,
    BrilligBinaryOp, DataFlowGraph, GlobalSpace, Instruction, ReservedRegisters,
    SingleAddrVariable, Type, Value,
};

pub(crate) struct BrilligGlobals<'global> {
    pub(crate) brillig_context: &'global mut BrilligContext<FieldElement, GlobalSpace>,

    brillig_globals: HashMap<ValueId, BrilligVariable>,
}

impl<'global> BrilligGlobals<'global> {
    pub(crate) fn get_globals(self) -> HashMap<ValueId, BrilligVariable> {
        self.brillig_globals
    }

    pub(crate) fn create_brillig_globals(&mut self, globals: &DataFlowGraph) {
        for (id, value) in globals.values_iter() {
            match value {
                Value::NumericConstant { constant, typ } => {
                    let new_variable =
                        allocate_value_with_type(self.brillig_context, Type::Numeric(*typ));
                    self.brillig_context
                        .const_instruction(new_variable.extract_single_addr(), *constant);

                    self.brillig_globals.insert(id, new_variable);
                }
                Value::Instruction { instruction, .. } => {
                    let result = globals.instruction_results(*instruction)[0];
                    let instruction = &globals[*instruction];
                    match &instruction {
                        Instruction::MakeArray { elements: array, typ } => {
                            let new_variable =
                                allocate_value_with_type(self.brillig_context, typ.clone());
                            // Initialize the variable
                            match new_variable {
                                BrilligVariable::BrilligArray(brillig_array) => {
                                    self.brillig_context.codegen_initialize_array(brillig_array);
                                }
                                BrilligVariable::BrilligVector(vector) => {
                                    let size = self
                                        .brillig_context
                                        .make_usize_constant_instruction(array.len().into());
                                    self.brillig_context
                                        .codegen_initialize_vector(vector, size, None);
                                    self.brillig_context.deallocate_single_addr(size);
                                }
                                _ => unreachable!(
                                    "ICE: Cannot initialize array value created as {new_variable:?}"
                                ),
                            };

                            // Write the items
                            let items_pointer = self
                                .brillig_context
                                .codegen_make_array_or_vector_items_pointer(new_variable);

                            self.initialize_constant_array(array, typ, items_pointer);

                            self.brillig_context.deallocate_register(items_pointer);

                            self.brillig_globals.insert(result, new_variable);
                        }
                        _ => {
                            unreachable!("Expected MakeArray instruction but got {instruction:#?}")
                        }
                    }
                }
                _ => {
                    panic!("got something other than numeric constant")
                }
            }
        }
    }

    fn initialize_constant_array(
        &mut self,
        data: &im::Vector<ValueId>,
        typ: &Type,
        pointer: MemoryAddress,
        // brillig_globals: &HashMap<ValueId, BrilligVariable>,
        // globals: &DataFlowGraph,
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
            dbg!("initializing runtime");
            self.initialize_constant_array_runtime(item_types, first_item, item_count, pointer);
        } else {
            dbg!("initializing comptime");
            self.initialize_constant_array_comptime(data, pointer);
        }
    }

    fn initialize_constant_array_runtime(
        &mut self,
        item_types: Arc<Vec<Type>>,
        item_to_repeat: Vec<ValueId>,
        item_count: usize,
        pointer: MemoryAddress,
    ) {
        let mut subitem_to_repeat_variables = Vec::with_capacity(item_types.len());
        for subitem_id in item_to_repeat.into_iter() {
            subitem_to_repeat_variables.push(
                *self
                    .brillig_globals
                    .get(&subitem_id)
                    .unwrap_or_else(|| panic!("ICE: ValueId {subitem_id} is not available")),
            );
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

    fn initialize_constant_array_comptime(
        &mut self,
        data: &im::Vector<crate::ssa::ir::map::Id<Value>>,
        pointer: MemoryAddress,
    ) {
        // Allocate a register for the iterator
        let write_pointer_register = self.brillig_context.allocate_register();

        self.brillig_context.mov_instruction(write_pointer_register, pointer);

        for (element_idx, element_id) in data.iter().enumerate() {
            let element_variable = *self
                .brillig_globals
                .get(&element_id)
                .unwrap_or_else(|| panic!("ICE: ValueId {element_id} is not available"));
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
}
