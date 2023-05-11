use std::collections::{HashMap, HashSet};

use acvm::FieldElement;
use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::InsertInstructionResult,
        function::Function,
        instruction::{InstructionId, TerminatorInstruction},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn unroll_loops(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            Context::new(function).unroll_loops();
        }
        self
    }
}

struct Context<'f> {
    visited_blocks: HashSet<BasicBlockId>,
    values: HashMap<ValueId, ValueId>,
    function: &'f mut Function,

    current_block: BasicBlockId,

    /// The original ControlFlowGraph of this function before it was modified
    /// by this loop unrolling pass.
    cfg: ControlFlowGraph,
    inlined_loop_blocks: HashSet<BasicBlockId>,
}

impl<'f> Context<'f> {
    fn new(function: &'f mut Function) -> Self {
        Self {
            visited_blocks: HashSet::new(),
            values: HashMap::new(),
            current_block: function.entry_block(),
            inlined_loop_blocks: HashSet::new(),
            cfg: ControlFlowGraph::with_function(function),
            function,
        }
    }

    fn unroll_loops(&mut self) {
        let block = &self.function.dfg[self.current_block];
        self.visited_blocks.insert(self.current_block);
        println!("Visited {}", self.current_block);

        match block.terminator() {
            // TODO Remove the clone
            Some(TerminatorInstruction::Jmp { destination, arguments }) => {
                self.handle_jmp(*destination, &arguments.clone(), false);
            }
            Some(TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
            }) => {
                self.handle_jmpif(*condition, *then_destination, *else_destination);
            }
            Some(TerminatorInstruction::Return { return_values: _ }) => (),
            None => unreachable!("Block has no terminator"),
        }
    }

    ///
    /// entry -> a \
    ///            |-> c
    ///       -> b /
    ///
    ///     V----|
    /// entry -> a
    ///       -> b
    ///

    fn handle_jmp(
        &mut self,
        destination: BasicBlockId,
        arguments: &[ValueId],
        conditional_jmp: bool,
    ) {
        let non_looping_predecessor_count = self.count_non_looping_predecessors(destination);

        if !conditional_jmp && non_looping_predecessor_count <= 1 {
            // Inline the block
            println!("Directly inlining {destination}");
            self.inline_instructions_from_block(&arguments, destination);
        } else {
            println!("Switching to {destination}");
            self.current_block = destination;
        }
        self.unroll_loops();
    }

    fn count_non_looping_predecessors(&mut self, block: BasicBlockId) -> usize {
        let predecessors = self.cfg.predecessors(block);

        predecessors.filter(|pred| !self.reachable_from(*pred, *pred, &mut HashSet::new())).count()
    }

    fn reachable_from(
        &self,
        current_block: BasicBlockId,
        target: BasicBlockId,
        visited: &mut HashSet<BasicBlockId>,
    ) -> bool {
        if visited.contains(&current_block) {
            return false;
        }

        visited.insert(current_block);

        for successor in self.cfg.successors(current_block) {
            if successor == target {
                return true;
            }
            if self.reachable_from(successor, target, visited) {
                return true;
            }
        }

        false
    }

    fn handle_jmpif(
        &mut self,
        condition: ValueId,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
    ) {
        match self.get_constant(condition) {
            Some(constant) => {
                let next_block = if constant.is_zero() { else_block } else { then_block };
                self.inlined_loop_blocks.insert(self.current_block);
                println!("Constant jmpif to {next_block}");
                self.handle_jmp(next_block, &[], false);
            }
            None => {
                // We only allow dynamic branching if we're not going in a loop
                let verify = |block| {
                    let looped = self.visited_blocks.contains(block);
                    assert!(!looped, "Dynamic loops are unsupported - {block} was already visited");
                };

                verify(&then_block);
                verify(&else_block);

                println!("Condition = {condition}");
                println!("Non-constant jmpif to {then_block} or {else_block}");

                self.current_block = then_block;
                self.handle_jmp(then_block, &[], true);
                self.current_block = else_block;
                self.handle_jmp(else_block, &[], true);
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

    fn inline_instructions_from_block(
        &mut self,
        jmp_args: &[ValueId],
        source_block_id: BasicBlockId,
    ) {
        let dest_block = self.current_block;
        let source_block = &self.function.dfg[source_block_id];
        assert_eq!(
            source_block.parameters().len(),
            jmp_args.len(),
            "Parameter len != arg len when inlining block {source_block_id} into {dest_block}"
        );

        // Map each parameter to its new value
        for (param, arg) in source_block.parameters().iter().zip(jmp_args) {
            self.values.insert(*param, *arg);
        }

        let instructions = source_block.instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute the
        // block parameter values.
        for instruction in instructions {
            self.push_instruction(instruction);
        }

        let terminator = self.function.dfg[source_block_id].terminator()
            .expect("Expected each block during the simplify_cfg optimization to have a terminator instruction")
            .map_values(|id| self.get_value(id));

        self.function.dfg.set_block_terminator(dest_block, terminator);
    }

    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.function.dfg[id].map_values(|id| self.get_value(id));
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let new_results = self.function.dfg.insert_instruction_and_results(
            instruction,
            self.current_block,
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
                println!("result {} -> {}", old_results[0], new_result);
                values.insert(old_results[0], new_result);
            }
            InsertInstructionResult::Results(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    println!("result {} -> {}", old_result, new_result);
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }
}
