//! This file holds the pass to convert from Noir's SSA IR to ACIR.
use crate::ssa_refactor::ir::instruction::Instruction;

use super::ssa_gen::Ssa;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context {}

/// The output of the Acir-gen pass
pub struct Acir {}

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

    fn convert_ssa(&mut self, _ssa: Ssa) -> Acir {
        // When converting SSA to ACIR, we expect the legalization pass in the SSA module
        // to ensure the following:
        // - All functions will be inlined
        // - All basic blocks will be inlined
        //
        // When generating ACIR, we therefore only need to look at the entry block's
        // instructions.
        let entry_func = _ssa.functions.first().expect("expected at least one function");
        let entry_block_id = entry_func.entry_block();
        let entry_block = &entry_func.dfg[entry_block_id];

        // Instruction Ids for all instructions in the entry block
        let instruction_ids = entry_block.instructions();
        for ins_id in instruction_ids {
            let ins = &entry_func.dfg[*ins_id];
            match ins {
                Instruction::Binary(_) => todo!(),
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
}
