use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        value::{Value, ValueId},
    },
    ssa_builder::FunctionBuilder,
    ssa_gen::Ssa,
};

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
        let mut context = InlineContext::new(main_function);
        let main_id = main_function.id();
        context.inline_function(self, main_id, &[])
    }
}

/// The context for the function inlining pass.
///
/// This works using an internal FunctionBuilder to build a new main function from scratch.
/// Doing it this way properly handles importing instructions between functions and lets us
/// reuse the existing API at the cost of essentially cloning each of main's instructions.
struct InlineContext {
    recursion_level: u32,
    functions_to_keep: HashSet<FunctionId>,
    builder: FunctionBuilder,
}

/// The per-function inlining context contains information that is only valid for one function.
/// For example, each function has its own DataFlowGraph, and thus each function needs a translation
/// layer to translate between BlockId to BlockId for the current function and the function to
/// inline into. The same goes for ValueIds, InstructionIds, and for storing other data like
/// parameter to argument mappings.
struct PerFunctionContext<'function> {
    /// The source function is the function we're currently inlining into the function being built.
    source_function: &'function Function,

    /// The shared inlining context for all functions. This notably contains the FunctionBuilder used
    /// to build the function we're inlining into.
    context: &'function mut InlineContext,

    /// Maps ValueIds in the function being inlined to the new ValueIds to use in the function
    /// being inlined into. This mapping also contains the mapping from parameter values to
    /// argument values.
    values: HashMap<ValueId, ValueId>,

    /// Maps BasicBlockIds in the function being inlined to the new BasicBlockIds to use in the
    /// function being inlined into.
    blocks: HashMap<BasicBlockId, BasicBlockId>,

    /// Maps InstructionIds from the function being inlined to the function being inlined into.
    instructions: HashMap<InstructionId, InstructionId>,
}

impl InlineContext {
    /// Create a new context object for the function inlining pass.
    /// This starts off with an empty mapping of instructions for main's parameters.
    ///
    /// main: The main function of the program to start inlining into. Only this function
    ///       and all functions still reachable from it will be returned when inlining is finished.
    fn new(main: &Function) -> InlineContext {
        Self {
            recursion_level: 0,
            builder: FunctionBuilder::new(main.name().to_owned(), main.id()),
            functions_to_keep: HashSet::new(),
        }
    }

    fn inline_function(&mut self, ssa: &Ssa, id: FunctionId, arguments: &[ValueId]) {
        self.recursion_level += 1;

        if self.recursion_level > RECURSION_LIMIT {
            panic!(
                "Attempted to recur more than {RECURSION_LIMIT} times during function inlining."
            );
        }

        let source_function = &ssa.functions[&id];
        let mut context = PerFunctionContext::new(self, source_function, arguments);
        context.inline_blocks(ssa);
    }
}

impl<'function> PerFunctionContext<'function> {
    /// Create a new PerFunctionContext from the source function.
    /// The value and block mappings for this context are initially empty except
    /// for containing the mapping between parameters in the source_function and
    /// the arguments of the destination function.
    fn new(
        context: &'function mut InlineContext,
        source_function: &'function Function,
        arguments: &[ValueId],
    ) -> Self {
        let entry = source_function.entry_block();
        let parameters = source_function.dfg.block_parameters(entry);
        assert_eq!(parameters.len(), arguments.len());

        Self {
            context,
            source_function,
            values: parameters.iter().copied().zip(arguments.iter().copied()).collect(),
            blocks: HashMap::new(),
            instructions: HashMap::new(),
        }
    }

    /// Translates a ValueId from the function being inlined to a ValueId of the function
    /// being inlined into. Note that this expects value ids for all Value::Instruction and
    /// Value::Param values are already handled as a result of previous inlining of instructions
    /// and blocks respectively. If these assertions trigger it means a value is being used before
    /// the instruction or block that defines the value is inserted.
    fn translate_value(&mut self, id: ValueId) -> ValueId {
        if let Some(value) = self.values.get(&id) {
            return *value;
        }

        let new_value = match &self.source_function.dfg[id] {
            Value::Instruction { .. } => {
                unreachable!("All Value::Instructions should already be known during inlining after creating the original inlined instruction")
            }
            Value::Param { .. } => {
                unreachable!("All Value::Params should already be known from previous calls to translate_block")
            }
            Value::NumericConstant { constant, typ } => {
                let value = self.source_function.dfg[*constant].value();
                self.context.builder.numeric_constant(value, *typ)
            }
            Value::Function(function) => self.context.builder.import_function(*function),
            Value::Intrinsic(intrinsic) => self.context.builder.import_intrinsic_id(*intrinsic),
        };

        self.values.insert(id, new_value);
        new_value
    }

    /// Translate a block id from the source function to one of the target function.
    ///
    /// If the block isn't already known, this will insert a new block into the target function
    /// with the same parameter types as the source block.
    fn translate_block(&mut self, id: BasicBlockId) -> BasicBlockId {
        if let Some(block) = self.blocks.get(&id) {
            return *block;
        }

        // The block is not already present in the function being inlined into so we must create it.
        // The block's instructions are not copied over as they will be copied later in inlining.
        let new_block = self.context.builder.insert_block();
        let original_parameters = self.source_function.dfg.block_parameters(id);

        for parameter in original_parameters {
            let typ = self.source_function.dfg.type_of_value(*parameter);
            let new_parameter = self.context.builder.add_block_parameter(new_block, typ);
            self.values.insert(*parameter, new_parameter);
        }

        new_block
    }

    /// Try to retrieve the function referred to by the given Id.
    /// Expects that the given ValueId belongs to the source_function.
    ///
    /// Returns None if the id is not known to refer to a function.
    fn get_function(&mut self, mut id: ValueId) -> Option<FunctionId> {
        id = self.translate_value(id);
        match self.context.builder[id] {
            Value::Function(id) => Some(id),
            _ => None,
        }
    }

    /// Inline all reachable blocks within the source_function into the destination function.
    fn inline_blocks(&mut self, ssa: &Ssa) {
        let mut seen_blocks = HashSet::new();
        let mut block_queue = vec![self.source_function.entry_block()];

        while let Some(block_id) = block_queue.pop() {
            self.context.builder.switch_to_block(block_id);
            seen_blocks.insert(block_id);

            self.inline_block(ssa, block_id);
            self.handle_terminator_instruction(block_id);
        }
    }

    /// Inline each instruction in the given block into the function being inlined into.
    /// This may recurse if it finds another function to inline if a call instruction is within this block.
    fn inline_block(&mut self, ssa: &Ssa, block_id: BasicBlockId) {
        let block = &self.source_function.dfg[block_id];
        for id in block.instructions() {
            match &self.source_function.dfg[*id] {
                Instruction::Call { func, arguments } => match self.get_function(*func) {
                    Some(id) => self.context.inline_function(ssa, id, arguments),
                    None => self.push_instruction(*id),
                },
                _ => self.push_instruction(*id),
            }
        }
    }

    /// Push the given instruction from the source_function into the current block of the
    /// function being inlined into.
    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.source_function.dfg[id].map_values(|id| self.translate_value(id));
        let results = self.source_function.dfg.instruction_results(id);

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(results, |result| self.source_function.dfg.type_of_value(*result)));

        let new_results = self.context.builder.insert_instruction(instruction, ctrl_typevars);

        assert_eq!(results.len(), new_results.len());
        for (result, new_result) in results.iter().zip(new_results) {
            self.values.insert(*result, *new_result);
        }
    }

    /// Handle the given terminator instruction from the given source function block.
    /// This will push any new blocks to the destination function as needed, add them
    /// to the block queue, and set the terminator instruction for the current block.
    fn handle_terminator_instruction(&mut self, block_id: BasicBlockId) {
        match self.source_function.dfg[block_id].terminator() {
            Some(TerminatorInstruction::Jmp { destination, arguments }) => {
                let destination = self.translate_block(*destination);
                let arguments = vecmap(arguments, |arg| self.translate_value(*arg));
                self.context.builder.terminate_with_jmp(destination, arguments);
            }
            Some(TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
            }) => todo!(),
            Some(TerminatorInstruction::Return { .. }) => (),
            None => unreachable!("Block has no terminator instruction"),
        }
    }
}
