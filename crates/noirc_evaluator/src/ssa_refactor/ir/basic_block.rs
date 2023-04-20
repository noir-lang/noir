use super::{
    instruction::{Instruction, InstructionId, TerminatorInstruction},
    map::Id,
};

/// A Basic block is a maximal collection of instructions
/// such that there are only jumps at the end of block
/// and one can only enter the block from the beginning.
///
/// This means that if one instruction is executed in a basic
/// block, then all instructions are executed. ie single-entry single-exit.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub(crate) struct BasicBlock {
    /// Parameters to the basic block.
    /// The relevant values can be created with this block's id
    /// and the index of the parameter, so we only need to remember
    /// the number of parameters here.
    parameter_count: usize,

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
    pub(super) fn new(parameter_count: usize) -> Self {
        Self { parameter_count, instructions: Vec::new(), is_sealed: false, terminator: None }
    }
}
