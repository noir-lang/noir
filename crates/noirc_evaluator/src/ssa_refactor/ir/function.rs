use crate::ssa_refactor::basic_block::{BasicBlock, BasicBlockId};

use super::instruction::Instruction;

use noirc_errors::Location;
use std::collections::HashMap;

/// A function holds a list of instructions.
/// These instructions are further grouped into
/// Basic blocks
#[derive(Debug)]
pub(crate) struct Function {
    /// Basic blocks associated to this particular function
    basic_blocks: HashMap<BasicBlockId, BasicBlock>,

    /// Maps instructions to source locations
    source_locations: HashMap<Instruction, Location>,

    /// The first basic block in the function
    entry_block: BasicBlockId,
}

/// FunctionId is a reference for a function
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct FunctionId(pub(crate) u32);
