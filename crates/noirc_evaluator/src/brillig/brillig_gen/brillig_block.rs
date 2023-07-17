use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};
use crate::ssa_refactor::ir::function::FunctionId;
use crate::ssa_refactor::ir::instruction::Intrinsic;
use crate::ssa_refactor::ir::types::CompositeType;
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
use super::brillig_fn::FunctionContext;

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
                let condition = self.convert_ssa_value(*condition, dfg);
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
                    let destination = self.convert_ssa_value(*dest, dfg);
                    let source = self.convert_ssa_value(*src, dfg);
                    self.brillig_context.mov_instruction(destination, source);
                }
                self.brillig_context
                    .jump_instruction(self.create_block_label_for_current_function(*destination));
            }
            TerminatorInstruction::Return { return_values } => {
                let return_registers: Vec<_> = return_values
                    .iter()
                    .map(|value_id| self.convert_ssa_value(*value_id, dfg))
                    .collect();
                self.brillig_context.return_instruction(&return_registers);
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
                Type::Numeric(_) | Type::Array(..) | Type::Reference => {
                    self.function_context.get_or_create_register(self.brillig_context, *param_id);
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
                let result_ids = dfg.instruction_results(instruction_id);
                let result_register = self
                    .function_context
                    .get_or_create_register(self.brillig_context, result_ids[0]);
                self.convert_ssa_binary(binary, dfg, result_register);
            }
            Instruction::Constrain(value) => {
                let condition = self.convert_ssa_value(*value, dfg);
                self.brillig_context.constrain_instruction(condition);
            }
            Instruction::Allocate => {
                let value = dfg.instruction_results(instruction_id)[0];
                let address_register =
                    self.function_context.get_or_create_register(self.brillig_context, value);
                self.brillig_context.allocate_instruction(address_register);
            }
            Instruction::Store { address, value } => {
                let address_register = self.convert_ssa_value(*address, dfg);
                let source_register = self.convert_ssa_value(*value, dfg);

                self.brillig_context.store_instruction(address_register, source_register);
            }
            Instruction::Load { address } => {
                let target_register = self.function_context.get_or_create_register(
                    self.brillig_context,
                    dfg.instruction_results(instruction_id)[0],
                );
                let address_register = self.convert_ssa_value(*address, dfg);

                self.brillig_context.load_instruction(target_register, address_register);
            }
            Instruction::Not(value) => {
                let condition = self.convert_ssa_value(*value, dfg);
                let result_ids = dfg.instruction_results(instruction_id);
                let result_register = self
                    .function_context
                    .get_or_create_register(self.brillig_context, result_ids[0]);
                let bit_size = Self::get_bit_size_from_ssa_type(dfg.type_of_value(*value));
                self.brillig_context.not_instruction(condition, bit_size, result_register);
            }
            Instruction::Call { func, arguments } => match &dfg[*func] {
                Value::ForeignFunction(func_name) => {
                    let result_ids = dfg.instruction_results(instruction_id);

                    let input_registers = vecmap(arguments, |value_id| {
                        self.convert_ssa_value_to_register_value_or_array(*value_id, dfg)
                    });
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
                    let function_arguments: Vec<RegisterIndex> =
                        vecmap(arguments, |arg| self.convert_ssa_value(*arg, dfg));
                    let result_ids = dfg.instruction_results(instruction_id);

                    // Create label for the function that will be called
                    let label_of_function_to_call =
                        FunctionContext::function_id_to_function_label(*func_id);

                    let saved_registers =
                        self.brillig_context.pre_call_save_registers_prep_args(&function_arguments);

                    // Call instruction, which will interpret above registers 0..num args
                    self.brillig_context.add_external_call_instruction(label_of_function_to_call);

                    // Important: resolve after pre_call_save_registers_prep_args
                    // This ensures we don't save the results to registers unnecessarily.
                    let result_registers = vecmap(result_ids, |a| {
                        self.function_context.get_or_create_register(self.brillig_context, *a)
                    });
                    assert!(
                        !saved_registers.iter().any(|x| result_registers.contains(x)),
                        "should not save registers used as function results"
                    );
                    self.brillig_context
                        .post_call_prep_returns_load_registers(&result_registers, &saved_registers);
                }
                Value::Intrinsic(Intrinsic::BlackBox(bb_func)) => {
                    let function_arguments = vecmap(arguments, |arg| {
                        self.convert_ssa_value_to_register_value_or_array(*arg, dfg)
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
                _ => {
                    unreachable!("unsupported function call type {:?}", dfg[*func])
                }
            },
            Instruction::Truncate { value, .. } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self
                    .function_context
                    .get_or_create_register(self.brillig_context, result_ids[0]);
                let source = self.convert_ssa_value(*value, dfg);
                self.brillig_context.truncate_instruction(destination, source);
            }
            Instruction::Cast(value, target_type) => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self
                    .function_context
                    .get_or_create_register(self.brillig_context, result_ids[0]);
                let source = self.convert_ssa_value(*value, dfg);
                self.convert_cast(destination, source, target_type, &dfg.type_of_value(*value));
            }
            Instruction::ArrayGet { array, index } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self
                    .function_context
                    .get_or_create_register(self.brillig_context, result_ids[0]);
                let array_register = self.convert_ssa_value(*array, dfg);
                let index_register = self.convert_ssa_value(*index, dfg);
                self.brillig_context.array_get(array_register, index_register, destination);
            }
            // Array set operation in SSA returns a new array that is a copy of the parameter array
            // With a specific value changed.
            Instruction::ArraySet { array, index, value } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self
                    .function_context
                    .get_or_create_register(self.brillig_context, result_ids[0]);

                // First issue a array copy to the destination
                let array_size = compute_size_of_type(&dfg.type_of_value(*array));
                self.brillig_context.allocate_fixed_length_array(destination, array_size);
                let source_array_register: RegisterIndex = self.convert_ssa_value(*array, dfg);
                let size_register = self.brillig_context.make_constant(array_size.into());
                self.brillig_context.copy_array_instruction(
                    source_array_register,
                    destination,
                    size_register,
                );

                // Then set the value in the newly created array
                let index_register = self.convert_ssa_value(*index, dfg);
                let value_register = self.convert_ssa_value(*value, dfg);
                self.brillig_context.array_set(destination, index_register, value_register);
            }
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };
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
                let value_register = self.convert_ssa_value(value_id, dfg);
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
                    self.brillig_context.binary_instruction(
                        iterator_register,
                        size_of_item_register,
                        iterator_register,
                        BrilligBinaryOp::Integer {
                            op: BinaryIntOp::Add,
                            bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
                        },
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

        let left = self.convert_ssa_value(binary.lhs, dfg);
        let right = self.convert_ssa_value(binary.rhs, dfg);

        let brillig_binary_op =
            convert_ssa_binary_op_to_brillig_binary_op(binary.operator, binary_type);

        self.brillig_context.binary_instruction(left, right, result_register, brillig_binary_op);
    }

    /// Converts an SSA `ValueId` into a `RegisterIndex`.
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> RegisterIndex {
        let value = &dfg[value_id];

        let register = match value {
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.
                self.function_context.get_or_create_register(self.brillig_context, value_id)
            }
            Value::NumericConstant { constant, .. } => {
                let register_index =
                    self.function_context.get_or_create_register(self.brillig_context, value_id);

                self.brillig_context.const_instruction(register_index, (*constant).into());
                register_index
            }
            Value::Array { .. } => {
                let address_register = self.brillig_context.allocate_register();
                self.brillig_context.allocate_fixed_length_array(
                    address_register,
                    compute_size_of_type(&dfg.type_of_value(value_id)),
                );
                self.store_in_memory(address_register, value_id, dfg);
                address_register
            }
            _ => {
                todo!("ICE: Should have been in cache {value:?}")
            }
        };
        register
    }

    fn convert_ssa_value_to_register_value_or_array(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let register_index = self.convert_ssa_value(value_id, dfg);
        let typ = dfg[value_id].get_type();
        match typ {
            Type::Numeric(_) => RegisterOrMemory::RegisterIndex(register_index),
            Type::Array(..) => RegisterOrMemory::HeapArray(HeapArray {
                pointer: register_index,
                size: compute_size_of_type(&typ),
            }),
            _ => {
                unreachable!("type not supported for conversion into brillig register")
            }
        }
    }

    fn allocate_external_call_result(
        &mut self,
        result: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let typ = dfg[result].get_type();
        match typ {
            Type::Numeric(_) => RegisterOrMemory::RegisterIndex(
                self.function_context.get_or_create_register(self.brillig_context, result),
            ),
            Type::Array(..) => {
                let pointer =
                    self.function_context.get_or_create_register(self.brillig_context, result);
                let array_size_in_memory = compute_size_of_type(&typ);
                self.brillig_context.allocate_fixed_length_array(pointer, array_size_in_memory);
                RegisterOrMemory::HeapArray(HeapArray { pointer, size: array_size_in_memory })
            }
            Type::Slice(_) => {
                let pointer =
                    self.function_context.get_or_create_register(self.brillig_context, result);
                let array_size_register = self.brillig_context.allocate_register();
                // Set the pointer to the current stack frame
                // The stack pointer will then be update by the caller of this method
                // once the external call is resolved and the array size is known
                self.brillig_context.set_array_pointer(pointer);
                RegisterOrMemory::HeapVector(HeapVector { pointer, size: array_size_register })
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

/// Computes the size of an SSA composite type
fn compute_size_of_composite_type(typ: &CompositeType) -> usize {
    typ.iter().map(compute_size_of_type).sum()
}

/// Finds out the size of a given SSA type
/// This is needed to store values in memory
pub(crate) fn compute_size_of_type(typ: &Type) -> usize {
    match typ {
        Type::Numeric(_) => 1,
        Type::Array(types, item_count) => compute_size_of_composite_type(types) * item_count,
        _ => todo!("ICE: Type not supported {typ:?}"),
    }
}
