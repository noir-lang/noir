use crate::brillig::brillig_ir::brillig_variable::{
    type_to_heap_value_type, BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable,
};
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};
use crate::ssa::ir::dfg::CallStack;
use crate::ssa::ir::instruction::ConstrainError;
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
use acvm::brillig_vm::brillig::HeapVector;
use acvm::{acir::AcirField, FieldElement};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use iter_extended::vecmap;
use num_bigint::BigUint;

use super::brillig_black_box::convert_black_box_call;
use super::brillig_block_variables::BlockVariables;
use super::brillig_fn::FunctionContext;

/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligBlock<'block> {
    pub(crate) function_context: &'block mut FunctionContext,
    /// The basic block that is being converted
    pub(crate) block_id: BasicBlockId,
    /// Context for creating brillig opcodes
    pub(crate) brillig_context: &'block mut BrilligContext,
    /// Tracks the available variable during the codegen of the block
    pub(crate) variables: BlockVariables,
    /// For each instruction, the set of values that are not used anymore after it.
    pub(crate) last_uses: HashMap<InstructionId, HashSet<ValueId>>,
}

impl<'block> BrilligBlock<'block> {
    /// Converts an SSA Basic block into a sequence of Brillig opcodes
    pub(crate) fn compile(
        function_context: &'block mut FunctionContext,
        brillig_context: &'block mut BrilligContext,
        block_id: BasicBlockId,
        dfg: &DataFlowGraph,
    ) {
        let live_in = function_context.liveness.get_live_in(&block_id);
        let variables = BlockVariables::new(live_in.clone());

        brillig_context.set_allocated_registers(
            variables
                .get_available_variables(function_context)
                .into_iter()
                .flat_map(|variable| variable.extract_registers())
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
    fn create_block_label_for_current_function(&self, block_id: BasicBlockId) -> String {
        Self::create_block_label(self.function_context.function_id, block_id)
    }
    /// Creates a unique label for a block using the function Id and the block ID.
    ///
    /// We implicitly assume that the function ID and the block ID is enough
    /// for us to create a unique label across functions and blocks.
    ///
    /// This is so that during linking there are no duplicates or labels being overwritten.
    fn create_block_label(function_id: FunctionId, block_id: BasicBlockId) -> String {
        format!("{function_id}-{block_id}")
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
        match terminator_instruction {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
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
                    self.pass_variable(source, destination);
                }
                self.brillig_context.jump_instruction(
                    self.create_block_label_for_current_function(*destination_block),
                );
            }
            TerminatorInstruction::Return { return_values, .. } => {
                let return_registers: Vec<_> = return_values
                    .iter()
                    .flat_map(|value_id| {
                        let return_variable = self.convert_ssa_value(*value_id, dfg);
                        return_variable.extract_registers()
                    })
                    .collect();
                self.brillig_context.codegen_return(&return_registers);
            }
        }
    }

    /// Passes an arbitrary variable from the registers of the source to the registers of the destination
    fn pass_variable(&mut self, source: BrilligVariable, destination: BrilligVariable) {
        match (source, destination) {
            (
                BrilligVariable::SingleAddr(source_var),
                BrilligVariable::SingleAddr(destination_var),
            ) => {
                self.brillig_context.mov_instruction(destination_var.address, source_var.address);
            }
            (
                BrilligVariable::BrilligArray(BrilligArray {
                    pointer: source_pointer,
                    size: _,
                    rc: source_rc,
                }),
                BrilligVariable::BrilligArray(BrilligArray {
                    pointer: destination_pointer,
                    size: _,
                    rc: destination_rc,
                }),
            ) => {
                self.brillig_context.mov_instruction(destination_pointer, source_pointer);
                self.brillig_context.mov_instruction(destination_rc, source_rc);
            }
            (
                BrilligVariable::BrilligVector(BrilligVector {
                    pointer: source_pointer,
                    size: source_size,
                    rc: source_rc,
                }),
                BrilligVariable::BrilligVector(BrilligVector {
                    pointer: destination_pointer,
                    size: destination_size,
                    rc: destination_rc,
                }),
            ) => {
                self.brillig_context.mov_instruction(destination_pointer, source_pointer);
                self.brillig_context.mov_instruction(destination_size, source_size);
                self.brillig_context.mov_instruction(destination_rc, source_rc);
            }
            (_, _) => {
                unreachable!("ICE: Cannot pass value from {:?} to {:?}", source, destination);
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
                let condition = SingleAddrVariable {
                    address: self.brillig_context.allocate_register(),
                    bit_size: 1,
                };

                self.convert_ssa_binary(
                    &Binary { lhs: *lhs, rhs: *rhs, operator: BinaryOp::Eq },
                    dfg,
                    condition,
                );
                match assert_message {
                    Some(ConstrainError::UserDefined(selector, values)) => {
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
                            selector.as_u64(),
                        );
                    }
                    Some(ConstrainError::Intrinsic(message)) => {
                        self.brillig_context.codegen_constrain(condition, Some(message.clone()));
                    }
                    None => {
                        self.brillig_context.codegen_constrain(condition, None);
                    }
                }
                self.brillig_context.deallocate_single_addr(condition);
            }
            Instruction::Allocate => {
                let result_value = dfg.instruction_results(instruction_id)[0];
                let address_register = self.variables.define_single_addr_variable(
                    self.function_context,
                    self.brillig_context,
                    result_value,
                    dfg,
                );
                match dfg.type_of_value(result_value) {
                    Type::Reference(element) => match *element {
                        Type::Array(..) => {
                            self.brillig_context
                                .codegen_allocate_array_reference(address_register.address);
                        }
                        Type::Slice(..) => {
                            self.brillig_context
                                .codegen_allocate_vector_reference(address_register.address);
                        }
                        _ => {
                            self.brillig_context
                                .codegen_allocate_single_addr_reference(address_register.address);
                        }
                    },
                    _ => {
                        unreachable!("ICE: Allocate on non-reference type")
                    }
                }
            }
            Instruction::Store { address, value } => {
                let address_var = self.convert_ssa_single_addr_value(*address, dfg);
                let source_variable = self.convert_ssa_value(*value, dfg);

                self.brillig_context.codegen_store_variable(address_var.address, source_variable);
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
                    .codegen_load_variable(target_variable, address_variable.address);
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

                    let input_registers = vecmap(arguments, |value_id| {
                        self.convert_ssa_value(*value_id, dfg).to_value_or_array()
                    });
                    let input_value_types = vecmap(arguments, |value_id| {
                        let value_type = dfg.type_of_value(*value_id);
                        type_to_heap_value_type(&value_type)
                    });
                    let output_registers = vecmap(result_ids, |value_id| {
                        self.allocate_external_call_result(*value_id, dfg).to_value_or_array()
                    });
                    let output_value_types = vecmap(result_ids, |value_id| {
                        let value_type = dfg.type_of_value(*value_id);
                        type_to_heap_value_type(&value_type)
                    });
                    self.brillig_context.foreign_call_instruction(
                        func_name.to_owned(),
                        &input_registers,
                        &input_value_types,
                        &output_registers,
                        &output_value_types,
                    );

                    for (i, output_register) in output_registers.iter().enumerate() {
                        if let ValueOrArray::HeapVector(HeapVector { size, .. }) = output_register {
                            // Update the stack pointer so that we do not overwrite
                            // dynamic memory returned from other external calls
                            self.brillig_context.increase_free_memory_pointer_instruction(*size);

                            // Update the dynamic slice length maintained in SSA
                            if let ValueOrArray::MemoryAddress(len_index) = output_registers[i - 1]
                            {
                                let element_size = dfg[result_ids[i]].get_type().element_size();
                                self.brillig_context.mov_instruction(len_index, *size);
                                self.brillig_context.codegen_usize_op_in_place(
                                    len_index,
                                    BrilligBinaryOp::UnsignedDiv,
                                    element_size,
                                );
                            } else {
                                unreachable!("ICE: a vector must be preceded by a register containing its length");
                            }
                        }
                        // Single values and allocation of fixed sized arrays has already been handled
                        // inside of `allocate_external_call_result`
                    }
                }
                Value::Function(func_id) => {
                    let result_ids = dfg.instruction_results(instruction_id);
                    self.convert_ssa_function_call(*func_id, arguments, dfg, result_ids);
                }
                Value::Intrinsic(Intrinsic::BlackBox(bb_func)) => {
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
                                if !matches!(dfg.type_of_value(arguments[i + 1]), Type::Slice(_)) {
                                    arguments_no_slice_len.push(*arg);
                                }
                            } else {
                                arguments_no_slice_len.push(*arg);
                            }
                        } else {
                            arguments_no_slice_len.push(*arg);
                        }
                    }

                    let function_arguments =
                        vecmap(&arguments_no_slice_len, |arg| self.convert_ssa_value(*arg, dfg));
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
                Value::Intrinsic(Intrinsic::ArrayLen) => {
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
                        self.convert_ssa_array_len(arguments[0], result_variable.address, dfg);
                    }
                }
                Value::Intrinsic(Intrinsic::AsSlice) => {
                    let source_variable = self.convert_ssa_value(arguments[0], dfg);
                    let result_ids = dfg.instruction_results(instruction_id);
                    let destination_len_variable = self.variables.define_single_addr_variable(
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
                    let source_size_as_register =
                        self.convert_ssa_array_set(source_variable, destination_variable, None);

                    // we need to explicitly set the destination_len_variable
                    self.brillig_context
                        .mov_instruction(destination_len_variable.address, source_size_as_register);
                    self.brillig_context.deallocate_register(source_size_as_register);
                }
                Value::Intrinsic(
                    Intrinsic::SlicePushBack
                    | Intrinsic::SlicePopBack
                    | Intrinsic::SlicePushFront
                    | Intrinsic::SlicePopFront
                    | Intrinsic::SliceInsert
                    | Intrinsic::SliceRemove,
                ) => {
                    self.convert_ssa_slice_intrinsic_call(
                        dfg,
                        &dfg[dfg.resolve(*func)],
                        instruction_id,
                        arguments,
                    );
                }
                Value::Intrinsic(Intrinsic::ToRadix(endianness)) => {
                    let source = self.convert_ssa_single_addr_value(arguments[0], dfg);

                    let radix: u32 = dfg
                        .get_numeric_constant(arguments[1])
                        .expect("Radix should be known")
                        .try_to_u64()
                        .expect("Radix should fit in u64")
                        .try_into()
                        .expect("Radix should be u32");

                    let limb_count: usize = dfg
                        .get_numeric_constant(arguments[2])
                        .expect("Limb count should be known")
                        .try_to_u64()
                        .expect("Limb count should fit in u64")
                        .try_into()
                        .expect("Limb count should fit in usize");

                    let results = dfg.instruction_results(instruction_id);

                    let target_len = self.variables.define_single_addr_variable(
                        self.function_context,
                        self.brillig_context,
                        results[0],
                        dfg,
                    );

                    let target_vector = self
                        .variables
                        .define_variable(
                            self.function_context,
                            self.brillig_context,
                            results[1],
                            dfg,
                        )
                        .extract_vector();

                    // Update the user-facing slice length
                    self.brillig_context
                        .usize_const_instruction(target_len.address, limb_count.into());

                    self.brillig_context.codegen_to_radix(
                        source,
                        target_vector,
                        radix,
                        limb_count,
                        matches!(endianness, Endian::Big),
                        8,
                    );
                }
                Value::Intrinsic(Intrinsic::ToBits(endianness)) => {
                    let source = self.convert_ssa_single_addr_value(arguments[0], dfg);
                    let limb_count: usize = dfg
                        .get_numeric_constant(arguments[1])
                        .expect("Limb count should be known")
                        .try_to_u64()
                        .expect("Limb count should fit in u64")
                        .try_into()
                        .expect("Limb count should fit in usize");

                    let results = dfg.instruction_results(instruction_id);

                    let target_len_variable = self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        results[0],
                        dfg,
                    );
                    let target_len = target_len_variable.extract_single_addr();

                    let target_vector = match self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        results[1],
                        dfg,
                    ) {
                        BrilligVariable::BrilligArray(array) => {
                            self.brillig_context.array_to_vector_instruction(&array)
                        }
                        BrilligVariable::BrilligVector(vector) => vector,
                        BrilligVariable::SingleAddr(..) => unreachable!("ICE: ToBits on non-array"),
                    };

                    // Update the user-facing slice length
                    self.brillig_context
                        .usize_const_instruction(target_len.address, limb_count.into());

                    self.brillig_context.codegen_to_radix(
                        source,
                        target_vector,
                        2,
                        limb_count,
                        matches!(endianness, Endian::Big),
                        1,
                    );
                }
                _ => {
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
                let array_pointer = match array_variable {
                    BrilligVariable::BrilligArray(BrilligArray { pointer, .. }) => pointer,
                    BrilligVariable::BrilligVector(BrilligVector { pointer, .. }) => pointer,
                    _ => unreachable!("ICE: array get on non-array"),
                };

                let index_variable = self.convert_ssa_single_addr_value(*index, dfg);
                self.validate_array_index(array_variable, index_variable);
                self.retrieve_variable_from_array(
                    array_pointer,
                    index_variable,
                    destination_variable,
                );
            }
            Instruction::ArraySet { array, index, value, mutable: _ } => {
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
                self.validate_array_index(source_variable, index_register);
                let source_size_as_register = self.convert_ssa_array_set(
                    source_variable,
                    destination_variable,
                    Some((index_register.address, value_variable)),
                );
                self.brillig_context.deallocate_register(source_size_as_register);
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
                let rc_register = match self.convert_ssa_value(*value, dfg) {
                    BrilligVariable::BrilligArray(BrilligArray { rc, .. })
                    | BrilligVariable::BrilligVector(BrilligVector { rc, .. }) => rc,
                    other => unreachable!("ICE: increment rc on non-array: {other:?}"),
                };
                self.brillig_context.codegen_usize_op_in_place(
                    rc_register,
                    BrilligBinaryOp::Add,
                    1,
                );
            }
            Instruction::DecrementRc { value } => {
                let rc_register = match self.convert_ssa_value(*value, dfg) {
                    BrilligVariable::BrilligArray(BrilligArray { rc, .. })
                    | BrilligVariable::BrilligVector(BrilligVector { rc, .. }) => rc,
                    other => unreachable!("ICE: decrement rc on non-array: {other:?}"),
                };
                self.brillig_context.codegen_usize_op_in_place(
                    rc_register,
                    BrilligBinaryOp::Sub,
                    1,
                );
            }
            Instruction::EnableSideEffects { .. } => {
                todo!("enable_side_effects not supported by brillig")
            }
            Instruction::IfElse { .. } => {
                unreachable!("IfElse instructions should not be possible in brillig")
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
        // Convert the arguments to registers casting those to the types of the receiving function
        let argument_registers: Vec<MemoryAddress> = arguments
            .iter()
            .flat_map(|argument_id| self.convert_ssa_value(*argument_id, dfg).extract_registers())
            .collect();

        // Create label for the function that will be called
        let label_of_function_to_call = FunctionContext::function_id_to_function_label(func_id);

        let variables_to_save = self.variables.get_available_variables(self.function_context);

        let saved_registers = self
            .brillig_context
            .codegen_pre_call_save_registers_prep_args(&argument_registers, &variables_to_save);

        // We don't save and restore constants, so we dump them before a external call since the callee might use the registers where they are allocated.
        self.variables.dump_constants();

        // Call instruction, which will interpret above registers 0..num args
        self.brillig_context.add_external_call_instruction(label_of_function_to_call);

        // Important: resolve after pre_call_save_registers_prep_args
        // This ensures we don't save the results to registers unnecessarily.

        // Allocate the registers for the variables where we are assigning the returns
        let variables_assigned_to = vecmap(result_ids, |result_id| {
            self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                *result_id,
                dfg,
            )
        });

        // Collect the registers that should have been returned
        let returned_registers: Vec<MemoryAddress> = variables_assigned_to
            .iter()
            .flat_map(|returned_variable| returned_variable.extract_registers())
            .collect();

        assert!(
            !saved_registers.iter().any(|x| returned_registers.contains(x)),
            "should not save registers used as function results"
        );

        // puts the returns into the returned_registers and restores saved_registers
        self.brillig_context
            .codegen_post_call_prep_returns_load_registers(&returned_registers, &saved_registers);
    }

    fn validate_array_index(
        &mut self,
        array_variable: BrilligVariable,
        index_register: SingleAddrVariable,
    ) {
        let (size_as_register, should_deallocate_size) = match array_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, .. }) => {
                (self.brillig_context.make_usize_constant_instruction(size.into()), true)
            }
            BrilligVariable::BrilligVector(BrilligVector { size, .. }) => {
                (SingleAddrVariable::new_usize(size), false)
            }
            _ => unreachable!("ICE: validate array index on non-array"),
        };

        let condition = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);

        self.brillig_context.memory_op_instruction(
            index_register.address,
            size_as_register.address,
            condition.address,
            BrilligBinaryOp::LessThan,
        );

        self.brillig_context
            .codegen_constrain(condition, Some("Array index out of bounds".to_owned()));

        if should_deallocate_size {
            self.brillig_context.deallocate_single_addr(size_as_register);
        }
        self.brillig_context.deallocate_single_addr(condition);
    }

    pub(crate) fn retrieve_variable_from_array(
        &mut self,
        array_pointer: MemoryAddress,
        index_var: SingleAddrVariable,
        destination_variable: BrilligVariable,
    ) {
        match destination_variable {
            BrilligVariable::SingleAddr(destination_register) => {
                self.brillig_context.codegen_array_get(
                    array_pointer,
                    index_var,
                    destination_register.address,
                );
            }
            BrilligVariable::BrilligArray(..) | BrilligVariable::BrilligVector(..) => {
                let reference = self.brillig_context.allocate_register();
                self.brillig_context.codegen_array_get(array_pointer, index_var, reference);
                self.brillig_context.codegen_load_variable(destination_variable, reference);
                self.brillig_context.deallocate_register(reference);
            }
        }
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
        opt_index_and_value: Option<(MemoryAddress, BrilligVariable)>,
    ) -> MemoryAddress {
        let destination_pointer = match destination_variable {
            BrilligVariable::BrilligArray(BrilligArray { pointer, .. }) => pointer,
            BrilligVariable::BrilligVector(BrilligVector { pointer, .. }) => pointer,
            _ => unreachable!("ICE: array_set SSA returns non-array"),
        };

        let reference_count = match source_variable {
            BrilligVariable::BrilligArray(BrilligArray { rc, .. })
            | BrilligVariable::BrilligVector(BrilligVector { rc, .. }) => rc,
            _ => unreachable!("ICE: array_set SSA on non-array"),
        };

        let (source_pointer, source_size_as_register) = match source_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, pointer, rc: _ }) => {
                let source_size_register = self.brillig_context.allocate_register();
                self.brillig_context.usize_const_instruction(source_size_register, size.into());
                (pointer, source_size_register)
            }
            BrilligVariable::BrilligVector(BrilligVector { size, pointer, rc: _ }) => {
                let source_size_register = self.brillig_context.allocate_register();
                self.brillig_context.mov_instruction(source_size_register, size);
                (pointer, source_size_register)
            }
            _ => unreachable!("ICE: array_set SSA on non-array"),
        };

        // Here we want to compare the reference count against 1.
        let one = self.brillig_context.make_usize_constant_instruction(1_usize.into());
        let condition = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            reference_count,
            one.address,
            condition,
            BrilligBinaryOp::Equals,
        );
        self.brillig_context.codegen_branch(condition, |ctx, cond| {
            if cond {
                // Reference count is 1, we can mutate the array directly
                ctx.mov_instruction(destination_pointer, source_pointer);
            } else {
                // First issue a array copy to the destination
                ctx.codegen_allocate_array(destination_pointer, source_size_as_register);

                ctx.codegen_copy_array(
                    source_pointer,
                    destination_pointer,
                    SingleAddrVariable::new(
                        source_size_as_register,
                        BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
                    ),
                );
            }
        });

        match destination_variable {
            BrilligVariable::BrilligArray(BrilligArray { rc: target_rc, .. }) => {
                self.brillig_context.usize_const_instruction(target_rc, 1_usize.into());
            }
            BrilligVariable::BrilligVector(BrilligVector {
                size: target_size,
                rc: target_rc,
                ..
            }) => {
                self.brillig_context.mov_instruction(target_size, source_size_as_register);
                self.brillig_context.usize_const_instruction(target_rc, 1_usize.into());
            }
            _ => unreachable!("ICE: array_set SSA on non-array"),
        }

        if let Some((index_register, value_variable)) = opt_index_and_value {
            // Then set the value in the newly created array
            self.store_variable_in_array(
                destination_pointer,
                SingleAddrVariable::new_usize(index_register),
                value_variable,
            );
        }

        self.brillig_context.deallocate_register(condition);
        source_size_as_register
    }

    pub(crate) fn store_variable_in_array_with_ctx(
        ctx: &mut BrilligContext,
        destination_pointer: MemoryAddress,
        index_register: SingleAddrVariable,
        value_variable: BrilligVariable,
    ) {
        match value_variable {
            BrilligVariable::SingleAddr(value_variable) => {
                ctx.codegen_array_set(destination_pointer, index_register, value_variable.address);
            }
            BrilligVariable::BrilligArray(_) => {
                let reference: MemoryAddress = ctx.allocate_register();
                ctx.codegen_allocate_array_reference(reference);
                ctx.codegen_store_variable(reference, value_variable);
                ctx.codegen_array_set(destination_pointer, index_register, reference);
                ctx.deallocate_register(reference);
            }
            BrilligVariable::BrilligVector(_) => {
                let reference = ctx.allocate_register();
                ctx.codegen_allocate_vector_reference(reference);
                ctx.codegen_store_variable(reference, value_variable);
                ctx.codegen_array_set(destination_pointer, index_register, reference);
                ctx.deallocate_register(reference);
            }
        }
    }

    pub(crate) fn store_variable_in_array(
        &mut self,
        destination_pointer: MemoryAddress,
        index_variable: SingleAddrVariable,
        value_variable: BrilligVariable,
    ) {
        Self::store_variable_in_array_with_ctx(
            self.brillig_context,
            destination_pointer,
            index_variable,
            value_variable,
        );
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
        let source_variable = self.convert_ssa_value(slice_id, dfg);
        let source_vector = self.convert_array_or_vector_to_vector(source_variable);

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
            dfg[binary.lhs].get_type(),
            dfg[binary.rhs].get_type(),
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
                    self.convert_signed_division(left, right, result_variable);
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
            BinaryOp::Shr => BrilligBinaryOp::Shr,
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

    /// Splits a two's complement signed integer in the sign bit and the absolute value.
    /// For example, -6 i8 (11111010) is split to 00000110 (6, absolute value) and 1 (is_negative).
    fn absolute_value(
        &mut self,
        num: SingleAddrVariable,
        absolute_value: SingleAddrVariable,
        result_is_negative: SingleAddrVariable,
    ) {
        let max_positive = self
            .brillig_context
            .make_constant_instruction(((1_u128 << (num.bit_size - 1)) - 1).into(), num.bit_size);

        // Compute if num is negative
        self.brillig_context.binary_instruction(
            max_positive,
            num,
            result_is_negative,
            BrilligBinaryOp::LessThan,
        );

        // Two's complement of num
        let zero = self.brillig_context.make_constant_instruction(0_usize.into(), num.bit_size);
        let twos_complement =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), num.bit_size);
        self.brillig_context.binary_instruction(zero, num, twos_complement, BrilligBinaryOp::Sub);

        // absolute_value = result_is_negative ? twos_complement : num
        self.brillig_context.conditional_mov_instruction(
            absolute_value.address,
            result_is_negative.address,
            twos_complement.address,
            num.address,
        );

        self.brillig_context.deallocate_single_addr(zero);
        self.brillig_context.deallocate_single_addr(max_positive);
        self.brillig_context.deallocate_single_addr(twos_complement);
    }

    fn convert_signed_division(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let left_is_negative = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
        let left_abs_value =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);

        let right_is_negative =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
        let right_abs_value =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), right.bit_size);

        let result_is_negative =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);

        // Compute both absolute values
        self.absolute_value(left, left_abs_value, left_is_negative);
        self.absolute_value(right, right_abs_value, right_is_negative);

        // Perform the division on the absolute values
        self.brillig_context.binary_instruction(
            left_abs_value,
            right_abs_value,
            result,
            BrilligBinaryOp::UnsignedDiv,
        );

        // Compute result sign
        self.brillig_context.binary_instruction(
            left_is_negative,
            right_is_negative,
            result_is_negative,
            BrilligBinaryOp::Xor,
        );

        // If result has to be negative, perform two's complement
        self.brillig_context.codegen_if(result_is_negative.address, |ctx| {
            let zero = ctx.make_constant_instruction(0_usize.into(), result.bit_size);
            ctx.binary_instruction(zero, result, result, BrilligBinaryOp::Sub);
            ctx.deallocate_single_addr(zero);
        });

        self.brillig_context.deallocate_single_addr(left_is_negative);
        self.brillig_context.deallocate_single_addr(left_abs_value);
        self.brillig_context.deallocate_single_addr(right_is_negative);
        self.brillig_context.deallocate_single_addr(right_abs_value);
        self.brillig_context.deallocate_single_addr(result_is_negative);
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
        self.convert_signed_division(left, right, scratch_var_i);

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
                if let Some(variable) = self.variables.get_constant(value_id, dfg) {
                    variable
                } else {
                    let new_variable =
                        self.variables.allocate_constant(self.brillig_context, value_id, dfg);

                    self.brillig_context
                        .const_instruction(new_variable.extract_single_addr(), *constant);
                    new_variable
                }
            }
            Value::Array { array, .. } => {
                if let Some(variable) = self.variables.get_constant(value_id, dfg) {
                    variable
                } else {
                    let new_variable =
                        self.variables.allocate_constant(self.brillig_context, value_id, dfg);

                    // Initialize the variable
                    let pointer = match new_variable {
                        BrilligVariable::BrilligArray(brillig_array) => {
                            self.brillig_context.codegen_allocate_fixed_length_array(
                                brillig_array.pointer,
                                array.len(),
                            );
                            self.brillig_context
                                .usize_const_instruction(brillig_array.rc, 1_usize.into());

                            brillig_array.pointer
                        }
                        BrilligVariable::BrilligVector(vector) => {
                            self.brillig_context
                                .usize_const_instruction(vector.size, array.len().into());
                            self.brillig_context
                                .codegen_allocate_array(vector.pointer, vector.size);
                            self.brillig_context.usize_const_instruction(vector.rc, 1_usize.into());

                            vector.pointer
                        }
                        _ => unreachable!(
                            "ICE: Cannot initialize array value created as {new_variable:?}"
                        ),
                    };

                    // Write the items

                    // Allocate a register for the iterator
                    let iterator_register =
                        self.brillig_context.make_usize_constant_instruction(0_usize.into());

                    for element_id in array.iter() {
                        let element_variable = self.convert_ssa_value(*element_id, dfg);
                        // Store the item in memory
                        self.store_variable_in_array(pointer, iterator_register, element_variable);
                        // Increment the iterator
                        self.brillig_context.codegen_usize_op_in_place(
                            iterator_register.address,
                            BrilligBinaryOp::Add,
                            1,
                        );
                    }

                    self.brillig_context.deallocate_single_addr(iterator_register);

                    new_variable
                }
            }
            Value::Function(_) => {
                // For the debugger instrumentation we want to allow passing
                // around values representing function pointers, even though
                // there is no interaction with the function possible given that
                // value.
                let new_variable =
                    self.variables.allocate_constant(self.brillig_context, value_id, dfg);

                self.brillig_context.const_instruction(
                    new_variable.extract_single_addr(),
                    value_id.to_usize().into(),
                );
                new_variable
            }
            Value::Intrinsic(_) | Value::ForeignFunction(_) => {
                todo!("ICE: Cannot convert value {value:?}")
            }
        }
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
        match typ {
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
                self.brillig_context.codegen_allocate_fixed_length_array(array.pointer, array.size);
                self.brillig_context.usize_const_instruction(array.rc, 1_usize.into());

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
                self.brillig_context.usize_const_instruction(vector.rc, 1_usize.into());

                variable
            }
            _ => {
                unreachable!("ICE: unsupported return type for black box call {typ:?}")
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
            BrilligVariable::BrilligVector(BrilligVector { size, .. }) => {
                self.brillig_context.codegen_usize_op(
                    size,
                    result_register,
                    BrilligBinaryOp::UnsignedDiv,
                    element_size,
                );
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
