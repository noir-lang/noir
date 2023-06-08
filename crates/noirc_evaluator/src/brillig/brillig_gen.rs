use super::artifact::BrilligArtifact;
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
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, Value as BrilligValue,
        RegisterValueOrArray,
    },
    FieldElement,
};
use iter_extended::vecmap;
use std::collections::HashMap;

#[derive(Default)]
/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligGen {
    obj: BrilligArtifact,
    /// A usize indicating the latest un-used register.
    latest_register: usize,
    /// Map from SSA values to Register Indices.
    ssa_value_to_register: HashMap<ValueId, RegisterIndex>,
}

impl BrilligGen {
    /// Adds a brillig instruction to the brillig byte code
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    /// Gets a `RegisterIndex` for a `ValueId`, if one already exists
    /// or creates a new `RegisterIndex` using the latest available
    /// free register.
    fn get_or_create_register(&mut self, value: ValueId) -> RegisterIndex {
        if let Some(register_index) = self.ssa_value_to_register.get(&value) {
            return *register_index;
        }

        let register = self.create_register();

        // Cache the `ValueId` so that if we call it again, it will
        // return the register that has just been created.
        //
        // WARNING: This assumes that a register has not been
        // modified. If a MOV instruction has overwritten the value
        // at a register, then this cache will be invalid.
        self.ssa_value_to_register.insert(value, register);

        register
    }

    /// Creates a new register.
    fn create_register(&mut self) -> RegisterIndex {
        let register = RegisterIndex::from(self.latest_register);
        self.latest_register += 1;
        register
    }

    /// Converts an SSA Basic block into a sequence of Brillig opcodes
    fn convert_block(&mut self, block_id: BasicBlockId, dfg: &DataFlowGraph) {
        self.obj.add_block_label(block_id);
        let block = &dfg[dbg!(block_id)];
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
        self.obj.add_unresolved_jump(target);
        self.push_code(BrilligOpcode::Jump { location: 0 });
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    fn jump_if(&mut self, condition: RegisterIndex, target: BasicBlockId) {
        self.obj.add_unresolved_jump(target);
        self.push_code(BrilligOpcode::JumpIf { condition, location: 0 });
    }

    /// Converts the SSA return instruction into the necessary BRillig return
    /// opcode.
    ///
    /// For Brillig, the return is implicit; The caller will take `N` values from
    /// the Register starting at register index 0. `N` indicates the number of
    /// return values expected.
    fn convert_ssa_return(&mut self, return_values: &[ValueId], dfg: &DataFlowGraph) {
        // Check if the program returns the `Unit/None` type.
        // This type signifies that the program returns nothing.
        let is_return_unit_type =
            return_values.len() == 1 && dfg.type_of_value(return_values[0]) == Type::Unit;
        if is_return_unit_type {
            return;
        }

        for (destination_index, value_id) in return_values.iter().enumerate() {
            let return_register = self.convert_ssa_value(*value_id, dfg);
            if destination_index > self.latest_register {
                self.latest_register = destination_index;
            }
            self.push_code(BrilligOpcode::Mov {
                destination: destination_index.into(),
                source: return_register,
            });
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
            Instruction::Not(value) => {
                let result_ids = dfg.instruction_results(instruction_id);
                let result_register = self.get_or_create_register(result_ids[0]);

                assert_eq!(
                    dfg.type_of_value(*value),
                    Type::bool(),
                    "not operator can only be applied to boolean values"
                );

                let one = self.make_constant(FieldElement::one());
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

                let input_registers = vecmap(arguments,|value_id| self.convert_ssa_value(*value_id, dfg));
                let output_registers = vecmap(result_ids,|value_id| self.convert_ssa_value(*value_id, dfg));

                let opcode = BrilligOpcode::ForeignCall { 
                    function: func.to_owned(), 
                    destination: RegisterValueOrArray::RegisterIndex(output_registers[0]), 
                    input: RegisterValueOrArray::RegisterIndex(input_registers[0]),
                };
                self.push_code(opcode);
            }
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };
    }

    /// Returns a register which holds the value of a constant
    fn make_constant(&mut self, constant: FieldElement) -> RegisterIndex {
        let register = self.create_register();

        let const_opcode =
            BrilligOpcode::Const { destination: register, value: BrilligValue::from(constant) };
        self.push_code(const_opcode);

        register
    }

    /// Converts the Binary instruction into a sequence of Brillig opcodes.
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
        result_register: RegisterIndex,
    ) {
        let left_type = dfg[binary.lhs].get_type();
        let right_type = dfg[binary.rhs].get_type();
        if left_type != right_type {
            todo!("ICE: Binary operands must have the same type")
        }

        let left = self.convert_ssa_value(binary.lhs, dfg);
        let right = self.convert_ssa_value(binary.rhs, dfg);

        let brillig_binary_op =
            BrilligBinaryOp::convert_ssa_binary_op_to_brillig_binary_op(binary.operator, left_type);
        match brillig_binary_op {
            BrilligBinaryOp::Field { op } => {
                let opcode = BrilligOpcode::BinaryFieldOp {
                    op,
                    destination: result_register,
                    lhs: left,
                    rhs: right,
                };
                self.push_code(opcode);
            }
            BrilligBinaryOp::Integer { op, bit_size } => {
                let opcode = BrilligOpcode::BinaryIntOp {
                    op,
                    destination: result_register,
                    bit_size,
                    lhs: left,
                    rhs: right,
                };
                self.push_code(opcode);
            }
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
                self.push_code(BrilligOpcode::Const {
                    destination: register_index,
                    value: BrilligValue::from(*constant),
                });
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

        brillig.push_code(BrilligOpcode::Stop);
        dbg!(brillig.obj.byte_code.clone());
        brillig.obj
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

/// Type to encapsulate the binary operation types in Brillig
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
}

impl BrilligBinaryOp {
    /// Convert an SSA binary operation into:
    /// - Brillig Binary Integer Op, if it is a integer type
    /// - Brillig Binary Field Op, if it is a field type
    fn convert_ssa_binary_op_to_brillig_binary_op(ssa_op: BinaryOp, typ: Type) -> BrilligBinaryOp {
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

        fn binary_op_to_field_op(op: BinaryOp) -> BinaryFieldOp {
            match op {
                BinaryOp::Add => BinaryFieldOp::Add,
                BinaryOp::Sub => BinaryFieldOp::Sub,
                BinaryOp::Mul => BinaryFieldOp::Mul,
                BinaryOp::Div => BinaryFieldOp::Div,
                BinaryOp::Eq => BinaryFieldOp::Equals,
                _ => unreachable!(
                "Field type cannot be used with {op}. This should have been caught by the frontend"
            ),
            }
        }
        fn binary_op_to_int_op(op: BinaryOp, is_signed: bool) -> BinaryIntOp {
            match op {
                BinaryOp::Add => BinaryIntOp::Add,
                BinaryOp::Sub => BinaryIntOp::Sub,
                BinaryOp::Mul => BinaryIntOp::Mul,
                BinaryOp::Div => {
                    if is_signed {
                        BinaryIntOp::SignedDiv
                    } else {
                        BinaryIntOp::UnsignedDiv
                    }
                },
                BinaryOp::Mod => todo!("This is not supported by Brillig. It should either be added into Brillig or legalized by the SSA IR"),
                BinaryOp::Eq => BinaryIntOp::Equals,
                BinaryOp::Lt => BinaryIntOp::LessThan,
                BinaryOp::And => BinaryIntOp::And,
                BinaryOp::Or => BinaryIntOp::Or,
                BinaryOp::Xor => BinaryIntOp::Xor,
                BinaryOp::Shl => BinaryIntOp::Shl,
                BinaryOp::Shr => BinaryIntOp::Shr,
            }
        }
        // If bit size is available then it is a binary integer operation
        match bit_size_signedness {
            Some((bit_size, is_signed)) => {
                let binary_int_op = binary_op_to_int_op(ssa_op, is_signed);
                BrilligBinaryOp::Integer { op: binary_int_op, bit_size }
            }
            None => {
                let binary_field_op = binary_op_to_field_op(ssa_op);
                BrilligBinaryOp::Field { op: binary_field_op }
            }
        }
    }
}
