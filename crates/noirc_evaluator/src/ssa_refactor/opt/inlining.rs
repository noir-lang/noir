//! This module defines the function inlining pass for the SSA IR.
//! The purpose of this pass is to inline the instructions of each function call
//! within the function caller. If all function calls are known, there will only
//! be a single function remaining when the pass finishes.
use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        dfg::InsertInstructionResult,
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
    pub(crate) fn inline_functions(self) -> Ssa {
        InlineContext::new(&self).inline_all(self)
    }
}

/// The context for the function inlining pass.
///
/// This works using an internal FunctionBuilder to build a new main function from scratch.
/// Doing it this way properly handles importing instructions between functions and lets us
/// reuse the existing API at the cost of essentially cloning each of main's instructions.
struct InlineContext {
    recursion_level: u32,
    builder: FunctionBuilder,

    /// True if we failed to inline at least one call. If this is still false when finishing
    /// inlining we can remove all other functions from the resulting Ssa struct and keep only
    /// the function that was inlined into.
    failed_to_inline_a_call: bool,
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

    /// True if we're currently working on the main function.
    inlining_main: bool,
}

impl InlineContext {
    /// Create a new context object for the function inlining pass.
    /// This starts off with an empty mapping of instructions for main's parameters.
    /// The function being inlined into will always be the main function, although it is
    /// actually a copy that is created in case the original main is still needed from a function
    /// that could not be inlined calling it.
    fn new(ssa: &Ssa) -> InlineContext {
        let main_name = ssa.main().name().to_owned();
        let builder = FunctionBuilder::new(main_name, ssa.next_id.next());
        Self { builder, recursion_level: 0, failed_to_inline_a_call: false }
    }

    /// Start inlining the main function and all functions reachable from it.
    fn inline_all(mut self, ssa: Ssa) -> Ssa {
        let main = ssa.main();
        let mut context = PerFunctionContext::new(&mut self, main);
        context.inlining_main = true;

        // The main block is already inserted so we have to add it to context.blocks and add
        // its parameters here. Failing to do so would cause context.translate_block() to add
        // a fresh block for the entry block rather than use the existing one.
        let entry_block = context.context.builder.current_function.entry_block();
        let original_parameters = context.source_function.parameters();

        for parameter in original_parameters {
            let typ = context.source_function.dfg.type_of_value(*parameter);
            let new_parameter = context.context.builder.add_block_parameter(entry_block, typ);
            context.values.insert(*parameter, new_parameter);
        }

        context.blocks.insert(context.source_function.entry_block(), entry_block);
        context.inline_blocks(&ssa);
        self.finish(ssa)
    }

    /// Inlines a function into the current function and returns the translated return values
    /// of the inlined function.
    fn inline_function(
        &mut self,
        ssa: &Ssa,
        id: FunctionId,
        arguments: &[ValueId],
    ) -> Vec<ValueId> {
        self.recursion_level += 1;

        if self.recursion_level > RECURSION_LIMIT {
            panic!(
                "Attempted to recur more than {RECURSION_LIMIT} times during function inlining."
            );
        }

        let source_function = &ssa.functions[&id];
        let mut context = PerFunctionContext::new(self, source_function);

        let parameters = source_function.parameters();
        assert_eq!(parameters.len(), arguments.len());
        context.values = parameters.iter().copied().zip(arguments.iter().copied()).collect();

        let current_block = context.context.builder.current_block();
        context.blocks.insert(source_function.entry_block(), current_block);

        context.inline_blocks(ssa)
    }

    /// Finish inlining and return the new Ssa struct with the inlined version of main.
    /// If any functions failed to inline, they are not removed from the final Ssa struct.
    fn finish(self, mut ssa: Ssa) -> Ssa {
        let mut new_ssa = self.builder.finish();
        assert_eq!(new_ssa.functions.len(), 1);

        // If we failed to inline any call, any function may still be reachable so we
        // don't remove any from the final program. We could be more precise here and
        // do a reachability analysis but it should be fine to keep the extra functions
        // around longer if they are not called.
        if self.failed_to_inline_a_call {
            let new_main = new_ssa.functions.pop_first().unwrap().1;
            ssa.main_id = new_main.id();
            ssa.functions.insert(new_main.id(), new_main);
            ssa
        } else {
            new_ssa
        }
    }
}

impl<'function> PerFunctionContext<'function> {
    /// Create a new PerFunctionContext from the source function.
    /// The value and block mappings for this context are initially empty except
    /// for containing the mapping between parameters in the source_function and
    /// the arguments of the destination function.
    fn new(context: &'function mut InlineContext, source_function: &'function Function) -> Self {
        Self {
            context,
            source_function,
            blocks: HashMap::new(),
            instructions: HashMap::new(),
            values: HashMap::new(),
            inlining_main: false,
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
            value @ Value::Instruction { .. } => {
                unreachable!("All Value::Instructions should already be known during inlining after creating the original inlined instruction. Unknown value {id} = {value:?}")
            }
            value @ Value::Param { .. } => {
                unreachable!("All Value::Params should already be known from previous calls to translate_block. Unknown value {id} = {value:?}")
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
    fn translate_block(
        &mut self,
        source_block: BasicBlockId,
        block_queue: &mut Vec<BasicBlockId>,
    ) -> BasicBlockId {
        if let Some(block) = self.blocks.get(&source_block) {
            return *block;
        }

        // The block is not yet inlined, queue it
        block_queue.push(source_block);

        // The block is not already present in the function being inlined into so we must create it.
        // The block's instructions are not copied over as they will be copied later in inlining.
        let new_block = self.context.builder.insert_block();
        let original_parameters = self.source_function.dfg.block_parameters(source_block);

        for parameter in original_parameters {
            let typ = self.source_function.dfg.type_of_value(*parameter);
            let new_parameter = self.context.builder.add_block_parameter(new_block, typ);
            self.values.insert(*parameter, new_parameter);
        }

        self.blocks.insert(source_block, new_block);
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
            Value::Intrinsic(_) => None,
            _ => {
                self.context.failed_to_inline_a_call = true;
                None
            }
        }
    }

    /// Inline all reachable blocks within the source_function into the destination function.
    fn inline_blocks(&mut self, ssa: &Ssa) -> Vec<ValueId> {
        let mut seen_blocks = HashSet::new();
        let mut block_queue = vec![self.source_function.entry_block()];

        let mut function_return = None;

        while let Some(source_block) = block_queue.pop() {
            let translated_block = self.translate_block(source_block, &mut block_queue);
            self.context.builder.switch_to_block(translated_block);

            seen_blocks.insert(source_block);
            self.inline_block(ssa, source_block);

            if let Some(ret) = self.handle_terminator_instruction(source_block, &mut block_queue) {
                function_return = Some(ret);
            }
        }

        if let Some((block, values)) = function_return {
            self.context.builder.switch_to_block(block);
            values
        } else {
            unreachable!("Inlined function had no return instruction")
        }
    }

    /// Inline each instruction in the given block into the function being inlined into.
    /// This may recurse if it finds another function to inline if a call instruction is within this block.
    fn inline_block(&mut self, ssa: &Ssa, block_id: BasicBlockId) {
        let block = &self.source_function.dfg[block_id];
        for id in block.instructions() {
            match &self.source_function.dfg[*id] {
                Instruction::Call { func, arguments } => match self.get_function(*func) {
                    Some(function) => self.inline_function(ssa, *id, function, arguments),
                    None => self.push_instruction(*id),
                },
                _ => self.push_instruction(*id),
            }
        }
    }

    /// Inline a function call and remember the inlined return values in the values map
    fn inline_function(
        &mut self,
        ssa: &Ssa,
        call_id: InstructionId,
        function: FunctionId,
        arguments: &[ValueId],
    ) {
        let old_results = self.source_function.dfg.instruction_results(call_id);
        let arguments = vecmap(arguments, |arg| self.translate_value(*arg));
        let new_results = self.context.inline_function(ssa, function, &arguments);
        let new_results = InsertInstructionResult::Results(&new_results);
        Self::insert_new_instruction_results(&mut self.values, old_results, new_results);
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
        Self::insert_new_instruction_results(&mut self.values, results, new_results);
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

    /// Handle the given terminator instruction from the given source function block.
    /// This will push any new blocks to the destination function as needed, add them
    /// to the block queue, and set the terminator instruction for the current block.
    ///
    /// If the terminator instruction was a Return, this will return the block this instruction
    /// was in as well as the values that were returned.
    fn handle_terminator_instruction(
        &mut self,
        block_id: BasicBlockId,
        block_queue: &mut Vec<BasicBlockId>,
    ) -> Option<(BasicBlockId, Vec<ValueId>)> {
        match self.source_function.dfg[block_id].unwrap_terminator() {
            TerminatorInstruction::Jmp { destination, arguments } => {
                let destination = self.translate_block(*destination, block_queue);
                let arguments2 = vecmap(arguments, |arg| self.translate_value(*arg));
                self.context.builder.terminate_with_jmp(destination, arguments2);
                None
            }
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                let condition = self.translate_value(*condition);
                let then_block = self.translate_block(*then_destination, block_queue);
                let else_block = self.translate_block(*else_destination, block_queue);
                self.context.builder.terminate_with_jmpif(condition, then_block, else_block);
                None
            }
            TerminatorInstruction::Return { return_values } => {
                let return_values = vecmap(return_values, |value| self.translate_value(*value));
                if self.inlining_main {
                    self.context.builder.terminate_with_return(return_values.clone());
                }
                let block_id = self.translate_block(block_id, block_queue);
                Some((block_id, return_values))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ssa_refactor::{
        ir::{instruction::BinaryOp, map::Id, types::Type},
        ssa_builder::FunctionBuilder,
    };

    #[test]
    fn basic_inlining() {
        // fn foo {
        //   b0():
        //     v0 = call bar()
        //     return v0
        // }
        // fn bar {
        //   b0():
        //     return 72
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("foo".into(), foo_id);

        let bar_id = Id::test_new(1);
        let bar = builder.import_function(bar_id);
        let results = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        builder.terminate_with_return(results);

        builder.new_function("bar".into(), bar_id);
        let expected_return = 72u128;
        let seventy_two = builder.field_constant(expected_return);
        builder.terminate_with_return(vec![seventy_two]);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        let inlined = ssa.inline_functions();
        assert_eq!(inlined.functions.len(), 1);
    }

    #[test]
    fn complex_inlining() {
        // This SSA is from issue #1327 which previously failed to inline properly
        //
        // fn main f0 {
        //   b0(v0: Field):
        //     v7 = call f2(f1)
        //     v13 = call f3(v7)
        //     v16 = call v13(v0)
        //     return v16
        // }
        // fn square f1 {
        //   b0(v0: Field):
        //     v2 = mul v0, v0
        //     return v2
        // }
        // fn id1 f2 {
        //   b0(v0: function):
        //     return v0
        // }
        // fn id2 f3 {
        //   b0(v0: function):
        //     return v0
        // }
        let main_id = Id::test_new(0);
        let square_id = Id::test_new(1);
        let id1_id = Id::test_new(2);
        let id2_id = Id::test_new(3);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let main_v0 = builder.add_parameter(Type::field());

        let main_f1 = builder.import_function(square_id);
        let main_f2 = builder.import_function(id1_id);
        let main_f3 = builder.import_function(id2_id);

        let main_v7 = builder.insert_call(main_f2, vec![main_f1], vec![Type::Function])[0];
        let main_v13 = builder.insert_call(main_f3, vec![main_v7], vec![Type::Function])[0];
        let main_v16 = builder.insert_call(main_v13, vec![main_v0], vec![Type::field()])[0];
        builder.terminate_with_return(vec![main_v16]);

        // Compiling square f1
        builder.new_function("square".into(), square_id);
        let square_v0 = builder.add_parameter(Type::field());
        let square_v2 = builder.insert_binary(square_v0, BinaryOp::Mul, square_v0);
        builder.terminate_with_return(vec![square_v2]);

        // Compiling id1 f2
        builder.new_function("id1".into(), id1_id);
        let id1_v0 = builder.add_parameter(Type::Function);
        builder.terminate_with_return(vec![id1_v0]);

        // Compiling id2 f3
        builder.new_function("id2".into(), id2_id);
        let id2_v0 = builder.add_parameter(Type::Function);
        builder.terminate_with_return(vec![id2_v0]);

        // Done, now we test that we can successfully inline all functions.
        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        let inlined = ssa.inline_functions();
        assert_eq!(inlined.functions.len(), 1);
    }
}
