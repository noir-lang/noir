use acvm::FieldElement;

use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId,
    function::{Function, FunctionId},
    instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
    types::Type,
    value::{Value, ValueId},
};

use super::{
    ir::instruction::{InstructionId, Intrinsic},
    ssa_gen::Ssa,
};

/// The per-function context for each ssa function being generated.
///
/// This is split from the global SsaBuilder context to allow each function
/// to be potentially built concurrently.
///
/// Contrary to the name, this struct has the capacity to build as many
/// functions as needed, although it is limited to one function at a time.
pub(crate) struct FunctionBuilder {
    current_function: Function,
    current_block: BasicBlockId,
    finished_functions: Vec<Function>,
}

impl FunctionBuilder {
    pub(crate) fn new(function_name: String, function_id: FunctionId) -> Self {
        let new_function = Function::new(function_name, function_id);
        let current_block = new_function.entry_block();

        Self { current_function: new_function, current_block, finished_functions: Vec::new() }
    }

    /// Finish the current function and create a new function
    pub(crate) fn new_function(&mut self, name: String, function_id: FunctionId) {
        let new_function = Function::new(name, function_id);
        self.current_block = new_function.entry_block();

        let old_function = std::mem::replace(&mut self.current_function, new_function);
        self.finished_functions.push(old_function);
    }

    pub(crate) fn finish(mut self) -> Ssa {
        self.finished_functions.push(self.current_function);
        Ssa::new(self.finished_functions)
    }

    pub(crate) fn add_parameter(&mut self, typ: Type) -> ValueId {
        let entry = self.current_function.entry_block();
        self.current_function.dfg.add_block_parameter(entry, typ)
    }

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(
        &mut self,
        value: impl Into<FieldElement>,
        typ: Type,
    ) -> ValueId {
        self.current_function.dfg.make_constant(value.into(), typ)
    }

    /// Insert a numeric constant into the current function of type Field
    pub(crate) fn field_constant(&mut self, value: impl Into<FieldElement>) -> ValueId {
        self.numeric_constant(value.into(), Type::field())
    }

    pub(crate) fn type_of_value(&self, value: ValueId) -> Type {
        self.current_function.dfg.type_of_value(value)
    }

    pub(crate) fn insert_block(&mut self) -> BasicBlockId {
        self.current_function.dfg.make_block()
    }

    pub(crate) fn add_block_parameter(&mut self, block: BasicBlockId, typ: Type) -> ValueId {
        self.current_function.dfg.add_block_parameter(block, typ)
    }

    /// Inserts a new instruction at the end of the current block and returns its results
    fn insert_instruction(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> &[ValueId] {
        let id = self.current_function.dfg.make_instruction(instruction, ctrl_typevars);
        self.current_function.dfg.insert_instruction_in_block(self.current_block, id);
        self.current_function.dfg.instruction_results(id)
    }

    /// Switch to inserting instructions in the given block.
    /// Expects the given block to be within the same function. If you want to insert
    /// instructions into a new function, call new_function instead.
    pub(crate) fn switch_to_block(&mut self, block: BasicBlockId) {
        self.current_block = block;
    }

    /// Insert an allocate instruction at the end of the current block, allocating the
    /// given amount of field elements. Returns the result of the allocate instruction,
    /// which is always a Reference to the allocated data.
    pub(crate) fn insert_allocate(&mut self, size_to_allocate: u32) -> ValueId {
        self.insert_instruction(Instruction::Allocate { size: size_to_allocate }, None)[0]
    }

    /// Insert a Load instruction at the end of the current block, loading from the given offset
    /// of the given address which should point to a previous Allocate instruction. Note that
    /// this is limited to loading a single value. Loading multiple values (such as a tuple)
    /// will require multiple loads.
    /// 'offset' is in units of FieldElements here. So loading the fourth FieldElement stored in
    /// an array will have an offset of 3.
    /// Returns the element that was loaded.
    pub(crate) fn insert_load(
        &mut self,
        mut address: ValueId,
        offset: ValueId,
        type_to_load: Type,
    ) -> ValueId {
        if let Some(offset) = self.current_function.dfg.get_numeric_constant(offset) {
            if !offset.is_zero() {
                let offset = self.field_constant(offset);
                address = self.insert_binary(address, BinaryOp::Add, offset);
            }
        };
        self.insert_instruction(Instruction::Load { address }, Some(vec![type_to_load]))[0]
    }

    /// Insert a Store instruction at the end of the current block, storing the given element
    /// at the given address. Expects that the address points somewhere
    /// within a previous Allocate instruction.
    pub(crate) fn insert_store(&mut self, address: ValueId, value: ValueId) {
        self.insert_instruction(Instruction::Store { address, value }, None);
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    pub(crate) fn insert_binary(
        &mut self,
        lhs: ValueId,
        operator: BinaryOp,
        rhs: ValueId,
    ) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.insert_instruction(instruction, None)[0]
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.insert_instruction(Instruction::Not(rhs), None)[0]
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: Type) -> ValueId {
        self.insert_instruction(Instruction::Cast(value, typ), None)[0]
    }

    /// Insert a constrain instruction at the end of the current block.
    pub(crate) fn insert_constrain(&mut self, boolean: ValueId) {
        self.insert_instruction(Instruction::Constrain(boolean), None);
    }

    /// Insert a call instruction a the end of the current block and return
    /// the results of the call.
    pub(crate) fn insert_call(
        &mut self,
        func: ValueId,
        arguments: Vec<ValueId>,
        result_types: Vec<Type>,
    ) -> &[ValueId] {
        self.insert_instruction(Instruction::Call { func, arguments }, Some(result_types))
    }

    /// Terminates the current block with the given terminator instruction
    fn terminate_block_with(&mut self, terminator: TerminatorInstruction) {
        self.current_function.dfg.set_block_terminator(self.current_block, terminator);
    }

    /// Terminate the current block with a jmp instruction to jmp to the given
    /// block with the given arguments.
    pub(crate) fn terminate_with_jmp(
        &mut self,
        destination: BasicBlockId,
        arguments: Vec<ValueId>,
    ) {
        self.terminate_block_with(TerminatorInstruction::Jmp { destination, arguments });
    }

    /// Terminate the current block with a jmpif instruction to jmp with the given arguments
    /// block with the given arguments.
    pub(crate) fn terminate_with_jmpif(
        &mut self,
        condition: ValueId,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
    ) {
        self.terminate_block_with(TerminatorInstruction::JmpIf {
            condition,
            then_destination,
            else_destination,
        });
    }

    /// Terminate the current block with a return instruction
    pub(crate) fn terminate_with_return(&mut self, return_values: Vec<ValueId>) {
        self.terminate_block_with(TerminatorInstruction::Return { return_values });
    }

    /// Returns a ValueId pointing to the given function or imports the function
    /// into the current function if it was not already, and returns that ID.
    pub(crate) fn import_function(&mut self, function: FunctionId) -> ValueId {
        self.current_function.dfg.import_function(function)
    }

    /// Retrieve a value reference to the given intrinsic operation.
    /// Returns None if there is no intrinsic matching the given name.
    pub(crate) fn import_intrinsic(&mut self, name: &str) -> Option<ValueId> {
        Intrinsic::lookup(name)
            .map(|intrinsic| self.current_function.dfg.import_intrinsic(intrinsic))
    }

    /// Removes the given instruction from the current block or panic otherwise.
    pub(crate) fn remove_instruction_from_current_block(&mut self, instruction: InstructionId) {
        self.current_function.dfg[self.current_block].remove_instruction(instruction);
    }
}

impl std::ops::Index<ValueId> for FunctionBuilder {
    type Output = Value;

    fn index(&self, id: ValueId) -> &Self::Output {
        &self.current_function.dfg[id]
    }
}

impl std::ops::Index<InstructionId> for FunctionBuilder {
    type Output = Instruction;

    fn index(&self, id: InstructionId) -> &Self::Output {
        &self.current_function.dfg[id]
    }
}
