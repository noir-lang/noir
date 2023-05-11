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
    pub(crate) fn unroll(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            Context::new(function).simplify_function_cfg();
        }
        self
    }
}

struct Context<'f> {
    visited_blocks: HashSet<BasicBlockId>,
    values: HashMap<ValueId, ValueId>,
    function: &'f mut Function,

    current_block: BasicBlockId,

    /// This CFG is the original CFG before the pass modifies each block
    cfg: ControlFlowGraph,
}

enum Job {
    MergeIf { 
        block_with_brif: BasicBlockId, 
        merge_point: BasicBlockId,

        // The last block where the condition of the brif instruction can be assumed to be true.
        // This block should always be a direct predecessor of merge_point
        final_then_block: BasicBlockId,

        // The last block where the condition of the brif instruction can be assumed to be false.
        // This block should always be a direct predecessor of merge_point
        final_else_block: BasicBlockId,
    },
    UnrollLoop {
        pre_loop: BasicBlockId,
        loop_start: BasicBlockId,
        loop_body: BasicBlockId,
        loop_end: BasicBlockId,
    },
    /// A transitive block is one that looks like B in A -> B -> {C, D, ..}.
    /// That is, it is unconditionally branched to by exactly 1 predecessor.
    /// We can merge these blocks and remove the unnecessary br to turn the
    /// new cfg into AB -> {C, D, ..}
    RemoveTransitiveBlock {
        before: BasicBlockId,
        transitive: BasicBlockId,
        after: BasicBlockId,
    }
}

// fn main f2 {
//   b0(v0: u1):
//     jmpif v0 then: b2, else: b3
//   b2():
//     jmp b4(Field 6 (v17))
//   b4(v1: Field):
//     v4 = eq v1, Field 6 (v3)
//     constrain v4
//     jmp b6(Field 0 (v8))
//   b6(v7: Field):
//     v9 = lt v7, Field 2 (v6)
//     jmpif v9 then: b7, else: b8
//   b7():
//     v13 = call println(v7)
//     v15 = add v7, Field 1 (v14)
//     jmp b6(v15)
//   b8():
//     jmp b5(unit 0 (v10))
//   b5(v11: unit):
//     return unit 0 (v16)
//   b3():
//     jmp b4(Field 7 (v2))
// }
//
//
// b0 -> b2           v----|
//          => b4 -> b6 -> b7
//    -> b3
//                      -> b8 -> b5
//
// MergeIf(b0, b2, b3, b4)
// UnrollLoop(b4, b6, b7, b8)
// RemoveTransitiveBlock(b8, b5)

impl<'f> Context<'f> {
    fn new(function: &'f mut Function) -> Self {
        Self {
            visited_blocks: HashSet::new(),
            values: HashMap::new(),
            cfg: ControlFlowGraph::with_function(function),
            current_block: function.entry_block(),
            function,
        }
    }

    fn simplify_function_cfg(&mut self) {
        let block = &self.function.dfg[self.current_block];
        self.visited_blocks.insert(self.current_block);

        match block.terminator() {
            // TODO Remove the clone
            Some(TerminatorInstruction::Jmp { destination, arguments }) => {
                self.handle_jmp(*destination, &arguments.clone());
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

    fn handle_jmp(
        &mut self,
        destination: BasicBlockId,
        arguments: &[ValueId],
    ) {
        self.inline_instructions_from_block(&arguments, destination);
        self.simplify_function_cfg();
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
                self.handle_jmp(next_block, &[]);
            }
            None => {
                // We only allow dynamic branching if we're not going in a loop
                assert!(
                    !self.visited_blocks.contains(&then_block),
                    "Dynamic loops are unsupported - block {then_block} was already visited"
                );
                assert!(
                    !self.visited_blocks.contains(&else_block),
                    "Dynamic loops are unsupported - block {else_block} was already visited"
                );

                self.current_block = then_block;
                self.handle_jmp(then_block, &[]);
                self.current_block = else_block;
                self.handle_jmp(else_block, &[]);
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
