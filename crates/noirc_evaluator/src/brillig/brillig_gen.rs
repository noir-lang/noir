use super::brillig_ir::{
    artifact::{BrilligArtifact, UnresolvedJumpLocation},
    BrilligBinaryOp, BrilligContext,
};
use crate::ssa_refactor::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction},
    post_order::PostOrder,
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, RegisterValueOrArray,
        Value as BrilligValue,
    },
    FieldElement,
};
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
    #[deprecated(note = " this module will not longer push to the bytecode")]
    /// Adds a brillig instruction to the brillig byte code
    fn push_code(&mut self, code: BrilligOpcode) {
        self.context.push_opcode(code);
    }

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
        self.context.add_block_label(block_id);
        let block = &dfg[block_id];
        self.convert_block_params(block, dfg);

        for instruction_id in block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg);
        }

        // Jump to the next block
        let jump = block.terminator().expect("block is expected to be constructed");
        match jump {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                let condition = self.convert_ssa_value(*condition, dfg);
                self.jump_if(condition, *then_destination);
                self.jump(*else_destination);
            }
            TerminatorInstruction::Jmp { destination, arguments } => {
                let target = &dfg[*destination];
                for (src, dest) in arguments.iter().zip(target.parameters()) {
                    let destination = self.convert_ssa_value(*dest, dfg);
                    let source = self.convert_ssa_value(*src, dfg);
                    self.push_code(BrilligOpcode::Mov { destination, source });
                }
                self.jump(*destination);
            }
            TerminatorInstruction::Return { return_values } => {
                self.convert_ssa_return(return_values, dfg);
            }
        }
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    fn jump(&mut self, target: BasicBlockId) {
        self.context.add_unresolved_jump(UnresolvedJumpLocation::Block(target));
        self.push_code(BrilligOpcode::Jump { location: 0 });
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    fn jump_if(&mut self, condition: RegisterIndex, target: BasicBlockId) {
        self.context.add_unresolved_jump(UnresolvedJumpLocation::Block(target));
        self.push_code(BrilligOpcode::JumpIf { condition, location: 0 });
    }

    /// Converts the SSA return instruction into the necessary Brillig return
    /// opcode.
    fn convert_ssa_return(&mut self, return_values: &[ValueId], dfg: &DataFlowGraph) {
        let return_registers: Vec<_> =
            return_values.iter().map(|value_id| self.convert_ssa_value(*value_id, dfg)).collect();
        self.context.return_instruction(&return_registers);
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
                let result_register = self.get_or_create_register(result_ids[0]);
                self.convert_ssa_binary(binary, dfg, result_register);
            }
            Instruction::Constrain(value) => {
                let condition = self.convert_ssa_value(*value, dfg);
                // jump to the relative location after the trap
                self.context.add_unresolved_jump(UnresolvedJumpLocation::Relative(2));
                self.push_code(BrilligOpcode::JumpIf { condition, location: 0 });
                self.push_code(BrilligOpcode::Trap);
            }
            Instruction::Allocate => {
                let pointer_register =
                    self.get_or_create_register(dfg.instruction_results(instruction_id)[0]);
                self.allocate_array(pointer_register, 1);
            }
            Instruction::Store { address, value } => {
                let address_register = self.convert_ssa_value(*address, dfg);
                let value_register = self.convert_ssa_value(*value, dfg);
                self.push_code(BrilligOpcode::Store {
                    destination_pointer: address_register,
                    source: value_register,
                });
            }
            Instruction::Load { address } => {
                let target_register =
                    self.get_or_create_register(dfg.instruction_results(instruction_id)[0]);
                let address_register = self.convert_ssa_value(*address, dfg);
                self.push_code(BrilligOpcode::Load {
                    destination: target_register,
                    source_pointer: address_register,
                });
            }
            Instruction::Not(value) => {
                let result_ids = dfg.instruction_results(instruction_id);
                let result_register = self.get_or_create_register(result_ids[0]);

                assert_eq!(
                    dfg.type_of_value(*value),
                    Type::bool(),
                    "not operator can only be applied to boolean values"
                );

                let one = self.context.make_constant(BrilligValue::from(FieldElement::one()));
                let condition = self.convert_ssa_value(*value, dfg);

                // Compile !x as (1 - x)
                let opcode = BrilligOpcode::BinaryIntOp {
                    destination: result_register,
                    op: BinaryIntOp::Sub,
                    bit_size: 1,
                    lhs: one,
                    rhs: condition,
                };
                self.push_code(opcode);
            }
            Instruction::ForeignCall { func, arguments } => {
                let result_ids = dfg.instruction_results(instruction_id);

                let input_registers =
                    vecmap(arguments, |value_id| self.convert_ssa_value(*value_id, dfg));
                let output_registers =
                    vecmap(result_ids, |value_id| self.convert_ssa_value(*value_id, dfg));

                let opcode = BrilligOpcode::ForeignCall {
                    function: func.to_owned(),
                    destination: RegisterValueOrArray::RegisterIndex(output_registers[0]),
                    input: RegisterValueOrArray::RegisterIndex(input_registers[0]),
                };
                self.push_code(opcode);
            }
            Instruction::Truncate { value, .. } => {
                // Effectively a no-op because brillig already has implicit truncation on integer
                // operations. We need only copy the value to it's destination.
                let result_ids = dfg.instruction_results(instruction_id);
                let destination = self.get_or_create_register(result_ids[0]);
                let source = self.convert_ssa_value(*value, dfg);
                self.push_code(BrilligOpcode::Mov { destination, source });
            }
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };
    }

    fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let array_pointer = self.context.allocate_memory(size as usize);
        self.push_code(BrilligOpcode::Const {
            destination: pointer_register,
            value: BrilligValue::from(array_pointer),
        });
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

        self.context.binary_instruction(left, right, result_register, brillig_binary_op);
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

                self.context.const_instruction(register_index, BrilligValue::from(*constant));
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

        // TODO: shouldn't this be added when we process a return instruction
        brillig.push_code(BrilligOpcode::Stop);
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
}

/// Returns the type of the operation considering the types of the operands
/// TODO: SSA issues binary operations between fields and integers.
/// This probably should be explicitly casted in SSA to avoid having to coerce at this level.
pub(crate) fn type_of_binary_operation(lhs_type: Type, rhs_type: Type) -> Type {
    match (lhs_type, rhs_type) {
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
