use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{ssa_gen::Ssa, ir::{instruction::Instruction, value::{ValueId, Value}, dfg::DataFlowGraph, function::FunctionId, basic_block::BasicBlockId}};

/// An arbitrary limit to the maximum number of recursive call
/// frames at any point in time.
const RECURSION_LIMIT: u32 = 1000;

impl Ssa {
    /// Inline all functions within the IR.
    ///
    /// In the case of recursive functions, this will attempt
    /// to recursively inline until the RECURSION_LIMIT is reached.
    ///
    /// Functions are recursively inlined into main until either we finish
    /// inlining all functions or we encounter a function whose function id is not known.
    /// When the later happens, the call instruction is kept in addition to the function
    /// it refers to. The function it refers to is kept unmodified without any inlining
    /// changes. This is because if the function's id later becomes known by a later
    /// pass, we would need to re-run all of inlining anyway to inline it, so we might
    /// as well save the work for later instead of performing it twice.
    pub(crate) fn inline_functions(&mut self) {
        let main_function = self.main();
        let mut context = InlineContext::new(main_function.entry_block(), main_function.id());

        let blocks = vecmap(main_function.dfg.basic_blocks_iter(), |(id, _)| id);

        for block in blocks {
            let instructions = main_function.dfg[block].instructions();

            let mut new_instructions = Vec::with_capacity(instructions.len());

            for (index, instruction) in instructions.iter().copied().enumerate() {
                match &main_function.dfg[instruction] {
                    Instruction::Call { func, arguments } => {
                        match context.get_function(*func, &main_function.dfg) {
                            Some(id) => {
                                main_function.dfg.split_block_at(block, instruction);
                                context.inline_function(self, id, arguments)
                            }
                            None => new_instructions.push(instruction),
                        }
                    },
                    _ => new_instructions.push(instruction),
                }
            }
        }
    }
}

struct InlineContext {
    recursion_level: u32,
    argument_values: Vec<HashMap<ValueId, ValueId>>,
    current_block: BasicBlockId,
    visited_blocks: HashSet<BasicBlockId>,
    functions_to_keep: HashSet<FunctionId>,
}

impl InlineContext {
    /// Create a new context object for the function inlining pass.
    /// This starts off with an empty mapping of instructions for main's parameters.
    fn new(current_block: BasicBlockId, main: FunctionId) -> InlineContext {
        let mut visited_blocks = HashSet::new();
        visited_blocks.insert(current_block);

        let mut functions_to_keep = HashSet::new();
        functions_to_keep.insert(main);

        Self {
            recursion_level: 0,
            argument_values: vec![HashMap::new()],
            current_block,
            visited_blocks,
            functions_to_keep,
        }
    }

    fn current_function_arguments(&self) -> &HashMap<ValueId, ValueId> {
        self.argument_values.last()
            .expect("Expected there to always be argument values for the current function being inlined")
    }

    fn get_function(&self, mut id: ValueId, dfg: &DataFlowGraph) -> Option<FunctionId> {
        if let Some(new_id) = self.current_function_arguments().get(&id) {
            id = *new_id;
        }

        match dfg[id] {
            Value::Function(id) => Some(id),
            _ => None,
        }
    }

    fn inline_function(&mut self, ssa: &Ssa, id: FunctionId, arguments: &[ValueId]) {
        let target_function = &ssa.functions[&id];
        let current_block = target_function.entry_block();

        let parameters = target_function.dfg.block_parameters(current_block);
        assert_eq!(parameters.len(), arguments.len());

        let argument_map = parameters.iter().copied().zip(arguments.iter().copied()).collect();
        self.argument_values.push(argument_map);

        let instructions = target_function.dfg[current_block].instructions();

        let mut new_instructions = Vec::with_capacity(instructions.len());

        for id in instructions {
            match &target_function.dfg[*id] {
                Instruction::Call { func, arguments } => {
                    match self.get_function(*func, &target_function.dfg) {
                        Some(id) => self.inline_function(ssa, id, arguments),
                        None => new_instructions.push(*id),
                    }
                },
                _ => new_instructions.push(*id),
            }
        }

        self.argument_values.pop();
    }
}
