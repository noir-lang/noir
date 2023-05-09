use std::collections::{HashSet, HashMap};

use acvm::FieldElement;

use crate::ssa_refactor::{ssa_gen::Ssa, ir::{function::Function, cfg::ControlFlowGraph, basic_block::BasicBlockId, value::ValueId, instruction::TerminatorInstruction}};


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
    let mut context = Context::new(function);
    context.simplify_function_cfg(current_block);
}

struct Context<'f> {
    visited_blocks: HashSet<BasicBlockId>,
    values: HashMap<ValueId, ValueId>,
    function: &'f mut Function,
}

impl<'f> Context<'f> {
    fn new(function: &'f mut Function) -> Self {
        Self {
            visited_blocks: HashSet::new(),
            values: HashMap::new(),
            function,
        }
    }

    fn simplify_function_cfg(&mut self, current_block: BasicBlockId) {
        let block = &self.function.dfg[current_block];
        let successors: Vec<_> = block.successors().collect();
        self.visited_blocks.insert(current_block);

        if successors.len() == 1 {
            let source_block = successors[0];
            self.inline_instructions_from_block(current_block, source_block);
            self.simplify_function_cfg(current_block);
        } else if successors.len() > 1 {
            if let Some(TerminatorInstruction::JmpIf { condition, then_destination, else_destination }) = block.terminator() {
                match self.get_constant(*condition) {
                    Some(constant) => {
                        let next_block = if constant.is_zero() { *else_destination } else { *then_destination };
                        self.inline_instructions_from_block(current_block, next_block);
                        self.simplify_function_cfg(current_block);
                    },
                    None => todo!(),
                }

            } else {
                unreachable!("Only JmpIf terminators should have more than 1 successor")
            }
        }
    }

    fn get_value(&self, value: ValueId) -> ValueId {
        self.values.get(&value).copied().unwrap_or(value)
    }

    fn get_constant(&self, value: ValueId) -> Option<FieldElement> {
        let value = self.get_value(value);
        self.function.dfg.get_numeric_constant(value)
    }

    /// TODO: Translate block parameters
    fn inline_instructions_from_block(&mut self, dest_block: BasicBlockId, source_block: BasicBlockId) {
        let instructions = self.function.dfg[source_block].instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute the
        // block parameter values.
        for instruction in instructions {
            self.function.dfg.insert_instruction_in_block(dest_block, instruction);
        }

        let terminator = self.function.dfg[source_block].terminator().expect("Expected each block during the simplify_cfg optimization to have a terminator instruction").clone();
        self.function.dfg.set_block_terminator(dest_block, terminator);
    }
}
