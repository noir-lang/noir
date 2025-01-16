use std::collections::BTreeSet;
use std::sync::Arc;

use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::InlineType;
use serde::{Deserialize, Serialize};

use super::basic_block::BasicBlockId;
use super::dfg::{DataFlowGraph, GlobalsGraph};
use super::instruction::TerminatorInstruction;
use super::map::Id;
use super::types::Type;
use super::value::ValueId;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub(crate) enum RuntimeType {
    // A noir function, to be compiled in ACIR and executed by ACVM
    Acir(InlineType),
    // Unconstrained function, to be compiled to brillig and executed by the Brillig VM
    Brillig(InlineType),
}

impl RuntimeType {
    /// Returns whether the runtime type represents an entry point.
    /// We return `false` for InlineType::Inline on default, which is true
    /// in all cases except for main. `main` should be supported with special
    /// handling in any places where this function determines logic.
    pub(crate) fn is_entry_point(&self) -> bool {
        match self {
            RuntimeType::Acir(inline_type) => inline_type.is_entry_point(),
            RuntimeType::Brillig(_) => true,
        }
    }

    pub(crate) fn is_inline_always(&self) -> bool {
        matches!(
            self,
            RuntimeType::Acir(InlineType::InlineAlways)
                | RuntimeType::Brillig(InlineType::InlineAlways)
        )
    }

    pub(crate) fn is_no_predicates(&self) -> bool {
        matches!(
            self,
            RuntimeType::Acir(InlineType::NoPredicates)
                | RuntimeType::Brillig(InlineType::NoPredicates)
        )
    }

    pub(crate) fn is_brillig(&self) -> bool {
        matches!(self, RuntimeType::Brillig(_))
    }

    pub(crate) fn is_acir(&self) -> bool {
        matches!(self, RuntimeType::Acir(_))
    }
}

impl Default for RuntimeType {
    fn default() -> Self {
        RuntimeType::Acir(InlineType::default())
    }
}

/// A function holds a list of instructions.
/// These instructions are further grouped into Basic blocks
///
/// All functions outside of the current function are seen as external.
/// To reference external functions its FunctionId can be used but this
/// cannot be checked for correctness until inlining is performed.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Function {
    /// The first basic block in the function
    entry_block: BasicBlockId,

    /// Name of the function for debugging only
    name: String,

    id: Option<FunctionId>,

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
        Self { name, id: Some(id), entry_block, dfg }
    }

    /// Globals are generated using the same codegen process as functions.
    /// To avoid a recursive global context we should create a pseudo function to mock a globals context.
    pub(crate) fn new_for_globals() -> Self {
        let mut dfg = DataFlowGraph::default();
        let entry_block = dfg.make_block();
        Self { name: "globals".to_owned(), id: None, entry_block, dfg }
    }

    /// Creates a new function as a clone of the one passed in with the passed in id.
    pub(crate) fn clone_with_id(id: FunctionId, another: &Function) -> Self {
        let dfg = another.dfg.clone();
        let entry_block = another.entry_block;
        Self { name: another.name.clone(), id: Some(id), entry_block, dfg }
    }

    /// Takes the signature (function name & runtime) from a function but does not copy the body.
    pub(crate) fn clone_signature(id: FunctionId, another: &Function) -> Self {
        let mut new_function = Function::new(another.name.clone(), id);
        new_function.set_runtime(another.runtime());
        new_function.set_globals(another.dfg.globals.clone());
        new_function
    }

    /// The name of the function.
    /// Used exclusively for debugging purposes.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The id of the function.
    pub(crate) fn id(&self) -> FunctionId {
        self.id.expect("FunctionId should be initialized")
    }

    /// Runtime type of the function.
    pub(crate) fn runtime(&self) -> RuntimeType {
        self.dfg.runtime()
    }

    /// Set runtime type of the function.
    pub(crate) fn set_runtime(&mut self, runtime: RuntimeType) {
        self.dfg.set_runtime(runtime);
    }

    pub(crate) fn set_globals(&mut self, globals: Arc<GlobalsGraph>) {
        self.dfg.globals = globals;
    }

    pub(crate) fn is_no_predicates(&self) -> bool {
        match self.runtime() {
            RuntimeType::Acir(inline_type) => matches!(inline_type, InlineType::NoPredicates),
            RuntimeType::Brillig(_) => false,
        }
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
            if let Some(TerminatorInstruction::Return { return_values, .. }) = terminator {
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

    /// Finds the block of the function with the Return instruction
    pub(crate) fn find_last_block(&self) -> BasicBlockId {
        for block in self.reachable_blocks() {
            if matches!(self.dfg[block].terminator(), Some(TerminatorInstruction::Return { .. })) {
                return block;
            }
        }

        unreachable!("SSA Function {} has no reachable return instruction!", self.id())
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function::clone_with_id(self.id(), self)
    }
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeType::Acir(inline_type) => write!(f, "acir({inline_type})"),
            RuntimeType::Brillig(inline_type) => write!(f, "brillig({inline_type})"),
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

#[test]
fn sign_smoke() {
    let mut signature = Signature::default();

    signature.params.push(Type::Numeric(super::types::NumericType::NativeField));
    signature.returns.push(Type::Numeric(super::types::NumericType::Unsigned { bit_size: 32 }));
}
