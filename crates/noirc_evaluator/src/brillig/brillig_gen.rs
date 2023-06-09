use super::{
    artifact::BrilligArtifact,
    binary::{type_of_binary_operation, BrilligBinaryOp},
    memory::BrilligMemory,
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
        BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, Value as BrilligValue,
    },
    FieldElement,
};
use std::collections::HashMap;

#[derive(Default)]
/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligGen {
    obj: BrilligArtifact,
    /// A usize indicating the latest un-used register.
    latest_register: usize,
    /// Map from SSA values to Register Indices.
    ssa_value_to_register: HashMap<ValueId, RegisterIndex>,
    /// Tracks memory allocations
    memory: BrilligMemory,
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
        self.push_code(BrilligOpcode::Stop);
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
            _ => todo!("ICE: Instruction not supported {instruction:?}"),
        };
    }

    fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let array_pointer = self.memory.allocate(size as usize);
        self.push_code(BrilligOpcode::Const {
            destination: pointer_register,
            value: BrilligValue::from(array_pointer),
        });
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
        let binary_type =
            type_of_binary_operation(dfg[binary.lhs].get_type(), dfg[binary.rhs].get_type());

        let left = self.convert_ssa_value(binary.lhs, dfg);
        let right = self.convert_ssa_value(binary.rhs, dfg);

        // Process modulo operator separately as there is no
        // Brillig modulo operator and the result is multiple
        // brillig opcodes.
        if let BinaryOp::Mod = binary.operator {
            match binary_type {
                Type::Numeric(NumericType::Unsigned { bit_size }) => {
                    self.convert_integer_mod(result_register, left, right, bit_size, false);
                    return;
                }
                Type::Numeric(NumericType::Signed { bit_size }) => {
                    self.convert_integer_mod(result_register, left, right, bit_size, true);
                    return;
                }
                _ => unimplemented!("ICE: Modulo operation not supported for type {binary_type:?}"),
            }
        }

        let brillig_binary_op = BrilligBinaryOp::convert_ssa_binary_op_to_brillig_binary_op(
            binary.operator,
            binary_type,
        );
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

    /// Computes left % right by emitting the necessary Brillig opcodes.
    ///
    /// This is done by using the following formula:
    ///
    /// a % b = a - (b * (a / b))
    fn convert_integer_mod(
        &mut self,
        result_register: RegisterIndex,
        left: RegisterIndex,
        right: RegisterIndex,
        bit_size: u32,
        signed: bool,
    ) {
        let scratch_register_i = self.create_register();
        let scratch_register_j = self.create_register();

        // i = left / right
        self.push_code(BrilligOpcode::BinaryIntOp {
            op: match signed {
                true => BinaryIntOp::SignedDiv,
                false => BinaryIntOp::UnsignedDiv,
            },
            destination: scratch_register_i,
            bit_size,
            lhs: left,
            rhs: right,
        });

        // j = i * right
        self.push_code(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Mul,
            destination: scratch_register_j,
            bit_size,
            lhs: scratch_register_i,
            rhs: right,
        });

        // result_register = left - j
        self.push_code(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Sub,
            destination: result_register,
            bit_size,
            lhs: left,
            rhs: scratch_register_j,
        });
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
