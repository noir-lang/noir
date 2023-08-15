use std::collections::BTreeSet;

use iter_extended::vecmap;

use super::basic_block::BasicBlockId;
use super::dfg::DataFlowGraph;
use super::instruction::TerminatorInstruction;
use super::map::Id;
use super::types::Type;
use super::value::ValueId;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(crate) enum RuntimeType {
    // A noir function, to be compiled in ACIR and executed by ACVM
    Acir,
    // Unconstrained function, to be compiled to brillig and executed by the Brillig VM
    Brillig,
}

/// A function holds a list of instructions.
/// These instructions are further grouped into Basic blocks
///
/// All functions outside of the current function are seen as external.
/// To reference external functions its FunctionId can be used but this
/// cannot be checked for correctness until inlining is performed.
#[derive(Debug)]
pub(crate) struct Function {
    /// The first basic block in the function
    entry_block: BasicBlockId,

    /// Name of the function for debugging only
    name: String,

    id: FunctionId,

    runtime: RuntimeType,

    /// The DataFlowGraph holds the majority of data pertaining to the function
    /// including its blocks, instructions, and values.
    pub(crate) dfg: DataFlowGraph,
}

impl Function {
    /// Creates a new function with an automatically inserted entry block.
    ///
    /// Note that any parameters or attributes of the function must be manually added later.
    pub(crate) fn new(name: String, id: FunctionId) -> Self {
        let mut dfg = DataFlowGraph::default();
        let entry_block = dfg.make_block();
        Self { name, id, entry_block, dfg, runtime: RuntimeType::Acir }
    }

    /// The name of the function.
    /// Used exclusively for debugging purposes.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The id of the function.
    pub(crate) fn id(&self) -> FunctionId {
        self.id
    }

    /// Runtime type of the function.
    pub(crate) fn runtime(&self) -> RuntimeType {
        self.runtime
    }

    /// Set runtime type of the function.
    pub(crate) fn set_runtime(&mut self, runtime: RuntimeType) {
        self.runtime = runtime;
    }

    /// Retrieves the entry block of a function.
    ///
    /// A function's entry block contains the instructions
    /// to be executed first when the function is called.
    /// The function's parameters are also stored as the
    /// entry block's parameters.
    pub(crate) fn entry_block(&self) -> BasicBlockId {
        self.entry_block
    }

    /// Returns the parameters of this function.
    /// The parameters will always match that of this function's entry block.
    pub(crate) fn parameters(&self) -> &[ValueId] {
        self.dfg.block_parameters(self.entry_block)
    }

    /// Returns the return types of this function.
    pub(crate) fn returns(&self) -> &[ValueId] {
        let blocks = self.reachable_blocks();
        let mut function_return_values = None;
        for block in blocks {
            let terminator = self.dfg[block].terminator();
            if let Some(TerminatorInstruction::Return { return_values }) = terminator {
                function_return_values = Some(return_values);
                break;
            }
        }
        function_return_values
            .expect("Expected a return instruction, as function construction is finished")
    }

    /// Collects all the reachable blocks of this function.
    ///
    /// Note that self.dfg.basic_blocks_iter() iterates over all blocks,
    /// whether reachable or not. This function should be used if you
    /// want to iterate only reachable blocks.
    pub(crate) fn reachable_blocks(&self) -> BTreeSet<BasicBlockId> {
        let mut blocks = BTreeSet::new();
        let mut stack = vec![self.entry_block];

        while let Some(block) = stack.pop() {
            if blocks.insert(block) {
                stack.extend(self.dfg[block].successors());
            }
        }
        blocks
    }

    pub(crate) fn signature(&self) -> Signature {
        let params = vecmap(self.parameters(), |param| self.dfg.type_of_value(*param));
        let returns = vecmap(self.returns(), |ret| self.dfg.type_of_value(*ret));
        Signature { params, returns }
    }
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeType::Acir => write!(f, "acir"),
            RuntimeType::Brillig => write!(f, "brillig"),
        }
    }
}

/// FunctionId is a reference for a function
///
/// This Id is how each function refers to other functions
/// within Call instructions.
pub(crate) type FunctionId = Id<Function>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
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
