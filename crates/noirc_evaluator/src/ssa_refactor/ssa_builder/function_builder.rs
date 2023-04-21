use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId,
    function::{Function, FunctionId},
    types::Type,
    value::ValueId,
};

use super::SharedBuilderContext;

/// The per-function context for each ssa function being generated.
///
/// This is split from the global SsaBuilder context to allow each function
/// to be potentially built concurrently.
///
/// Contrary to the name, this struct has the capacity to build as many
/// functions as needed, although it is limited to one function at a time.
pub(crate) struct FunctionBuilder<'ssa> {
    global_context: &'ssa SharedBuilderContext,

    current_function: Function,
    current_function_id: FunctionId,

    current_block: BasicBlockId,

    finished_functions: Vec<(FunctionId, Function)>,
}

impl<'ssa> FunctionBuilder<'ssa> {
    pub(crate) fn new(context: &'ssa SharedBuilderContext) -> Self {
        let new_function = Function::new();
        let current_block = new_function.entry_block();

        Self {
            global_context: context,
            current_function: new_function,
            current_function_id: context.next_function(),
            current_block,
            finished_functions: Vec::new(),
        }
    }

    /// Finish the current function and create a new function
    pub(crate) fn new_function(&mut self) {
        let new_function = Function::new();
        let old_function = std::mem::replace(&mut self.current_function, new_function);

        self.finished_functions.push((self.current_function_id, old_function));
        self.current_function_id = self.global_context.next_function();
    }

    pub(crate) fn finish(mut self) -> Vec<(FunctionId, Function)> {
        self.finished_functions.push((self.current_function_id, self.current_function));
        self.finished_functions
    }

    pub(crate) fn add_parameter(&mut self, typ: Type) -> ValueId {
        let entry = self.current_function.entry_block();
        self.current_function.dfg.add_block_parameter(entry, typ)
    }
}
