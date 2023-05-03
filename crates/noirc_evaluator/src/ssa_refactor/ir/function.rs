use std::collections::HashMap;

use super::basic_block::BasicBlockId;
use super::dfg::DataFlowGraph;
use super::instruction::InstructionId;
use super::map::Id;
use super::types::Type;

use noirc_errors::Location;

/// A function holds a list of instructions.
/// These instructions are further grouped into Basic blocks
///
/// Like Crane-lift all functions outside of the current function is seen as external.
/// To reference external functions, one must first import the function signature
/// into the current function's context.
#[derive(Debug)]
pub(crate) struct Function {
    /// Maps instructions to source locations
    source_locations: HashMap<InstructionId, Location>,

    /// The first basic block in the function
    entry_block: BasicBlockId,

    /// Name of the function for debugging only
    name: String,

    id: FunctionId,

    pub(crate) dfg: DataFlowGraph,
}

impl Function {
    /// Creates a new function with an automatically inserted entry block.
    ///
    /// Note that any parameters to the function must be manually added later.
    pub(crate) fn new(name: String, id: FunctionId) -> Self {
        let mut dfg = DataFlowGraph::default();
        let entry_block = dfg.make_block();
        Self { name, source_locations: HashMap::new(), id, entry_block, dfg }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn id(&self) -> FunctionId {
        self.id
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

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::printer::display_function(self, f)
    }
}

#[test]
fn sign_smoke() {
    let mut signature = Signature::default();

    signature.params.push(Type::Numeric(super::types::NumericType::NativeField));
    signature.returns.push(Type::Numeric(super::types::NumericType::Unsigned { bit_size: 32 }));
}
