//! This module defines the [Ssa::check_for_underconstrained_values] pass.
//!
//! The pass detects whether there are Brillig calls which are not connected to circuit inputs.
use crate::ssa::checks::is_numeric_constant;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Hint, Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use crate::ssa::visit_once_deque::VisitOnceDeque;
use noirc_artifacts::ssa::{InternalBug, SsaReport};
use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;
use std::collections::{BTreeMap, HashSet};

/// Union-Find (disjoint set) data structure for efficiently computing connected components.
/// Uses path compression and union by rank for near-O(1) amortized operations.
struct UnionFind {
    parent: HashMap<ValueId, ValueId>,
    rank: HashMap<ValueId, u32>,
}

impl UnionFind {
    fn new() -> Self {
        Self { parent: HashMap::default(), rank: HashMap::default() }
    }

    /// Ensure a value exists in the union-find.
    fn make_set(&mut self, v: ValueId) {
        self.parent.entry(v).or_insert(v);
    }

    /// Find the root representative of the set containing `v`, with path compression.
    fn find(&mut self, v: ValueId) -> ValueId {
        let p = self.parent[&v];
        if p == v {
            return v;
        }
        let root = self.find(p);
        self.parent.insert(v, root);
        root
    }

    /// Union the sets containing `a` and `b`. Uses union by rank.
    fn union(&mut self, a: ValueId, b: ValueId) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        let rank_a = *self.rank.entry(ra).or_insert(0);
        let rank_b = *self.rank.entry(rb).or_insert(0);
        if rank_a < rank_b {
            self.parent.insert(ra, rb);
        } else {
            self.parent.insert(rb, ra);
            if rank_a == rank_b {
                *self.rank.entry(ra).or_insert(0) += 1;
            }
        }
    }

    /// Union all values in the iterator into one set.
    fn union_all(&mut self, values: impl IntoIterator<Item = ValueId>) {
        let mut first = None;
        for v in values {
            self.make_set(v);
            match first {
                None => first = Some(v),
                Some(f) => self.union(f, v),
            }
        }
    }
}

impl Ssa {
    /// This function provides an SSA pass that detects if the final function has any subgraphs independent from inputs and outputs.
    /// If this is the case, then part of the final circuit can be completely replaced by any other passing circuit,
    /// since there are no constraints ensuring connections.
    ///
    /// Go through each top-level non-Brillig function and detect if it has independent subgraphs.
    #[tracing::instrument(level = "trace", skip(self))]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn check_for_underconstrained_values(&mut self) -> Vec<SsaReport> {
        self.functions
            .values()
            .map(|f| f.id())
            .par_bridge()
            .flat_map(|fid| {
                let function_to_process = &self.functions[&fid];
                match function_to_process.runtime() {
                    RuntimeType::Acir { .. } => check_for_underconstrained_values_within_function(
                        function_to_process,
                        &self.functions,
                    ),
                    RuntimeType::Brillig(_) => Vec::new(),
                }
            })
            .collect()
    }
}

/// Detect independent subgraphs (not connected to function inputs or outputs) and return a vector of bug reports if some are found
fn check_for_underconstrained_values_within_function(
    function: &Function,
    all_functions: &BTreeMap<FunctionId, Function>,
) -> Vec<SsaReport> {
    let mut context = Context::default();

    context.compute_sets_of_connected_value_ids(function, all_functions);

    // Find roots connected to function inputs/outputs
    let connected_roots = context.find_roots_connected_to_function_inputs_or_outputs(function);

    // Check each brillig return value: if it's in a disconnected component
    // and any of its corresponding inputs are in a different component, report a bug
    let mut warnings: Vec<SsaReport> = Vec::new();
    for (brillig_output, argument_ids) in &context.brillig_return_to_argument {
        // If the output isn't in the union-find, it was never used by any subsequent
        // instruction. The brillig constraints check handles that case.
        if !context.uf.parent.contains_key(brillig_output) {
            continue;
        }
        let output_root = context.uf.find(*brillig_output);
        // Skip outputs that are connected to function inputs/outputs
        if connected_roots.contains(&output_root) {
            continue;
        }
        // Check if any input is in a different component (or absent from the UF entirely)
        let has_disconnected_input = argument_ids.iter().any(|inp| {
            if context.uf.parent.contains_key(inp) {
                context.uf.find(*inp) != output_root
            } else {
                true
            }
        });
        if has_disconnected_input {
            warnings.push(SsaReport::Bug(InternalBug::IndependentSubgraph {
                call_stack: function.dfg.get_instruction_call_stack(
                    context.brillig_return_to_instruction_id[brillig_output],
                ),
            }));
        }
    }
    warnings
}

struct Context {
    block_queue: VisitOnceDeque,
    uf: UnionFind,
    brillig_return_to_argument: im::HashMap<ValueId, Vec<ValueId>>,
    brillig_return_to_instruction_id: im::HashMap<ValueId, InstructionId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            block_queue: VisitOnceDeque::default(),
            uf: UnionFind::new(),
            brillig_return_to_argument: im::HashMap::default(),
            brillig_return_to_instruction_id: im::HashMap::default(),
        }
    }
}

impl Context {
    /// Compute sets of variable ValueIds that are connected with constraints
    ///
    /// Additionally, store information about Brillig calls in the context
    fn compute_sets_of_connected_value_ids(
        &mut self,
        function: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        self.block_queue.push_back(function.entry_block());
        while let Some(block) = self.block_queue.pop_back() {
            self.connect_value_ids_in_block(function, block, all_functions);
        }
    }

    /// Find roots of components that contain function inputs or outputs
    fn find_roots_connected_to_function_inputs_or_outputs(
        &mut self,
        function: &Function,
    ) -> HashSet<ValueId> {
        let returns = function.returns().unwrap_or_default();
        let input_output_values: Vec<_> = function
            .parameters()
            .iter()
            .chain(returns)
            .copied()
            .filter(|id| !is_numeric_constant(function, *id))
            .filter(|id| self.uf.parent.contains_key(id))
            .collect();
        input_output_values.into_iter().map(|id| self.uf.find(id)).collect()
    }

    /// Go through each instruction in the block and union ValueIds connected through that instruction
    ///
    /// Additionally, this function adds mappings of Brillig return values to call arguments and instruction ids from calls to Brillig functions in the block
    fn connect_value_ids_in_block(
        &mut self,
        function: &Function,
        block: BasicBlockId,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) {
        let instructions = function.dfg[block].instructions();

        for instruction in instructions {
            let mut instruction_values: Vec<ValueId> = Vec::new();

            // Collect non-constant instruction arguments
            function.dfg[*instruction].for_each_value(|value_id| {
                if !is_numeric_constant(function, value_id) {
                    instruction_values.push(value_id);
                }
            });
            // And non-constant results
            for value_id in function.dfg.instruction_results(*instruction) {
                if !is_numeric_constant(function, *value_id) {
                    instruction_values.push(*value_id);
                }
            }

            // For most instructions we just connect inputs and outputs
            match &function.dfg[*instruction] {
                Instruction::ArrayGet { .. }
                | Instruction::ArraySet { .. }
                | Instruction::Binary(..)
                | Instruction::Cast(..)
                | Instruction::Constrain(..)
                | Instruction::ConstrainNotEqual(..)
                | Instruction::IfElse { .. }
                | Instruction::Load { .. }
                | Instruction::Not(..)
                | Instruction::Store { .. }
                | Instruction::Truncate { .. }
                | Instruction::MakeArray { .. } => {
                    self.uf.union_all(instruction_values);
                }

                Instruction::Call { func: func_id, arguments: argument_ids } => {
                    match &function.dfg[*func_id] {
                        Value::Intrinsic(intrinsic) => match intrinsic {
                            Intrinsic::ApplyRangeConstraint
                            | Intrinsic::AssertConstant
                            | Intrinsic::AsWitness
                            | Intrinsic::IsUnconstrained => {}
                            Intrinsic::ArrayLen
                            | Intrinsic::ArrayAsStrUnchecked
                            | Intrinsic::ArrayRefCount
                            | Intrinsic::AsVector
                            | Intrinsic::BlackBox(..)
                            | Intrinsic::Hint(Hint::BlackBox)
                            | Intrinsic::DerivePedersenGenerators
                            | Intrinsic::VectorInsert
                            | Intrinsic::VectorPushBack
                            | Intrinsic::VectorPushFront
                            | Intrinsic::VectorPopBack
                            | Intrinsic::VectorPopFront
                            | Intrinsic::VectorRefCount
                            | Intrinsic::VectorRemove
                            | Intrinsic::StaticAssert
                            | Intrinsic::StrAsBytes
                            | Intrinsic::ToBits(..)
                            | Intrinsic::ToRadix(..)
                            | Intrinsic::FieldLessThan => {
                                self.uf.union_all(instruction_values);
                            }
                        },
                        Value::Function(callee) => match all_functions[callee].runtime() {
                            RuntimeType::Brillig(_) => {
                                // For calls to Brillig functions we memorize the mapping of results to argument ValueId's and InstructionId's
                                // The latter are needed to produce the callstack later
                                for result in
                                    function.dfg.instruction_results(*instruction).iter().filter(
                                        |value_id| !is_numeric_constant(function, **value_id),
                                    )
                                {
                                    self.brillig_return_to_argument
                                        .insert(*result, argument_ids.clone());
                                    self.brillig_return_to_instruction_id
                                        .insert(*result, *instruction);
                                }
                            }
                            RuntimeType::Acir(..) => {
                                self.uf.union_all(instruction_values);
                            }
                        },
                        Value::ForeignFunction(..) => {
                            panic!(
                                "Should not be able to reach foreign function from non-Brillig functions, {func_id} in function {}",
                                function.name()
                            );
                        }
                        Value::Instruction { .. }
                        | Value::NumericConstant { .. }
                        | Value::Param { .. }
                        | Value::Global(_) => {
                            panic!(
                                "At the point we are running disconnect there shouldn't be any other values as arguments"
                            )
                        }
                    }
                }
                Instruction::Allocate
                | Instruction::DecrementRc { .. }
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. }
                | Instruction::Noop
                | Instruction::RangeCheck { .. } => {}
            }
        }

        self.block_queue.extend(function.dfg[block].successors());
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::Ssa;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    /// Test that a connected function raises no warnings
    fn test_simple_connected_function() {
        let program = r#"
        acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v2 = add v0, Field 1
                v3 = mul v1, Field 2
                v4 = eq v2, v3
                return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_underconstrained_values();
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where the results of a call to a Brillig function are not connected to main function inputs or outputs
    /// This should be detected.
    fn test_simple_function_with_disconnected_part() {
        let program = r#"
        acir(inline) fn main f0 {
            b0(v0: Field, v1: Field):
                v2 = add v0, Field 1
                v3 = mul v1, Field 2
                v4 = call f1(v2, v3) -> Field
                v5 = add v4, Field 2
                return
        }

        brillig(inline) fn br f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_underconstrained_values();
        assert_eq!(ssa_level_warnings.len(), 1);
    }
}
