use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE,
};
use crate::ssa::ir::dfg::CallStack;
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
use acvm::acir::brillig::{BinaryFieldOp, BinaryIntOp, HeapArray, RegisterIndex, RegisterOrMemory};
use acvm::brillig_vm::brillig::HeapVector;
use acvm::FieldElement;
use iter_extended::vecmap;

use super::brillig_black_box::convert_black_box_call;
use super::brillig_fn::FunctionContext;

/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligBlock<'block> {
    pub(crate) function_context: &'block mut FunctionContext,
    /// The basic block that is being converted
    pub(crate) block_id: BasicBlockId,
    /// Context for creating brillig opcodes
    pub(crate) brillig_context: &'block mut BrilligContext,
}

impl<'block> BrilligBlock<'block> {
    /// Converts an SSA Basic block into a sequence of Brillig opcodes
    pub(crate) fn compile(
        function_context: &'block mut FunctionContext,
        brillig_context: &'block mut BrilligContext,
        block_id: BasicBlockId,
        dfg: &DataFlowGraph,
    ) {
        let mut brillig_block = BrilligBlock { function_context, block_id, brillig_context };

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

    fn get_bit_size_from_ssa_type(typ: Type) -> u32 {
        match typ {
            Type::Numeric(num_type) => match num_type {
                NumericType::Signed { bit_size } | NumericType::Unsigned { bit_size } => bit_size,
                NumericType::NativeField => FieldElement::max_num_bits(),
            },
            _ => unreachable!("ICE bitwise not on a non numeric type"),
        }
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
        format!("{}-{}", function_id, block_id)
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
                let condition = self.convert_ssa_register_value(*condition, dfg);
                self.brillig_context.jump_if_instruction(
                    condition,
                    self.create_block_label_for_current_function(*then_destination),
                );
                self.brillig_context.jump_instruction(
                    self.create_block_label_for_current_function(*else_destination),
                );
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                let target = &dfg[*destination];
                for (src, dest) in arguments.iter().zip(target.parameters()) {
                    // Destination variable might have already been created by another block that jumps to this target
                    let destination = self.function_context.get_or_create_variable(
                        self.brillig_context,
                        *dest,
                        dfg,
                    );
                    let source = self.convert_ssa_value(*src, dfg);
                    self.pass_variable(source, destination);
                }
                self.brillig_context
                    .jump_instruction(self.create_block_label_for_current_function(*destination));
            }
            TerminatorInstruction::Return { return_values } => {
                let return_registers: Vec<_> = return_values
                    .iter()
                    .flat_map(|value_id| {
                        let return_variable = self.convert_ssa_value(*value_id, dfg);
                        self.function_context.extract_registers(return_variable)
                    })
                    .collect();
                self.brillig_context.return_instruction(&return_registers);
            }
        }
    }

    /// Passes an arbitrary variable from the registers of the source to the registers of the destination
    fn pass_variable(&mut self, source: RegisterOrMemory, destination: RegisterOrMemory) {
        match (source, destination) {
            (
                RegisterOrMemory::RegisterIndex(source_register),
                RegisterOrMemory::RegisterIndex(destination_register),
            ) => {
                self.brillig_context.mov_instruction(destination_register, source_register);
            }
            (
                RegisterOrMemory::HeapArray(HeapArray { pointer: source_pointer, .. }),
                RegisterOrMemory::HeapArray(HeapArray { pointer: destination_pointer, .. }),
            ) => {
                self.brillig_context.mov_instruction(destination_pointer, source_pointer);
            }
            (
                RegisterOrMemory::HeapVector(HeapVector {
                    pointer: source_pointer,
                    size: source_size,
                }),
                RegisterOrMemory::HeapVector(HeapVector {
                    pointer: destination_pointer,
                    size: destination_size,
                }),
            ) => {
                self.brillig_context.mov_instruction(destination_pointer, source_pointer);
                self.brillig_context.mov_instruction(destination_size, source_size);
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
                Type::Numeric(_) | Type::Array(..) | Type::Slice(..) | Type::Reference => {
                    // This parameter variable might have already been created by another block that jumps to this one.
                    self.function_context.get_or_create_variable(
                        self.brillig_context,
                        *param_id,
                        dfg,
                    );
                }
                _ => {
                    todo!("ICE: Param type not supported")
                }
            }
        }
    }

    /// Converts an SSA instruction into a sequence of Brillig opcodes.
    fn convert_ssa_instruction(&mut self, instruction_id: InstructionId, dfg: &DataFlowGraph) {
        let instruction = &dfg[instruction_id];
        self.brillig_context.set_call_stack(dfg.get_call_stack(instruction_id));

        match instruction {
            Instruction::Binary(binary) => {
                let result_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                self.convert_ssa_binary(binary, dfg, result_register);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                let condition = self.brillig_context.allocate_register();

                self.convert_ssa_binary(
                    &Binary { lhs: *lhs, rhs: *rhs, operator: BinaryOp::Eq },
                    dfg,
                    condition,
                );

                self.brillig_context.constrain_instruction(condition, assert_message.clone());
                self.brillig_context.deallocate_register(condition);
            }
            Instruction::Allocate => {
                let result_value = dfg.instruction_results(instruction_id)[0];
                let address_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    result_value,
                    dfg,
                );
                self.brillig_context.allocate_variable_instruction(address_register);
            }
            Instruction::Store { address, value } => {
                let address_register = self.convert_ssa_register_value(*address, dfg);
                let source_variable = self.convert_ssa_value(*value, dfg);

                self.brillig_context.store_variable_instruction(address_register, source_variable);
            }
            Instruction::Load { address } => {
                let target_variable = self.function_context.create_variable(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );

                let address_register = self.convert_ssa_register_value(*address, dfg);

                self.brillig_context.load_variable_instruction(target_variable, address_register);
            }
            Instruction::Not(value) => {
                let condition_register = self.convert_ssa_register_value(*value, dfg);
                let result_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                let bit_size = Self::get_bit_size_from_ssa_type(dfg.type_of_value(*value));
                self.brillig_context.not_instruction(condition_register, bit_size, result_register);
            }
            Instruction::Call { func, arguments } => match &dfg[*func] {
                Value::ForeignFunction(func_name) => {
                    let result_ids = dfg.instruction_results(instruction_id);

                    let input_registers =
                        vecmap(arguments, |value_id| self.convert_ssa_value(*value_id, dfg));
                    let output_registers = vecmap(result_ids, |value_id| {
                        self.allocate_external_call_result(*value_id, dfg)
                    });
                    self.brillig_context.foreign_call_instruction(
                        func_name.to_owned(),
                        &input_registers,
                        &output_registers,
                    );

                    for (i, output_register) in output_registers.iter().enumerate() {
                        if let RegisterOrMemory::HeapVector(HeapVector { size, .. }) =
                            output_register
                        {
                            // Update the stack pointer so that we do not overwrite
                            // dynamic memory returned from other external calls
                            self.brillig_context.update_stack_pointer(*size);

                            // Update the dynamic slice length maintained in SSA
                            if let RegisterOrMemory::RegisterIndex(len_index) =
                                output_registers[i - 1]
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
                    self.convert_ssa_function_call(*func_id, arguments, dfg, instruction_id);
                }
                Value::Intrinsic(Intrinsic::BlackBox(bb_func)) => {
                    let function_arguments =
                        vecmap(arguments, |arg| self.convert_ssa_value(*arg, dfg));
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
                    let result_register = self.function_context.create_register_variable(
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
                        let len_register_index =
                            self.function_context.extract_register(len_variable);
                        self.brillig_context.mov_instruction(result_register, len_register_index);
                    } else {
                        self.convert_ssa_array_len(arguments[0], result_register, dfg);
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
                    let source = self.convert_ssa_register_value(arguments[0], dfg);
                    let radix = self.convert_ssa_register_value(arguments[1], dfg);
                    let limb_count = self.convert_ssa_register_value(arguments[2], dfg);

                    let results = dfg.instruction_results(instruction_id);

                    let target_len_variable = self.function_context.get_or_create_variable(
                        self.brillig_context,
                        results[0],
                        dfg,
                    );
                    let target_len = self.function_context.extract_register(target_len_variable);

                    let target_slice = self.function_context.create_variable(
                        self.brillig_context,
                        results[1],
                        dfg,
                    );

                    let heap_vec = self.brillig_context.extract_heap_vector(target_slice);

                    // Update the user-facing slice length
                    self.brillig_context.mov_instruction(target_len, limb_count);

                    self.brillig_context.radix_instruction(
                        source,
                        heap_vec,
                        radix,
                        limb_count,
                        matches!(endianness, Endian::Big),
                    );
                }
                Value::Intrinsic(Intrinsic::ToBits(endianness)) => {
                    let source = self.convert_ssa_register_value(arguments[0], dfg);
                    let limb_count = self.convert_ssa_register_value(arguments[1], dfg);

                    let results = dfg.instruction_results(instruction_id);

                    let target_len_variable = self.function_context.get_or_create_variable(
                        self.brillig_context,
                        results[0],
                        dfg,
                    );
                    let target_len = self.function_context.extract_register(target_len_variable);

                    let target_slice = self.function_context.create_variable(
                        self.brillig_context,
                        results[1],
                        dfg,
                    );

                    let radix = self.brillig_context.make_constant(2_usize.into());
                    let heap_vec = self.brillig_context.extract_heap_vector(target_slice);

                    // Update the user-facing slice length
                    self.brillig_context.mov_instruction(target_len, limb_count);

                    self.brillig_context.radix_instruction(
                        source,
                        heap_vec,
                        radix,
                        limb_count,
                        matches!(endianness, Endian::Big),
                    );

                    self.brillig_context.deallocate_register(radix);
                }
                _ => {
                    unreachable!("unsupported function call type {:?}", dfg[*func])
                }
            },
            Instruction::Truncate { value, .. } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );
                let source_register = self.convert_ssa_register_value(*value, dfg);
                self.brillig_context.truncate_instruction(destination_register, source_register);
            }
            Instruction::Cast(value, target_type) => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );
                let source_register = self.convert_ssa_register_value(*value, dfg);
                self.convert_cast(
                    destination_register,
                    source_register,
                    target_type,
                    &dfg.type_of_value(*value),
                );
            }
            Instruction::ArrayGet { array, index } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination_variable =
                    self.function_context.create_variable(self.brillig_context, result_ids[0], dfg);

                let array_variable = self.convert_ssa_value(*array, dfg);
                let array_pointer = match array_variable {
                    RegisterOrMemory::HeapArray(HeapArray { pointer, .. }) => pointer,
                    RegisterOrMemory::HeapVector(HeapVector { pointer, .. }) => pointer,
                    _ => unreachable!("ICE: array get on non-array"),
                };

                let index_register = self.convert_ssa_register_value(*index, dfg);
                self.retrieve_variable_from_array(
                    array_pointer,
                    index_register,
                    destination_variable,
                );
            }
            Instruction::ArraySet { array, index, value } => {
                let source_variable = self.convert_ssa_value(*array, dfg);
                let index_register = self.convert_ssa_register_value(*index, dfg);
                let value_variable = self.convert_ssa_value(*value, dfg);

                let result_ids = dfg.instruction_results(instruction_id);
                let destination_variable =
                    self.function_context.create_variable(self.brillig_context, result_ids[0], dfg);

                self.convert_ssa_array_set(
                    source_variable,
                    destination_variable,
                    index_register,
                    value_variable,
                );
            }
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };

        self.brillig_context.set_call_stack(CallStack::new());
    }

    fn convert_ssa_function_call(
        &mut self,
        func_id: FunctionId,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        instruction_id: InstructionId,
    ) {
        // Convert the arguments to registers casting those to the types of the receiving function
        let argument_registers: Vec<RegisterIndex> = arguments
            .iter()
            .flat_map(|argument_id| {
                let variable_to_pass = self.convert_ssa_value(*argument_id, dfg);
                self.function_context.extract_registers(variable_to_pass)
            })
            .collect();

        let result_ids = dfg.instruction_results(instruction_id);

        // Create label for the function that will be called
        let label_of_function_to_call = FunctionContext::function_id_to_function_label(func_id);

        let saved_registers =
            self.brillig_context.pre_call_save_registers_prep_args(&argument_registers);

        // Call instruction, which will interpret above registers 0..num args
        self.brillig_context.add_external_call_instruction(label_of_function_to_call);

        // Important: resolve after pre_call_save_registers_prep_args
        // This ensures we don't save the results to registers unnecessarily.

        // Allocate the registers for the variables where we are assigning the returns
        let variables_assigned_to = vecmap(result_ids, |result_id| {
            self.function_context.create_variable(self.brillig_context, *result_id, dfg)
        });

        // Collect the registers that should have been returned
        let returned_registers: Vec<RegisterIndex> = variables_assigned_to
            .iter()
            .flat_map(|returned_variable| {
                self.function_context.extract_registers(*returned_variable)
            })
            .collect();

        assert!(
            !saved_registers.iter().any(|x| returned_registers.contains(x)),
            "should not save registers used as function results"
        );

        // puts the returns into the returned_registers and restores saved_registers
        self.brillig_context
            .post_call_prep_returns_load_registers(&returned_registers, &saved_registers);
    }

    pub(crate) fn retrieve_variable_from_array(
        &mut self,
        array_pointer: RegisterIndex,
        index_register: RegisterIndex,
        destination_variable: RegisterOrMemory,
    ) {
        match destination_variable {
            RegisterOrMemory::RegisterIndex(destination_register) => {
                self.brillig_context.array_get(array_pointer, index_register, destination_register);
            }
            RegisterOrMemory::HeapArray(HeapArray { pointer, .. }) => {
                self.brillig_context.array_get(array_pointer, index_register, pointer);
            }
            RegisterOrMemory::HeapVector(..) => {
                // Vectors are stored as references inside arrays to be able to match SSA indexes
                let reference = self.brillig_context.allocate_register();
                self.brillig_context.array_get(array_pointer, index_register, reference);
                self.brillig_context.load_variable_instruction(destination_variable, reference);
                self.brillig_context.deallocate_register(reference);
            }
        }
    }

    /// Array set operation in SSA returns a new array or slice that is a copy of the parameter array or slice
    /// With a specific value changed.
    fn convert_ssa_array_set(
        &mut self,
        source_variable: RegisterOrMemory,
        destination_variable: RegisterOrMemory,
        index_register: RegisterIndex,
        value_variable: RegisterOrMemory,
    ) {
        let destination_pointer = match destination_variable {
            RegisterOrMemory::HeapArray(HeapArray { pointer, .. }) => pointer,
            RegisterOrMemory::HeapVector(HeapVector { pointer, .. }) => pointer,
            _ => unreachable!("ICE: array set returns non-array"),
        };

        // First issue a array copy to the destination
        let (source_pointer, source_size_as_register) = match source_variable {
            RegisterOrMemory::HeapArray(HeapArray { size, pointer }) => {
                let source_size_register = self.brillig_context.allocate_register();
                self.brillig_context.const_instruction(source_size_register, size.into());
                (pointer, source_size_register)
            }
            RegisterOrMemory::HeapVector(HeapVector { size, pointer }) => {
                let source_size_register = self.brillig_context.allocate_register();
                self.brillig_context.mov_instruction(source_size_register, size);
                (pointer, source_size_register)
            }
            _ => unreachable!("ICE: array set on non-array"),
        };

        self.brillig_context
            .allocate_array_instruction(destination_pointer, source_size_as_register);

        self.brillig_context.copy_array_instruction(
            source_pointer,
            destination_pointer,
            source_size_as_register,
        );

        if let RegisterOrMemory::HeapVector(HeapVector { size: target_size, .. }) =
            destination_variable
        {
            self.brillig_context.mov_instruction(target_size, source_size_as_register);
        }

        // Then set the value in the newly created array
        self.store_variable_in_array(destination_pointer, index_register, value_variable);

        self.brillig_context.deallocate_register(source_size_as_register);
    }

    pub(crate) fn store_variable_in_array(
        &mut self,
        destination_pointer: RegisterIndex,
        index_register: RegisterIndex,
        value_variable: RegisterOrMemory,
    ) {
        match value_variable {
            RegisterOrMemory::RegisterIndex(value_register) => {
                self.brillig_context.array_set(destination_pointer, index_register, value_register);
            }
            RegisterOrMemory::HeapArray(HeapArray { pointer, .. }) => {
                self.brillig_context.array_set(destination_pointer, index_register, pointer);
            }
            RegisterOrMemory::HeapVector(_) => {
                // Vectors are stored as references inside arrays to be able to match SSA indexes
                let reference = self.brillig_context.allocate_register();
                self.brillig_context.allocate_variable_instruction(reference);
                self.brillig_context.store_variable_instruction(reference, value_variable);
                self.brillig_context.array_set(destination_pointer, index_register, reference);
                self.brillig_context.deallocate_register(reference);
            }
        }
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
                let target_len = match self.function_context.get_or_create_variable(
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    RegisterOrMemory::RegisterIndex(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[1], dfg);

                let target_vector = self.brillig_context.extract_heap_vector(target_variable);
                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(target_len, arguments[0], dfg, BinaryIntOp::Add);

                self.slice_push_back_operation(target_vector, source_vector, &item_values);
            }
            Value::Intrinsic(Intrinsic::SlicePushFront) => {
                let target_len = match self.function_context.get_or_create_variable(
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    RegisterOrMemory::RegisterIndex(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[1], dfg);
                let target_vector = self.brillig_context.extract_heap_vector(target_variable);
                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(target_len, arguments[0], dfg, BinaryIntOp::Add);

                self.slice_push_front_operation(target_vector, source_vector, &item_values);
            }
            Value::Intrinsic(Intrinsic::SlicePopBack) => {
                let target_len = match self.function_context.get_or_create_variable(
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    RegisterOrMemory::RegisterIndex(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[1], dfg);

                let target_vector = self.brillig_context.extract_heap_vector(target_variable);

                let pop_variables = vecmap(&results[2..element_size + 2], |result| {
                    self.function_context.create_variable(self.brillig_context, *result, dfg)
                });

                self.update_slice_length(target_len, arguments[0], dfg, BinaryIntOp::Sub);

                self.slice_pop_back_operation(target_vector, source_vector, &pop_variables);
            }
            Value::Intrinsic(Intrinsic::SlicePopFront) => {
                let target_len = match self.function_context.get_or_create_variable(
                    self.brillig_context,
                    results[element_size],
                    dfg,
                ) {
                    RegisterOrMemory::RegisterIndex(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let pop_variables = vecmap(&results[0..element_size], |result| {
                    self.function_context.create_variable(self.brillig_context, *result, dfg)
                });

                let target_variable = self.function_context.create_variable(
                    self.brillig_context,
                    results[element_size + 1],
                    dfg,
                );
                let target_vector = self.brillig_context.extract_heap_vector(target_variable);

                self.update_slice_length(target_len, arguments[0], dfg, BinaryIntOp::Sub);

                self.slice_pop_front_operation(target_vector, source_vector, &pop_variables);
            }
            Value::Intrinsic(Intrinsic::SliceInsert) => {
                let target_len = match self.function_context.get_or_create_variable(
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    RegisterOrMemory::RegisterIndex(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_id = results[1];
                let target_variable =
                    self.function_context.create_variable(self.brillig_context, target_id, dfg);

                let target_vector = self.brillig_context.extract_heap_vector(target_variable);

                // Remove if indexing in insert is changed to flattened indexing
                // https://github.com/noir-lang/noir/issues/1889#issuecomment-1668048587
                let user_index = self.convert_ssa_register_value(arguments[2], dfg);

                let converted_index = self.brillig_context.make_constant(element_size.into());

                self.brillig_context.memory_op(
                    converted_index,
                    user_index,
                    converted_index,
                    BinaryIntOp::Mul,
                );

                let items = vecmap(&arguments[3..element_size + 3], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(target_len, arguments[0], dfg, BinaryIntOp::Add);

                self.slice_insert_operation(target_vector, source_vector, converted_index, &items);
                self.brillig_context.deallocate_register(converted_index);
            }
            Value::Intrinsic(Intrinsic::SliceRemove) => {
                let target_len = match self.function_context.get_or_create_variable(
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    RegisterOrMemory::RegisterIndex(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_id = results[1];

                let target_variable =
                    self.function_context.create_variable(self.brillig_context, target_id, dfg);
                let target_vector = self.brillig_context.extract_heap_vector(target_variable);

                // Remove if indexing in remove is changed to flattened indexing
                // https://github.com/noir-lang/noir/issues/1889#issuecomment-1668048587
                let user_index = self.convert_ssa_register_value(arguments[2], dfg);

                let converted_index = self.brillig_context.make_constant(element_size.into());
                self.brillig_context.memory_op(
                    converted_index,
                    user_index,
                    converted_index,
                    BinaryIntOp::Mul,
                );

                let removed_items = vecmap(&results[2..element_size + 2], |result| {
                    self.function_context.create_variable(self.brillig_context, *result, dfg)
                });

                self.update_slice_length(target_len, arguments[0], dfg, BinaryIntOp::Sub);

                self.slice_remove_operation(
                    target_vector,
                    source_vector,
                    converted_index,
                    &removed_items,
                );

                self.brillig_context.deallocate_register(converted_index);
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
        target_len: RegisterIndex,
        source_value: ValueId,
        dfg: &DataFlowGraph,
        binary_op: BinaryIntOp,
    ) {
        let source_len_variable =
            self.function_context.get_or_create_variable(self.brillig_context, source_value, dfg);
        let source_len = self.function_context.extract_register(source_len_variable);

        self.brillig_context.usize_op(source_len, target_len, binary_op, 1);
    }

    /// Converts an SSA cast to a sequence of Brillig opcodes.
    /// Casting is only necessary when shrinking the bit size of a numeric value.
    fn convert_cast(
        &mut self,
        destination: RegisterIndex,
        source: RegisterIndex,
        target_type: &Type,
        source_type: &Type,
    ) {
        fn numeric_to_bit_size(typ: &NumericType) -> u32 {
            match typ {
                NumericType::Signed { bit_size } | NumericType::Unsigned { bit_size } => *bit_size,
                NumericType::NativeField => FieldElement::max_num_bits(),
            }
        }
        // Casting is only valid for numeric types
        // This should be checked by the frontend, so we panic if this is the case
        let (source_numeric_type, target_numeric_type) = match (source_type, target_type) {
            (Type::Numeric(source_numeric_type), Type::Numeric(target_numeric_type)) => {
                (source_numeric_type, target_numeric_type)
            }
            _ => unimplemented!("The cast operation is only valid for integers."),
        };
        let source_bit_size = numeric_to_bit_size(source_numeric_type);
        let target_bit_size = numeric_to_bit_size(target_numeric_type);
        // Casting from a larger bit size to a smaller bit size (narrowing cast)
        // requires a cast instruction.
        // If its a widening cast, ie casting from a smaller bit size to a larger bit size
        // we simply put a mov instruction as a no-op
        //
        // Field elements by construction always have the largest bit size
        // This means that casting to a Field element, will always be a widening cast
        // and therefore a no-op. Conversely, casting from a Field element
        // will always be a narrowing cast and therefore a cast instruction
        if source_bit_size > target_bit_size {
            self.brillig_context.cast_instruction(destination, source, target_bit_size);
        } else {
            self.brillig_context.mov_instruction(destination, source);
        }
    }

    /// Converts the Binary instruction into a sequence of Brillig opcodes.
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
        result_register: RegisterIndex,
    ) {
        let binary_type =
            type_of_binary_operation(dfg[binary.lhs].get_type(), dfg[binary.rhs].get_type());

        let mut left = self.convert_ssa_register_value(binary.lhs, dfg);
        let mut right = self.convert_ssa_register_value(binary.rhs, dfg);

        let brillig_binary_op =
            convert_ssa_binary_op_to_brillig_binary_op(binary.operator, &binary_type);

        // Some binary operations with fields are issued by the compiler, such as loop comparisons, cast those to the bit size here
        // TODO Remove after fixing https://github.com/noir-lang/noir/issues/1979
        if let (
            BrilligBinaryOp::Integer { bit_size, .. },
            Type::Numeric(NumericType::NativeField),
        ) = (&brillig_binary_op, &binary_type)
        {
            let new_lhs = self.brillig_context.allocate_register();
            let new_rhs = self.brillig_context.allocate_register();

            self.brillig_context.cast_instruction(new_lhs, left, *bit_size);
            self.brillig_context.cast_instruction(new_rhs, right, *bit_size);

            left = new_lhs;
            right = new_rhs;
        }

        self.brillig_context.binary_instruction(left, right, result_register, brillig_binary_op);
    }

    /// Converts an SSA `ValueId` into a `RegisterOrMemory`. Initializes if necessary.
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> RegisterOrMemory {
        let value = &dfg[dfg.resolve(value_id)];

        match value {
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.
                self.function_context.get_variable(value_id, dfg)
            }
            Value::NumericConstant { constant, .. } => {
                // Constants might have been converted previously or not, so we get or create and
                // (re)initialize the value inside.
                let new_variable = self.function_context.get_or_create_variable(
                    self.brillig_context,
                    value_id,
                    dfg,
                );
                let register_index = self.function_context.extract_register(new_variable);

                self.brillig_context.const_instruction(register_index, (*constant).into());
                new_variable
            }
            Value::Array { array, .. } => {
                let new_variable = self.function_context.get_or_create_variable(
                    self.brillig_context,
                    value_id,
                    dfg,
                );

                // Initialize the variable
                let pointer = match new_variable {
                    RegisterOrMemory::HeapArray(heap_array) => {
                        self.brillig_context
                            .allocate_fixed_length_array(heap_array.pointer, array.len());

                        heap_array.pointer
                    }
                    RegisterOrMemory::HeapVector(heap_vector) => {
                        self.brillig_context
                            .const_instruction(heap_vector.size, array.len().into());
                        self.brillig_context
                            .allocate_array_instruction(heap_vector.pointer, heap_vector.size);

                        heap_vector.pointer
                    }
                    _ => unreachable!(
                        "ICE: Cannot initialize array value created as {new_variable:?}"
                    ),
                };

                // Write the items

                // Allocate a register for the iterator
                let iterator_register = self.brillig_context.make_constant(0_usize.into());

                for element_id in array.iter() {
                    let element_variable = self.convert_ssa_value(*element_id, dfg);
                    // Store the item in memory
                    self.store_variable_in_array(pointer, iterator_register, element_variable);
                    // Increment the iterator
                    self.brillig_context.usize_op_in_place(iterator_register, BinaryIntOp::Add, 1);
                }

                self.brillig_context.deallocate_register(iterator_register);

                new_variable
            }
            _ => {
                todo!("ICE: Cannot convert value {value:?}")
            }
        }
    }

    /// Converts an SSA `ValueId` into a `RegisterIndex`. Initializes if necessary.
    fn convert_ssa_register_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterIndex {
        let variable = self.convert_ssa_value(value_id, dfg);
        self.function_context.extract_register(variable)
    }

    fn allocate_external_call_result(
        &mut self,
        result: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let typ = dfg[result].get_type();
        match typ {
            Type::Numeric(_) => {
                self.function_context.create_variable(self.brillig_context, result, dfg)
            }

            Type::Array(..) => {
                let variable =
                    self.function_context.create_variable(self.brillig_context, result, dfg);
                let array = self.function_context.extract_heap_array(variable);
                self.brillig_context.allocate_fixed_length_array(array.pointer, array.size);
                variable
            }
            Type::Slice(_) => {
                let variable =
                    self.function_context.create_variable(self.brillig_context, result, dfg);
                let vector = self.brillig_context.extract_heap_vector(variable);

                // Set the pointer to the current stack frame
                // The stack pointer will then be updated by the caller of this method
                // once the external call is resolved and the array size is known
                self.brillig_context.set_array_pointer(vector.pointer);
                variable
            }
            _ => {
                unreachable!("ICE: unsupported return type for black box call {typ:?}")
            }
        }
    }

    /// Gets the "user-facing" length of an array.
    /// An array of structs with two fields would be stored as an 2 * array.len() heap array/heap vector.
    /// So we divide the length by the number of subitems in an item to get the user-facing length.
    fn convert_ssa_array_len(
        &mut self,
        array_id: ValueId,
        result_register: RegisterIndex,
        dfg: &DataFlowGraph,
    ) {
        let array_variable = self.convert_ssa_value(array_id, dfg);
        let element_size = dfg.type_of_value(array_id).element_size();

        match array_variable {
            RegisterOrMemory::HeapArray(HeapArray { size, .. }) => {
                self.brillig_context
                    .const_instruction(result_register, (size / element_size).into());
            }
            RegisterOrMemory::HeapVector(HeapVector { size, .. }) => {
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
/// TODO: SSA issues binary operations between fields and integers.
/// This probably should be explicitly casted in SSA to avoid having to coerce at this level.
pub(crate) fn type_of_binary_operation(lhs_type: &Type, rhs_type: &Type) -> Type {
    match (lhs_type, rhs_type) {
        (_, Type::Function) | (Type::Function, _) => {
            unreachable!("Functions are invalid in binary operations")
        }
        (_, Type::Reference) | (Type::Reference, _) => {
            unreachable!("References are invalid in binary operations")
        }
        (_, Type::Array(..)) | (Type::Array(..), _) => {
            unreachable!("Arrays are invalid in binary operations")
        }
        (_, Type::Slice(..)) | (Type::Slice(..), _) => {
            unreachable!("Arrays are invalid in binary operations")
        }
        // If either side is a Field constant then, we coerce into the type
        // of the other operand
        (Type::Numeric(NumericType::NativeField), typ)
        | (typ, Type::Numeric(NumericType::NativeField)) => typ.clone(),
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
) -> BrilligBinaryOp {
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
            BinaryOp::Add => BrilligBinaryOp::Field { op: BinaryFieldOp::Add },
            BinaryOp::Sub => BrilligBinaryOp::Field { op: BinaryFieldOp::Sub },
            BinaryOp::Mul => BrilligBinaryOp::Field { op: BinaryFieldOp::Mul },
            BinaryOp::Div => BrilligBinaryOp::Field { op: BinaryFieldOp::Div },
            BinaryOp::Eq => BrilligBinaryOp::Field { op: BinaryFieldOp::Equals },
            BinaryOp::Lt => BrilligBinaryOp::Integer {
                op: BinaryIntOp::LessThan,
                bit_size: BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE,
            },
            _ => unreachable!(
                "Field type cannot be used with {op}. This should have been caught by the frontend"
            ),
        }
    }

    fn binary_op_to_int_op(op: BinaryOp, bit_size: u32, is_signed: bool) -> BrilligBinaryOp {
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
            BinaryOp::Mod => {
                return BrilligBinaryOp::Modulo { is_signed_integer: is_signed, bit_size }
            }
            BinaryOp::Eq => BinaryIntOp::Equals,
            BinaryOp::Lt => BinaryIntOp::LessThan,
            BinaryOp::And => BinaryIntOp::And,
            BinaryOp::Or => BinaryIntOp::Or,
            BinaryOp::Xor => BinaryIntOp::Xor,
        };

        BrilligBinaryOp::Integer { op: operation, bit_size }
    }

    // If bit size is available then it is a binary integer operation
    match bit_size_signedness {
        Some((bit_size, is_signed)) => binary_op_to_int_op(ssa_op, *bit_size, is_signed),
        None => binary_op_to_field_op(ssa_op),
    }
}
