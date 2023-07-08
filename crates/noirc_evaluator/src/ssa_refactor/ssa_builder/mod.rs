use std::{borrow::Cow, rc::Rc};

use acvm::FieldElement;

use crate::ssa_refactor::ir::{
    basic_block::BasicBlockId,
    function::{Function, FunctionId},
    instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
    types::Type,
    value::{Value, ValueId},
};

use super::{
    ir::{
        basic_block::BasicBlock,
        dfg::InsertInstructionResult,
        function::RuntimeType,
        instruction::{InstructionId, Intrinsic},
        types::CompositeType,
    },
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
    /// The current function being built.
    /// A function builder can be re-used to build different functions.
    ///
    /// Note: What is the benefit of this vs creating a function builder per function?
    pub(super) current_function: Function,
    /// The current block being built.
    current_block: BasicBlockId,
    /// List of all of the functions that one has built.
    finished_functions: Vec<Function>,
}

impl FunctionBuilder {
    /// Creates a new FunctionBuilder to build the function with the given FunctionId.
    ///
    /// This creates the new function internally so there is no need to call .new_function()
    /// right after constructing a new FunctionBuilder.
    pub(crate) fn new(
        function_name: String,
        function_id: FunctionId,
        runtime: RuntimeType,
    ) -> Self {
        let mut new_function = Function::new(function_name, function_id);
        new_function.set_runtime(runtime);
        let current_block = new_function.entry_block();

        Self { current_function: new_function, current_block, finished_functions: Vec::new() }
    }

    /// Finish the current function by storing it in a `finished_function` vector and create a new function.
    ///
    /// A FunctionBuilder can always only work on one function at a time, so care
    /// should be taken not to finish a function that is still in progress by calling
    /// new_function before the current function is finished.
    fn new_function_with_type(
        &mut self,
        name: String,
        function_id: FunctionId,
        runtime_type: RuntimeType,
    ) {
        let mut new_function = Function::new(name, function_id);
        new_function.set_runtime(runtime_type);
        self.current_block = new_function.entry_block();

        let old_function = std::mem::replace(&mut self.current_function, new_function);
        self.finished_functions.push(old_function);
    }

    /// Finish the current function and create a new ACIR function.
    pub(crate) fn new_function(&mut self, name: String, function_id: FunctionId) {
        self.new_function_with_type(name, function_id, RuntimeType::Acir);
    }

    /// Finish the current function and create a new unconstrained function.
    pub(crate) fn new_brillig_function(&mut self, name: String, function_id: FunctionId) {
        self.new_function_with_type(name, function_id, RuntimeType::Brillig);
    }

    /// Consume the `FunctionBuilder` returning all the functions it has generated.
    pub(crate) fn finish(mut self) -> Ssa {
        self.finished_functions.push(self.current_function);
        Ssa::new(self.finished_functions)
    }

    /// Add a parameter to the current function with the given parameter type.
    /// Returns the newly-added parameter.
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

    /// Insert an array constant into the current function with the given element values.
    pub(crate) fn array_constant(
        &mut self,
        elements: im::Vector<ValueId>,
        // arrays are homogenous, so there is a single type that all elements abide by
        element_type: Rc<CompositeType>,
    ) -> ValueId {
        self.current_function.dfg.make_array(elements, element_type)
    }

    /// Returns the type of the given value.
    pub(crate) fn type_of_value(&self, value: ValueId) -> Type {
        self.current_function.dfg.type_of_value(value)
    }

    /// Insert a new block into the current function and return it.
    /// Note: This block is unreachable until another block is set to jump to it.
    pub(crate) fn insert_block(&mut self) -> BasicBlockId {
        self.current_function.dfg.make_block()
    }

    /// Adds a parameter with the given type to the given block.
    /// Returns the newly-added parameter.
    pub(crate) fn add_block_parameter(&mut self, block: BasicBlockId, typ: Type) -> ValueId {
        self.current_function.dfg.add_block_parameter(block, typ)
    }

    /// Returns the parameters of the given block in the current function.
    pub(crate) fn block_parameters(&self, block: BasicBlockId) -> &[ValueId] {
        self.current_function.dfg.block_parameters(block)
    }

    /// Inserts a new instruction at the end of the current block and returns its results
    ///
    /// TODO: Noted elsewhere this is also doing simplification, can we do this separately? tradeoffs?
    pub(crate) fn insert_instruction(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> InsertInstructionResult {
        self.current_function.dfg.insert_instruction_and_results(
            instruction,
            self.current_block,
            ctrl_typevars,
        )
    }

    /// Switch to inserting instructions in the given block.
    ///
    /// Expects the given block to be within the same function. If you want to insert
    /// instructions into a new function, call `new_function` instead.
    pub(crate) fn switch_to_block(&mut self, block: BasicBlockId) {
        self.current_block = block;
    }

    /// Returns the block currently being modified
    pub(crate) fn current_block(&mut self) -> BasicBlockId {
        self.current_block
    }

    /// Insert an allocate instruction at the end of the current block.
    /// Returns the result of the allocate instruction,
    /// which is always a Reference to the allocated data.
    pub(crate) fn insert_allocate(&mut self) -> ValueId {
        // TODO: Rust has .first on vectors which is confusing because
        // TODO that returns Option<T> whereas this returns T and expects there
        // TODO to be only one T.Perhaps change this to be `first_and_only`
        // TODO or something to not conflate this with .first, but also
        // TODO to convey the fact that its the only one; `exactly_one` ?
        self.insert_instruction(Instruction::Allocate, None).first()
    }

    /// Insert a Load instruction at the end of the current block.
    /// TODO: check previous description was not outdated.
    /// TODO: where do we check that `address` is a Reference?
    pub(crate) fn insert_load(&mut self, address: ValueId, type_to_load: Type) -> ValueId {
        self.insert_instruction(Instruction::Load { address }, Some(vec![type_to_load])).first()
    }

    /// Insert a Store instruction at the end of the current block.
    /// Storing the given element at the given address.
    /// TODO: check previous description was not outdated
    /// TODO: where do we check that `address` is a Reference?
    /// TODO: where do we check that `value` is the correct type for `address?
    /// TODO: Is `address` still correct terminology, given that we are using stores for mutable variables?
    pub(crate) fn insert_store(&mut self, address: ValueId, value: ValueId) {
        self.insert_instruction(Instruction::Store { address, value }, None);
    }

    /// Insert a binary instruction at the end of the current block.
    ///
    /// Returns the result of the binary instruction.
    ///
    /// All binary instructions return one result.
    pub(crate) fn insert_binary(
        &mut self,
        lhs: ValueId,
        operator: BinaryOp,
        rhs: ValueId,
    ) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.insert_instruction(instruction, None).first()
    }

    /// Insert a not instruction at the end of the current block.
    ///
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.insert_instruction(Instruction::Not(rhs), None).first()
    }

    /// Insert a cast instruction at the end of the current block.
    ///
    /// Returns the result of the cast instruction.
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: Type) -> ValueId {
        self.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a truncate instruction at the end of the current block.
    ///
    /// Returns the result of the truncate instruction.
    pub(crate) fn insert_truncate(
        &mut self,
        value: ValueId,
        bit_size: u32,
        max_bit_size: u32,
    ) -> ValueId {
        self.insert_instruction(Instruction::Truncate { value, bit_size, max_bit_size }, None)
            .first()
    }

    /// Insert a constrain instruction at the end of the current block.
    pub(crate) fn insert_constrain(&mut self, boolean: ValueId) {
        let results = self.insert_instruction(Instruction::Constrain(boolean), None).results();
        assert!(results.is_empty(), "constrain instructions do not return any results");
        // TODO: maybe put as a method on results like `.first`
    }

    /// Insert a call instruction at the end of the current block.
    ///
    /// Returns the results of the call.
    pub(crate) fn insert_call(
        &mut self,
        func: ValueId,
        arguments: Vec<ValueId>,
        result_types: Vec<Type>,
    ) -> Cow<[ValueId]> {
        self.insert_instruction(Instruction::Call { func, arguments }, Some(result_types)).results()
    }

    /// Insert array_get instruction at the end of the current block.
    ///
    /// This will extract an element from `array` at position `index`.
    ///
    /// Returns the `ValueId` of the fetched element.
    pub(crate) fn insert_array_get(
        &mut self,
        array: ValueId,
        index: ValueId,
        element_type: Type,
    ) -> ValueId {
        let element_type = Some(vec![element_type]);
        self.insert_instruction(Instruction::ArrayGet { array, index }, element_type).first()
    }

    /// Insert an array_set instruction to the end of the block.
    ///
    /// This will create a new array with the given index replaced with a `value`.
    /// Note: This will not modify `array`. Arrays are immutable in SSA.
    ///
    /// Returns the `ValueId` of the newly created array. This will be a reference.
    pub(crate) fn insert_array_set(
        &mut self,
        array: ValueId,
        index: ValueId,
        value: ValueId,
    ) -> ValueId {
        self.insert_instruction(Instruction::ArraySet { array, index, value }, None).first()
    }

    /// Terminates the current block with the given terminator instruction.
    ///
    /// This is used to denote the block being completed, since basic blocks
    /// can only have control flow instructions at the end of the block.
    fn terminate_block_with(&mut self, terminator: TerminatorInstruction) {
        self.current_function.dfg.set_block_terminator(self.current_block, terminator);
    }

    /// Terminate the current block with a jump instruction.
    ///
    /// Jump to the given block with the given arguments.
    pub(crate) fn terminate_with_jmp(
        &mut self,
        destination: BasicBlockId,
        arguments: Vec<ValueId>,
    ) {
        self.terminate_block_with(TerminatorInstruction::Jmp { destination, arguments });
    }

    /// Terminate the current block with a conditional jump instruction.
    ///
    /// Jump to the `then` block if the condition is true, else jump to
    /// the `else` block.
    ///
    /// TODO: where are the block arguments being supplied?
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

    /// Returns a `ValueId` pointing to the given function.
    ///
    /// If the function has already been imported, its `ValueId` will be returned,
    /// else a new `ValueId` will be returned.
    pub(crate) fn import_function(&mut self, function: FunctionId) -> ValueId {
        self.current_function.dfg.import_function(function)
    }

    /// Returns a `ValueId` pointing to the given oracle/foreign function.
    ///
    /// If the function has already been imported, its `ValueId` will be returned,
    /// else a new `ValueId` will be returned.
    pub(crate) fn import_foreign_function(&mut self, function: &str) -> ValueId {
        self.current_function.dfg.import_foreign_function(function)
    }

    /// Return a `ValueId` to the given intrinsic operation.
    pub(crate) fn import_intrinsic_id(&mut self, intrinsic: Intrinsic) -> ValueId {
        self.current_function.dfg.import_intrinsic(intrinsic)
    }

    /// Removes the given instruction from the current block or panics otherwise.
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

impl std::ops::Index<BasicBlockId> for FunctionBuilder {
    type Output = BasicBlock;

    fn index(&self, id: BasicBlockId) -> &Self::Output {
        &self.current_function.dfg[id]
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use acvm::FieldElement;

    use crate::ssa_refactor::ir::{
        function::RuntimeType,
        instruction::{Endian, Intrinsic},
        map::Id,
        types::Type,
        value::Value,
    };

    use super::FunctionBuilder;

    #[test]
    fn insert_constant_call() {
        // `bits` should be an array of constants [1, 1, 1, 0...]:
        // let x = 7;
        // let bits = x.to_le_bits(8);
        //
        // This is because when we insert an instruction, we are checking to see if that
        // instruction can be simplified. When the arguments are constant, we can compute this
        // at compile time.
        // TODO(NOTE): We can do this for blackbox functions too
        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let one = builder.numeric_constant(FieldElement::one(), Type::bool());
        let zero = builder.numeric_constant(FieldElement::zero(), Type::bool());

        let to_bits_id = builder.import_intrinsic_id(Intrinsic::ToBits(Endian::Little));
        let input = builder.numeric_constant(FieldElement::from(7_u128), Type::field());
        let length = builder.numeric_constant(FieldElement::from(8_u128), Type::field());
        let result_types = vec![Type::Array(Rc::new(vec![Type::bool()]), 8)];
        let call_result = builder.insert_call(to_bits_id, vec![input, length], result_types)[0];

        let array = match &builder.current_function.dfg[call_result] {
            Value::Array { array, .. } => array,
            _ => panic!(),
        };

        assert_eq!(array[0], one);
        assert_eq!(array[1], one);
        assert_eq!(array[2], one);
        assert_eq!(array[3], zero);
    }
}
