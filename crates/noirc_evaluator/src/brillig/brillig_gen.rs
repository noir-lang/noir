use super::brillig_ir::{artifact::BrilligArtifact, BrilligBinaryOp, BrilligContext};
use crate::ssa_refactor::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction},
    post_order::PostOrder,
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::acir::brillig_vm::{BinaryFieldOp, BinaryIntOp, RegisterIndex, RegisterValueOrArray};
use iter_extended::vecmap;
use std::collections::HashMap;

#[derive(Default)]
/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligGen {
    /// Context for creating brillig opcodes
    context: BrilligContext,
    /// Map from SSA values to Register Indices.
    ssa_value_to_register: HashMap<ValueId, RegisterIndex>,
}

impl BrilligGen {
    /// Gets a `RegisterIndex` for a `ValueId`, if one already exists
    /// or creates a new `RegisterIndex` using the latest available
    /// free register.
    fn get_or_create_register(&mut self, value: ValueId) -> RegisterIndex {
        if let Some(register_index) = self.ssa_value_to_register.get(&value) {
            return *register_index;
        }

        let register = self.context.create_register();

        // Cache the `ValueId` so that if we call it again, it will
        // return the register that has just been created.
        //
        // WARNING: This assumes that a register has not been
        // modified. If a MOV instruction has overwritten the value
        // at a register, then this cache will be invalid.
        self.ssa_value_to_register.insert(value, register);

        register
    }

    /// Converts an SSA Basic block into a sequence of Brillig opcodes
    fn convert_block(&mut self, block_id: BasicBlockId, dfg: &DataFlowGraph) {
        // Add a label for this block
        self.context.add_label_to_next_opcode(block_id);

        // Convert the block parameters
        let block = &dfg[block_id];
        self.convert_block_params(block, dfg);

        // Convert all of the instructions int the block
        for instruction_id in block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg);
        }

        // Process the block's terminator instruction
        let terminator_instruction =
            block.terminator().expect("block is expected to be constructed");
        self.convert_ssa_terminator(terminator_instruction, dfg);
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
                self.context.jump_if_instruction(condition, then_destination);
                self.context.jump_instruction(else_destination);
            }
            TerminatorInstruction::Jmp { destination, arguments } => {
                let target = &dfg[*destination];
                for (src, dest) in arguments.iter().zip(target.parameters()) {
                    let destination = self.convert_ssa_value(*dest, dfg);
                    let source = self.convert_ssa_value(*src, dfg);
                    self.context.mov_instruction(destination, source);
                }
                self.context.jump_instruction(destination);
            }
            TerminatorInstruction::Return { return_values } => {
                let return_registers: Vec<_> = return_values
                    .iter()
                    .map(|value_id| self.convert_ssa_value(*value_id, dfg))
                    .collect();
                self.context.return_instruction(&return_registers);
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
                Type::Numeric(_) => {
                    self.get_or_create_register(*param_id);
                }
                Type::Array(_, size) => {
                    let pointer_register = self.get_or_create_register(*param_id);
                    self.context.allocate_array(pointer_register, *size as u32, true);
                }
                _ => {
                    todo!("ICE: Param type not supported {param_type:?}")
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
                let result_register = self.get_or_create_register(result_ids[0]);
                self.convert_ssa_binary(binary, dfg, result_register);
            }
            Instruction::Constrain(value) => {
                let condition = self.convert_ssa_value(*value, dfg);
                self.context.constrain_instruction(condition);
            }
            Instruction::Allocate => {
                let pointer_register =
                    self.get_or_create_register(dfg.instruction_results(instruction_id)[0]);
                self.context.allocate_array(pointer_register, 1, false);
            }
            Instruction::Store { address, value } => {
                let address_register = self.convert_ssa_value(*address, dfg);
                let value_register = self.convert_ssa_value(*value, dfg);
                self.context.store_instruction(address_register, value_register);
            }
            Instruction::Load { address } => {
                let target_register =
                    self.get_or_create_register(dfg.instruction_results(instruction_id)[0]);
                let address_register = self.convert_ssa_value(*address, dfg);
                self.context.load_instruction(target_register, address_register);
            }
            Instruction::Not(value) => {
                assert_eq!(
                    dfg.type_of_value(*value),
                    Type::bool(),
                    "not operator can only be applied to boolean values"
                );
                let condition = self.convert_ssa_value(*value, dfg);
                let result_ids = dfg.instruction_results(instruction_id);
                let result_register = self.get_or_create_register(result_ids[0]);

                self.context.not_instruction(condition, result_register);
            }
            Instruction::Call { func, arguments } => match &dfg[*func] {
                Value::ForeignFunction(func_name) => {
                    let result_ids = dfg.instruction_results(instruction_id);

                    let input_registers = vecmap(arguments, |value_id| {
                        self.convert_ssa_value_to_register_value_or_array(*value_id, dfg)
                    });
                    let output_registers = vecmap(result_ids, |value_id| {
                        self.convert_ssa_value_to_register_value_or_array(*value_id, dfg)
                    });

                    self.context.foreign_call_instruction(
                        func_name.to_owned(),
                        &input_registers,
                        &output_registers,
                    );
                }
                _ => {
                    unreachable!("only foreign function calls supported in unconstrained functions")
                }
            },
            Instruction::Truncate { value, .. } => {
                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self.get_or_create_register(result_ids[0]);
                let source = self.convert_ssa_value(*value, dfg);
                self.context.truncate_instruction(destination, source);
            }
            Instruction::ArrayGet { array, index } => {
                let array_ptr = self.convert_ssa_value(*array, dfg);
                let index = self.convert_ssa_value(*index, dfg);

                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self.get_or_create_register(result_ids[0]);

                self.context.array_get(array_ptr, index, destination);
            }
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };
    }

    /// Converts the Binary instruction into a sequence of Brillig opcodes.
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
        // TODO: For array addition, perhaps this will need
        // TODO to be multiple registers
        result_register: RegisterIndex,
    ) {
        let lhs_type = dfg[binary.lhs].get_type();
        let rhs_type = dfg[binary.rhs].get_type();

        let left = self.convert_ssa_value(binary.lhs, dfg);
        let right = self.convert_ssa_value(binary.rhs, dfg);

        let binary_type = type_of_binary_operation(lhs_type, rhs_type);
        // For operations involving arrays, we handle as a special case
        if let Type::Array(element_type, num_elements) = binary_type {
            return self.convert_ssa_binary_array(
                &element_type,
                left,
                right,
                num_elements as u32,
                binary.operator,
                result_register,
            );
        }

        let brillig_binary_op =
            convert_ssa_binary_op_to_brillig_binary_op(binary.operator, binary_type);

        self.context.binary_instruction(left, right, result_register, brillig_binary_op);
    }

    /// Handles binary operations that involve arrays as the operands.
    fn convert_ssa_binary_array(
        &mut self,
        element_type: &[Type],
        lhs_array_ptr: RegisterIndex,
        rhs_array_ptr: RegisterIndex,
        num_elements: u32,
        operator: BinaryOp,
        // TODO: For array addition, perhaps this will need
        // TODO to be multiple registers
        result_register: RegisterIndex,
    ) {
        // For now, we only support arrays of single element types
        assert_eq!(
            element_type.len(),
            1,
            "array has a composite type as its element, which we do not support yet"
        );
        let atomic_type = element_type[0].clone();

        // For now we set the maximum size of an array to be 2^64
        // This is of course arbitrary and can be changed.
        //
        // This just means that when we create an index to iterate over the array,
        // we will be interpreting the index as a 64 bit integer.
        const MAX_BIT_SIZE_ARRAY: u32 = 64;

        // Label to denote the start of the loop which we will use to
        // store the bytecode being generated for the for loop
        const START_LOOP_LABEL: &str = "start_loop";
        // Label to denote code that will be executed after the loop
        const EXIT_LOOP_LABEL: &str = "exit_loop";

        match operator {
            BinaryOp::Eq => {
                // Set the result to true
                self.context.const_instruction(result_register, 1u128.into());

                // Set index to 0 to start
                let index = self.context.create_register();
                self.context.const_instruction(index, 0u128.into());

                // Load number of elements into a register
                let num_elements_register = self.context.create_register();
                self.context
                    .const_instruction(num_elements_register, (num_elements as u128).into());

                // Create a register to store the current array elements
                let lhs_array_element = self.context.create_register();
                let rhs_array_element = self.context.create_register();

                // Create a label for the start of the loop
                self.context.add_label_to_next_opcode(START_LOOP_LABEL);

                // Check if index < num_elements
                //
                // If it is not, then we jump to the exit label
                let index_less_than_array_len = self.context.create_register();
                self.context.binary_instruction(
                    index,
                    num_elements_register,
                    index_less_than_array_len,
                    BrilligBinaryOp::Integer {
                        op: BinaryIntOp::LessThan,
                        bit_size: MAX_BIT_SIZE_ARRAY,
                    },
                );
                self.context.not_instruction(index_less_than_array_len, index_less_than_array_len);
                self.context.jump_if_instruction(index_less_than_array_len, EXIT_LOOP_LABEL);

                // Load the current array element into the register
                self.context.array_get(lhs_array_ptr, index, lhs_array_element);
                self.context.array_get(rhs_array_ptr, index, rhs_array_element);

                // Increment the index register
                let one = self.context.make_constant(1u128.into());
                self.context.binary_instruction(
                    index,
                    one,
                    index,
                    BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: MAX_BIT_SIZE_ARRAY },
                );

                // Check if the elements loaded are equal and store them in the result_register
                //
                // This code is actually more general and checks an arbitrary binary operation
                let binary_op = convert_ssa_binary_op_to_brillig_binary_op(operator, atomic_type);
                self.context.binary_instruction(
                    lhs_array_element,
                    rhs_array_element,
                    result_register,
                    binary_op,
                );

                // Jump back to start of the loop, if the elements are equal
                self.context.jump_if_instruction(result_register, START_LOOP_LABEL);

                // Add the exit_loop label to mark the end of the loop
                self.context.add_label_to_next_opcode(EXIT_LOOP_LABEL);
            }
            _ => unimplemented!("binary operation {operator} is not implemented for arrays"),
        }
    }

    /// Converts an SSA `ValueId` into a `RegisterIndex`.
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> RegisterIndex {
        let value = &dfg[value_id];
        let register = match value {
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.
                self.get_or_create_register(value_id)
            }
            Value::NumericConstant { constant, .. } => {
                let register_index = self.get_or_create_register(value_id);

                self.context.const_instruction(register_index, (*constant).into());
                register_index
            }
            _ => {
                todo!("ICE: Should have been in cache {value:?}")
            }
        };
        register
    }

    /// Compiles an SSA function into a Brillig artifact which
    /// contains a sequence of SSA opcodes.
    pub(crate) fn compile(func: &Function) -> BrilligArtifact {
        let mut brillig = BrilligGen::default();

        brillig.convert_ssa_function(func);

        brillig.context.artifact()
    }

    /// Converting an SSA function into Brillig bytecode.
    ///
    /// TODO: Change this to use `dfg.basic_blocks_iter` which will return an
    /// TODO iterator of all of the basic blocks.
    /// TODO(Jake): what order is this ^
    fn convert_ssa_function(&mut self, func: &Function) {
        let mut reverse_post_order = Vec::new();
        reverse_post_order.extend_from_slice(PostOrder::with_function(func).as_slice());
        reverse_post_order.reverse();

        for block in reverse_post_order {
            self.convert_block(block, &func.dfg);
        }
    }

    fn convert_ssa_value_to_register_value_or_array(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterValueOrArray {
        let register_index = self.convert_ssa_value(value_id, dfg);
        let typ = dfg[value_id].get_type();
        match typ {
            Type::Numeric(_) => RegisterValueOrArray::RegisterIndex(register_index),
            Type::Array(_, size) => RegisterValueOrArray::HeapArray(register_index, size),
            Type::Unit => RegisterValueOrArray::RegisterIndex(register_index),
            _ => {
                unreachable!("type not supported for conversion into brillig register")
            }
        }
    }
}

/// Returns the type of the operation considering the types of the operands
/// TODO: SSA issues binary operations between fields and integers.
/// This probably should be explicitly casted in SSA to avoid having to coerce at this level.
pub(crate) fn type_of_binary_operation(lhs_type: Type, rhs_type: Type) -> Type {
    match (&lhs_type, &rhs_type) {
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
        // If both sides are arrays, then we also expect their types to be the same.
        (Type::Array(_, _), Type::Array(_, _)) => {
            assert_eq!(
                lhs_type, rhs_type,
                "lhs and rhs types in a binary operation are always the same"
            );
            lhs_type
        }
        (lhs_type, rhs_type) => {
            unreachable!(
                "ICE: Binary operation between types {:?} and {:?} is not allowed",
                lhs_type, rhs_type
            )
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
