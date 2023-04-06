use super::instructions::Instruction;
use crate::ssa_ref::{basic_blocks::BasicBlock, cfg::BasicBlockId};
use acvm::acir::BlackBoxFunc;
use noirc_errors::Location;
use std::collections::HashMap;

/// A function holds a list of instructions.
/// These instructions are further grouped into
/// Basic blocks
pub struct Function {
    /// Basic blocks associated to this particular function
    basic_blocks: HashMap<BasicBlockId, BasicBlock>,

    /// Maps instructions to source locations
    source_locations: HashMap<Instruction, Location>,

    /// The first basic block in the function
    entry_block: BasicBlockId,
}

/// FunctionId is a reference for a function
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);
