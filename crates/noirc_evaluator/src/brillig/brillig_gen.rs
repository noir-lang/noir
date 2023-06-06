use std::collections::HashMap;

use crate::ssa_refactor::ir::{
    basic_block::BasicBlock,
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction},
    map::Id,
    types::{NumericType, Type},
    value::Value,
};

use super::artifact::BrilligArtifact;

use acvm::acir::brillig_vm::{
    BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, Value as BrilligValue,
};
#[derive(Default)]
/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligGen {
    obj: BrilligArtifact,
    latest_register: usize,
    ssa_value_to_register: HashMap<Id<Value>, RegisterIndex>,
}

impl BrilligGen {
    /// Adds a brillig instruction to the brillig code base
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    fn get_or_create_register(&mut self, value: Id<Value>) -> RegisterIndex {
        match self.ssa_value_to_register.get(&value) {
            Some(register) => *register,
            None => {
                let register = self.latest_register;
                self.latest_register += 1;
                let register = RegisterIndex::from(register);
                self.ssa_value_to_register.insert(value, register);
                register
            }
        }
    }

    fn convert_block(&mut self, block: &BasicBlock, dfg: &DataFlowGraph) {
        self.convert_block_params(block, dfg);

        for instruction_id in block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg);
        }

        self.convert_ssa_return(block, dfg);
    }

    fn convert_ssa_return(&mut self, block: &BasicBlock, dfg: &DataFlowGraph) {
        let return_values = match block.terminator().unwrap() {
            TerminatorInstruction::Return { return_values } => return_values,
            _ => todo!("ICE: Unsupported return"),
        };

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

    fn convert_ssa_instruction(&mut self, instruction_id: InstructionId, dfg: &DataFlowGraph) {
        let instruction = &dfg[instruction_id];

        match instruction {
            Instruction::Binary(binary) => {
                let result_ids = dfg.instruction_results(instruction_id);
                let result_register = self.get_or_create_register(result_ids[0]);
                self.convert_ssa_binary(binary, dfg, result_register);
            }
            _ => todo!("ICE: Instruction not supported"),
        };
    }

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

        let opcode = match left_type {
            Type::Numeric(numeric_type) => match numeric_type {
                NumericType::NativeField => {
                    let op = match binary.operator {
                        BinaryOp::Add => BinaryFieldOp::Add,
                        BinaryOp::Sub => BinaryFieldOp::Sub,
                        BinaryOp::Mul => BinaryFieldOp::Mul,
                        BinaryOp::Div => BinaryFieldOp::Div,
                        BinaryOp::Eq => BinaryFieldOp::Equals,
                        _ => todo!("ICE: Binary operator not supported for field type"),
                    };
                    BrilligOpcode::BinaryFieldOp {
                        op,
                        destination: result_register,
                        lhs: left,
                        rhs: right,
                    }
                }
                NumericType::Signed { bit_size } | NumericType::Unsigned { bit_size } => {
                    let op = match binary.operator {
                        BinaryOp::Add => BinaryIntOp::Add,
                        BinaryOp::Sub => BinaryIntOp::Sub,
                        BinaryOp::Mul => BinaryIntOp::Mul,
                        BinaryOp::Div => match numeric_type {
                            NumericType::Signed { .. } => BinaryIntOp::SignedDiv,
                            NumericType::Unsigned { .. } => BinaryIntOp::UnsignedDiv,
                            _ => unreachable!("ICE: Binary type not supported"),
                        },
                        BinaryOp::Eq => BinaryIntOp::Equals,
                        BinaryOp::Lt => BinaryIntOp::LessThan,
                        BinaryOp::Shl => BinaryIntOp::Shl,
                        BinaryOp::Shr => BinaryIntOp::Shr,
                        BinaryOp::Xor => BinaryIntOp::Xor,
                        BinaryOp::Or => BinaryIntOp::Or,
                        BinaryOp::And => BinaryIntOp::And,
                        _ => todo!(),
                    };
                    BrilligOpcode::BinaryIntOp {
                        op,
                        destination: result_register,
                        bit_size,
                        lhs: left,
                        rhs: right,
                    }
                }
            },
            _ => {
                todo!("ICE: Binary type not supported")
            }
        };

        self.push_code(opcode);
    }

    fn convert_ssa_value(&mut self, value_id: Id<Value>, dfg: &DataFlowGraph) -> RegisterIndex {
        let value = &dfg[value_id];

        let register = match value {
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.
                self.ssa_value_to_register[&value_id]
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

    pub(crate) fn compile(func: &Function) -> BrilligArtifact {
        let mut brillig = BrilligGen::default();

        let dfg = &func.dfg;

        brillig.convert_block(&dfg[func.entry_block()], dfg);

        brillig.push_code(BrilligOpcode::Stop);

        brillig.obj
    }
}
