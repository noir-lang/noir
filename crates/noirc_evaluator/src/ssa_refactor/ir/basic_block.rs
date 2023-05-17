use super::{
    instruction::{InstructionId, TerminatorInstruction},
    map::Id,
    value::ValueId,
};

/// A Basic block is a maximal collection of instructions
/// such that there are only jumps at the end of block
/// and one can only enter the block from the beginning.
///
/// This means that if one instruction is executed in a basic
/// block, then all instructions are executed. ie single-entry single-exit.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct BasicBlock {
    /// Parameters to the basic block.
    parameters: Vec<ValueId>,

    /// Instructions in the basic block.
    instructions: Vec<InstructionId>,

    /// The terminating instruction for the basic block.
    ///
    /// This will be a control flow instruction. This is only
    /// None if the block is still being constructed.
    terminator: Option<TerminatorInstruction>,
}

/// An identifier for a Basic Block.
pub(crate) type BasicBlockId = Id<BasicBlock>;

impl BasicBlock {
    /// Create a new BasicBlock with the given instructions.
    /// Parameters can also be added later via BasicBlock::add_parameter
    pub(crate) fn new(instructions: Vec<InstructionId>) -> Self {
        Self { parameters: Vec::new(), instructions, terminator: None }
    }

    /// Returns the parameters of this block
    pub(crate) fn parameters(&self) -> &[ValueId] {
        &self.parameters
    }

    /// Adds a parameter to this BasicBlock.
    /// Expects that the ValueId given should refer to a Value::Param
    /// instance with its position equal to self.parameters.len().
    pub(crate) fn add_parameter(&mut self, parameter: ValueId) {
        self.parameters.push(parameter);
    }

    /// Insert an instruction at the end of this block
    pub(crate) fn insert_instruction(&mut self, instruction: InstructionId) {
        self.instructions.push(instruction);
    }

    /// Retrieve a reference to all instructions in this block.
    pub(crate) fn instructions(&self) -> &[InstructionId] {
        &self.instructions
    }

    /// Retrieve a mutable reference to all instructions in this block.
    pub(crate) fn instructions_mut(&mut self) -> &mut Vec<InstructionId> {
        &mut self.instructions
    }

    /// Sets the terminator instruction of this block.
    ///
    /// A properly-constructed block will always terminate with a TerminatorInstruction -
    /// which either jumps to another block or returns from the current function. A block
    /// will only have no terminator if it is still under construction.
    pub(crate) fn set_terminator(&mut self, terminator: TerminatorInstruction) {
        self.terminator = Some(terminator);
    }

    /// Returns the terminator of this block.
    ///
    /// Once this block has finished construction, this is expected to always be Some.
    pub(crate) fn terminator(&self) -> Option<&TerminatorInstruction> {
        self.terminator.as_ref()
    }

    /// Iterate over all the successors of the currently block, as determined by
    /// the blocks jumped to in the terminator instruction. If there is no terminator
    /// instruction yet, this will iterate 0 times.
    pub(crate) fn successors(
        &self,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + DoubleEndedIterator {
        match &self.terminator {
            Some(TerminatorInstruction::Jmp { destination, .. }) => vec![*destination].into_iter(),
            Some(TerminatorInstruction::JmpIf { then_destination, else_destination, .. }) => {
                vec![*then_destination, *else_destination].into_iter()
            }
            Some(TerminatorInstruction::Return { .. }) => vec![].into_iter(),
            None => vec![].into_iter(),
        }
    }

    /// Removes the given instruction from this block if present or panics otherwise.
    pub(crate) fn remove_instruction(&mut self, instruction: InstructionId) {
        // Iterate in reverse here as an optimization since remove_instruction is most
        // often called to remove instructions at the end of a block.
        let index =
            self.instructions.iter().rev().position(|id| *id == instruction).unwrap_or_else(|| {
                panic!("remove_instruction: No such instruction {instruction:?} in block")
            });
        self.instructions.remove(index);
    }
}
