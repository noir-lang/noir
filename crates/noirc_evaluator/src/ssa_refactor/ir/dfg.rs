use super::{
    basic_block::{BasicBlock, BasicBlockId},
    constant::{NumericConstant, NumericConstantId},
    function::Signature,
    instruction::{Instruction, InstructionId},
    map::{DenseMap, Id, SecondaryMap, TwoWayMap},
    types::Type,
    value::{Value, ValueId},
};

use acvm::FieldElement;
use iter_extended::vecmap;

#[derive(Debug, Default)]
/// A convenience wrapper to store `Value`s.
pub(crate) struct ValueList(Vec<Id<Value>>);

impl ValueList {
    /// Inserts an element to the back of the list and
    /// returns the `position`
    pub(crate) fn push(&mut self, value: ValueId) -> usize {
        self.0.push(value);
        self.len() - 1
    }

    /// Returns the number of values in the list.
    fn len(&self) -> usize {
        self.0.len()
    }

    /// Removes all items from the list.
    fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns the ValueId's as a slice.
    pub(crate) fn as_slice(&self) -> &[ValueId] {
        &self.0
    }
}

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
    results: SecondaryMap<Instruction, ValueList>,

    /// Storage for all of the values defined in this
    /// function.
    values: DenseMap<Value>,

    /// Storage for all constants used within a function.
    /// Each constant is unique, attempting to insert the same constant
    /// twice will return the same ConstantId.
    constants: TwoWayMap<NumericConstant>,

    /// Function signatures of external methods
    signatures: DenseMap<Signature>,

    /// All blocks in a function
    blocks: DenseMap<BasicBlock>,
}

impl DataFlowGraph {
    /// Creates a new basic block with no parameters.
    /// After being created, the block is unreachable in the current function
    /// until another block is made to jump to it.
    pub(crate) fn new_block(&mut self) -> BasicBlockId {
        self.blocks.insert(BasicBlock::new(Vec::new()))
    }

    /// Creates a new basic block with the given parameters.
    /// After being created, the block is unreachable in the current function
    /// until another block is made to jump to it.
    pub(crate) fn new_block_with_parameters(
        &mut self,
        parameter_types: impl Iterator<Item = Type>,
    ) -> BasicBlockId {
        self.blocks.insert_with_id(|entry_block| {
            let parameters = vecmap(parameter_types.enumerate(), |(position, typ)| {
                self.values.insert(Value::Param { block: entry_block, position, typ })
            });

            BasicBlock::new(parameters)
        })
    }

    pub(crate) fn block_parameters(&self, block: BasicBlockId) -> &[ValueId] {
        self.blocks[block].parameters()
    }

    /// Inserts a new instruction into the DFG.
    /// This does not add the instruction to the block or populate the instruction's result list
    pub(crate) fn make_instruction(&mut self, instruction_data: Instruction) -> InstructionId {
        let id = self.instructions.insert(instruction_data);
        // Create a new vector to store the potential results for the instruction.
        self.results.insert(id, Default::default());
        id
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

    /// Attaches results to the instruction, clearing any previous results.
    ///
    /// Returns the results of the instruction
    pub(crate) fn make_instruction_results(
        &mut self,
        instruction_id: InstructionId,
        ctrl_typevar: Type,
    ) -> &[ValueId] {
        // Clear all of the results instructions associated with this
        // instruction.
        self.results.get_mut(&instruction_id).expect("all instructions should have a `result` allocation when instruction was added to the DFG").clear();

        // Get all of the types that this instruction produces
        // and append them as results.
        let typs = self.instruction_result_types(instruction_id, ctrl_typevar);

        for typ in typs {
            self.append_result(instruction_id, typ);
        }

        self.results.get_mut(&instruction_id)
            .expect("all instructions should have a `result` allocation when instruction was added to the DFG")
            .as_slice()
    }

    /// Return the result types of this instruction.
    ///
    /// For example, an addition instruction will return
    /// one type which is the type of the operands involved.
    /// This is the `ctrl_typevar` in this case.
    fn instruction_result_types(
        &self,
        instruction_id: InstructionId,
        ctrl_typevar: Type,
    ) -> Vec<Type> {
        // Check if it is a call instruction. If so, we don't support that yet
        let ins_data = &self.instructions[instruction_id];
        match ins_data {
            Instruction::Call { .. } => todo!("function calls are not supported yet"),
            ins => ins.return_types(ctrl_typevar),
        }
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
        let actual_res_position = results.push(value_id);
        assert_eq!(actual_res_position, expected_res_position);
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

    pub(crate) fn add_block_parameter(&mut self, block_id: BasicBlockId, typ: Type) -> Id<Value> {
        let block = &mut self.blocks[block_id];
        let position = block.parameters().len();
        let parameter = self.values.insert(Value::Param { block: block_id, position, typ });
        block.add_parameter(parameter);
        parameter
    }

    pub(crate) fn insert_instruction_in_block(
        &mut self,
        block: BasicBlockId,
        instruction: InstructionId,
    ) {
        self.blocks[block].insert_instruction(instruction);
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

#[cfg(test)]
mod tests {
    use super::DataFlowGraph;
    use crate::ssa_refactor::ir::{
        instruction::Instruction,
        types::{NumericType, Type},
    };

    #[test]
    fn make_instruction() {
        let mut dfg = DataFlowGraph::default();
        let ins = Instruction::Allocate { size: 20 };
        let ins_id = dfg.make_instruction(ins);

        let num_results =
            dfg.make_instruction_results(ins_id, Type::Numeric(NumericType::NativeField)).len();

        let results = dfg.instruction_results(ins_id);
        assert_eq!(results.len(), num_results);
    }
}
