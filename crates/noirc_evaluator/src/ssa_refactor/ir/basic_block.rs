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

    /// A basic block is considered sealed
    /// if no further predecessors will be added to it.
    /// Since only filled blocks can have successors,
    /// predecessors are always filled.
    is_sealed: bool,

    /// The terminating instruction for the basic block.
    ///
    /// This will be a control flow instruction. This is only
    /// None if the block is still being constructed.
    terminator: Option<TerminatorInstruction>,
}

/// An identifier for a Basic Block.
pub(crate) type BasicBlockId = Id<BasicBlock>;

impl BasicBlock {
    pub(crate) fn new(parameters: Vec<ValueId>) -> Self {
        Self { parameters, instructions: Vec::new(), is_sealed: false, terminator: None }
    }

    pub(crate) fn parameters(&self) -> &[ValueId] {
        &self.parameters
    }

    pub(crate) fn add_parameter(&mut self, parameter: ValueId) {
        self.parameters.push(parameter);
    }

    /// Insert an instruction at the end of this block
    pub(crate) fn insert_instruction(&mut self, instruction: InstructionId) {
        self.instructions.push(instruction);
    }

    pub(crate) fn instructions(&self) -> &[InstructionId] {
        &self.instructions
    }

    pub(crate) fn set_terminator(&mut self, terminator: TerminatorInstruction) {
        self.terminator = Some(terminator);
    }

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
}
