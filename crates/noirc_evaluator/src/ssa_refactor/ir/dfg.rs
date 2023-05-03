use std::collections::HashMap;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    constant::{NumericConstant, NumericConstantId},
    function::{FunctionId, Signature},
    instruction::{
        Instruction, InstructionId, InstructionResultType, Intrinsic, TerminatorInstruction,
    },
    map::{DenseMap, Id, TwoWayMap},
    types::Type,
    value::{Value, ValueId},
};

use acvm::FieldElement;

/// The DataFlowGraph contains most of the actual data in a function including
/// its blocks, instructions, and values. This struct is largely responsible for
/// owning most data in a function and handing out Ids to this data that can be
/// shared without worrying about ownership.
#[derive(Debug, Default)]
pub(crate) struct DataFlowGraph {
    /// All of the instructions in a function
    instructions: DenseMap<Instruction>,

    /// Stores the results for a particular instruction.
    ///
    /// An instruction may return multiple values
    /// and for this, we will also use the cranelift strategy
    /// to fetch them via indices.
    ///
    /// Currently, we need to define them in a better way
    /// Call instructions require the func signature, but
    /// other instructions may need some more reading on my part
    results: HashMap<InstructionId, Vec<ValueId>>,

    /// Storage for all of the values defined in this
    /// function.
    values: DenseMap<Value>,

    /// Storage for all constants used within a function.
    /// Each constant is unique, attempting to insert the same constant
    /// twice will return the same ConstantId.
    constants: TwoWayMap<NumericConstant>,

    /// Contains each function that has been imported into the current function.
    /// Each function's Value::Function is uniqued here so any given FunctionId
    /// will always have the same ValueId within this function.
    functions: HashMap<FunctionId, ValueId>,

    /// Contains each intrinsic that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given intrinsic is always
    /// represented by only 1 ValueId within this function.
    intrinsics: HashMap<Intrinsic, ValueId>,

    /// Function signatures of external methods
    signatures: DenseMap<Signature>,

    /// All blocks in a function
    blocks: DenseMap<BasicBlock>,
}

impl DataFlowGraph {
    /// Creates a new basic block with no parameters.
    /// After being created, the block is unreachable in the current function
    /// until another block is made to jump to it.
    pub(crate) fn make_block(&mut self) -> BasicBlockId {
        self.blocks.insert(BasicBlock::new(Vec::new()))
    }

    /// Get an iterator over references to each basic block within the dfg, paired with the basic
    /// block's id.
    ///
    /// The pairs are order by id, which is not guaranteed to be meaningful.
    pub(crate) fn basic_blocks_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (BasicBlockId, &BasicBlock)> {
        self.blocks.iter()
    }

    pub(crate) fn block_parameters(&self, block: BasicBlockId) -> &[ValueId] {
        self.blocks[block].parameters()
    }

    /// Inserts a new instruction into the DFG.
    /// This does not add the instruction to the block.
    /// Returns the id of the new instruction and its results.
    ///
    /// Populates the instruction's results with the given ctrl_typevars if the instruction
    /// is a Load, Call, or Intrinsic. Otherwise the instruction's results will be known
    /// by the instruction itself and None can safely be passed for this parameter.
    pub(crate) fn make_instruction(
        &mut self,
        instruction_data: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> InstructionId {
        let id = self.instructions.insert(instruction_data);
        self.make_instruction_results(id, ctrl_typevars);
        id
    }

    /// Replace an instruction id with another.
    ///
    /// This function should generally be avoided if possible in favor of inserting new
    /// instructions since it does not check whether the instruction results of the removed
    /// instruction are still in use. Users of this function thus need to ensure the old
    /// instruction's results are no longer in use or are otherwise compatible with the
    /// new instruction's result count and types.
    pub(crate) fn replace_instruction(&mut self, id: Id<Instruction>, instruction: Instruction) {
        self.instructions[id] = instruction;
    }

    /// Insert a value into the dfg's storage and return an id to reference it.
    /// Until the value is used in an instruction it is unreachable.
    pub(crate) fn make_value(&mut self, value: Value) -> ValueId {
        self.values.insert(value)
    }

    /// Creates a new constant value, or returns the Id to an existing one if
    /// one already exists.
    pub(crate) fn make_constant(&mut self, value: FieldElement, typ: Type) -> ValueId {
        let constant = self.constants.insert(NumericConstant::new(value));
        self.values.insert(Value::NumericConstant { constant, typ })
    }

    /// Gets or creates a ValueId for the given FunctionId.
    pub(crate) fn import_function(&mut self, function: FunctionId) -> ValueId {
        if let Some(existing) = self.functions.get(&function) {
            return *existing;
        }
        self.values.insert(Value::Function(function))
    }

    /// Gets or creates a ValueId for the given Intrinsic.
    pub(crate) fn import_intrinsic(&mut self, intrinsic: Intrinsic) -> ValueId {
        if let Some(existing) = self.intrinsics.get(&intrinsic) {
            return *existing;
        }
        self.values.insert(Value::Intrinsic(intrinsic))
    }

    /// Attaches results to the instruction, clearing any previous results.
    ///
    /// This does not normally need to be called manually as it is called within
    /// make_instruction automatically.
    ///
    /// Returns the results of the instruction
    pub(crate) fn make_instruction_results(
        &mut self,
        instruction_id: InstructionId,
        ctrl_typevars: Option<Vec<Type>>,
    ) {
        self.results.insert(instruction_id, Default::default());

        // Get all of the types that this instruction produces
        // and append them as results.
        let typs = self.instruction_result_types(instruction_id, ctrl_typevars);

        for typ in typs {
            self.append_result(instruction_id, typ);
        }
    }

    /// Return the result types of this instruction.
    ///
    /// In the case of Load, Call, and Intrinsic, the function's result
    /// type may be unknown. In this case, the given ctrl_typevars are returned instead.
    /// ctrl_typevars is taken in as an Option since it is common to omit them when getting
    /// the type of an instruction that does not require them. Compared to passing an empty Vec,
    /// Option has the benefit of panicking if it is accidentally used for a Call instruction,
    /// rather than silently returning the empty Vec and continuing.
    fn instruction_result_types(
        &self,
        instruction_id: InstructionId,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> Vec<Type> {
        let instruction = &self.instructions[instruction_id];
        match instruction.result_type() {
            InstructionResultType::Known(typ) => vec![typ],
            InstructionResultType::Operand(value) => vec![self.type_of_value(value)],
            InstructionResultType::None => vec![],
            InstructionResultType::Unknown => {
                ctrl_typevars.expect("Control typevars required but not given")
            }
        }
    }

    /// Returns the type of a given value
    pub(crate) fn type_of_value(&self, value: ValueId) -> Type {
        self.values[value].get_type()
    }

    /// Appends a result type to the instruction.
    pub(crate) fn append_result(&mut self, instruction_id: InstructionId, typ: Type) -> ValueId {
        let results = self.results.get_mut(&instruction_id).unwrap();
        let expected_res_position = results.len();

        let value_id = self.values.insert(Value::Instruction {
            typ,
            position: expected_res_position,
            instruction: instruction_id,
        });

        // Add value to the list of results for this instruction
        results.push(value_id);
        value_id
    }

    /// Returns the number of instructions
    /// inserted into functions.
    pub(crate) fn num_instructions(&self) -> usize {
        self.instructions.len()
    }

    /// Returns all of result values which are attached to this instruction.
    pub(crate) fn instruction_results(&self, instruction_id: InstructionId) -> &[ValueId] {
        self.results.get(&instruction_id).expect("expected a list of Values").as_slice()
    }

    /// Add a parameter to the given block
    pub(crate) fn add_block_parameter(&mut self, block_id: BasicBlockId, typ: Type) -> Id<Value> {
        let block = &mut self.blocks[block_id];
        let position = block.parameters().len();
        let parameter = self.values.insert(Value::Param { block: block_id, position, typ });
        block.add_parameter(parameter);
        parameter
    }

    /// Insert an instruction at the end of a given block.
    /// If the block already has a terminator, the instruction is inserted before the terminator.
    pub(crate) fn insert_instruction_in_block(
        &mut self,
        block: BasicBlockId,
        instruction: InstructionId,
    ) {
        self.blocks[block].insert_instruction(instruction);
    }

    /// Returns the field element represented by this value if it is a numeric constant.
    /// Returns None if the given value is not a numeric constant.
    pub(crate) fn get_numeric_constant(&self, value: Id<Value>) -> Option<FieldElement> {
        self.get_numeric_constant_with_type(value).map(|(value, _typ)| value)
    }

    /// Returns the field element and type represented by this value if it is a numeric constant.
    /// Returns None if the given value is not a numeric constant.
    pub(crate) fn get_numeric_constant_with_type(
        &self,
        value: Id<Value>,
    ) -> Option<(FieldElement, Type)> {
        match self.values[value] {
            Value::NumericConstant { constant, typ } => Some((self[constant].value(), typ)),
            _ => None,
        }
    }

    /// Sets the terminator instruction for the given basic block
    pub(crate) fn set_block_terminator(
        &mut self,
        block: BasicBlockId,
        terminator: TerminatorInstruction,
    ) {
        self.blocks[block].set_terminator(terminator);
    }

    /// Splits the given block in two at the given instruction, returning the Id of the new block.
    /// This will remove the given instruction and place every instruction after it into a new block
    /// with the same terminator as the old block. The old block is modified to stop
    /// before the instruction to remove and to unconditionally branch to the new block.
    /// This function is useful during function inlining to remove the call instruction
    /// while opening a spot at the end of the current block to insert instructions into.
    ///
    /// Example (before):
    ///   block1: a; b; c; d; e; jmp block5
    ///
    /// After self.split_block_at(block1, c):
    ///   block1: a; b; jmp block2
    ///   block2: d; e; jmp block5
    pub(crate) fn split_block_at(&mut self, block: BasicBlockId, instruction_to_remove: InstructionId) -> BasicBlockId {
        let split_block = &mut self.blocks[block];

        let mut instructions = split_block.instructions().iter();
        let index = instructions.position(|id| *id == instruction_to_remove).unwrap_or_else(|| {
            panic!("No instruction found with id {instruction_to_remove:?} in block {block:?}")
        });

        let instructions = split_block.instructions_mut().drain(index..).collect();
        split_block.remove_instruction(instruction_to_remove);

        self.blocks.insert(BasicBlock::new(instructions))
    }
}

impl std::ops::Index<InstructionId> for DataFlowGraph {
    type Output = Instruction;
    fn index(&self, id: InstructionId) -> &Self::Output {
        &self.instructions[id]
    }
}

impl std::ops::Index<ValueId> for DataFlowGraph {
    type Output = Value;
    fn index(&self, id: ValueId) -> &Self::Output {
        &self.values[id]
    }
}

impl std::ops::Index<NumericConstantId> for DataFlowGraph {
    type Output = NumericConstant;
    fn index(&self, id: NumericConstantId) -> &Self::Output {
        &self.constants[id]
    }
}

impl std::ops::Index<BasicBlockId> for DataFlowGraph {
    type Output = BasicBlock;
    fn index(&self, id: BasicBlockId) -> &Self::Output {
        &self.blocks[id]
    }
}

impl std::ops::IndexMut<BasicBlockId> for DataFlowGraph {
    /// Get a mutable reference to a function's basic block for the given id.
    fn index_mut(&mut self, id: BasicBlockId) -> &mut BasicBlock {
        &mut self.blocks[id]
    }
}

#[cfg(test)]
mod tests {
    use super::DataFlowGraph;
    use crate::ssa_refactor::ir::instruction::Instruction;

    #[test]
    fn make_instruction() {
        let mut dfg = DataFlowGraph::default();
        let ins = Instruction::Allocate { size: 20 };
        let ins_id = dfg.make_instruction(ins, None);

        let results = dfg.instruction_results(ins_id);
        assert_eq!(results.len(), 1);
    }
}
