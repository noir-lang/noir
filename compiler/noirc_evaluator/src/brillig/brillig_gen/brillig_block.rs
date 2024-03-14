use crate::brillig::brillig_ir::brillig_variable::{
    type_to_heap_value_type, BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable,
};
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};
use crate::ssa::ir::dfg::CallStack;
use crate::ssa::ir::instruction::ConstrainError;
use crate::ssa::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    dfg::DataFlowGraph,
    function::FunctionId,
    instruction::{
        Binary, BinaryOp, Endian, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
    },
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::acir::brillig::{BinaryFieldOp, BinaryIntOp, MemoryAddress, ValueOrArray};
use acvm::brillig_vm::brillig::HeapVector;
use acvm::FieldElement;
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
        let variables =
            BlockVariables::new(live_in.clone(), function_context.all_block_parameters());

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

        // Convert the block parameters
        let block = &dfg[self.block_id];
        self.convert_block_params(block, dfg);

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
                    let destination = self.variables.get_block_param(
                        self.function_context,
                        *destination_block,
                        *dest,
                        dfg,
                    );
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
                self.brillig_context.return_instruction(&return_registers);
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

    /// Converts SSA Block parameters into Brillig Registers.
    fn convert_block_params(&mut self, block: &BasicBlock, dfg: &DataFlowGraph) {
        for param_id in block.parameters() {
            let value = &dfg[*param_id];
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
                    self.variables.get_block_param(
                        self.function_context,
                        self.block_id,
                        *param_id,
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
                let assert_message = if let Some(error) = assert_message {
                    match error.as_ref() {
                        ConstrainError::Static(string) => Some(string.clone()),
                        ConstrainError::Dynamic(call_instruction) => {
                            let Instruction::Call { func, arguments } = call_instruction else {
                                unreachable!("expected a call instruction")
                            };

                            let Value::Function(func_id) = &dfg[*func] else {
                                unreachable!("expected a function value")
                            };

                            self.convert_ssa_function_call(*func_id, arguments, dfg, &[]);

                            // Dynamic assert messages are handled in the generated function call.
                            // We then don't need to attach one to the constrain instruction.
                            None
                        }
                    }
                } else {
                    None
                };

                let condition = SingleAddrVariable {
                    address: self.brillig_context.allocate_register(),
                    bit_size: 1,
                };

                self.convert_ssa_binary(
                    &Binary { lhs: *lhs, rhs: *rhs, operator: BinaryOp::Eq },
                    dfg,
                    condition,
                );

                self.brillig_context.constrain_instruction(condition, assert_message);
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
                                .allocate_array_reference_instruction(address_register.address);
                        }
                        Type::Slice(..) => {
                            self.brillig_context
                                .allocate_vector_reference_instruction(address_register.address);
                        }
                        _ => {
                            self.brillig_context.allocate_single_addr_reference_instruction(
                                address_register.address,
                            );
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

                self.brillig_context
                    .store_variable_instruction(address_var.address, source_variable);
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
                    .load_variable_instruction(target_variable, address_variable.address);
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
                            self.brillig_context.update_stack_pointer(*size);

                            // Update the dynamic slice length maintained in SSA
                            if let ValueOrArray::MemoryAddress(len_index) = output_registers[i - 1]
                            {
                                let element_size = dfg[result_ids[i]].get_type().element_size();
                                self.brillig_context.mov_instruction(len_index, *size);
                                self.brillig_context.usize_op_in_place(
                                    len_index,
                                    BinaryIntOp::UnsignedDiv,
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
                    let radix = self.convert_ssa_single_addr_value(arguments[1], dfg);
                    let limb_count = self.convert_ssa_single_addr_value(arguments[2], dfg);

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
                    self.brillig_context.mov_instruction(target_len.address, limb_count.address);

                    self.brillig_context.radix_instruction(
                        source,
                        target_vector,
                        radix,
                        limb_count,
                        matches!(endianness, Endian::Big),
                    );
                }
                Value::Intrinsic(Intrinsic::ToBits(endianness)) => {
                    let source = self.convert_ssa_single_addr_value(arguments[0], dfg);
                    let limb_count = self.convert_ssa_single_addr_value(arguments[1], dfg);

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
                            self.brillig_context.array_to_vector(&array)
                        }
                        BrilligVariable::BrilligVector(vector) => vector,
                        BrilligVariable::SingleAddr(..) => unreachable!("ICE: ToBits on non-array"),
                    };

                    let radix = self.brillig_context.make_constant(2_usize.into(), 32);

                    // Update the user-facing slice length
                    self.brillig_context.mov_instruction(target_len.address, limb_count.address);

                    self.brillig_context.radix_instruction(
                        source,
                        target_vector,
                        radix,
                        limb_count,
                        matches!(endianness, Endian::Big),
                    );

                    self.brillig_context.deallocate_single_addr(radix);
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
                self.brillig_context.truncate_instruction(
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
            Instruction::ArraySet { array, index, value, .. } => {
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

                self.convert_ssa_array_set(
                    source_variable,
                    destination_variable,
                    index_register.address,
                    value_variable,
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
                    let right = self.brillig_context.make_constant(
                        FieldElement::from_be_bytes_reduce(&max.to_bytes_be()).into(),
                        FieldElement::max_num_bits(),
                    );

                    // Check if lte max
                    let brillig_binary_op = BrilligBinaryOp::Integer(BinaryIntOp::LessThanEquals);
                    let condition =
                        SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                    self.brillig_context.binary_instruction(
                        left,
                        right,
                        condition,
                        brillig_binary_op,
                    );

                    self.brillig_context.constrain_instruction(condition, assert_message.clone());
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
                self.brillig_context.usize_op_in_place(rc_register, BinaryIntOp::Add, 1);
            }
            Instruction::DecrementRc { value } => {
                let rc_register = match self.convert_ssa_value(*value, dfg) {
                    BrilligVariable::BrilligArray(BrilligArray { rc, .. })
                    | BrilligVariable::BrilligVector(BrilligVector { rc, .. }) => rc,
                    other => unreachable!("ICE: decrement rc on non-array: {other:?}"),
                };
                self.brillig_context.usize_op_in_place(rc_register, BinaryIntOp::Sub, 1);
            }
            Instruction::EnableSideEffects { .. } => {
                todo!("enable_side_effects not supported by brillig")
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
            .pre_call_save_registers_prep_args(&argument_registers, &variables_to_save);

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
            .post_call_prep_returns_load_registers(&returned_registers, &saved_registers);
    }

    fn validate_array_index(
        &mut self,
        array_variable: BrilligVariable,
        index_register: SingleAddrVariable,
    ) {
        let (size_as_register, should_deallocate_size) = match array_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, .. }) => {
                (self.brillig_context.make_usize_constant(size.into()), true)
            }
            BrilligVariable::BrilligVector(BrilligVector { size, .. }) => {
                (SingleAddrVariable::new_usize(size), false)
            }
            _ => unreachable!("ICE: validate array index on non-array"),
        };

        let condition = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);

        self.brillig_context.memory_op(
            index_register.address,
            size_as_register.address,
            condition.address,
            BinaryIntOp::LessThan,
        );

        self.brillig_context
            .constrain_instruction(condition, Some("Array index out of bounds".to_owned()));

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
                self.brillig_context.array_get(
                    array_pointer,
                    index_var,
                    destination_register.address,
                );
            }
            BrilligVariable::BrilligArray(..) | BrilligVariable::BrilligVector(..) => {
                let reference = self.brillig_context.allocate_register();
                self.brillig_context.array_get(array_pointer, index_var, reference);
                self.brillig_context.load_variable_instruction(destination_variable, reference);
                self.brillig_context.deallocate_register(reference);
            }
        }
    }

    /// Array set operation in SSA returns a new array or slice that is a copy of the parameter array or slice
    /// With a specific value changed.
    fn convert_ssa_array_set(
        &mut self,
        source_variable: BrilligVariable,
        destination_variable: BrilligVariable,
        index_register: MemoryAddress,
        value_variable: BrilligVariable,
    ) {
        let destination_pointer = match destination_variable {
            BrilligVariable::BrilligArray(BrilligArray { pointer, .. }) => pointer,
            BrilligVariable::BrilligVector(BrilligVector { pointer, .. }) => pointer,
            _ => unreachable!("ICE: array set returns non-array"),
        };

        let reference_count = match source_variable {
            BrilligVariable::BrilligArray(BrilligArray { rc, .. })
            | BrilligVariable::BrilligVector(BrilligVector { rc, .. }) => rc,
            _ => unreachable!("ICE: array set on non-array"),
        };

        let (source_pointer, source_size_as_register) = match source_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, pointer, rc: _ }) => {
                let source_size_register = self.brillig_context.allocate_register();
                self.brillig_context.usize_const(source_size_register, size.into());
                (pointer, source_size_register)
            }
            BrilligVariable::BrilligVector(BrilligVector { size, pointer, rc: _ }) => {
                let source_size_register = self.brillig_context.allocate_register();
                self.brillig_context.mov_instruction(source_size_register, size);
                (pointer, source_size_register)
            }
            _ => unreachable!("ICE: array set on non-array"),
        };

        // Here we want to compare the reference count against 1.
        let one = self.brillig_context.make_usize_constant(1_usize.into());
        let condition = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(
            reference_count,
            one.address,
            condition,
            BinaryIntOp::Equals,
        );
        self.brillig_context.branch_instruction(condition, |ctx, cond| {
            if cond {
                // Reference count is 1, we can mutate the array directly
                ctx.mov_instruction(destination_pointer, source_pointer);
            } else {
                // First issue a array copy to the destination
                ctx.allocate_array_instruction(destination_pointer, source_size_as_register);

                ctx.copy_array_instruction(
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
                self.brillig_context.usize_const(target_rc, 1_usize.into());
            }
            BrilligVariable::BrilligVector(BrilligVector {
                size: target_size,
                rc: target_rc,
                ..
            }) => {
                self.brillig_context.mov_instruction(target_size, source_size_as_register);
                self.brillig_context.usize_const(target_rc, 1_usize.into());
            }
            _ => unreachable!("ICE: array set on non-array"),
        }

        // Then set the value in the newly created array
        self.store_variable_in_array(
            destination_pointer,
            SingleAddrVariable::new_usize(index_register),
            value_variable,
        );

        self.brillig_context.deallocate_register(source_size_as_register);
        self.brillig_context.deallocate_register(condition);
    }

    pub(crate) fn store_variable_in_array_with_ctx(
        ctx: &mut BrilligContext,
        destination_pointer: MemoryAddress,
        index_register: SingleAddrVariable,
        value_variable: BrilligVariable,
    ) {
        match value_variable {
            BrilligVariable::SingleAddr(value_variable) => {
                ctx.array_set(destination_pointer, index_register, value_variable.address);
            }
            BrilligVariable::BrilligArray(_) => {
                let reference: MemoryAddress = ctx.allocate_register();
                ctx.allocate_array_reference_instruction(reference);
                ctx.store_variable_instruction(reference, value_variable);
                ctx.array_set(destination_pointer, index_register, reference);
                ctx.deallocate_register(reference);
            }
            BrilligVariable::BrilligVector(_) => {
                let reference = ctx.allocate_register();
                ctx.allocate_vector_reference_instruction(reference);
                ctx.store_variable_instruction(reference, value_variable);
                ctx.array_set(destination_pointer, index_register, reference);
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

                self.update_slice_length(target_len.address, arguments[0], dfg, BinaryIntOp::Add);

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

                self.update_slice_length(target_len.address, arguments[0], dfg, BinaryIntOp::Add);

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

                self.update_slice_length(target_len.address, arguments[0], dfg, BinaryIntOp::Sub);

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

                self.update_slice_length(target_len.address, arguments[0], dfg, BinaryIntOp::Sub);

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

                let converted_index = self.brillig_context.make_usize_constant(element_size.into());

                self.brillig_context.memory_op(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BinaryIntOp::Mul,
                );

                let items = vecmap(&arguments[3..element_size + 3], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(target_len.address, arguments[0], dfg, BinaryIntOp::Add);

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

                let converted_index = self.brillig_context.make_usize_constant(element_size.into());
                self.brillig_context.memory_op(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BinaryIntOp::Mul,
                );

                let removed_items = vecmap(&results[2..element_size + 2], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_slice_length(target_len.address, arguments[0], dfg, BinaryIntOp::Sub);

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
        binary_op: BinaryIntOp,
    ) {
        let source_len_variable = self.convert_ssa_value(source_value, dfg);
        let source_len = source_len_variable.extract_single_addr();

        self.brillig_context.usize_op(source_len.address, target_len, binary_op, 1);
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
        let binary_type =
            type_of_binary_operation(dfg[binary.lhs].get_type(), dfg[binary.rhs].get_type());

        let left = self.convert_ssa_single_addr_value(binary.lhs, dfg);
        let right = self.convert_ssa_single_addr_value(binary.rhs, dfg);

        let (brillig_binary_op, is_signed) =
            convert_ssa_binary_op_to_brillig_binary_op(binary.operator, &binary_type);

        self.brillig_context.binary_instruction(left, right, result_variable, brillig_binary_op);

        self.add_overflow_check(brillig_binary_op, left, right, result_variable, is_signed);
    }

    fn add_overflow_check(
        &mut self,
        binary_operation: BrilligBinaryOp,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
        is_signed: bool,
    ) {
        let (op, bit_size) = if let BrilligBinaryOp::Integer(op) = binary_operation {
            // Bit size is checked at compile time to be equal for left and right
            (op, left.bit_size)
        } else {
            return;
        };

        match (op, is_signed) {
            (BinaryIntOp::Add, false) => {
                let condition =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                // Check that lhs <= result
                self.brillig_context.binary_instruction(
                    left,
                    result,
                    condition,
                    BrilligBinaryOp::Integer(BinaryIntOp::LessThanEquals),
                );
                self.brillig_context.constrain_instruction(
                    condition,
                    Some("attempt to add with overflow".to_string()),
                );
                self.brillig_context.deallocate_single_addr(condition);
            }
            (BinaryIntOp::Sub, false) => {
                let condition =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                // Check that rhs <= lhs
                self.brillig_context.binary_instruction(
                    right,
                    left,
                    condition,
                    BrilligBinaryOp::Integer(BinaryIntOp::LessThanEquals),
                );
                self.brillig_context.constrain_instruction(
                    condition,
                    Some("attempt to subtract with overflow".to_string()),
                );
                self.brillig_context.deallocate_single_addr(condition);
            }
            (BinaryIntOp::Mul, false) => {
                // Multiplication overflow is only possible for bit sizes > 1
                if bit_size > 1 {
                    let is_right_zero =
                        SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                    let zero = self.brillig_context.make_constant(0_usize.into(), bit_size);
                    self.brillig_context.binary_instruction(
                        zero,
                        right,
                        is_right_zero,
                        BrilligBinaryOp::Integer(BinaryIntOp::Equals),
                    );
                    self.brillig_context.if_not_instruction(is_right_zero.address, |ctx| {
                        let condition = SingleAddrVariable::new(ctx.allocate_register(), 1);
                        let division = SingleAddrVariable::new(ctx.allocate_register(), bit_size);
                        // Check that result / rhs == lhs
                        ctx.binary_instruction(
                            result,
                            right,
                            division,
                            BrilligBinaryOp::Integer(BinaryIntOp::UnsignedDiv),
                        );
                        ctx.binary_instruction(
                            division,
                            left,
                            condition,
                            BrilligBinaryOp::Integer(BinaryIntOp::Equals),
                        );
                        ctx.constrain_instruction(
                            condition,
                            Some("attempt to multiply with overflow".to_string()),
                        );
                        ctx.deallocate_single_addr(condition);
                        ctx.deallocate_single_addr(division);
                    });
                    self.brillig_context.deallocate_single_addr(is_right_zero);
                    self.brillig_context.deallocate_single_addr(zero);
                }
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
                        .const_instruction(new_variable.extract_single_addr(), (*constant).into());
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
                            self.brillig_context
                                .allocate_fixed_length_array(brillig_array.pointer, array.len());
                            self.brillig_context.usize_const(brillig_array.rc, 1_usize.into());

                            brillig_array.pointer
                        }
                        BrilligVariable::BrilligVector(vector) => {
                            self.brillig_context.usize_const(vector.size, array.len().into());
                            self.brillig_context
                                .allocate_array_instruction(vector.pointer, vector.size);
                            self.brillig_context.usize_const(vector.rc, 1_usize.into());

                            vector.pointer
                        }
                        _ => unreachable!(
                            "ICE: Cannot initialize array value created as {new_variable:?}"
                        ),
                    };

                    // Write the items

                    // Allocate a register for the iterator
                    let iterator_register =
                        self.brillig_context.make_usize_constant(0_usize.into());

                    for element_id in array.iter() {
                        let element_variable = self.convert_ssa_value(*element_id, dfg);
                        // Store the item in memory
                        self.store_variable_in_array(pointer, iterator_register, element_variable);
                        // Increment the iterator
                        self.brillig_context.usize_op_in_place(
                            iterator_register.address,
                            BinaryIntOp::Add,
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
                self.brillig_context.allocate_fixed_length_array(array.pointer, array.size);
                self.brillig_context.usize_const(array.rc, 1_usize.into());

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
                self.brillig_context.set_array_pointer(vector.pointer);
                self.brillig_context.usize_const(vector.rc, 1_usize.into());

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
                self.brillig_context.usize_const(result_register, (size / element_size).into());
            }
            BrilligVariable::BrilligVector(BrilligVector { size, .. }) => {
                self.brillig_context.usize_op(
                    size,
                    result_register,
                    BinaryIntOp::UnsignedDiv,
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
pub(crate) fn type_of_binary_operation(lhs_type: &Type, rhs_type: &Type) -> Type {
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
        (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
            assert_eq!(
                lhs_type, rhs_type,
                "lhs and rhs types in a binary operation are always the same"
            );
            Type::Numeric(*lhs_type)
        }
    }
}

/// Convert an SSA binary operation into:
/// - Brillig Binary Integer Op, if it is a integer type
/// - Brillig Binary Field Op, if it is a field type
pub(crate) fn convert_ssa_binary_op_to_brillig_binary_op(
    ssa_op: BinaryOp,
    typ: &Type,
) -> (BrilligBinaryOp, bool) {
    // First get the bit size and whether its a signed integer, if it is a numeric type
    // if it is not,then we return None, indicating that
    // it is a Field.
    let bit_size_signedness = match typ {
          Type::Numeric(numeric_type) => match numeric_type {
              NumericType::Signed { bit_size } => Some((bit_size, true)),
              NumericType::Unsigned { bit_size } => Some((bit_size, false)),
              NumericType::NativeField => None,
          },
          _ => unreachable!("only numeric types are allowed in binary operations. References are handled separately"),
      };

    fn binary_op_to_field_op(op: BinaryOp) -> BrilligBinaryOp {
        match op {
            BinaryOp::Add => BrilligBinaryOp::Field(BinaryFieldOp::Add),
            BinaryOp::Sub => BrilligBinaryOp::Field(BinaryFieldOp::Sub),
            BinaryOp::Mul => BrilligBinaryOp::Field(BinaryFieldOp::Mul),
            BinaryOp::Div => BrilligBinaryOp::Field(BinaryFieldOp::Div),
            BinaryOp::Eq => BrilligBinaryOp::Field(BinaryFieldOp::Equals),
            _ => unreachable!(
                "Field type cannot be used with {op}. This should have been caught by the frontend"
            ),
        }
    }

    fn binary_op_to_int_op(op: BinaryOp, is_signed: bool) -> BrilligBinaryOp {
        let operation = match op {
            BinaryOp::Add => BinaryIntOp::Add,
            BinaryOp::Sub => BinaryIntOp::Sub,
            BinaryOp::Mul => BinaryIntOp::Mul,
            BinaryOp::Div => {
                if is_signed {
                    BinaryIntOp::SignedDiv
                } else {
                    BinaryIntOp::UnsignedDiv
                }
            }
            BinaryOp::Mod => return BrilligBinaryOp::Modulo { is_signed_integer: is_signed },
            BinaryOp::Eq => BinaryIntOp::Equals,
            BinaryOp::Lt => BinaryIntOp::LessThan,
            BinaryOp::And => BinaryIntOp::And,
            BinaryOp::Or => BinaryIntOp::Or,
            BinaryOp::Xor => BinaryIntOp::Xor,
            BinaryOp::Shl => BinaryIntOp::Shl,
            BinaryOp::Shr => BinaryIntOp::Shr,
        };

        BrilligBinaryOp::Integer(operation)
    }

    // If bit size is available then it is a binary integer operation
    match bit_size_signedness {
        Some((_, is_signed)) => (binary_op_to_int_op(ssa_op, is_signed), is_signed),
        None => (binary_op_to_field_op(ssa_op), false),
    }
}
