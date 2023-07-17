use crate::brillig::brillig_gen::brillig_slice_ops::{
    convert_array_or_vector_to_vector, slice_push_back_operation,
};
use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext};
use crate::ssa_refactor::ir::function::FunctionId;
use crate::ssa_refactor::ir::instruction::Intrinsic;
use crate::ssa_refactor::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    dfg::DataFlowGraph,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction},
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::acir::brillig::{BinaryFieldOp, BinaryIntOp, HeapArray, RegisterIndex, RegisterOrMemory};
use acvm::brillig_vm::brillig::HeapVector;
use acvm::FieldElement;
use iter_extended::vecmap;

use super::brillig_black_box::convert_black_box_call;
use super::brillig_fn::{compute_size_of_composite_type, FunctionContext};
use super::brillig_slice_ops::{
    slice_insert_operation, slice_pop_back_operation, slice_pop_front_operation,
    slice_push_front_operation, slice_remove_operation,
};

/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligBlock<'block> {
    function_context: &'block mut FunctionContext,
    /// The basic block that is being converted
    block_id: BasicBlockId,
    /// Context for creating brillig opcodes
    brillig_context: &'block mut BrilligContext,
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
            TerminatorInstruction::Jmp { destination, arguments } => {
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

        match instruction {
            Instruction::Binary(binary) => {
                let result_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                self.convert_ssa_binary(binary, dfg, result_register);
            }
            Instruction::Constrain(value) => {
                let condition = self.convert_ssa_register_value(*value, dfg);
                self.brillig_context.constrain_instruction(condition);
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

                    for output_register in output_registers {
                        if let RegisterOrMemory::HeapVector(HeapVector { size, .. }) =
                            output_register
                        {
                            // Update the stack pointer so that we do not overwrite
                            // dynamic memory returned from other external calls
                            self.brillig_context.update_stack_pointer(size);
                        }
                        // Single values and allocation of fixed sized arrays has already been handled
                        // inside of `allocate_external_call_result`
                    }
                }
                Value::Function(func_id) => {
                    let argument_registers: Vec<RegisterIndex> = arguments
                        .iter()
                        .flat_map(|arg| {
                            let arg = self.convert_ssa_value(*arg, dfg);
                            self.function_context.extract_registers(arg)
                        })
                        .collect();
                    let result_ids = dfg.instruction_results(instruction_id);

                    // Create label for the function that will be called
                    let label_of_function_to_call =
                        FunctionContext::function_id_to_function_label(*func_id);

                    let saved_registers =
                        self.brillig_context.pre_call_save_registers_prep_args(&argument_registers);

                    // Call instruction, which will interpret above registers 0..num args
                    self.brillig_context.add_external_call_instruction(label_of_function_to_call);

                    // Important: resolve after pre_call_save_registers_prep_args
                    // This ensures we don't save the results to registers unnecessarily.
                    let result_registers: Vec<RegisterIndex> = result_ids
                        .iter()
                        .flat_map(|arg| {
                            let arg = self.function_context.create_variable(
                                self.brillig_context,
                                *arg,
                                dfg,
                            );
                            self.function_context.extract_registers(arg)
                        })
                        .collect();

                    assert!(
                        !saved_registers.iter().any(|x| result_registers.contains(x)),
                        "should not save registers used as function results"
                    );
                    self.brillig_context
                        .post_call_prep_returns_load_registers(&result_registers, &saved_registers);
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
                    let param = self.convert_ssa_value(arguments[0], dfg);
                    self.brillig_context.length_of_variable_instruction(param, result_register);
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
                        &dfg[*func],
                        instruction_id,
                        arguments,
                    );
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
                let destination_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    result_ids[0],
                    dfg,
                );

                let array_variable = self.convert_ssa_value(*array, dfg);
                let array_pointer = match array_variable {
                    RegisterOrMemory::HeapArray(HeapArray { pointer, .. }) => pointer,
                    RegisterOrMemory::HeapVector(HeapVector { pointer, .. }) => pointer,
                    _ => unreachable!("ICE: array get on non-array"),
                };

                let index_register = self.convert_ssa_register_value(*index, dfg);
                self.brillig_context.array_get(array_pointer, index_register, destination_register);
            }
            Instruction::ArraySet { array, index, value } => {
                let source_variable = self.convert_ssa_value(*array, dfg);
                let index_register = self.convert_ssa_register_value(*index, dfg);
                let value_register = self.convert_ssa_register_value(*value, dfg);

                let result_ids = dfg.instruction_results(instruction_id);
                let destination_variable =
                    self.function_context.create_variable(self.brillig_context, result_ids[0], dfg);

                self.convert_ssa_array_set(
                    source_variable,
                    destination_variable,
                    index_register,
                    value_register,
                );
            }
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };
    }

    /// Array set operation in SSA returns a new array or slice that is a copy of the parameter array or slice
    /// With a specific value changed.
    fn convert_ssa_array_set(
        &mut self,
        source_variable: RegisterOrMemory,
        destination_variable: RegisterOrMemory,
        index_register: RegisterIndex,
        value_register: RegisterIndex,
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
        self.brillig_context.array_set(destination_pointer, index_register, value_register);

        self.brillig_context.deallocate_register(source_size_as_register);
    }

    /// Convert the SSA slice operations to brillig slice operations
    fn convert_ssa_slice_intrinsic_call(
        &mut self,
        dfg: &DataFlowGraph,
        intrinsic: &Value,
        instruction_id: InstructionId,
        arguments: &[ValueId],
    ) {
        let source_variable = self.convert_ssa_value(arguments[0], dfg);
        let source_vector =
            convert_array_or_vector_to_vector(self.brillig_context, source_variable);

        match intrinsic {
            Value::Intrinsic(Intrinsic::SlicePushBack) => {
                let target_variable = self.function_context.create_variable(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                let target_vector = self.function_context.extract_heap_vector(target_variable);
                let item_value = self.convert_ssa_register_value(arguments[1], dfg);
                slice_push_back_operation(
                    self.brillig_context,
                    target_vector,
                    source_vector,
                    item_value,
                );
            }
            Value::Intrinsic(Intrinsic::SlicePushFront) => {
                let target_variable = self.function_context.create_variable(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                    dfg,
                );
                let target_vector = self.function_context.extract_heap_vector(target_variable);
                let item_value = self.convert_ssa_register_value(arguments[1], dfg);
                slice_push_front_operation(
                    self.brillig_context,
                    target_vector,
                    source_vector,
                    item_value,
                );
            }
            Value::Intrinsic(Intrinsic::SlicePopBack) => {
                let results = dfg.instruction_results(instruction_id);

                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[0], dfg);
                let target_vector = self.function_context.extract_heap_vector(target_variable);

                let pop_item = self.function_context.create_register_variable(
                    self.brillig_context,
                    results[1],
                    dfg,
                );

                slice_pop_back_operation(
                    self.brillig_context,
                    target_vector,
                    source_vector,
                    pop_item,
                );
            }
            Value::Intrinsic(Intrinsic::SlicePopFront) => {
                let results = dfg.instruction_results(instruction_id);

                let pop_item = self.function_context.create_register_variable(
                    self.brillig_context,
                    results[0],
                    dfg,
                );
                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[1], dfg);
                let target_vector = self.function_context.extract_heap_vector(target_variable);

                slice_pop_front_operation(
                    self.brillig_context,
                    target_vector,
                    source_vector,
                    pop_item,
                );
            }
            Value::Intrinsic(Intrinsic::SliceInsert) => {
                let results = dfg.instruction_results(instruction_id);
                let index = self.convert_ssa_register_value(arguments[1], dfg);
                let item = self.convert_ssa_register_value(arguments[2], dfg);
                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[0], dfg);

                let target_vector = self.function_context.extract_heap_vector(target_variable);
                slice_insert_operation(
                    self.brillig_context,
                    target_vector,
                    source_vector,
                    index,
                    item,
                );
            }
            Value::Intrinsic(Intrinsic::SliceRemove) => {
                let results = dfg.instruction_results(instruction_id);
                let index = self.convert_ssa_register_value(arguments[1], dfg);

                let target_variable =
                    self.function_context.create_variable(self.brillig_context, results[0], dfg);
                let target_vector = self.function_context.extract_heap_vector(target_variable);

                let removed_item_register = self.function_context.create_register_variable(
                    self.brillig_context,
                    results[1],
                    dfg,
                );

                slice_remove_operation(
                    self.brillig_context,
                    target_vector,
                    source_vector,
                    index,
                    removed_item_register,
                );
            }
            _ => unreachable!("ICE: Slice operation not supported"),
        }
    }

    /// This function allows storing a Value in memory starting at the address specified by the
    /// address_register. The value can be a single value or an array. The function will recursively
    /// store the value in memory.
    fn store_in_memory(
        &mut self,
        address_register: RegisterIndex,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) {
        let value = &dfg[value_id];
        match value {
            Value::Param { .. } | Value::Instruction { .. } | Value::NumericConstant { .. } => {
                let value_register = self.convert_ssa_register_value(value_id, dfg);
                self.brillig_context.store_instruction(address_register, value_register);
            }
            Value::Array { array, element_type } => {
                // Allocate a register for the iterator
                let iterator_register = self.brillig_context.allocate_register();
                // Set the iterator to the address of the array
                self.brillig_context.mov_instruction(iterator_register, address_register);

                let size_of_item_register = self
                    .brillig_context
                    .make_constant(compute_size_of_composite_type(element_type).into());

                for element_id in array.iter() {
                    // Store the item in memory
                    self.store_in_memory(iterator_register, *element_id, dfg);
                    // Increment the iterator by the size of the items
                    self.brillig_context.memory_op(
                        iterator_register,
                        size_of_item_register,
                        iterator_register,
                        BinaryIntOp::Add,
                    );
                }
            }
            _ => unimplemented!("ICE: Value {:?} not storeable in memory", value),
        }
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

        let left = self.convert_ssa_register_value(binary.lhs, dfg);
        let right = self.convert_ssa_register_value(binary.rhs, dfg);

        let brillig_binary_op =
            convert_ssa_binary_op_to_brillig_binary_op(binary.operator, binary_type);

        self.brillig_context.binary_instruction(left, right, result_register, brillig_binary_op);
    }

    /// Converts an SSA `ValueId` into a `RegisterOrMemory`. Initializes if necessary.
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> RegisterOrMemory {
        let value = &dfg[value_id];

        let variable = match value {
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.
                self.function_context.get_variable(value_id)
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
            Value::Array { .. } => {
                let new_variable =
                    self.function_context.create_variable(self.brillig_context, value_id, dfg);
                let heap_array = self.function_context.extract_heap_array(new_variable);

                self.brillig_context
                    .allocate_fixed_length_array(heap_array.pointer, heap_array.size);
                self.store_in_memory(heap_array.pointer, value_id, dfg);
                new_variable
            }
            _ => {
                todo!("ICE: Cannot convert value {value:?}")
            }
        };
        variable
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
                let vector = self.function_context.extract_heap_vector(variable);

                // Set the pointer to the current stack frame
                // The stack pointer will then be update by the caller of this method
                // once the external call is resolved and the array size is known
                self.brillig_context.set_array_pointer(vector.pointer);
                variable
            }
            _ => {
                unreachable!("ICE: unsupported return type for black box call {typ:?}")
            }
        }
    }
}

/// Returns the type of the operation considering the types of the operands
/// TODO: SSA issues binary operations between fields and integers.
/// This probably should be explicitly casted in SSA to avoid having to coerce at this level.
pub(crate) fn type_of_binary_operation(lhs_type: Type, rhs_type: Type) -> Type {
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
        | (typ, Type::Numeric(NumericType::NativeField)) => typ,
        // If both sides are numeric type, then we expect their types to be
        // the same.
        (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
            assert_eq!(
                lhs_type, rhs_type,
                "lhs and rhs types in a binary operation are always the same"
            );
            Type::Numeric(lhs_type)
        }
    }
}

/// Convert an SSA binary operation into:
/// - Brillig Binary Integer Op, if it is a integer type
/// - Brillig Binary Field Op, if it is a field type
pub(crate) fn convert_ssa_binary_op_to_brillig_binary_op(
    ssa_op: BinaryOp,
    typ: Type,
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
        let operation = match op {
            BinaryOp::Add => BinaryFieldOp::Add,
            BinaryOp::Sub => BinaryFieldOp::Sub,
            BinaryOp::Mul => BinaryFieldOp::Mul,
            BinaryOp::Div => BinaryFieldOp::Div,
            BinaryOp::Eq => BinaryFieldOp::Equals,
            _ => unreachable!(
                "Field type cannot be used with {op}. This should have been caught by the frontend"
            ),
        };

        BrilligBinaryOp::Field { op: operation }
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
            BinaryOp::Shl => BinaryIntOp::Shl,
            BinaryOp::Shr => BinaryIntOp::Shr,
        };

        BrilligBinaryOp::Integer { op: operation, bit_size }
    }

    // If bit size is available then it is a binary integer operation
    match bit_size_signedness {
        Some((bit_size, is_signed)) => binary_op_to_int_op(ssa_op, bit_size, is_signed),
        None => binary_op_to_field_op(ssa_op),
    }
}
