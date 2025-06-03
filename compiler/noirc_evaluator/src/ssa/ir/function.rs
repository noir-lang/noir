use std::collections::BTreeSet;
use std::sync::Arc;

use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::InlineType;
use serde::{Deserialize, Serialize};

use super::basic_block::BasicBlockId;
use super::dfg::{DataFlowGraph, GlobalsGraph};
use super::instruction::{BinaryOp, Instruction, TerminatorInstruction};
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
    pub(crate) fn returns(&self) -> &[ValueId] {
        for block in self.reachable_blocks() {
            let terminator = self.dfg[block].terminator();
            if let Some(TerminatorInstruction::Return { return_values, .. }) = terminator {
                return return_values;
            }
        }
        &[]
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

    pub(crate) fn num_instructions(&self) -> usize {
        self.reachable_blocks()
            .iter()
            .map(|block| {
                let block = &self.dfg[*block];
                block.instructions().len() + block.terminator().is_some() as usize
            })
            .sum()
    }

    /// Iterate over the numeric constants in the function.
    pub fn constants(&self) -> impl Iterator<Item = (&FieldElement, &NumericType)> {
        let local = self.dfg.values_iter();
        let global = self.dfg.globals.values_iter();
        local.chain(global).filter_map(|(_, value)| {
            if let Value::NumericConstant { constant, typ } = value {
                Some((constant, typ))
            } else {
                None
            }
        })
    }

    /// Asserts that the [`Function`] is well formed.
    ///
    /// Panics on malformed functions.
    pub(crate) fn assert_valid(&self) {
        self.assert_single_return_block();
        self.validate_signed_arithmetic_invariants();
    }

    /// Checks that the function has only one return block.
    fn assert_single_return_block(&self) {
        let reachable_blocks = self.reachable_blocks();

        // We assume that all functions have a single block which terminates with a `return` instruction.
        let return_blocks: BTreeSet<_> = reachable_blocks
            .iter()
            .filter(|block| {
                // All blocks must have a terminator instruction of some sort.
                let terminator = self.dfg[**block].terminator().unwrap_or_else(|| {
                    panic!("Function {} has no terminator in block {block}", self.id())
                });
                matches!(terminator, TerminatorInstruction::Return { .. })
            })
            .collect();
        if return_blocks.len() > 1 {
            panic!("Function {} has multiple return blocks {return_blocks:?}", self.id())
        }
    }

    /// Validates that any checked signed add/sub is followed by the expected truncate.
    fn validate_signed_arithmetic_invariants(&self) {
        // State for tracking the last signed binary addition/subtraction
        let mut signed_binary_op = None;
        for block in self.reachable_blocks() {
            for instruction in self.dfg[block].instructions() {
                match &self.dfg[*instruction] {
                    Instruction::Binary(binary) => {
                        signed_binary_op = None;

                        match binary.operator {
                            // We are only validating addition/subtraction
                            BinaryOp::Add { unchecked: false }
                            | BinaryOp::Sub { unchecked: false } => {}
                            // Otherwise, move onto the next instruction
                            _ => continue,
                        }

                        // Assume rhs_type is the same as lhs_type
                        let lhs_type = self.dfg.type_of_value(binary.lhs);
                        if let Type::Numeric(NumericType::Signed { bit_size }) = lhs_type {
                            let results = self.dfg.instruction_results(*instruction);
                            signed_binary_op = Some((bit_size, results[0]));
                        }
                    }
                    Instruction::Truncate { value, bit_size, max_bit_size } => {
                        let Some((signed_op_bit_size, signed_op_res)) = signed_binary_op.take()
                        else {
                            continue;
                        };
                        assert_eq!(
                            *bit_size, signed_op_bit_size,
                            "ICE: Correct truncate must follow the result of a checked signed add/sub"
                        );
                        assert_eq!(
                            *max_bit_size,
                            *bit_size + 1,
                            "ICE: Correct truncate must follow the result of a checked signed add/sub"
                        );
                        assert_eq!(
                            *value, signed_op_res,
                            "ICE: Correct truncate must follow the result of a checked signed add/sub"
                        );
                    }
                    _ => {
                        signed_binary_op = None;
                    }
                }
            }
        }
        if signed_binary_op.is_some() {
            panic!("ICE: Truncate must follow the result of a checked signed add/sub");
        }
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

/// Iterate over every Value in this DFG in no particular order, including unused Values,
/// for testing purposes.
pub fn function_values_iter(func: &Function) -> impl DoubleEndedIterator<Item = (ValueId, &Value)> {
    func.dfg.values_iter()
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

#[cfg(test)]
mod validation {
    use crate::ssa::ssa_gen::Ssa;

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_sub_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_sub_brillig() {
        // This matches the test above we just want to make sure it holds in the Brillig runtime as well as ACIR
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_add_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_add_brillig() {
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(
        expected = "ICE: Correct truncate must follow the result of a checked signed add/sub"
    )]
    fn signed_sub_bad_truncate_bit_size() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 32 bits, max_bit_size: 33
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(
        expected = "ICE: Correct truncate must follow the result of a checked signed add/sub"
    )]
    fn signed_sub_bad_truncate_max_bit_size() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 18
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_sub_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_sub_brillig() {
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_add_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_add_brillig() {
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }
}
