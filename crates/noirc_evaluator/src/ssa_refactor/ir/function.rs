use crate::ssa_refactor::basic_block::{BasicBlock, BasicBlockId};

use super::instruction::Instruction;

use noirc_errors::Location;
use std::{
    collections::{hash_map, HashMap},
    ops::Index,
};

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

impl Function {
    /// Get an iterator over the ids of the basic blocks within the function.
    ///
    /// The ids are iterated in no meaningful order.
    pub(crate) fn basic_block_ids_iter(&self) -> BasicBlockIdsIter {
        BasicBlockIdsIter(self.basic_blocks.keys())
    }
}

impl Index<BasicBlockId> for Function {
    type Output = BasicBlock;
    /// Get a function's basic block for the given id.
    fn index<'a>(&'a self, id: BasicBlockId) -> &'a BasicBlock {
        &self.basic_blocks[&id]
    }
}

/// An iterator over a function's basic block ids. The iterator type is `BasicBlockId`.
pub(crate) struct BasicBlockIdsIter<'a>(hash_map::Keys<'a, BasicBlockId, BasicBlock>);

impl<'a> Iterator for BasicBlockIdsIter<'a> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<BasicBlockId> {
        self.0.next().map(|k| *k)
    }
}

/// FunctionId is a reference for a function
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct FunctionId(pub(crate) u32);
