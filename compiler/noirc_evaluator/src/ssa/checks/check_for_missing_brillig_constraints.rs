//! This module defines the [Ssa::check_for_missing_brillig_constraints] method.
//!
//! It verifies that the output of Brillig calls is connected to the inputs of the calls
//! by assertions; in other words, that the circuit has constraints that the output is
//! correct, given the inputs.
//!
//! To do so, it tracks the ancestry of every expression, and checks that any
//! variable which is an output of a Brillig call has a descendant which appears
//! in an assertion, where the other side has an ancestor that is an input of the call.
//!
//! Essentially, to consider a particular Brillig call constrained, we are looking
//! for a constraint where the ancestors of the constraint arguments intersect both of:
//! * the descendants of the results of the call (outputs)
//! * the ancestors of the arguments of the call (inputs)
//!
//! For example take the following graph of variables feeding into calls:
//! ```text
//!   v1     v2      v3
//!    \   /  \    /
//!     \ /    \  /
//!      v4     v5 = call(v2, v3)
//!      |\     |
//!      | \    |
//!      |  \   |
//!      |   \  |
//!      |    \ |
//!      |      v6 = call(v5, v4)
//!      |     /
//!      |    /
//!      |   /
//!      |  /
//! constrain(v4, v6)
//! ```
//!
//! Both calls are considered constrained:
//! * The output of the 2nd call (v6) is constrained directly against its input (v4)
//! * The output of the 1st call (v5) has a descendant (v6) which is constrained against
//!   a value (v4) that has an ancestor (v2) which is also an ancestor of an argument of
//!   of the call itself.
//!
//! The goal isn't to verify that the constraint is correct, just that some (indirect)
//! connection between inputs and outputs is made.
use crate::ssa::checks::is_numeric_constant;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::post_order::PostOrder;
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use acvm::AcirField;
use noirc_artifacts::ssa::{InternalBug, SsaReport};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

impl Ssa {
    /// Detect Brillig calls left unconstrained with manual asserts
    /// and return a vector of bug reports if any have been found
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn check_for_missing_brillig_constraints(
        &mut self,
        _enable_lookback: bool,
    ) -> Vec<SsaReport> {
        println!("{self}");

        // Skip the check if there are no Brillig functions involved
        if !self.functions.values().any(|func| func.runtime().is_brillig()) {
            return vec![];
        }

        self.functions
            .values()
            .filter(|func| func.runtime().is_acir() && has_call_to_brillig(func, &self.functions))
            .par_bridge()
            .flat_map(|func| {
                Context::new(func)
                    .build_ancestors(func, &self.functions)
                    .build_tainted(func, &self.functions)
                    .into_warnings(func)
            })
            .collect()
    }
}

type AncestorMap = HashMap<ValueId, HashSet<ValueId>>;

/// Outputs of a Brillig call and their descendants.
#[derive(Debug)]
struct TaintedDescendants {
    /// Inputs of the call.
    ///
    /// To consider the call constrained, the constraint must be on a value which has
    /// an ancestry that intersects with the ancestry of an argument.
    arguments: Vec<ValueId>,
    /// Non-array outputs of the call.
    ///
    /// To consider the output constrained, we have to find a constraint such that
    /// the output is an ancestor of the constrained value.
    single_outputs: HashSet<ValueId>,
    /// Array outputs of the call, tracked per element, accumulating their individual
    /// dependencies.
    ///
    /// To consider an element constrained, we have to find a constraint such that
    /// the constrained value appears in the descendants.
    array_outputs: HashMap<(ValueId, u32), HashSet<ValueId>>,
}

impl TaintedDescendants {
    fn new(func: &Function, arguments: Vec<ValueId>, result_ids: &[ValueId]) -> Self {
        let max_array_size: u32 = crate::ssa::ir::dfg::MAX_ELEMENTS.try_into().unwrap();
        let mut single_outputs = HashSet::new();
        let mut array_outputs = HashMap::new();
        for result_id in result_ids {
            match func.dfg.try_get_array_length(*result_id) {
                // If the result value is an array, create an empty descendant set for
                // every element to be accessed further on and record the indices
                // of the resulting sets for future reference
                Some(length) if length.0 <= max_array_size => {
                    for i in 0..length.0 {
                        array_outputs.insert((*result_id, i), HashSet::new());
                    }
                }
                // For very large array_outputs or non-array_outputs, treat the whole result as a single value
                // to avoid memory/time issues when tracking individual elements
                Some(_) | None => {
                    single_outputs.insert(*result_id);
                }
            }
        }
        Self { arguments, single_outputs, array_outputs }
    }

    /// Whether there are any unconstrained results left.
    fn is_constrained(&self) -> bool {
        self.single_outputs.is_empty() && self.array_outputs.is_empty()
    }

    /// Try to constrain some of the outputs if:
    /// * one of the constrained values is a descendant of the output, and
    /// * another constrained value shares an ancestor with an input, and it is not tainted
    ///
    /// Exceptions to this rule are:
    /// * if there are no input arguments (they were all numeric constants, or there were no args)
    /// * if there is only one constrained value (an output against a constant)
    ///
    /// Return `true` if all outputs have been constrained.
    fn try_constrain(
        &mut self,
        constrained_values: &[ValueId],
        ancestors: &AncestorMap,
        all_tainted: &HashSet<ValueId>,
    ) -> bool {
        dbg!(&self);
        dbg!(&ancestors);
        dbg!(&all_tainted);

        let is_against_const = constrained_values.len() == 1;
        let is_const_args = self.arguments.is_empty();

        // Make sure this constraint has something to do with the inputs,
        // unless there are no inputs, or the output is against a constant.
        if !is_against_const
            && !is_const_args
            && !self.arguments_intersect(constrained_values, ancestors, all_tainted)
        {
            return false;
        }

        // Remove any results that have been directly or indirectly constrained.
        self.single_outputs.retain(|output| {
            !constrained_values.iter().any(|value| {
                output == value || ancestors.get(value).map_or(false, |a| a.contains(output))
            })
        });
        self.array_outputs.retain(|_, descendants| {
            !constrained_values.iter().any(|value| descendants.contains(value))
        });

        self.is_constrained()
    }

    /// Whether one of the constrained values:
    /// * shares an ancestor with a call argument, and
    /// * is not a descendant of the outputs, and
    /// * is not tainted
    fn arguments_intersect(
        &self,
        constrained_values: &[ValueId],
        ancestors: &AncestorMap,
        all_tainted: &HashSet<ValueId>,
    ) -> bool {
        for constrained in constrained_values {
            if all_tainted.contains(constrained) {
                // Allowing these would mean we could constrain the output of one call
                // with the output of another Brillig call.
                continue;
            }
            if self.is_descendant_of_outputs(ancestors, constrained) {
                // Allowing these would trivially connect outputs to inputs of the same call.
                continue;
            }
            // Check if this constraint is directly on one of the inputs.
            if self.arguments.iter().any(|a| a == constrained) {
                return true;
            }
            // Check if one of the inputs shares an ancestor with the constraint.
            let Some(constrained_ancestors) = ancestors.get(constrained) else {
                continue;
            };
            for arg in &self.arguments {
                if constrained_ancestors.contains(arg) {
                    return true;
                }
                let Some(arg_ancestors) = ancestors.get(arg) else {
                    continue;
                };
                if arg_ancestors.contains(constrained)
                    || intersecting(arg_ancestors, constrained_ancestors)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Whether the value is a descendant of the call outputs.
    fn is_descendant_of_outputs(&self, ancestors: &AncestorMap, value: &ValueId) -> bool {
        self.single_outputs.contains(value)
            || ancestors
                .get(value)
                .map_or(false, |a| self.single_outputs.iter().any(|o| a.contains(o)))
            || self.array_outputs.values().any(|d| d.contains(value))
    }

    /// Add to the descendants of a particular array element.
    fn extend_array_result(&mut self, array: ValueId, index: u32, results: &[ValueId]) {
        if let Some(descendants) = self.array_outputs.get_mut(&(array, index)) {
            descendants.extend(results);
        }
    }
}

struct Context {
    /// Block IDs in Post Order.
    post_order: Vec<BasicBlockId>,

    /// Ancestors of the values that we are interested in.
    ///
    /// We are interested in the ancestry of variables which either:
    /// * have constraints on them, or
    /// * are inputs to a Brillig call.
    ///
    /// Using that information, we can consider a constraint to cover an output of a Brillig call if:
    /// * the output is in the ancestry of a constrained value, and
    /// * the ancestors of a constrained value intersects the ancestors of one of the inputs of the call
    ///
    /// The ancestors of a value do not include the value itself.
    ancestors: AncestorMap,

    /// Descendants of Brillig calls.
    tainted: HashMap<InstructionId, TaintedDescendants>,
}

impl Context {
    fn new(func: &Function) -> Self {
        Self {
            post_order: PostOrder::with_function(func).into_vec(),
            ancestors: Default::default(),
            tainted: Default::default(),
        }
    }

    /// Traverse blocks and instruction bottom-up to build up the ancestry of values.
    fn build_ancestors(
        mut self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> Self {
        // Union of all values we are tracking; helps skip values that are of no interest.
        let mut tracked_ids = HashSet::new();

        // Traverse block from the end towards the beginning, so we can expand the ancestry of the values bottom-up.
        for block_id in self.post_order.iter().copied() {
            // Traverse instructions in reverse so we know which values we want the ancestors for.
            for instruction_id in func.dfg[block_id].instructions().iter().rev() {
                let instruction = &func.dfg[*instruction_id];
                let result_ids = func.dfg.instruction_results(*instruction_id);

                // If any of the results is part of something we are tracking, add the inputs to their ancestry.
                for result_id in result_ids {
                    if is_numeric_constant(func, *result_id) || !tracked_ids.contains(result_id) {
                        continue;
                    }
                    for (id, ancestors) in &mut self.ancestors {
                        if id != result_id && !ancestors.contains(result_id) {
                            continue;
                        }
                        for value_id in instruction_arguments(func, instruction) {
                            ancestors.insert(value_id);
                            tracked_ids.insert(value_id);
                        }
                    }
                }

                let should_track = is_call_to_brillig(func, all_functions, instruction_id)
                    || is_constraint(func, instruction_id)
                    || is_side_effect(func, instruction);

                if should_track {
                    // Start tracking the ancestors of the inputs of the instruction.
                    // Skip the first value of calls, which is the function ID.
                    for value_id in instruction_arguments(func, instruction) {
                        self.ancestors.entry(value_id).or_default();
                        tracked_ids.insert(value_id);
                    }
                }

                if let Instruction::Constrain(v1, v2, _) = instruction
                    && !is_numeric_constant(func, *v1)
                    && !is_numeric_constant(func, *v2)
                {
                    Self::add_equivalence(&mut self.ancestors, *v1, *v2);
                }
            }
        }

        self
    }

    /// Traverse blocks an instructions top-down to build up the descendants of Brillig calls,
    /// trying to constrain them along the way based on the ancestry information.
    fn build_tainted(
        mut self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> Self {
        let mut all_tainted = HashSet::new();
        // Traverse in Reverse Post Order, ie. top-down.
        for block_id in self.post_order.clone().into_iter().rev() {
            let mut side_effects_var: Option<ValueId> = None;
            for instruction_id in func.dfg[block_id].instructions() {
                let instruction = &func.dfg[*instruction_id];
                let mut arguments = instruction_arguments(func, instruction);
                let results = instruction_results(func, instruction_id);

                // If we are under a side effect, extend ancestors and the args.
                if let Some(side_effects_var) = &side_effects_var {
                    self.extend_ancestors_with_side_effects(side_effects_var, &results);
                    arguments.push(*side_effects_var);
                }

                // Extend the descendants of Brillig calls.
                // This is only required for array output; for single outputs we can look at the ancestry.
                if !results.is_empty() {
                    // Look for ArrayGet instructions with a constant index,
                    // and if the array is the result of a tainted call,
                    // then add the result as a descendant of that particular index.
                    if let Instruction::ArrayGet { array, index } = instruction
                        && let Some(index) = func.dfg.get_numeric_constant(*index)
                        && let Some(index) = index.try_to_u32()
                    {
                        for tainted in self.tainted.values_mut() {
                            tainted.extend_array_result(*array, index, &results);
                        }
                    }

                    // Tainted values cannot be used to constrain Brillig output.
                    if arguments.iter().any(|a| all_tainted.contains(a)) {
                        all_tainted.extend(&results);
                    }
                }

                if is_call_to_brillig(func, all_functions, instruction_id) && !results.is_empty() {
                    let tainted = TaintedDescendants::new(func, arguments, &results);
                    self.tainted.insert(*instruction_id, tainted);
                    all_tainted.extend(&results);
                } else if is_constraint(func, instruction_id) && !self.tainted.is_empty() {
                    let constrained_values = instruction_arguments(func, instruction);
                    self.tainted.retain(|_, tainted| {
                        !tainted.try_constrain(&constrained_values, &self.ancestors, &all_tainted)
                    });
                } else if let Instruction::EnableSideEffectsIf { condition } = instruction {
                    side_effects_var =
                        (!is_numeric_constant(func, *condition)).then_some(*condition);
                }
            }
        }

        self
    }

    /// Every Brillig call not properly constrained should remain in the tainted set
    /// at this point. For each, emit a corresponding warning.
    fn into_warnings(self, function: &Function) -> Vec<SsaReport> {
        self.tainted
            .keys()
            .map(|brillig_call| {
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_instruction_call_stack(*brillig_call),
                })
            })
            .collect()
    }

    /// Add the ancestors of the current side effect variable to the ancestors of the current results.
    ///
    /// We weren't able to do this during bottom-up traversal, because we don't know the side effects
    /// when the results are defined.
    fn extend_ancestors_with_side_effects(
        &mut self,
        side_effects_var: &ValueId,
        results: &[ValueId],
    ) {
        let Some(side_effects_ancestors) = self.ancestors.get(side_effects_var).cloned() else {
            return;
        };
        for result in results {
            let Some(ancestors) = self.ancestors.get_mut(&result) else {
                continue;
            };
            ancestors.insert(*side_effects_var);
            ancestors.extend(&side_effects_ancestors);
        }
    }

    /// When we have `constrain v0 == v1`, then consider any follow up constraints
    /// on v0 or v1 as if it applied on both. This is because some SSA passes use
    /// constraint info to simplify values, and what was a constraint on v0 could
    /// end up being a constraint on v1.
    fn add_equivalence(ancestors: &mut AncestorMap, v1: ValueId, v2: ValueId) {
        /// If we have `c -> a`, insert `c -> b` as well.
        /// This way if we put a constraint on `c`, it affects anything calls
        /// that produced `b`, even if they had no relation to `a`.
        ///
        /// Not inserting `a -> b` and `b -> a` because that would make
        /// a constraint derive from the output of a call, disqualifying
        /// it from being a constraint on the inputs.
        fn go(ancestors: &mut AncestorMap, a: ValueId, b: ValueId) {
            for (c, ancestors) in ancestors.iter_mut() {
                if *c == b {
                    // Not inserting self-ancestry.
                    continue;
                }
                if ancestors.contains(&a) {
                    ancestors.insert(b);
                }
            }
        }
        go(ancestors, v1, v2);
        go(ancestors, v2, v1);
    }
}

/// Whether there is at least one instruction making a call to a Brillig function with non-empty results.
fn has_call_to_brillig(func: &Function, all_functions: &BTreeMap<FunctionId, Function>) -> bool {
    for block_id in func.reachable_blocks() {
        for instruction_id in func.dfg[block_id].instructions() {
            if is_call_to_brillig(func, all_functions, instruction_id) {
                return true;
            }
        }
    }
    false
}

/// Whether the instruction a call to a Brillig function with a non-empty results.
fn is_call_to_brillig(
    func: &Function,
    all_functions: &BTreeMap<FunctionId, Function>,
    instruction_id: &InstructionId,
) -> bool {
    let Instruction::Call { func: callee_id, .. } = func.dfg[*instruction_id] else {
        return false;
    };
    let Value::Function(callee_id) = func.dfg[callee_id] else {
        return false;
    };
    if !all_functions[&callee_id].runtime().is_brillig() {
        return false;
    }
    !func.dfg.instruction_results(*instruction_id).is_empty()
}

/// Whether an instruction puts constraints on its inputs.
fn is_constraint(func: &Function, instruction_id: &InstructionId) -> bool {
    let instruction = &func.dfg[*instruction_id];
    if matches!(
        instruction,
        Instruction::Constrain(..)
            | Instruction::ConstrainNotEqual(..)
            | Instruction::RangeCheck { .. }
    ) {
        return true;
    }
    let Instruction::Call { func: callee_id, .. } = instruction else {
        return false;
    };
    let Value::Intrinsic(intrinsic) = &func.dfg[*callee_id] else {
        return false;
    };
    matches!(intrinsic, Intrinsic::ApplyRangeConstraint | Intrinsic::AssertConstant)
}

/// Whether the instruction is a non-constant side effect variable.
fn is_side_effect(func: &Function, instruction: &Instruction) -> bool {
    let Instruction::EnableSideEffectsIf { condition } = instruction else {
        return false;
    };
    !is_numeric_constant(func, *condition)
}

/// Collect non-constant arguments of an instruction.
fn instruction_arguments(func: &Function, instruction: &Instruction) -> Vec<ValueId> {
    let mut arguments = Vec::new();
    // Skip the first value of calls, which is the function ID.
    let skip_first = matches!(instruction, Instruction::Call { .. });
    let mut is_first = true;
    instruction.for_each_value(|value_id| {
        if !(skip_first && is_first || is_numeric_constant(func, value_id)) {
            arguments.push(value_id);
        }
        is_first = false;
    });
    arguments
}

/// Collect non-constant results of an instruction.
fn instruction_results(func: &Function, instruction_id: &InstructionId) -> Vec<ValueId> {
    func.dfg
        .instruction_results(*instruction_id)
        .iter()
        .filter(|value| !is_numeric_constant(func, **value))
        .copied()
        .collect()
}

/// Return `true` if two sets have a non-empty intersection.
fn intersecting<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
    a.intersection(b).next().is_some()
}

#[cfg(test)]
mod tests {
    use crate::ssa::Ssa;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function is left unchecked with a later assert,
    /// by example of the program illustrating issue #5425 (simplified variant).
    fn test_underconstrained_value_detector_5425() {
        /*
        unconstrained fn maximum_price(options: [u32; 2]) -> u32 {
            let mut maximum_option = options[0];
            if (options[1] > options[0]) {
                maximum_option = options[1];
            }
            maximum_option
        }

        fn main(sandwiches: pub [u32; 2], drinks: pub [u32; 2], best_value: u32) {
            let most_expensive_sandwich = maximum_price(sandwiches);
            let mut sandwich_exists = false;
            sandwich_exists |= (sandwiches[0] == most_expensive_sandwich);
            sandwich_exists |= (sandwiches[1] == most_expensive_sandwich);
            assert(sandwich_exists);

            let most_expensive_drink = maximum_price(drinks);
            assert(
                best_value
                == (most_expensive_sandwich + most_expensive_drink)
            );
        }
        */
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v4: [u32; 2], v5: [u32; 2], v6: u32):
            inc_rc v4
            inc_rc v5
            v8 = call f1(v4) -> u32
            v9 = allocate -> &mut u1
            store u1 0 at v9
            v10 = load v9 -> u1
            v11 = array_get v4, index u32 0 -> u32
            v12 = eq v11, v8
            v13 = or v10, v12
            store v13 at v9
            v14 = load v9 -> u1
            v15 = array_get v4, index u32 1 -> u32
            v16 = eq v15, v8
            v17 = or v14, v16
            store v17 at v9
            v18 = load v9 -> u1
            constrain v18 == u1 1
            v19 = call f1(v5) -> u32
            v20 = add v8, v19
            constrain v6 == v20
            dec_rc v4
            dec_rc v5
            return
        }

        brillig(inline) fn maximum_price f1 {
          b0(v0: [u32; 2]):
            v2 = array_get v0, index u32 0 -> u32
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function returning multiple result values
    /// is left unchecked with a later assert involving all the results
    fn test_unchecked_multiple_results_brillig() {
        // First call is constrained properly, involving both results
        // Second call is insufficiently constrained, involving only one of the results
        // The Brillig function is fake, for simplicity's sake
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2, v3 = call f1(v0) -> (u32, u32)
            v4 = mul v2, v3
            constrain v4 == v0
            v5, v6 = call f1(v0) -> (u32, u32)
            v7 = mul v5, v5
            constrain v7 == v0
            return
        }

        brillig(inline) fn factor f1 {
          b0(v0: u32):
            return u32 0, u32 0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig function is called with a constant argument
    /// (should _not_ lead to a false positive failed check
    /// if all the results are constrained)
    fn test_checked_brillig_with_constant_arguments() {
        // The call is constrained properly, involving both results
        // (but the argument to the Brillig is a constant)
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3, v4 = call f1(Field 7) -> (u32, u32)
            v5 = mul v3, v4
            constrain v5 == v0
            return
        }

        brillig(inline) fn factor f1 {
          b0(v0: Field):
            return u32 0, u32 0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig function call is constrained with a range check
    /// (should _not_ lead to a false positive failed check)
    fn test_range_checked_brillig() {
        // The call is constrained properly with a range check, involving
        // both Brillig call argument and result
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            v3 = add v2, v0
            range_check v3 to 32 bits
            return
        }

        brillig(inline) fn dummy f1 {
          b0(v0: u32):
            return u32 0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig nested type result is insufficiently constrained
    /// (with a field constraint missing)
    fn test_nested_type_result_brillig() {
        /*
        struct Animal {
            legs: Field,
            eyes: u8,
            tag: Tag,
        }

        struct Tag {
            no: Field,
        }

        unconstrained fn foo(bar: Field) -> Animal {
            Animal {
                legs: 4,
                eyes: 2,
                tag: Tag { no: bar }
            }
        }

        fn main(x: Field) -> pub Animal {
            let dog = foo(x);
            assert(dog.legs == 4);
            assert(dog.tag.no == x);

            dog
        }
        */
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2, v3, v4 = call f1(v0) -> (Field, u8, Field)
            v6 = eq v2, Field 4
            constrain v2 == Field 4
            v10 = eq v4, v0
            constrain v4 == v0
            return v2, v3, v4
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            return Field 4, u8 2, v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where Brillig calls' root result values are constrained against
    /// each other (covers a false negative edge case)
    /// (https://github.com/noir-lang/noir/pull/6658#pullrequestreview-2482170066)
    fn test_root_result_intersection_false_negative() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v5 = call f1(v0, v1) -> Field
            v6 = eq v3, v5
            constrain v3 == v5
            v8 = add v3, v5
            return v8
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 2);
    }

    #[test]
    #[traced_test]
    /// Test EnableSideEffectsIf conditions affecting the dependency graph
    /// (SSA a bit convoluted to work around simplification breaking the flow
    /// of the parsed test code). Note that the side effect variable is a
    /// descendant of the output of the call, and the constraint is on a
    /// variable which is affected by the side effect variable.
    fn test_enable_side_effects_if_affecting_following_statements() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v5 = add v0, v1
            v6 = eq v3, v5
            v7 = add u1 1, u1 0
            enable_side_effects v6
            v8 = add v7, u1 1
            enable_side_effects u1 1
            constrain v8 == u1 2
            return v3
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test call result array elements being underconstrained
    fn test_brillig_result_array_missing_element_constraint() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v16 = call f1(v0) -> [u32; 3]
            v17 = array_get v16, index u32 0 -> u32
            constrain v17 == v0
            v19 = array_get v16, index u32 2 -> u32
            constrain v19 == v0
            return v17
        }

        brillig(inline) fn into_array f1 {
          b0(v0: u32):
            v4 = make_array [v0, v0, v0] : [u32; 3]
            return v4
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test call result array elements being constrained properly
    fn test_brillig_result_array_all_elements_constrained() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v16 = call f1(v0) -> [u32; 3]
            v17 = array_get v16, index u32 0 -> u32
            constrain v17 == v0
            v20 = array_get v16, index u32 1 -> u32
            constrain v20 == v0
            v19 = array_get v16, index u32 2 -> u32
            constrain v19 == v0
            return v17
        }

        brillig(inline) fn into_array f1 {
          b0(v0: u32):
            v4 = make_array [v0, v0, v0] : [u32; 3]
            return v4
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained (wrapper) Brillig calls not producing a false positive.
    ///
    /// A wrapper was considered something that passes all the outputs of
    /// one Brillig call as inputs to the next Brillig call.
    fn test_chained_brillig_calls_constrained_wrapped() {
        /*
        struct Animal {
            legs: Field,
            eyes: u8,
            tag: Tag,
        }

        struct Tag {
            no: Field,
        }

        unconstrained fn foo(x: Field) -> Animal {
            Animal {
                legs: 4,
                eyes: 2,
                tag: Tag { no: x }
            }
        }

        unconstrained fn bar(x: Animal) -> Animal {
            Animal {
                legs: x.legs,
                eyes: x.eyes,
                tag: Tag { no: x.tag.no + 1 }
            }
        }

        fn main(x: Field) -> pub Animal {
            let dog = bar(foo(x));
            assert(dog.legs == 4);
            assert(dog.eyes == 2);
            assert(dog.tag.no == x + 1);

            dog
        }
        */
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v27, v28, v29 = call f2(v0) -> (Field, u8, Field)
            v30, v31, v32 = call f1(v27, v28, v29) -> (Field, u8, Field)
            constrain v30 == Field 4
            constrain v31 == u8 2
            v35 = add v0, Field 1
            constrain v32 == v35
            return v30, v31, v32
        }

        brillig(inline) fn foo f2 {
          b0(v0: Field):
            return Field 4, u8 2, v0
        }

        brillig(inline) fn bar f1 {
          b0(v0: Field, v1: u8, v2: Field):
            v7 = add v2, Field 1
            return v0, v1, v7
        }

        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained Brillig calls.
    /// This is based on the diagram from the top of the module.
    fn test_chained_brillig_calls_constrained_mixed() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v3 = mul v0, v1
            v4 = call f1(v1, v2) -> Field
            v5 = call f1(v3, v4) -> Field
            constrain v3 == v5
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test for the argument descendants coming before Brillig calls themselves being
    /// registered as such
    fn test_brillig_argument_descendants_preceding_call() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = add v0, v1
            v5 = call f1(v0, v1) -> Field
            constrain v3 == v5
            return v3
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// No-result calls (e.g. print) shouldn't trigger the check
    fn test_no_result_brillig_calls() {
        let program = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(Field 1)
            return Field 1
        }
        acir(inline) fn println f1 {
          b0(v0: Field):
            call f2(u1 1, v0)
            return
        }
        brillig(inline) fn print_unconstrained f2 {
          b0(v0: u1, v1: Field):
            return
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(false);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test for programs equivalent to the below (#10547):
    ///
    /// ```noir
    /// unconstrained fn identity(input: u64) -> u64 {
    ///     input
    /// }
    ///
    /// pub fn main(input: u32) {
    ///     let casted_input = input as u64;
    ///     let input_copy = unsafe { identity(casted_input) };
    ///     assert_eq(input_copy as Field, casted_input as Field);
    /// }
    /// ```
    fn multiple_casts_on_brillig_input_does_not_result_in_warning() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
            b0(v0: u32):
            v1 = cast v0 as u64
            v3 = call f1(v1) -> u64
            v4 = cast v3 as Field
            v5 = cast v0 as Field
            constrain v4 == v5
            return
        }
        brillig(inline) predicate_pure fn identity f1 {
            b0(v0: u64):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn truncating_brillig_argument_does_not_result_in_warning() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
            b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = call f1(v1) -> Field
            constrain v2 == v0
            return
        }
        brillig(inline) predicate_pure fn identity32 f1 {
            b0(v0: Field):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn constrain_on_independent_variable_can_indirectly_clear_results() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = call f1(v0) -> u32
            constrain v3 == v1       // This constraint does not connect the input of f1 to the output, so it doesn't clear.
            v4 = lt v1, u32 1000000  // This is a constraint against a constant, so it would clear if it was directly v3.
            constrain v4 == u1 1     // Since we asserted that v3 equals v1, this should indirectly clear v3.
            return
        }
        brillig(inline) predicate_pure fn f f1 {
          b0(v0: u32):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn constrain_on_array_element_links_to_input_array() {
        // Regression test for https://github.com/noir-lang/noir/issues/11807
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v3 = call f1(v1) -> Field
            constrain v3 == v0
            return v3
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 1]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }

    #[test]
    #[traced_test]
    fn constrain_on_nested_array_element_links_to_input_array() {
        // Nested array variant: [[Field; 1]; 1] wrapping v0
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v2 = make_array [v1] : [[Field; 1]; 1]
            v4 = call f1(v2) -> Field
            constrain v4 == v0
            return v4
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [[Field; 1]; 1]):
            v2 = array_get v0, index u32 0 -> [Field; 1]
            v3 = array_get v2, index u32 0 -> Field
            return v3
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }

    #[test]
    #[traced_test]
    fn array_set_with_variable_index_constrain_against_set_value() {
        // Array built from constants, then array_set with a non-constant index
        // inserts v0. Brillig result constrained against v0.
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [Field 0, Field 0] : [Field; 2]
            v3 = array_set v2, index v1, value v0
            v4 = call f1(v3) -> Field
            constrain v4 == v0
            return v4
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: array_set value should be tracked as a call argument."
        );
    }

    #[test]
    #[traced_test]
    fn array_set_on_param_array_constrain_against_original_element() {
        // make_array [v0, v1], then array_set at non-constant index with v0.
        // Brillig result constrained against v0 (which is both in the original
        // make_array AND the array_set value).
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u32):
            v3 = make_array [v0, v1] : [Field; 2]
            v4 = array_set v3, index v2, value v0
            v5 = call f1(v4) -> Field
            constrain v5 == v0
            return v5
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: array_set on make_array with params, constrained against original element."
        );
    }

    #[test]
    #[traced_test]
    fn array_set_constrain_result_array_elements() {
        // Brillig returns an array. We array_get each element and constrain
        // against the values used in the array_set. Since the Brillig call's
        // result is an array, the checker uses array element tracking.
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u32):
            v3 = make_array [v0, Field 0] : [Field; 2]
            v4 = array_set v3, index v2, value v1
            v5 = call f1(v4) -> [Field; 2]
            v6 = array_get v5, index u32 0 -> Field
            v7 = array_get v5, index u32 1 -> Field
            constrain v6 == v0
            constrain v7 == v1
            return
        }
        brillig(inline) predicate_pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            return v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: both array elements constrained against inputs."
        );
    }

    #[test]
    #[traced_test]
    fn outputs_do_not_trivially_connect_to_inputs() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1, v2 = call f1(v0) -> (u32, u32)
            constrain v1 == v2
            return
        }
        brillig(inline) predicate_pure fn f f1 {
          b0(v0: u32):
            return v0, v0
        }
        "#;

        let mut ssa = Ssa::from_str(program).unwrap();
        let ssa_level_warnings = ssa.check_for_missing_brillig_constraints(true);
        assert_eq!(
            ssa_level_warnings.len(),
            1,
            "We are constraining the outputs, but they are *not* connected to the inputs"
        );
    }
}
