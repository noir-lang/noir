use acvm::FieldElement;

use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId,
    function::{Function, FunctionId},
    instruction::{Binary, BinaryOp, Instruction, InstructionId},
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
    pub(crate) fn new(function_name: String, context: &'ssa SharedBuilderContext) -> Self {
        let new_function = Function::new(function_name);
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
    pub(crate) fn new_function(&mut self, name: String) {
        let new_function = Function::new(name);
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

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(&mut self, value: FieldElement, typ: Type) -> ValueId {
        self.current_function.dfg.make_constant(value, typ)
    }

    /// Insert a numeric constant into the current function of type Field
    pub(crate) fn field_constant(&mut self, value: impl Into<FieldElement>) -> ValueId {
        self.numeric_constant(value.into(), Type::field())
    }

    fn insert_instruction(&mut self, instruction: Instruction) -> InstructionId {
        let id = self.current_function.dfg.make_instruction(instruction);
        self.current_function.dfg.insert_instruction_in_block(self.current_block, id);
        id
    }

    /// Insert an allocate instruction at the end of the current block, allocating the
    /// given amount of field elements. Returns the result of the allocate instruction,
    /// which is always a Reference to the allocated data.
    pub(crate) fn insert_allocate(&mut self, size_to_allocate: u32) -> ValueId {
        let id = self.insert_instruction(Instruction::Allocate { size: size_to_allocate });
        self.current_function.dfg.make_instruction_results(id, Type::Reference)[0]
    }

    /// Insert a Load instruction at the end of the current block, loading from the given address
    /// which should point to a previous Allocate instruction. Note that this is limited to loading
    /// a single value. Loading multiple values (such as a tuple) will require multiple loads.
    /// Returns the element that was loaded.
    pub(crate) fn insert_load(&mut self, address: ValueId, type_to_load: Type) -> ValueId {
        let id = self.insert_instruction(Instruction::Load { address });
        self.current_function.dfg.make_instruction_results(id, type_to_load)[0]
    }

    /// Insert a Store instruction at the end of the current block, storing the given element
    /// at the given address. Expects that the address points to a previous Allocate instruction.
    pub(crate) fn insert_store(&mut self, address: ValueId, value: ValueId) {
        self.insert_instruction(Instruction::Store { address, value });
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    pub(crate) fn insert_binary(
        &mut self,
        lhs: ValueId,
        operator: BinaryOp,
        rhs: ValueId,
        typ: Type,
    ) -> ValueId {
        let id = self.insert_instruction(Instruction::Binary(Binary { lhs, rhs, operator }));
        self.current_function.dfg.make_instruction_results(id, typ)[0]
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId, typ: Type) -> ValueId {
        let id = self.insert_instruction(Instruction::Not(rhs));
        self.current_function.dfg.make_instruction_results(id, typ)[0]
    }
}
