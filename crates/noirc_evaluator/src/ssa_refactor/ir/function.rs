use super::basic_block::{BasicBlock, BasicBlockId};
use super::dfg::DataFlowGraph;
use super::instruction::{Instruction, InstructionId};
use super::map::{DenseMap, Id, SecondaryMap};
use super::types::Type;

use noirc_errors::Location;
use std::collections::HashMap;

/// A function holds a list of instructions.
/// These instructions are further grouped into Basic blocks
///
/// Like Crane-lift all functions outside of the current function is seen as external.
/// To reference external functions, one must first import the function signature
/// into the current function's context.
#[derive(Debug)]
pub(crate) struct Function {
    /// Basic blocks associated to this particular function
    basic_blocks: DenseMap<BasicBlock>,

    /// Maps instructions to source locations
    source_locations: SecondaryMap<Instruction, Location>,

    /// The first basic block in the function
    entry_block: BasicBlockId,

    dfg: DataFlowGraph,
}

impl Function {
    pub(crate) fn new(parameter_count: usize) -> Self {
        let mut basic_blocks = DenseMap::default();
        let entry_block = basic_blocks.insert(BasicBlock::new(parameter_count));

        Self {
            basic_blocks,
            source_locations: SecondaryMap::new(),
            entry_block,
            dfg: DataFlowGraph::default(),
        }
    }

    pub(crate) fn entry_block(&self) -> BasicBlockId {
        self.entry_block
    }
}

/// FunctionId is a reference for a function
pub(crate) type FunctionId = Id<Function>;

#[derive(Debug, Default, Clone)]
pub(crate) struct Signature {
    pub(crate) params: Vec<Type>,
    pub(crate) returns: Vec<Type>,
}

#[test]
fn sign_smoke() {
    let mut signature = Signature::default();

    signature.params.push(Type::Numeric(super::types::NumericType::NativeField));
    signature.returns.push(Type::Numeric(super::types::NumericType::Unsigned { bit_size: 32 }));
}
