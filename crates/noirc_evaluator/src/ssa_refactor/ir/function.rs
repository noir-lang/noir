use std::ops::Index;

use super::basic_block::{BasicBlock, BasicBlockId};
use super::dfg::DataFlowGraph;
use super::instruction::Instruction;
use super::map::{DenseMap, Id, SecondaryMap};
use super::types::Type;
use super::value::Value;

use iter_extended::vecmap;
use noirc_errors::Location;

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
        let mut dfg = DataFlowGraph::default();
        let mut basic_blocks = DenseMap::default();

        // The parameters for each function are stored as the block parameters
        // of the function's entry block
        let entry_block = basic_blocks.insert_with_id(|entry_block| {
            // TODO: Give each parameter its correct type
            let parameters = vecmap(0..parameter_count, |i| {
                dfg.make_value(Value::Param { block: entry_block, position: i, typ: Type::Unit })
            });

            BasicBlock::new(parameters)
        });

        Self { basic_blocks, source_locations: SecondaryMap::new(), entry_block, dfg }
    }

    pub(crate) fn entry_block(&self) -> BasicBlockId {
        self.entry_block
    }
}

impl Function {
    /// Get an iterator over the ids of the basic blocks within the function.
    ///
    /// The ids are iterated in no meaningful order.
    pub(crate) fn basic_block_ids_iter(&self) -> impl Iterator<Item = BasicBlockId> {
        self.basic_blocks.ids_iter()
    }
}

impl Index<BasicBlockId> for Function {
    type Output = BasicBlock;
    /// Get a function's basic block for the given id.
    fn index(&self, id: BasicBlockId) -> &BasicBlock {
        &self.basic_blocks[id]
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
