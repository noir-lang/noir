use crate::ssa_refactor::{ssa_gen::Ssa, ir::{function::Function, cfg::ControlFlowGraph, basic_block::BasicBlockId}};


impl Ssa {
    pub(crate) fn simplify_cfg(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            simplify_function_cfg(function);
        }
        self
    }
}

fn simplify_function_cfg(function: &mut Function) {
    let current_block = function.entry_block();
    simplify_function_cfg_recursive(function, current_block);
}

fn simplify_function_cfg_recursive(function: &mut Function, current_block: BasicBlockId) {
    let successors: Vec<_> = function.dfg[current_block].successors().collect();

    if successors.len() == 1 {
        let source_block = successors[0];
        inline_instructions_from_block(function, current_block, source_block);
        simplify_function_cfg_recursive(function, current_block);
    } else {
        for block in successors {
            simplify_function_cfg_recursive(function, block);
        }
    }
}

/// TODO: Translate block parameters
fn inline_instructions_from_block(function: &mut Function, dest_block: BasicBlockId, source_block: BasicBlockId) {
    let instructions = function.dfg[source_block].instructions().to_vec();

    // We cannot directly append each instruction since we need to substitute the
    // block parameter values.
    for instruction in instructions {
        function.dfg.insert_instruction_in_block(dest_block, instruction);
    }

    let terminator = function.dfg[source_block].terminator().expect("Expected each block during the simplify_cfg optimization to have a terminator instruction").clone();
    function.dfg.set_block_terminator(dest_block, terminator);
}
