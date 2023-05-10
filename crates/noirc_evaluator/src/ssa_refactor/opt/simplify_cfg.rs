use std::collections::{HashMap, HashSet};

use acvm::FieldElement;
use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        dfg::InsertInstructionResult,
        function::Function,
        instruction::{InstructionId, TerminatorInstruction},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

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
        Self { visited_blocks: HashSet::new(), values: HashMap::new(), function }
    }

    fn simplify_function_cfg(&mut self, current_block: BasicBlockId) {
        let block = &self.function.dfg[current_block];
        self.visited_blocks.insert(current_block);

        match block.terminator() {
            Some(TerminatorInstruction::Jmp { destination, arguments }) => {
                let source_block = *destination;
                let arguments = arguments.clone(); // TODO Remove clone
                self.inline_instructions_from_block(current_block, &arguments, source_block);
                self.simplify_function_cfg(current_block);
            },
            Some(TerminatorInstruction::JmpIf { condition, then_destination, else_destination }) => {
                match self.get_constant(*condition) {
                    Some(constant) => {
                        let next_block =
                            if constant.is_zero() { *else_destination } else { *then_destination };
                        self.inline_instructions_from_block(current_block, &[], next_block);
                        self.simplify_function_cfg(current_block);
                    }
                    None => {
                        // We only allow dynamic branching if we're not going in a loop
                        assert!(!self.visited_blocks.contains(then_destination), "Dynamic loops are unsupported - block {then_destination} was already visited");
                        assert!(!self.visited_blocks.contains(else_destination), "Dynamic loops are unsupported - block {else_destination} was already visited");
                        let else_destination = *else_destination;

                        self.inline_instructions_from_block(current_block, &[], *then_destination);
                        self.simplify_function_cfg(current_block);
                        self.inline_instructions_from_block(current_block, &[], else_destination);
                        self.simplify_function_cfg(current_block);
                    }
                }
            },
            Some(TerminatorInstruction::Return { return_values: _ }) => (),
            None => unreachable!("Block has no terminator"),
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
    fn inline_instructions_from_block(
        &mut self,
        dest_block: BasicBlockId,
        jmp_args: &[ValueId],
        source_block_id: BasicBlockId,
    ) {
        let source_block = &self.function.dfg[source_block_id];
        assert_eq!(source_block.parameters().len(), jmp_args.len(), "Parameter len != arg len when inlining block {source_block_id} into {dest_block}");

        // Map each parameter to its new value
        for (param, arg) in source_block.parameters().iter().zip(jmp_args) {
            self.values.insert(*param, *arg);
        }

        let instructions = source_block.instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute the
        // block parameter values.
        for instruction in instructions {
            self.push_instruction(dest_block, instruction);
        }

        let terminator = self.function.dfg[source_block_id].terminator()
            .expect("Expected each block during the simplify_cfg optimization to have a terminator instruction")
            .map_values(|id| self.get_value(id));

        self.function.dfg.set_block_terminator(dest_block, terminator);
    }

    fn push_instruction(&mut self, current_block: BasicBlockId, id: InstructionId) {
        let instruction = self.function.dfg[id].map_values(|id| self.get_value(id));
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let new_results = self.function.dfg.insert_instruction_and_results(
            instruction,
            current_block,
            ctrl_typevars,
        );
        Self::insert_new_instruction_results(&mut self.values, &results, new_results);
    }

    /// Modify the values HashMap to remember the mapping between an instruction result's previous
    /// ValueId (from the source_function) and its new ValueId in the destination function.
    fn insert_new_instruction_results(
        values: &mut HashMap<ValueId, ValueId>,
        old_results: &[ValueId],
        new_results: InsertInstructionResult,
    ) {
        assert_eq!(old_results.len(), new_results.len());

        match new_results {
            InsertInstructionResult::SimplifiedTo(new_result) => {
                values.insert(old_results[0], new_result);
            }
            InsertInstructionResult::Results(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }
}
