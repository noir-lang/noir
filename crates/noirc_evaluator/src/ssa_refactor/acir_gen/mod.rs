//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use super::{
    ir::{
        dfg::DataFlowGraph,
        instruction::{Binary, BinaryOp},
        map::Id,
        value::Value,
    },
    ssa_gen::Ssa,
};
use crate::ssa_refactor::ir::instruction::Instruction;

use acvm::acir::circuit::opcodes::Opcode as AcirOpcode;
use acvm::acir::native_types::Expression;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {}

/// The output of the Acir-gen pass
pub struct Acir {
    opcodes: Vec<AcirOpcode>,
}

impl Ssa {
    pub(crate) fn into_acir(self) -> Acir {
        let mut context = Context::new();
        context.convert_ssa(self)
    }
}

impl Context {
    fn new() -> Self {
        Self {}
    }

    fn convert_ssa(&mut self, ssa: Ssa) -> Acir {
        // When converting SSA to ACIR, we expect the legalization pass in the SSA module
        // to ensure the following:
        // - All functions will be inlined
        // - All basic blocks will be inlined
        //
        // When generating ACIR, we therefore only need to look at the entry block's
        // instructions.
        let entry_func = ssa.functions.first().expect("expected at least one function");
        let entry_block_id = entry_func.entry_block();
        let dfg = &entry_func.dfg;
        let entry_block = &dfg[entry_block_id];

        // Instruction Ids for all instructions in the entry block
        let instruction_ids = entry_block.instructions();
        for ins_id in instruction_ids {
            let ins = &entry_func.dfg[*ins_id];
            match ins {
                Instruction::Binary(binary) => self.convert_ssa_binary(binary, dfg),
                Instruction::Cast(_, _) => todo!(),
                Instruction::Not(_) => todo!(),
                Instruction::Truncate { value, bit_size, max_bit_size } => todo!(),
                Instruction::Constrain(_) => todo!(),
                Instruction::Call { func, arguments } => {
                    todo!()
                }
                Instruction::Allocate { size } => todo!(),
                Instruction::Load { address } => todo!(),
                Instruction::Store { address, value } => {
                    todo!()
                }
            }
        }
        todo!()
    }

    fn convert_ssa_binary(&self, binary: &Binary, dfg: &DataFlowGraph) {
        let lhs = self.convert_ssa_value(&binary.lhs, &dfg);
        let rhs = self.convert_ssa_value(&binary.rhs, &dfg);
        match binary.operator {
            BinaryOp::Add => {}
            BinaryOp::Sub => todo!(),
            BinaryOp::Mul => todo!(),
            BinaryOp::Div => todo!(),
            BinaryOp::Mod => todo!(),
            BinaryOp::Eq => todo!(),
            BinaryOp::Lt => todo!(),
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::Xor => todo!(),
            BinaryOp::Shl => todo!(),
            BinaryOp::Shr => todo!(),
        }
    }

    fn convert_ssa_value(&self, value_id: &Id<Value>, dfg: &DataFlowGraph) -> Expression {
        match dfg[*value_id] {
            Value::Instruction { instruction, position, typ } => todo!(),
            Value::Param { block, position, typ } => todo!(),
            Value::NumericConstant { constant, typ } => todo!(),
            Value::Function(_) => todo!(),
            Value::Intrinsic(_) => todo!(),
        };
        todo!()
    }
}
