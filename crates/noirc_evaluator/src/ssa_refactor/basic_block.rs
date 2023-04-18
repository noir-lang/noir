use super::ir::instruction::{Instruction, TerminatorInstruction};

/// A Basic block is a maximal collection of instructions
/// such that there are only jumps at the end of block
/// and one can only enter the block from the beginning.
///
/// This means that if one instruction is executed in a basic
/// block, then all instructions are executed. ie single-entry single-exit.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct BasicBlock {
    /// Arguments to the basic block.
    phi_nodes: Vec<BlockArguments>,
    /// Instructions in the basic block.
    instructions: Vec<Instruction>,

    /// A basic block is considered sealed
    /// if no further predecessors will be added to it.
    /// Since only filled blocks can have successors,
    /// predecessors are always filled.
    is_sealed: bool,

    /// The terminating instruction for the basic block.
    ///
    /// This will be a control flow instruction.
    terminator: TerminatorInstruction,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// An identifier for a Basic Block.
pub(crate) struct BasicBlockId;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Arguments to the basic block.
/// We use the modern Crane-lift strategy
/// of representing phi nodes as basic block
/// arguments.
pub(crate) struct BlockArguments;
