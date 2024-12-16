use crate::brillig::brillig_ir::artifact::Label;
use crate::brillig::brillig_ir::brillig_variable::{
    type_to_heap_value_type, BrilligArray, BrilligVariable, SingleAddrVariable,
};

use crate::brillig::brillig_ir::registers::Stack;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};
use crate::ssa::ir::dfg::CallStack;
use crate::ssa::ir::instruction::{ConstrainError, Hint};
use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    function::FunctionId,
    instruction::{
        Binary, BinaryOp, Endian, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
    },
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::acir::brillig::{MemoryAddress, ValueOrArray};
use acvm::{acir::AcirField, FieldElement};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use iter_extended::vecmap;
use num_bigint::BigUint;
use std::sync::Arc;

use super::brillig_black_box::convert_black_box_call;
use super::brillig_block_variables::BlockVariables;
use super::brillig_fn::FunctionContext;
use super::constant_allocation::InstructionLocation;

/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligBlock<'block> {
    pub(crate) function_context: &'block mut FunctionContext,
    /// The basic block that is being converted
    pub(crate) block_id: BasicBlockId,
    /// Context for creating brillig opcodes
    pub(crate) brillig_context: &'block mut BrilligContext<FieldElement, Stack>,
    /// Tracks the available variable during the codegen of the block
    pub(crate) variables: BlockVariables,
    /// For each instruction, the set of values that are not used anymore after it.
    pub(crate) last_uses: HashMap<InstructionId, HashSet<ValueId>>,
}

impl<'block> BrilligBlock<'block> {
    /// Converts an SSA Basic block into a sequence of Brillig opcodes
    pub(crate) fn compile(
        function_context: &'block mut FunctionContext,
        brillig_context: &'block mut BrilligContext<FieldElement, Stack>,
        block_id: BasicBlockId,
        dfg: &DataFlowGraph,
    ) {
        let live_in = function_context.liveness.get_live_in(&block_id);
        let variables = BlockVariables::new(live_in.clone());

        brillig_context.set_allocated_registers(
            variables
                .get_available_variables(function_context)
                .into_iter()
                .map(|variable| variable.extract_register())
                .collect(),
        );
        let last_uses = function_context.liveness.get_last_uses(&block_id).clone();

        let mut brillig_block =
            BrilligBlock { function_context, block_id, brillig_context, variables, last_uses };

        brillig_block.convert_block(dfg);
    }

    fn convert_block(&mut self, dfg: &DataFlowGraph) {
        // Add a label for this block
        let block_label = self.create_block_label_for_current_function(self.block_id);
        self.brillig_context.enter_context(block_label);

        self.convert_block_params(dfg);

        let block = &dfg[self.block_id];

        // Convert all of the instructions into the block
        for instruction_id in block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg);
        }

        // Process the block's terminator instruction
        let terminator_instruction =
            block.terminator().expect("block is expected to be constructed");

        self.convert_ssa_terminator(terminator_instruction, dfg);
    }

    /// Creates a unique global label for a block.
    ///
    /// This uses the current functions's function ID and the block ID
    /// Making the assumption that the block ID passed in belongs to this
    /// function.
    fn create_block_label_for_current_function(&self, block_id: BasicBlockId) -> Label {
        Self::create_block_label(self.function_context.function_id, block_id)
    }
    /// Creates a unique label for a block using the function Id and the block ID.
    ///
    /// We implicitly assume that the function ID and the block ID is enough
    /// for us to create a unique label across functions and blocks.
    ///
    /// This is so that during linking there are no duplicates or labels being overwritten.
    fn create_block_label(function_id: FunctionId, block_id: BasicBlockId) -> Label {
        Label::block(function_id, block_id)
    }

    /// Converts an SSA terminator instruction into the necessary opcodes.
    ///
    /// TODO: document why the TerminatorInstruction::Return includes a stop instruction
    /// TODO along with the `Self::compile`
    fn convert_ssa_terminator(
        &mut self,
        terminator_instruction: &TerminatorInstruction,
        dfg: &DataFlowGraph,
    ) {
        self.initialize_constants(
            &self
                .function_context
                .constant_allocation
                .allocated_at_location(self.block_id, InstructionLocation::Terminator),
            dfg,
        );
        match terminator_instruction {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack: _,
            } => {
                let condition = self.convert_ssa_single_addr_value(*condition, dfg);
                self.brillig_context.jump_if_instruction(
                    condition.address,
                    self.create_block_label_for_current_function(*then_destination),
                );
                self.brillig_context.jump_instruction(
                    self.create_block_label_for_current_function(*else_destination),
                );
            }
            TerminatorInstruction::Jmp {
                destination: destination_block,
                arguments,
                call_stack: _,
            } => {
                let target_block = &dfg[*destination_block];
                for (src, dest) in arguments.iter().zip(target_block.parameters()) {
                    // Destinations are block parameters so they should have been allocated previously.
                    let destination =
                        self.variables.get_allocation(self.function_context, *dest, dfg);
                    let source = self.convert_ssa_value(*src, dfg);
                    self.brillig_context
                        .mov_instruction(destination.extract_register(), source.extract_register());
                }
                self.brillig_context.jump_instruction(
                    self.create_block_label_for_current_function(*destination_block),
                );
            }
            TerminatorInstruction::Return { return_values, .. } => {
                let return_registers = vecmap(return_values, |value_id| {
                    self.convert_ssa_value(*value_id, dfg).extract_register()
                });
                self.brillig_context.codegen_return(&return_registers);
            }
        }
    }

    /// Allocates the block parameters that the given block is defining
    fn convert_block_params(&mut self, dfg: &DataFlowGraph) {
        // We don't allocate the block parameters here, we allocate the parameters the block is defining.
        // Since predecessors to a block have to know where the parameters of the block are allocated to pass data to it,
        // the block parameters need to be defined/allocated before the given block. Variable liveness provides when the block parameters are defined.
        // For the entry block, the defined block params will be the params of the function + any extra params of blocks it's the immediate dominator of.
        for param_id in self.function_context.liveness.defined_block_params(&self.block_id) {
            let value = &dfg[param_id];
            let param_type = match value {
                Value::Param { typ, .. } => typ,
                _ => unreachable!("ICE: Only Param type values should appear in block parameters"),
            };
            match param_type {
                // Simple parameters and arrays are passed as already filled registers
                // In the case of arrays, the values should already be in memory and the register should
                // Be a valid pointer to the array.
                // For slices, two registers are passed, the pointer to the data and a register holding the size of the slice.
                Type::Numeric(_) | Type::Array(..) | Type::Slice(..) | Type::Reference(_) => {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        param_id,
                        dfg,
                    );
                }
                Type::Function => todo!("ICE: Type::Function Param not supported"),
            }
        }
    }

    /// Converts an SSA instruction into a sequence of Brillig opcodes.
    fn convert_ssa_instruction(&mut self, instruction_id: InstructionId, dfg: &DataFlowGraph) {
        let instruction = &dfg[instruction_id];
        self.brillig_context.set_call_stack(dfg.get_call_stack(instruction_id));

        self.initialize_constants(
            &self.function_context.constant_allocation.allocated_at_location(
                self.block_id,
                InstructionLocation::Instruction(instruction_id),
            ),
            dfg,
        );
        match instruction {
            Instruction::Binary(binary) => {
                let result_var = self.variables.define_single_addr_variable(
                    self.function_context,
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                self.convert_ssa_binary(binary, dfg, result_var);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                let (condition, deallocate) = match (
                    dfg.get_numeric_constant_with_type(*lhs),
                    dfg.get_numeric_constant_with_type(*rhs),
                ) {
                    // If the constraint is of the form `x == u1 1` then we can simply constrain `x` directly
                    (Some((constant, NumericType::Unsigned { bit_size: 1 })), None)
                        if constant == FieldElement::one() =>
                    {
                        (self.convert_ssa_single_addr_value(*rhs, dfg), false)
                    }
                    (None, Some((constant, NumericType::Unsigned { bit_size: 1 })))
                        if constant == FieldElement::one() =>
                    {
                        (self.convert_ssa_single_addr_value(*lhs, dfg), false)
                    }

                    // Otherwise we need to perform the equality explicitly.
                    _ => {
                        let condition = SingleAddrVariable {
                            address: self.brillig_context.allocate_register(),
                            bit_size: 1,
                        };
                        self.convert_ssa_binary(
                            &Binary { lhs: *lhs, rhs: *rhs, operator: BinaryOp::Eq },
                            dfg,
                            condition,
                        );

                        (condition, true)
                    }
                };

                match assert_message {
                    Some(ConstrainError::Dynamic(selector, _, values)) => {
                        let payload_values =
                            vecmap(values, |value| self.convert_ssa_value(*value, dfg));
                        let payload_as_params = vecmap(values, |value| {
                            let value_type = dfg.type_of_value(*value);
                            FunctionContext::ssa_type_to_parameter(&value_type)
                        });
                        self.brillig_context.codegen_constrain_with_revert_data(
                            condition,
                            payload_values,
                            payload_as_params,
                            *selector,
                        );
                    }
                    Some(ConstrainError::StaticString(message)) => {
                        self.brillig_context.codegen_constrain(condition, Some(message.clone()));
                    }
                    None => {
                        self.brillig_context.codegen_constrain(condition, None);
                    }
                }
                if deallocate {
                    self.brillig_context.deallocate_single_addr(condition);
                }
            }
            Instruction::Allocate => {
                let result_value = dfg.instruction_results(instruction_id)[0];
                let pointer = self.variables.define_single_addr_variable(
                    self.function_context,
                    self.brillig_context,
                    result_value,
                    dfg,
                );
                self.brillig_context.codegen_allocate_immediate_mem(pointer.address, 1);
            }
            Instruction::Store { address, value } => {
                let address_var = self.convert_ssa_single_addr_value(*address, dfg);
                let source_variable = self.convert_ssa_value(*value, dfg);

                self.brillig_context
                    .store_instruction(address_var.address, source_variable.extract_register());
            }
            Instruction::Load { address } => {
                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );

                let address_variable = self.convert_ssa_single_addr_value(*address, dfg);

                self.brillig_context
                    .load_instruction(target_variable.extract_register(), address_variable.address);
            }
            Instruction::Not(value) => {
                let condition_register = self.convert_ssa_single_addr_value(*value, dfg);
                let result_register = self.variables.define_single_addr_variable(
                    self.function_context,
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                self.brillig_context.not_instruction(condition_register, result_register);
            }
            Instruction::Call { func, arguments } => match &dfg[*func] {
                Value::ForeignFunction(func_name) => {
                    let result_ids = dfg.instruction_results(instruction_id);

                    let input_values = vecmap(arguments, |value_id| {
                        let variable = self.convert_ssa_value(*value_id, dfg);
                        self.brillig_context.variable_to_value_or_array(variable)
                    });
                    let input_value_types = vecmap(arguments, |value_id| {
                        let value_type = dfg.type_of_value(*value_id);
                        type_to_heap_value_type(&value_type)
                    });
                    let output_variables = vecmap(result_ids, |value_id| {
                        self.allocate_external_call_result(*value_id, dfg)
                    });
                    let output_values = vecmap(&output_variables, |variable| {
                        self.brillig_context.variable_to_value_or_array(*variable)
                    });
                    let output_value_types = vecmap(result_ids, |value_id| {
                        let value_type = dfg.type_of_value(*value_id);
                        type_to_heap_value_type(&value_type)
                    });
                    self.brillig_context.foreign_call_instruction(
                        func_name.to_owned(),
                        &input_values,
                        &input_value_types,
                        &output_values,
                        &output_value_types,
                    );

                    // Deallocate the temporary heap arrays and vectors of the inputs
                    for input_value in input_values {
                        match input_value {
                            ValueOrArray::HeapArray(array) => {
                                self.brillig_context.deallocate_heap_array(array);
                            }
                            ValueOrArray::HeapVector(vector) => {
                                self.brillig_context.deallocate_heap_vector(vector);
                            }
                            _ => {}
                        }
                    }

                    // Deallocate the temporary heap arrays and vectors of the outputs
                    for (i, (output_register, output_variable)) in
                        output_values.iter().zip(output_variables).enumerate()
                    {
                        match output_register {
                            // Returned vectors need to emit some bytecode to format the result as a BrilligVector
                            ValueOrArray::HeapVector(heap_vector) => {
                                self.brillig_context.initialize_externally_returned_vector(
                                    output_variable.extract_vector(),
                                    *heap_vector,
                                );
                                // Update the dynamic slice length maintained in SSA
                                if let ValueOrArray::MemoryAddress(len_index) = output_values[i - 1]
                                {
                                    let element_size = dfg[result_ids[i]].get_type().element_size();
                                    self.brillig_context
                                        .mov_instruction(len_index, heap_vector.size);
                                    self.brillig_context.codegen_usize_op_in_place(
                                        len_index,
                                        BrilligBinaryOp::UnsignedDiv,
                                        element_size,
                                    );
                                } else {
                                    unreachable!("ICE: a vector must be preceded by a register containing its length");
                                }
                                self.brillig_context.deallocate_heap_vector(*heap_vector);
                            }
                            ValueOrArray::HeapArray(array) => {
                                self.brillig_context.deallocate_heap_array(*array);
                            }
                            ValueOrArray::MemoryAddress(_) => {}
                        }
                    }
                }
                Value::Function(func_id) => {
                    let result_ids = dfg.instruction_results(instruction_id);
                    self.convert_ssa_function_call(*func_id, arguments, dfg, result_ids);
                }
                Value::Intrinsic(intrinsic) => {
                    // This match could be combined with the above but without it rust analyzer
                    // can't automatically insert any missing cases
                    match intrinsic {
                        Intrinsic::ArrayLen => {
                            let result_variable = self.variables.define_single_addr_variable(
                                self.function_context,
                                self.brillig_context,
                                dfg.instruction_results(instruction_id)[0],
                                dfg,
                            );
                            let param_id = arguments[0];
                            // Slices are represented as a tuple in the form: (length, slice contents).
                            // Thus, we can expect the first argument to a field in the case of a slice
                            // or an array in the case of an array.
                            if let Type::Numeric(_) = dfg.type_of_value(param_id) {
                                let len_variable = self.convert_ssa_value(arguments[0], dfg);
                                let length = len_variable.extract_single_addr();
                                self.brillig_context
                                    .mov_instruction(result_variable.address, length.address);
                            } else {
                                self.convert_ssa_array_len(
                                    arguments[0],
                                    result_variable.address,
                                    dfg,
                                );
                            }
                        }
                        Intrinsic::AsSlice => {
                            let source_variable = self.convert_ssa_value(arguments[0], dfg);
                            let result_ids = dfg.instruction_results(instruction_id);
                            let destination_len_variable =
                                self.variables.define_single_addr_variable(
                                    self.function_context,
                                    self.brillig_context,
                                    result_ids[0],
                                    dfg,
                                );
                            let destination_variable = self.variables.define_variable(
                                self.function_context,
                                self.brillig_context,
                                result_ids[1],
                                dfg,
                            );
                            let destination_vector = destination_variable.extract_vector();
                            let source_array = source_variable.extract_array();
                            let element_size = dfg.type_of_value(arguments[0]).element_size();

                            let source_size_register = self
                                .brillig_context
                                .make_usize_constant_instruction(source_array.size.into());

                            // we need to explicitly set the destination_len_variable
                            self.brillig_context.codegen_usize_op(
                                source_size_register.address,
                                destination_len_variable.address,
                                BrilligBinaryOp::UnsignedDiv,
                                element_size,
                            );

                            self.brillig_context.codegen_initialize_vector(
                                destination_vector,
                                source_size_register,
                                None,
                            );

                            // Items
                            let vector_items_pointer = self
                                .brillig_context
                                .codegen_make_vector_items_pointer(destination_vector);
                            let array_items_pointer =
                                self.brillig_context.codegen_make_array_items_pointer(source_array);

                            self.brillig_context.codegen_mem_copy(
                                array_items_pointer,
                                vector_items_pointer,
                                source_size_register,
                            );

                            self.brillig_context.deallocate_single_addr(source_size_register);
                            self.brillig_context.deallocate_register(vector_items_pointer);
                            self.brillig_context.deallocate_register(array_items_pointer);
                        }
                        Intrinsic::SlicePushBack
                        | Intrinsic::SlicePopBack
                        | Intrinsic::SlicePushFront
                        | Intrinsic::SlicePopFront
                        | Intrinsic::SliceInsert
                        | Intrinsic::SliceRemove => {
                            self.convert_ssa_slice_intrinsic_call(
                                dfg,
                                &dfg[dfg.resolve(*func)],
                                instruction_id,
                                arguments,
                            );
                        }
                        Intrinsic::ToBits(endianness) => {
                            let results = dfg.instruction_results(instruction_id);

                            let source = self.convert_ssa_single_addr_value(arguments[0], dfg);

                            let target_array = self
                                .variables
                                .define_variable(
                                    self.function_context,
                                    self.brillig_context,
                                    results[0],
                                    dfg,
                                )
                                .extract_array();

                            let two = self
                                .brillig_context
                                .make_usize_constant_instruction(2_usize.into());

                            self.brillig_context.codegen_to_radix(
                                source,
                                target_array,
                                two,
                                matches!(endianness, Endian::Little),
                                true,
                            );

                            self.brillig_context.deallocate_single_addr(two);
                        }

                        Intrinsic::ToRadix(endianness) => {
                            let results = dfg.instruction_results(instruction_id);

                            let source = self.convert_ssa_single_addr_value(arguments[0], dfg);
                            let radix = self.convert_ssa_single_addr_value(arguments[1], dfg);

                            let target_array = self
                                .variables
                                .define_variable(
                                    self.function_context,
                                    self.brillig_context,
                                    results[0],
                                    dfg,
                                )
                                .extract_array();

                            self.brillig_context.codegen_to_radix(
                                source,
                                target_array,
                                radix,
                                matches!(endianness, Endian::Little),
                                false,
                            );
                        }
                        Intrinsic::Hint(Hint::BlackBox) => {
                            let result_ids = dfg.instruction_results(instruction_id);
                            self.convert_ssa_identity_call(arguments, dfg, result_ids);
                        }
                        Intrinsic::BlackBox(bb_func) => {
                            // Slices are represented as a tuple of (length, slice contents).
                            // We must check the inputs to determine if there are slices
                            // and make sure that we pass the correct inputs to the black box function call.
                            // The loop below only keeps the slice contents, so that
                            // setting up a black box function with slice inputs matches the expected
                            // number of arguments specified in the function signature.
                            let mut arguments_no_slice_len = Vec::new();
                            for (i, arg) in arguments.iter().enumerate() {
                                if matches!(dfg.type_of_value(*arg), Type::Numeric(_)) {
                                    if i < arguments.len() - 1 {
                                        if !matches!(
                                            dfg.type_of_value(arguments[i + 1]),
                                            Type::Slice(_)
                                        ) {
                                            arguments_no_slice_len.push(*arg);
                                        }
                                    } else {
                                        arguments_no_slice_len.push(*arg);
                                    }
                                } else {
                                    arguments_no_slice_len.push(*arg);
                                }
                            }

                            let function_arguments = vecmap(&arguments_no_slice_len, |arg| {
                                self.convert_ssa_value(*arg, dfg)
                            });
                            let function_results = dfg.instruction_results(instruction_id);
                            let function_results = vecmap(function_results, |result| {
                                self.allocate_external_call_result(*result, dfg)
                            });
                            convert_black_box_call(
                                self.brillig_context,
                                bb_func,
                                &function_arguments,
                                &function_results,
                            );
                        }
                        // `Intrinsic::AsWitness` is used to provide hints to acir-gen on optimal expression splitting.
                        // It is then useless in the brillig runtime and so we can ignore it
                        Intrinsic::AsWitness => (),
                        Intrinsic::FieldLessThan => {
                            let lhs = self.convert_ssa_single_addr_value(arguments[0], dfg);
                            assert!(lhs.bit_size == FieldElement::max_num_bits());
                            let rhs = self.convert_ssa_single_addr_value(arguments[1], dfg);
                            assert!(rhs.bit_size == FieldElement::max_num_bits());

                            let results = dfg.instruction_results(instruction_id);
                            let destination = self
                                .variables
                                .define_variable(
                                    self.function_context,
                                    self.brillig_context,
                                    results[0],
                                    dfg,
                                )
                                .extract_single_addr();
                            assert!(destination.bit_size == 1);

                            self.brillig_context.binary_instruction(
                                lhs,
                                rhs,
                                destination,
                                BrilligBinaryOp::LessThan,
                            );
                        }
                        Intrinsic::ArrayRefCount | Intrinsic::SliceRefCount => {
                            let array = self.convert_ssa_value(arguments[0], dfg);
                            let result = dfg.instruction_results(instruction_id)[0];

                            let destination = self.variables.define_variable(
                                self.function_context,
                                self.brillig_context,
                                result,
                                dfg,
                            );
                            let destination = destination.extract_register();
                            let array = array.extract_register();
                            self.brillig_context.load_instruction(destination, array);
                        }
                        Intrinsic::FromField
                        | Intrinsic::AsField
                        | Intrinsic::IsUnconstrained
                        | Intrinsic::DerivePedersenGenerators
                        | Intrinsic::ApplyRangeConstraint
                        | Intrinsic::StrAsBytes
                        | Intrinsic::AssertConstant
                        | Intrinsic::StaticAssert
                        | Intrinsic::ArrayAsStrUnchecked => {
                            unreachable!("unsupported function call type {:?}", dfg[*func])
                        }
                    }
                }
                Value::Instruction { .. } | Value::Param { .. } | Value::NumericConstant { .. } => {
                    unreachable!("unsupported function call type {:?}", dfg[*func])
                }
            },
            Instruction::Truncate { value, bit_size, .. } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination_register = self.variables.define_single_addr_variable(
                    self.function_context,
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );
                let source_register = self.convert_ssa_single_addr_value(*value, dfg);
                self.brillig_context.codegen_truncate(
                    destination_register,
                    source_register,
                    *bit_size,
                );
            }
            Instruction::Cast(value, _) => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination_variable = self.variables.define_single_addr_variable(
                    self.function_context,
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );
                let source_variable = self.convert_ssa_single_addr_value(*value, dfg);
                self.convert_cast(destination_variable, source_variable);
            }
            Instruction::ArrayGet { array, index } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );

                let array_variable = self.convert_ssa_value(*array, dfg);

                let index_variable = self.convert_ssa_single_addr_value(*index, dfg);

                if !dfg.is_safe_index(*index, *array) {
                    self.validate_array_index(array_variable, index_variable);
                }

                let items_pointer =
                    self.brillig_context.codegen_make_array_or_vector_items_pointer(array_variable);

                self.brillig_context.codegen_load_with_offset(
                    items_pointer,
                    index_variable,
                    destination_variable.extract_register(),
                );

                self.brillig_context.deallocate_register(items_pointer);
            }
            Instruction::ArraySet { array, index, value, mutable } => {
                let source_variable = self.convert_ssa_value(*array, dfg);
                let index_register = self.convert_ssa_single_addr_value(*index, dfg);
                let value_variable = self.convert_ssa_value(*value, dfg);

                let result_ids = dfg.instruction_results(instruction_id);
                let destination_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );

                if !dfg.is_safe_index(*index, *array) {
                    self.validate_array_index(source_variable, index_register);
                }

                self.convert_ssa_array_set(
                    source_variable,
                    destination_variable,
                    index_register,
                    value_variable,
                    *mutable,
                );
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                let value = self.convert_ssa_single_addr_value(*value, dfg);
                // SSA generates redundant range checks. A range check with a max bit size >= value.bit_size will always pass.
                if value.bit_size > *max_bit_size {
                    // Cast original value to field
                    let left = SingleAddrVariable {
                        address: self.brillig_context.allocate_register(),
                        bit_size: FieldElement::max_num_bits(),
                    };
                    self.convert_cast(left, value);

                    // Create a field constant with the max
                    let max = BigUint::from(2_u128).pow(*max_bit_size) - BigUint::from(1_u128);
                    let right = self.brillig_context.make_constant_instruction(
                        FieldElement::from_be_bytes_reduce(&max.to_bytes_be()),
                        FieldElement::max_num_bits(),
                    );

                    // Check if lte max
                    let condition =
                        SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                    self.brillig_context.binary_instruction(
                        left,
                        right,
                        condition,
                        BrilligBinaryOp::LessThanEquals,
                    );

                    self.brillig_context.codegen_constrain(condition, assert_message.clone());
                    self.brillig_context.deallocate_single_addr(condition);
                    self.brillig_context.deallocate_single_addr(left);
                    self.brillig_context.deallocate_single_addr(right);
                }
            }
            Instruction::IncrementRc { value } => {
                let array_or_vector = self.convert_ssa_value(*value, dfg);
                let rc_register = self.brillig_context.allocate_register();

                // RC is always directly pointed by the array/vector pointer
                self.brillig_context
                    .load_instruction(rc_register, array_or_vector.extract_register());
                self.brillig_context.codegen_usize_op_in_place(
                    rc_register,
                    BrilligBinaryOp::Add,
                    1,
                );
                self.brillig_context
                    .store_instruction(array_or_vector.extract_register(), rc_register);
                self.brillig_context.deallocate_register(rc_register);
            }
            Instruction::DecrementRc { value } => {
                let array_or_vector = self.convert_ssa_value(*value, dfg);
                let rc_register = self.brillig_context.allocate_register();

                self.brillig_context
                    .load_instruction(rc_register, array_or_vector.extract_register());
                self.brillig_context.codegen_usize_op_in_place(
                    rc_register,
                    BrilligBinaryOp::Sub,
                    1,
                );
                self.brillig_context
                    .store_instruction(array_or_vector.extract_register(), rc_register);
                self.brillig_context.deallocate_register(rc_register);
            }
            Instruction::EnableSideEffectsIf { .. } => {
                todo!("enable_side_effects not supported by brillig")
            }
            Instruction::IfElse { .. } => {
                unreachable!("IfElse instructions should not be possible in brillig")
            }
            Instruction::MakeArray { elements: array, typ } => {
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
                            let size = self
                                .brillig_context
                                .make_usize_constant_instruction(array.len().into());
                            self.brillig_context.codegen_initialize_vector(vector, size, None);
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

                    self.initialize_constant_array(array, typ, dfg, items_pointer);

                    self.brillig_context.deallocate_register(items_pointer);
                }
            }
        };

        let dead_variables = self
            .last_uses
            .get(&instruction_id)
            .expect("Last uses for instruction should have been computed");

        for dead_variable in dead_variables {
            self.variables.remove_variable(
                dead_variable,
                self.function_context,
                self.brillig_context,
            );
        }
        self.brillig_context.set_call_stack(CallStack::new());
    }

    fn convert_ssa_function_call(
        &mut self,
        func_id: FunctionId,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) {
        let argument_variables =
            vecmap(arguments, |argument_id| self.convert_ssa_value(*argument_id, dfg));
        let return_variables = vecmap(result_ids, |result_id| {
            self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                *result_id,
                dfg,
            )
        });
        self.brillig_context.codegen_call(func_id, &argument_variables, &return_variables);
    }

    /// Copy the input arguments to the results.
    fn convert_ssa_identity_call(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) {
        let argument_variables =
            vecmap(arguments, |argument_id| self.convert_ssa_value(*argument_id, dfg));

        let return_variables = vecmap(result_ids, |result_id| {
            self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                *result_id,
                dfg,
            )
        });

        for (src, dst) in argument_variables.into_iter().zip(return_variables) {
            self.brillig_context.mov_instruction(dst.extract_register(), src.extract_register());
        }
    }

    fn validate_array_index(
        &mut self,
        array_variable: BrilligVariable,
        index_register: SingleAddrVariable,
    ) {
        let size = self.brillig_context.codegen_make_array_or_vector_length(array_variable);

        let condition = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);

        self.brillig_context.memory_op_instruction(
            index_register.address,
            size.address,
            condition.address,
            BrilligBinaryOp::LessThan,
        );

        self.brillig_context
            .codegen_constrain(condition, Some("Array index out of bounds".to_owned()));

        self.brillig_context.deallocate_single_addr(size);
        self.brillig_context.deallocate_single_addr(condition);
    }

    /// Array set operation in SSA returns a new array or slice that is a copy of the parameter array or slice
    /// With a specific value changed.
    ///
    /// Returns `source_size_as_register`, which is expected to be deallocated with:
    /// `self.brillig_context.deallocate_register(source_size_as_register)`
    fn convert_ssa_array_set(
        &mut self,
        source_variable: BrilligVariable,
        destination_variable: BrilligVariable,
        index_register: SingleAddrVariable,
        value_variable: BrilligVariable,
        mutable: bool,
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
        let items_pointer =
            self.brillig_context.codegen_make_array_or_vector_items_pointer(destination_for_store);

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

        self.brillig_context.deallocate_register(items_pointer);
    }

    /// Convert the SSA slice operations to brillig slice operations
    fn convert_ssa_slice_intrinsic_call(
        &mut self,
        dfg: &DataFlowGraph,
        intrinsic: &Value,
        instruction_id: InstructionId,
        arguments: &[ValueId],
    ) {
        let slice_id = arguments[1];
        let element_size = dfg.type_of_value(slice_id).element_size();
        let source_vector = self.convert_ssa_value(slice_id, dfg).extract_vector();

        let results = dfg.instruction_results(instruction_id);
        match intrinsic {
            Value::Intrinsic(Intrinsic::SlicePushBack) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[1],
                    dfg,
                );

                let target_vector = target_variable.extract_vector();
                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(
                    target_len.address,
                    arguments[0],
                    dfg,
                    BrilligBinaryOp::Add,
                );

                self.slice_push_back_operation(target_vector, source_vector, &item_values);
            }
            Value::Intrinsic(Intrinsic::SlicePushFront) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[1],
                    dfg,
                );
                let target_vector = target_variable.extract_vector();
                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(
                    target_len.address,
                    arguments[0],
                    dfg,
                    BrilligBinaryOp::Add,
                );

                self.slice_push_front_operation(target_vector, source_vector, &item_values);
            }
            Value::Intrinsic(Intrinsic::SlicePopBack) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[1],
                    dfg,
                );

                let target_vector = target_variable.extract_vector();

                let pop_variables = vecmap(&results[2..element_size + 2], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_slice_length(
                    target_len.address,
                    arguments[0],
                    dfg,
                    BrilligBinaryOp::Sub,
                );

                self.slice_pop_back_operation(target_vector, source_vector, &pop_variables);
            }
            Value::Intrinsic(Intrinsic::SlicePopFront) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[element_size],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let pop_variables = vecmap(&results[0..element_size], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[element_size + 1],
                    dfg,
                );
                let target_vector = target_variable.extract_vector();

                self.update_slice_length(
                    target_len.address,
                    arguments[0],
                    dfg,
                    BrilligBinaryOp::Sub,
                );

                self.slice_pop_front_operation(target_vector, source_vector, &pop_variables);
            }
            Value::Intrinsic(Intrinsic::SliceInsert) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_id = results[1];
                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    target_id,
                    dfg,
                );

                let target_vector = target_variable.extract_vector();

                // Remove if indexing in insert is changed to flattened indexing
                // https://github.com/noir-lang/noir/issues/1889#issuecomment-1668048587
                let user_index = self.convert_ssa_single_addr_value(arguments[2], dfg);

                let converted_index =
                    self.brillig_context.make_usize_constant_instruction(element_size.into());

                self.brillig_context.memory_op_instruction(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BrilligBinaryOp::Mul,
                );

                let items = vecmap(&arguments[3..element_size + 3], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(
                    target_len.address,
                    arguments[0],
                    dfg,
                    BrilligBinaryOp::Add,
                );

                self.slice_insert_operation(target_vector, source_vector, converted_index, &items);
                self.brillig_context.deallocate_single_addr(converted_index);
            }
            Value::Intrinsic(Intrinsic::SliceRemove) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_id = results[1];

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    target_id,
                    dfg,
                );
                let target_vector = target_variable.extract_vector();

                // Remove if indexing in remove is changed to flattened indexing
                // https://github.com/noir-lang/noir/issues/1889#issuecomment-1668048587
                let user_index = self.convert_ssa_single_addr_value(arguments[2], dfg);

                let converted_index =
                    self.brillig_context.make_usize_constant_instruction(element_size.into());
                self.brillig_context.memory_op_instruction(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BrilligBinaryOp::Mul,
                );

                let removed_items = vecmap(&results[2..element_size + 2], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_slice_length(
                    target_len.address,
                    arguments[0],
                    dfg,
                    BrilligBinaryOp::Sub,
                );

                self.slice_remove_operation(
                    target_vector,
                    source_vector,
                    converted_index,
                    &removed_items,
                );

                self.brillig_context.deallocate_single_addr(converted_index);
            }
            _ => unreachable!("ICE: Slice operation not supported"),
        }
    }

    /// Slices have a tuple structure (slice length, slice contents) to enable logic
    /// that uses dynamic slice lengths (such as with merging slices in the flattening pass).
    /// This method codegens an update to the slice length.
    ///
    /// The binary operation performed on the slice length is always an addition or subtraction of `1`.
    /// This is because the slice length holds the user length (length as displayed by a `.len()` call),
    /// and not a flattened length used internally to represent arrays of tuples.
    /// The length inside of `RegisterOrMemory::HeapVector` represents the entire flattened number
    /// of fields in the vector.
    fn update_slice_length(
        &mut self,
        target_len: MemoryAddress,
        source_value: ValueId,
        dfg: &DataFlowGraph,
        binary_op: BrilligBinaryOp,
    ) {
        let source_len_variable = self.convert_ssa_value(source_value, dfg);
        let source_len = source_len_variable.extract_single_addr();

        self.brillig_context.codegen_usize_op(source_len.address, target_len, binary_op, 1);
    }

    /// Converts an SSA cast to a sequence of Brillig opcodes.
    /// Casting is only necessary when shrinking the bit size of a numeric value.
    fn convert_cast(&mut self, destination: SingleAddrVariable, source: SingleAddrVariable) {
        // We assume that `source` is a valid `target_type` as it's expected that a truncate instruction was emitted
        // to ensure this is the case.

        self.brillig_context.cast_instruction(destination, source);
    }

    /// Converts the Binary instruction into a sequence of Brillig opcodes.
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
        result_variable: SingleAddrVariable,
    ) {
        let binary_type = type_of_binary_operation(
            dfg[binary.lhs].get_type().as_ref(),
            dfg[binary.rhs].get_type().as_ref(),
            binary.operator,
        );

        let left = self.convert_ssa_single_addr_value(binary.lhs, dfg);
        let right = self.convert_ssa_single_addr_value(binary.rhs, dfg);

        let (is_field, is_signed) = match binary_type {
            Type::Numeric(numeric_type) => match numeric_type {
                NumericType::Signed { .. } => (false, true),
                NumericType::Unsigned { .. } => (false, false),
                NumericType::NativeField => (true, false),
            },
            _ => unreachable!("only numeric types are allowed in binary operations. References are handled separately"),
        };

        let brillig_binary_op = match binary.operator {
            BinaryOp::Div => {
                if is_signed {
                    self.brillig_context.convert_signed_division(left, right, result_variable);
                    return;
                } else if is_field {
                    BrilligBinaryOp::FieldDiv
                } else {
                    BrilligBinaryOp::UnsignedDiv
                }
            }
            BinaryOp::Mod => {
                if is_signed {
                    self.convert_signed_modulo(left, right, result_variable);
                    return;
                } else {
                    BrilligBinaryOp::Modulo
                }
            }
            BinaryOp::Add => BrilligBinaryOp::Add,
            BinaryOp::Sub => BrilligBinaryOp::Sub,
            BinaryOp::Mul => BrilligBinaryOp::Mul,
            BinaryOp::Eq => BrilligBinaryOp::Equals,
            BinaryOp::Lt => {
                if is_signed {
                    self.convert_signed_less_than(left, right, result_variable);
                    return;
                } else {
                    BrilligBinaryOp::LessThan
                }
            }
            BinaryOp::And => BrilligBinaryOp::And,
            BinaryOp::Or => BrilligBinaryOp::Or,
            BinaryOp::Xor => BrilligBinaryOp::Xor,
            BinaryOp::Shl => BrilligBinaryOp::Shl,
            BinaryOp::Shr => {
                if is_signed {
                    self.convert_signed_shr(left, right, result_variable);
                    return;
                } else {
                    BrilligBinaryOp::Shr
                }
            }
        };

        self.brillig_context.binary_instruction(left, right, result_variable, brillig_binary_op);

        self.add_overflow_check(
            brillig_binary_op,
            left,
            right,
            result_variable,
            binary,
            dfg,
            is_signed,
        );
    }

    fn convert_signed_modulo(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let scratch_var_i =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);
        let scratch_var_j =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);

        // i = left / right
        self.brillig_context.convert_signed_division(left, right, scratch_var_i);

        // j = i * right
        self.brillig_context.binary_instruction(
            scratch_var_i,
            right,
            scratch_var_j,
            BrilligBinaryOp::Mul,
        );

        // result_register = left - j
        self.brillig_context.binary_instruction(left, scratch_var_j, result, BrilligBinaryOp::Sub);
        // Free scratch registers
        self.brillig_context.deallocate_single_addr(scratch_var_i);
        self.brillig_context.deallocate_single_addr(scratch_var_j);
    }

    fn convert_signed_less_than(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let biased_left =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);
        let biased_right =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), right.bit_size);

        let bias = self
            .brillig_context
            .make_constant_instruction((1_u128 << (left.bit_size - 1)).into(), left.bit_size);

        self.brillig_context.binary_instruction(left, bias, biased_left, BrilligBinaryOp::Add);
        self.brillig_context.binary_instruction(right, bias, biased_right, BrilligBinaryOp::Add);

        self.brillig_context.binary_instruction(
            biased_left,
            biased_right,
            result,
            BrilligBinaryOp::LessThan,
        );

        self.brillig_context.deallocate_single_addr(biased_left);
        self.brillig_context.deallocate_single_addr(biased_right);
        self.brillig_context.deallocate_single_addr(bias);
    }

    fn convert_signed_shr(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        // Check if left is negative
        let left_is_negative = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
        let max_positive = self
            .brillig_context
            .make_constant_instruction(((1_u128 << (left.bit_size - 1)) - 1).into(), left.bit_size);
        self.brillig_context.binary_instruction(
            max_positive,
            left,
            left_is_negative,
            BrilligBinaryOp::LessThan,
        );

        self.brillig_context.codegen_branch(left_is_negative.address, |ctx, is_negative| {
            if is_negative {
                let one = ctx.make_constant_instruction(1_u128.into(), left.bit_size);

                // computes 2^right
                let two = ctx.make_constant_instruction(2_u128.into(), left.bit_size);
                let two_pow = ctx.make_constant_instruction(1_u128.into(), left.bit_size);
                let right_u32 = SingleAddrVariable::new(ctx.allocate_register(), 32);
                ctx.cast(right_u32, right);
                let pow_body = |ctx: &mut BrilligContext<_, _>, _: SingleAddrVariable| {
                    ctx.binary_instruction(two_pow, two, two_pow, BrilligBinaryOp::Mul);
                };
                ctx.codegen_for_loop(None, right_u32.address, None, pow_body);

                // Right shift using division on 1-complement
                ctx.binary_instruction(left, one, result, BrilligBinaryOp::Add);
                ctx.convert_signed_division(result, two_pow, result);
                ctx.binary_instruction(result, one, result, BrilligBinaryOp::Sub);

                // Clean-up
                ctx.deallocate_single_addr(one);
                ctx.deallocate_single_addr(two);
                ctx.deallocate_single_addr(two_pow);
                ctx.deallocate_single_addr(right_u32);
            } else {
                ctx.binary_instruction(left, right, result, BrilligBinaryOp::Shr);
            }
        });

        self.brillig_context.deallocate_single_addr(left_is_negative);
    }

    #[allow(clippy::too_many_arguments)]
    fn add_overflow_check(
        &mut self,
        binary_operation: BrilligBinaryOp,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
        binary: &Binary,
        dfg: &DataFlowGraph,
        is_signed: bool,
    ) {
        let bit_size = left.bit_size;
        let max_lhs_bits = dfg.get_value_max_num_bits(binary.lhs);
        let max_rhs_bits = dfg.get_value_max_num_bits(binary.rhs);

        if bit_size == FieldElement::max_num_bits() {
            return;
        }

        match (binary_operation, is_signed) {
            (BrilligBinaryOp::Add, false) => {
                if std::cmp::max(max_lhs_bits, max_rhs_bits) < bit_size {
                    // `left` and `right` have both been casted up from smaller types and so cannot overflow.
                    return;
                }

                let condition =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                // Check that lhs <= result
                self.brillig_context.binary_instruction(
                    left,
                    result,
                    condition,
                    BrilligBinaryOp::LessThanEquals,
                );
                self.brillig_context
                    .codegen_constrain(condition, Some("attempt to add with overflow".to_string()));
                self.brillig_context.deallocate_single_addr(condition);
            }
            (BrilligBinaryOp::Sub, false) => {
                if dfg.is_constant(binary.lhs) && max_lhs_bits > max_rhs_bits {
                    // `left` is a fixed constant and `right` is restricted such that `left - right > 0`
                    // Note strict inequality as `right > left` while `max_lhs_bits == max_rhs_bits` is possible.
                    return;
                }

                let condition =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                // Check that rhs <= lhs
                self.brillig_context.binary_instruction(
                    right,
                    left,
                    condition,
                    BrilligBinaryOp::LessThanEquals,
                );
                self.brillig_context.codegen_constrain(
                    condition,
                    Some("attempt to subtract with overflow".to_string()),
                );
                self.brillig_context.deallocate_single_addr(condition);
            }
            (BrilligBinaryOp::Mul, false) => {
                if bit_size == 1 || max_lhs_bits + max_rhs_bits <= bit_size {
                    // Either performing boolean multiplication (which cannot overflow),
                    // or `left` and `right` have both been casted up from smaller types and so cannot overflow.
                    return;
                }

                let is_right_zero =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                let zero = self.brillig_context.make_constant_instruction(0_usize.into(), bit_size);
                self.brillig_context.binary_instruction(
                    zero,
                    right,
                    is_right_zero,
                    BrilligBinaryOp::Equals,
                );
                self.brillig_context.codegen_if_not(is_right_zero.address, |ctx| {
                    let condition = SingleAddrVariable::new(ctx.allocate_register(), 1);
                    let division = SingleAddrVariable::new(ctx.allocate_register(), bit_size);
                    // Check that result / rhs == lhs
                    ctx.binary_instruction(result, right, division, BrilligBinaryOp::UnsignedDiv);
                    ctx.binary_instruction(division, left, condition, BrilligBinaryOp::Equals);
                    ctx.codegen_constrain(
                        condition,
                        Some("attempt to multiply with overflow".to_string()),
                    );
                    ctx.deallocate_single_addr(condition);
                    ctx.deallocate_single_addr(division);
                });
                self.brillig_context.deallocate_single_addr(is_right_zero);
                self.brillig_context.deallocate_single_addr(zero);
            }
            _ => {}
        }
    }

    fn initialize_constants(&mut self, constants: &[ValueId], dfg: &DataFlowGraph) {
        for &constant_id in constants {
            self.convert_ssa_value(constant_id, dfg);
        }
    }

    /// Converts an SSA `ValueId` into a `RegisterOrMemory`. Initializes if necessary.
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> BrilligVariable {
        let value_id = dfg.resolve(value_id);
        let value = &dfg[value_id];

        match value {
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.

                self.variables.get_allocation(self.function_context, value_id, dfg)
            }
            Value::NumericConstant { constant, .. } => {
                // Constants might have been converted previously or not, so we get or create and
                // (re)initialize the value inside.
                if self.variables.is_allocated(&value_id) {
                    self.variables.get_allocation(self.function_context, value_id, dfg)
                } else {
                    let new_variable = self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        value_id,
                        dfg,
                    );

                    self.brillig_context
                        .const_instruction(new_variable.extract_single_addr(), *constant);
                    new_variable
                }
            }
            Value::Function(_) => {
                // For the debugger instrumentation we want to allow passing
                // around values representing function pointers, even though
                // there is no interaction with the function possible given that
                // value.
                let new_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    value_id,
                    dfg,
                );

                self.brillig_context.const_instruction(
                    new_variable.extract_single_addr(),
                    value_id.to_u32().into(),
                );
                new_variable
            }
            Value::Intrinsic(_) | Value::ForeignFunction(_) => {
                todo!("ICE: Cannot convert value {value:?}")
            }
        }
    }

    fn initialize_constant_array(
        &mut self,
        data: &im::Vector<ValueId>,
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

    fn initialize_constant_array_comptime(
        &mut self,
        data: &im::Vector<crate::ssa::ir::map::Id<Value>>,
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

    /// Converts an SSA `ValueId` into a `MemoryAddress`. Initializes if necessary.
    fn convert_ssa_single_addr_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> SingleAddrVariable {
        let variable = self.convert_ssa_value(value_id, dfg);
        variable.extract_single_addr()
    }

    fn allocate_external_call_result(
        &mut self,
        result: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let typ = dfg[result].get_type();
        match typ.as_ref() {
            Type::Numeric(_) => self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                result,
                dfg,
            ),

            Type::Array(..) => {
                let variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    result,
                    dfg,
                );
                let array = variable.extract_array();
                self.allocate_foreign_call_result_array(typ.as_ref(), array);

                variable
            }
            Type::Slice(_) => {
                let variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    result,
                    dfg,
                );
                let vector = variable.extract_vector();

                // Set the pointer to the current stack frame
                // The stack pointer will then be updated by the caller of this method
                // once the external call is resolved and the array size is known
                self.brillig_context.load_free_memory_pointer_instruction(vector.pointer);

                variable
            }
            _ => {
                unreachable!("ICE: unsupported return type for black box call {typ:?}")
            }
        }
    }

    fn allocate_foreign_call_result_array(&mut self, typ: &Type, array: BrilligArray) {
        let Type::Array(types, size) = typ else {
            unreachable!("ICE: allocate_foreign_call_array() expects an array, got {typ:?}")
        };

        self.brillig_context.codegen_initialize_array(array);

        let mut index = 0_usize;
        for _ in 0..*size {
            for element_type in types.iter() {
                match element_type {
                    Type::Array(_, nested_size) => {
                        let inner_array = BrilligArray {
                            pointer: self.brillig_context.allocate_register(),
                            size: *nested_size as usize,
                        };
                        self.allocate_foreign_call_result_array(element_type, inner_array);

                        // We add one since array.pointer points to [RC, ...items]
                        let idx =
                            self.brillig_context.make_usize_constant_instruction((index + 1).into()  );
                        self.brillig_context.codegen_store_with_offset(array.pointer, idx, inner_array.pointer);

                        self.brillig_context.deallocate_single_addr(idx);
                        self.brillig_context.deallocate_register(inner_array.pointer);
                    }
                    Type::Slice(_) => unreachable!("ICE: unsupported slice type in allocate_nested_array(), expects an array or a numeric type"),
                    _ => (),
                }
                index += 1;
            }
        }
    }

    /// Gets the "user-facing" length of an array.
    /// An array of structs with two fields would be stored as an 2 * array.len() array/vector.
    /// So we divide the length by the number of subitems in an item to get the user-facing length.
    fn convert_ssa_array_len(
        &mut self,
        array_id: ValueId,
        result_register: MemoryAddress,
        dfg: &DataFlowGraph,
    ) {
        let array_variable = self.convert_ssa_value(array_id, dfg);
        let element_size = dfg.type_of_value(array_id).element_size();

        match array_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, .. }) => {
                self.brillig_context
                    .usize_const_instruction(result_register, (size / element_size).into());
            }
            BrilligVariable::BrilligVector(vector) => {
                let size = self.brillig_context.codegen_make_vector_length(vector);

                self.brillig_context.codegen_usize_op(
                    size.address,
                    result_register,
                    BrilligBinaryOp::UnsignedDiv,
                    element_size,
                );

                self.brillig_context.deallocate_single_addr(size);
            }
            _ => {
                unreachable!("ICE: Cannot get length of {array_variable:?}")
            }
        }
    }
}

/// Returns the type of the operation considering the types of the operands
pub(crate) fn type_of_binary_operation(lhs_type: &Type, rhs_type: &Type, op: BinaryOp) -> Type {
    match (lhs_type, rhs_type) {
        (_, Type::Function) | (Type::Function, _) => {
            unreachable!("Functions are invalid in binary operations")
        }
        (_, Type::Reference(_)) | (Type::Reference(_), _) => {
            unreachable!("References are invalid in binary operations")
        }
        (_, Type::Array(..)) | (Type::Array(..), _) => {
            unreachable!("Arrays are invalid in binary operations")
        }
        (_, Type::Slice(..)) | (Type::Slice(..), _) => {
            unreachable!("Arrays are invalid in binary operations")
        }
        // If both sides are numeric type, then we expect their types to be
        // the same.
        (Type::Numeric(lhs_type), Type::Numeric(rhs_type))
            if op != BinaryOp::Shl && op != BinaryOp::Shr =>
        {
            assert_eq!(
                lhs_type, rhs_type,
                "lhs and rhs types in a binary operation are always the same but got {lhs_type} and {rhs_type}"
            );
            Type::Numeric(*lhs_type)
        }
        _ => lhs_type.clone(),
    }
}
