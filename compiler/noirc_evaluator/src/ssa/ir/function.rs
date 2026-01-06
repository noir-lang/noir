use std::collections::BTreeSet;
use std::sync::Arc;

use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::InlineType;
use serde::{Deserialize, Serialize};

use crate::ssa::ir::instruction::Instruction;
use crate::ssa::ir::post_order::PostOrder;

use super::basic_block::BasicBlockId;
use super::dfg::{DataFlowGraph, GlobalsGraph};
use super::instruction::TerminatorInstruction;
use super::map::Id;
use super::types::{NumericType, Type};
use super::value::{Value, ValueId};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RuntimeType {
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
    ///
    /// ## Important
    /// If a Brillig function is not main it requires special handling to determine
    /// whether it is an entry point. Brillig entry points can also be anywhere we start
    /// Brillig execution from an ACIR runtime. This requires analyzing the call sites of the ACIR runtime.
    pub(crate) fn is_entry_point(&self) -> bool {
        match self {
            RuntimeType::Acir(inline_type) | RuntimeType::Brillig(inline_type) => {
                inline_type.is_entry_point()
            }
        }
    }

    pub(crate) fn is_inline_always(&self) -> bool {
        matches!(
            self,
            RuntimeType::Acir(InlineType::InlineAlways)
                | RuntimeType::Brillig(InlineType::InlineAlways)
        )
    }

    pub(crate) fn is_inline_never(&self) -> bool {
        matches!(
            self,
            RuntimeType::Acir(InlineType::InlineNever)
                | RuntimeType::Brillig(InlineType::InlineNever)
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
pub struct Function {
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
        new_function.dfg.set_function_purities(another.dfg.function_purities.clone());
        new_function.dfg.brillig_arrays_offset = another.dfg.brillig_arrays_offset;
        new_function
    }

    /// The name of the function.
    /// Used exclusively for debugging purposes.
    pub fn name(&self) -> &str {
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
        self.runtime().is_no_predicates()
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
    /// None might be returned if the function ends up with all of its block
    /// terminators being `jmp`, `jmpif` or `unreachable`.
    pub(crate) fn returns(&self) -> Option<&[ValueId]> {
        for block in self.reachable_blocks() {
            let terminator = self.dfg[block].terminator();
            if let Some(TerminatorInstruction::Return { return_values, .. }) = terminator {
                return Some(return_values);
            }
        }
        None
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
        let returns =
            vecmap(self.returns().unwrap_or_default(), |ret| self.dfg.type_of_value(*ret));
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

    /// Total number of instructions in the reachable blocks of this function.
    pub(crate) fn num_instructions(&self) -> usize {
        self.reachable_blocks()
            .iter()
            .map(|block| {
                let block = &self.dfg[*block];
                block.instructions().len() + usize::from(block.terminator().is_some())
            })
            .sum()
    }

    pub fn view(&self) -> FunctionView {
        FunctionView(self)
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

/// Provide public access to certain aspects of a `Function` without bloating its API.
pub struct FunctionView<'a>(&'a Function);

impl<'a> FunctionView<'a> {
    /// Iterate over every Value in this DFG in no particular order, including unused Values,
    /// for testing purposes.
    pub fn values_iter(&self) -> impl DoubleEndedIterator<Item = (ValueId, &'a Value)> {
        self.0.dfg.values_iter()
    }

    /// Iterate over the blocks in the CFG in reverse-post-order.
    pub fn blocks_iter(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        let post_order = PostOrder::with_function(self.0);
        post_order.into_vec_reverse().into_iter()
    }

    /// Iterate over the successors of a blocks.
    pub fn block_successors_iter(
        &self,
        block_id: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> {
        let block = &self.0.dfg[block_id];
        block.successors()
    }

    /// Iterate over the functions called from a block.
    pub fn block_callees_iter(&self, block_id: BasicBlockId) -> impl Iterator<Item = FunctionId> {
        let block = &self.0.dfg[block_id];
        block.instructions().iter().map(|id| &self.0.dfg[*id]).filter_map(|instruction| {
            let Instruction::Call { func, .. } = instruction else {
                return None;
            };
            let Value::Function(func) = self.0.dfg[*func] else {
                return None;
            };
            Some(func)
        })
    }

    /// Iterate over the numeric constants in the function.
    pub fn constants(&self) -> impl Iterator<Item = (&FieldElement, &NumericType)> {
        let local = self.0.dfg.values_iter();
        let global = self.0.dfg.globals.values_iter();
        local.chain(global).filter_map(|(_, value)| {
            if let Value::NumericConstant { constant, typ } = value {
                Some((constant, typ))
            } else {
                None
            }
        })
    }

    pub fn has_data_bus_return_data(&self) -> bool {
        self.0.dfg.data_bus.return_data.is_some()
    }

    /// Return the types of the function parameters.
    pub fn parameter_types(&self) -> Vec<Type> {
        vecmap(self.0.parameters(), |p| self.0.dfg.type_of_value(*p))
    }

    /// Return the types of the returned values, if there are any.
    pub fn return_types(&self) -> Option<Vec<Type>> {
        self.0.returns().map(|rs| vecmap(rs, |p| self.0.dfg.type_of_value(*p)))
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

    signature.params.push(Type::Numeric(NumericType::NativeField));
    signature.returns.push(Type::Numeric(NumericType::Unsigned { bit_size: 32 }));
}
