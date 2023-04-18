use std::collections::HashMap;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    ir::{
        extfunc::{SigRef, Signature},
        instruction::{Instruction, InstructionId, Instructions},
        types::Typ,
        value::{Value, ValueId},
    },
};

#[derive(Debug, Default)]
/// A convenience wrapper to store `Value`s.
pub(crate) struct ValueList(Vec<ValueId>);

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
        self.0.clear()
    }
    /// Returns the ValueId's as a slice.
    pub(crate) fn as_slice(&self) -> &[ValueId] {
        &self.0
    }
}
#[derive(Debug, Default)]
pub(crate) struct DataFlowGraph {
    /// All of the instructions in a function
    instructions: Instructions,

    /// Stores the results for a particular instruction.
    ///
    /// An instruction may return multiple values
    /// and for this, we will also use the cranelift strategy
    /// to fetch them via indices.
    ///
    /// Currently, we need to define them in a better way
    /// Call instructions require the func signature, but
    /// other instructions may need some more reading on my part
    results: HashMap<InstructionId, ValueList>,

    /// Storage for all of the values defined in this
    /// function.
    values: HashMap<ValueId, Value>,

    /// Function signatures of external methods
    signatures: HashMap<SigRef, Signature>,

    /// All blocks in a function
    blocks: HashMap<BasicBlockId, BasicBlock>,
}

impl DataFlowGraph {
    /// Creates a new `empty` basic block
    pub(crate) fn new_block(&mut self) -> BasicBlockId {
        todo!()
    }

    /// Inserts a new instruction into the DFG.
    pub(crate) fn make_instruction(&mut self, instruction_data: Instruction) -> InstructionId {
        let id = self.instructions.add_instruction(instruction_data);

        // Create a new vector to store the potential results
        // for the instruction.
        self.results.insert(id, Default::default());

        id
    }

    /// Attaches results to the instruction.
    ///
    /// Returns the number of results that this instruction
    /// produces.
    pub(crate) fn make_instruction_results(
        &mut self,
        instruction_id: InstructionId,
        ctrl_typevar: Typ,
    ) -> usize {
        // Clear all of the results instructions associated with this
        // instruction.
        self.results.get_mut(&instruction_id).expect("all instructions should have a `result` allocation when instruction was added to the DFG").clear();

        // Get all of the types that this instruction produces
        // and append them as results.
        let typs = self.instruction_result_types(instruction_id, ctrl_typevar);
        let num_typs = typs.len();

        for typ in typs {
            self.append_result(instruction_id, typ);
        }

        num_typs
    }

    /// Return the result types of this instruction.
    ///
    /// For example, an addition instruction will return
    /// one type which is the type of the operands involved.
    /// This is the `ctrl_typevar` in this case.
    fn instruction_result_types(
        &self,
        instruction_id: InstructionId,
        ctrl_typevar: Typ,
    ) -> Vec<Typ> {
        // Check if it is a call instruction. If so, we don't support that yet
        let ins_data = self.instructions.get_instruction(instruction_id);
        return match ins_data {
            Instruction::Call { .. } => todo!("function calls are not supported yet"),
            ins => ins.return_types(ctrl_typevar),
        };
    }

    /// Appends a result type to the instruction.
    pub(crate) fn append_result(&mut self, instruction_id: InstructionId, typ: Typ) -> ValueId {
        let next_value_id = self.next_value();

        // Add value to the list of results for this instruction
        let res_position = self.results.get_mut(&instruction_id).unwrap().push(next_value_id);

        self.make_value(Value::Instruction {
            typ,
            position: res_position as u16,
            instruction: instruction_id,
        })
    }

    /// Stores a value and returns its `ValueId` reference.
    fn make_value(&mut self, data: Value) -> ValueId {
        let next_value = self.next_value();

        self.values.insert(next_value, data);

        next_value
    }

    /// Returns the next `ValueId`
    fn next_value(&self) -> ValueId {
        ValueId(self.values.len() as u32)
    }

    /// Returns the number of instructions
    /// inserted into functions.
    pub(crate) fn num_instructions(&self) -> usize {
        self.instructions.num_instructions()
    }

    /// Returns all of result values which are attached to this instruction.
    pub(crate) fn instruction_results(&self, instruction_id: InstructionId) -> &[ValueId] {
        &self.results.get(&instruction_id).expect("expected a list of Values").as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::DataFlowGraph;
    use crate::ssa_refactor::ir::{
        instruction::Instruction,
        types::{NumericType, Typ},
    };
    use acvm::FieldElement;

    #[test]
    fn make_instruction() {
        let mut dfg = DataFlowGraph::default();
        let ins = Instruction::Immediate { value: FieldElement::from(0u128) };
        let ins_id = dfg.make_instruction(ins);

        let num_results =
            dfg.make_instruction_results(ins_id, Typ::Numeric(NumericType::NativeField));

        let results = dfg.instruction_results(ins_id);

        assert_eq!(results.len(), num_results);
    }
}
