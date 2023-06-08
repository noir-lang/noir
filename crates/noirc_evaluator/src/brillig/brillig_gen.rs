use super::{
    artifact::BrilligArtifact,
    binary::{type_of_binary_operation, BrilligBinaryOp},
    memory::BrilligMemory,
};
use crate::ssa_refactor::ir::{
    basic_block::{BasicBlock, BasicBlockId},
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, Instruction, InstructionId, TerminatorInstruction},
    post_order::PostOrder,
    types::Type,
    value::{Value, ValueId},
};
use acvm::acir::brillig_vm::{Opcode as BrilligOpcode, RegisterIndex, Value as BrilligValue};
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

        let register = RegisterIndex::from(self.latest_register);
        self.ssa_value_to_register.insert(value, register);

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
            Instruction::Allocate { size } => {
                let pointer_register =
                    self.get_or_create_register(dfg.instruction_results(instruction_id)[0]);
                self.allocate_array(pointer_register, *size);
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
            _ => todo!("ICE: Instruction not supported"),
        };
    }

    fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let array_pointer = self.memory.allocate(size as usize);
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
