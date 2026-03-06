//! Replaces `array_set` instructions with `make_array` instructions inside "conditional
//! windows" when the result of the `array_set` is only used within that same "conditional window".
//!
//! A "conditional window" is the sequence of instructions between
//! `enable_side_effects v_cond` and the matching `enable_side_effects u1 1`.
//!
//! The optimization is correct only within the window because `array_set` under a conditional
//! predicate is a conditional operation: when the predicate is false the result equals the
//! original array, not the modified one. If the result were used outside the window (where
//! the predicate might be false) a plain `make_array` would produce the wrong value.
//!
//! When the result *is* exclusive to the window the replacement is safe, and it is
//! beneficial: it exposes the constant set-value directly as a `make_array` element so
//! that [`super::remove_if_else`]'s `ValueMerger` can short-circuit the conditional multiply for
//! that element rather than emitting a conditional `array_get`.
//!
//! Example – this SSA:
//!
//! ```ssa
//! enable_side_effects v_cond
//! v_set = array_set v_arr, index u32 2, value Field 99
//! enable_side_effects u1 1
//! v_out = if v_cond then v_set else (if v_not_cond) v_arr
//! ```
//!
//! becomes:
//!
//! ```ssa
//! enable_side_effects v_cond
//! v0 = array_get v_arr, index u32 0 -> Field
//! v1 = array_get v_arr, index u32 1 -> Field
//! v_set = make_array [v0, v1, Field 99] : [Field; 3]
//! enable_side_effects u1 1
//! v_out = if v_cond then v_set else (if v_not_cond) v_arr
//! ```
//!
//! Because `v_set` is only used within the "conditional window", and then just as the input
//! of an `if-then-else` instruction, the `array_set` can be executed unconditionally because
//! the [`super::remove_if_else`] pass that comes after this pass will merge `v_set` with `v_arr` make sure
//! to only use the values from `v_set` when `v_cond` is true.

use std::collections::{HashMap, HashSet};

use acvm::{AcirField, FieldElement, acir::brillig::lengths::ElementTypesLength};
use im::Vector;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId},
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Replaces qualifying `array_set` instructions with `make_array` instructions.
    /// See the [`array_set_to_make_array`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_set_window_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.array_set_window_optimization();
        }
        self
    }
}

impl Function {
    fn array_set_window_optimization(&mut self) {
        // This optimization only applies to ACIR functions (same precondition as
        // `remove_if_else` which this pass is designed to feed into).
        if self.runtime().is_brillig() {
            return;
        }

        #[cfg(debug_assertions)]
        array_set_to_make_array_pre_check(self);

        let block_id = self.entry_block();

        // Pass 1: find which array_set instructions are eligible.
        let candidates = find_candidates(&self.dfg, block_id);
        if candidates.is_empty() {
            return;
        }

        // Pass 2: replace each candidate array_set with array_gets + make_array.
        self.simple_optimization(|context| {
            let inst_id = context.instruction_id;
            if !candidates.contains(&inst_id) {
                return;
            }

            let Instruction::ArraySet { array, index, value, .. } = *context.instruction() else {
                unreachable!("candidate must be an ArraySet instruction");
            };

            let Some(const_index) =
                context.dfg.get_numeric_constant(index).and_then(|v| v.try_to_u32())
            else {
                unreachable!("candidate ArraySet index must be a constant u32");
            };

            let Type::Array(ref element_types, len) = context.dfg.type_of_value(array) else {
                unreachable!("candidate ArraySet array must be of array type");
            };

            let array_constant = context.dfg.get_array_constant(array);

            let element_types = element_types.clone();
            let element_count = ElementTypesLength(element_types.len() as u32);
            let total_elements = len * element_count;

            // Remove the array_set; we will emit replacement instructions instead.
            context.remove_current_instruction();
            let [old_result] = context.dfg.instruction_result(inst_id);

            // Build the element list for the make_array.
            let mut elements = Vector::new();
            for semi_flattened_index in 0..total_elements.0 {
                if semi_flattened_index == const_index {
                    elements.push_back(value);
                } else if let Some((array_element, _)) = &array_constant {
                    elements.push_back(array_element[semi_flattened_index as usize]);
                } else {
                    let element_index = (semi_flattened_index % element_count.0) as usize;
                    let element_type = element_types[element_index].clone();
                    let index = context.dfg.make_constant(
                        FieldElement::from(u128::from(semi_flattened_index)),
                        NumericType::length_type(),
                    );
                    let get = Instruction::ArrayGet { array, index };
                    let get_result =
                        context.insert_instruction(get, Some(vec![element_type])).first();
                    elements.push_back(get_result);
                }
            }

            let typ = Type::Array(element_types, len);
            let make_array = Instruction::MakeArray { elements, typ: typ.clone() };
            let new_result = context.insert_instruction(make_array, Some(vec![typ])).first();
            context.replace_value(old_result, new_result);
        });
    }
}

#[cfg(debug_assertions)]
fn array_set_to_make_array_pre_check(func: &Function) {
    // flatten_cfg must have run
    super::checks::assert_cfg_is_flattened(func);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ConditionalWindowId(usize);

#[derive(Debug, Copy, Clone)]
struct ConditionalWindow {
    id: ConditionalWindowId,
    predicate: ValueId,
}

#[derive(Debug)]
struct TrackedValue {
    window: ConditionalWindow,
    // The set of original array_set candidate results that this value transitively depends on.
    // If any of these candidates is disqualified, this value must be disqualified too.
    dependencies: im::HashSet<ValueId>,
}

/// Returns the set of `InstructionId`s for `array_set` instructions that are safe
/// to replace with `make_array + array_get` — i.e. those whose result (and any value
/// transitively derived from it within the same window) is used only within that window
/// or in an `IfElse` whose `then_condition` matches the window's predicate.
///
/// A "derived value" is the result of any instruction inside the window that takes a
/// candidate (or another derived value) as an input — e.g. a `call` or another
/// `array_set`. If a derived value escapes the window, the candidates it was derived
/// from are disqualified: replacing them with unconditional `make_array`s would lead to
/// incorrect results.
fn find_candidates(dfg: &DataFlowGraph, block_id: BasicBlockId) -> HashSet<InstructionId> {
    // Maps array_set candidate results → instruction id.
    let mut candidates: HashMap<ValueId, InstructionId> = HashMap::new();
    // Maps every "tracked" value (a candidate or a value derived from candidates within the same window).
    let mut tracked: HashMap<ValueId, TrackedValue> = HashMap::new();

    let mut current_window: Option<ConditionalWindow> = None;
    let mut window_counter: ConditionalWindowId = ConditionalWindowId(0);

    let instructions = dfg[block_id].instructions();

    for &instruction_id in instructions {
        let instruction = &dfg[instruction_id];

        // A window gets closed or opened. This check must be done before checking for escaped
        // values because a tracked value used in an `enable_side_effects` instruction should be
        // considered as escaping the previous window.
        if let Instruction::EnableSideEffectsIf { condition } = instruction {
            let is_unconditional = dfg.get_numeric_constant(*condition).is_some_and(|v| v.is_one());
            if is_unconditional {
                current_window = None;
            } else {
                window_counter = ConditionalWindowId(window_counter.0 + 1);
                current_window =
                    Some(ConditionalWindow { id: window_counter, predicate: *condition });
            }
        }

        // Check whether any input to this instruction is a tracked value being used in
        // a context that escapes the conditional window.
        instruction.for_each_value(|value| {
            if let Some(tracked_value) = tracked.get(&value) {
                // A `store` instruction writes the value into memory from which it can be
                // loaded after the window closes — treat this as an escape even when the
                // store is inside the same window.
                let escapes_via_store = matches!(
                    instruction,
                    Instruction::Store { value: stored, .. } if *stored == value
                );
                // Determine whether this value stays within the window it was defined in.
                let stays_within_window = if escapes_via_store {
                    // If the value is stored in memory it could escape its window
                    false
                } else {
                    // Otherwise, if the pair of if "condition/value" matches the tracked value and its window,
                    // then the value might escape the window but through an `if-else` that's going to
                    // be merged, in which case we consider this as not escaping the window.
                    match instruction {
                        // Check the "then-condition/then-value" case.
                        Instruction::IfElse { then_condition, then_value, .. }
                            if *then_value == value =>
                        {
                            *then_condition == tracked_value.window.predicate
                        }
                        // Check the "else-condition/else-value" pair
                        Instruction::IfElse { else_condition, else_value, .. }
                            if *else_value == value =>
                        {
                            *else_condition == tracked_value.window.predicate
                        }
                        _ => {
                            // Otherwise: safe as long as we are inside the tracked window.
                            current_window
                                .is_some_and(|window| window.id == tracked_value.window.id)
                        }
                    }
                };
                if !stays_within_window {
                    let tracked_value_dependencies = tracked_value.dependencies.clone();

                    candidates.remove(&value);
                    tracked.remove(&value);
                    for dependency in tracked_value_dependencies {
                        candidates.remove(&dependency);
                        tracked.remove(&dependency);
                    }
                }
            }
        });

        match instruction {
            Instruction::EnableSideEffectsIf { .. } => {
                // This was already handled above
            }
            Instruction::ArraySet { array, index, value, mutable: false } => {
                if let Some(window) = current_window {
                    let [result] = dfg.instruction_result(instruction_id);

                    // array_set with a constant in-bound index is a candidate
                    if dfg.is_safe_index(*index, *array) {
                        candidates.insert(result, instruction_id);
                    }

                    // Dependendencies of this array_set are the array_set result itself
                    // (so we don't have to add tracked values in addition to their dependencies
                    // later on) and any dependencies of the array and value (the index doesn't
                    // matter as it's an integer).
                    let mut dependencies = im::HashSet::new();
                    dependencies.insert(result);
                    for value in [array, value] {
                        if let Some(tracked_value) = tracked.get(value) {
                            dependencies = dependencies.union(tracked_value.dependencies.clone());
                        }
                    }
                    tracked.insert(result, TrackedValue { window, dependencies });
                }
            }
            Instruction::IfElse { .. } => {
                // We already disqualified candidates used in `IfElse` that escape the current
                // window, so there's no need to keep track of `IfElse` dependencies.
            }
            _ => {
                // For any other instruction inside a window: if it consumes tracked values,
                // its results are "derived" and must be tracked with the union of their deps.
                //
                // Exception: a `call` with reference-typed arguments may store any of its
                // arguments through those references, causing them to escape the window
                // through memory. Treat all tracked arguments as escaping in that case.
                if let Some(current_window) = current_window {
                    let results = dfg.instruction_results(instruction_id);

                    let is_call_with_ref_args =
                        if let Instruction::Call { arguments, .. } = instruction {
                            arguments.iter().any(|&arg| dfg.type_of_value(arg).contains_reference())
                        } else {
                            false
                        };

                    let mut dependencies = im::HashSet::new();
                    instruction.for_each_value(|value| {
                        if let Some(tracked_value) = tracked.get(&value) {
                            let tracked_value_dependencies = tracked_value.dependencies.clone();
                            if is_call_with_ref_args {
                                for value in tracked_value_dependencies {
                                    candidates.remove(&value);
                                    tracked.remove(&value);
                                }
                            } else if !results.is_empty() {
                                dependencies =
                                    dependencies.clone().union(tracked_value_dependencies);
                            }
                        }
                    });
                    if !dependencies.is_empty() {
                        for &result in results {
                            tracked.insert(
                                result,
                                TrackedValue {
                                    window: current_window,
                                    dependencies: dependencies.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    // Also check the block terminator: if a tracked value is used there it escapes.
    if let Some(terminator) = dfg[block_id].terminator() {
        terminator.for_each_value(|value| {
            if let Some(tracked_value) = tracked.get(&value) {
                let tracked_value_dependencies = tracked_value.dependencies.clone();
                for value in tracked_value_dependencies {
                    candidates.remove(&value);
                    tracked.remove(&value);
                }
            }
        });
    }

    candidates.into_values().collect()
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

    use super::Ssa;

    /// Basic case: array_set result is only used inside the conditional window.
    /// It should be replaced with make_array + array_gets.
    #[test]
    fn replaces_array_set_used_only_within_conditional_window() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v5 = array_set v0, index u32 1, value Field 99
            enable_side_effects u1 1
            v7 = if v1 then v5 else (if v2) v0
            return v7
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v4 = array_get v0, index u32 0 -> Field
            v6 = array_get v0, index u32 2 -> Field
            v8 = make_array [v4, Field 99, v6] : [Field; 3]
            enable_side_effects u1 1
            v10 = if v1 then v8 else (if v2) v0
            return v10
        }
        ");
    }

    /// The array_set result is used *after* the conditional window closes.
    /// The optimization must not apply.
    #[test]
    fn does_not_replace_array_set_used_after_window() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            enable_side_effects v1
            v4 = array_set v0, index u32 0, value Field 7
            enable_side_effects u1 1
            v6 = array_get v4, index u32 0 -> Field
            return v6
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// A chain of array_sets where each result feeds into the next, and the final
    /// result escapes the window. Neither the final nor the intermediate one can be
    /// replaced: replacing the intermediate `v2` would corrupt `v3`'s value when the
    /// predicate is false (its false-predicate result is `v2`, which would then be the
    /// wrong always-modified make_array instead of the original array).
    #[test]
    fn does_not_replace_array_set_chain_when_final_escapes_and_has_constant_index() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            enable_side_effects v1
            v4 = array_set v0, index u32 0, value Field 1
            v7 = array_set v4, index u32 1, value Field 2
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> Field
            return v9
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    #[test]
    fn does_not_replace_array_set_chain_when_final_escapes_and_has_unknown_index() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1, v10: u32):
            enable_side_effects v1
            v4 = array_set v0, index u32 0, value Field 1
            v7 = array_set v4, index v10, value Field 2
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> Field
            return v9
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// Both array_sets in the window feed into an IfElse — neither escapes.
    /// Both should be replaced.
    #[test]
    fn replaces_chain_of_array_sets_when_none_escape() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v3 = array_set v0, index u32 0, value Field 1
            v4 = array_set v3, index u32 1, value Field 2
            enable_side_effects u1 1
            v5 = if v1 then v4 else (if v2) v0
            return v5
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v4 = array_get v0, index u32 1 -> Field
            v6 = array_get v0, index u32 2 -> Field
            v8 = make_array [Field 1, v4, v6] : [Field; 3]
            v10 = make_array [Field 1, Field 2, v6] : [Field; 3]
            enable_side_effects u1 1
            v12 = if v1 then v10 else (if v2) v0
            return v12
        }
        ");
    }

    /// An example with an array_set + call where both are used exclusively in the conditional window
    #[test]
    fn replaces_array_set_that_feeds_poseidon2_inside_conditional_window() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u32, v2: u1, v3: u1, v4: u1, v5: u1):
            v7 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v9 = call poseidon2_permutation(v0) -> [Field; 4]
            v10 = if v3 then v9 else (if v2) v7
            enable_side_effects v4
            v13 = array_set v10, index u32 0, value Field 6
            v14 = call poseidon2_permutation(v13) -> [Field; 4]
            enable_side_effects u1 1
            v16 = if v4 then v14 else (if v5) v10
            return v16
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u32, v2: u1, v3: u1, v4: u1, v5: u1):
            v7 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v9 = call poseidon2_permutation(v0) -> [Field; 4]
            v10 = if v3 then v9 else (if v2) v7
            enable_side_effects v4
            v12 = array_get v10, index u32 1 -> Field
            v14 = array_get v10, index u32 2 -> Field
            v16 = array_get v10, index u32 3 -> Field
            v18 = make_array [Field 6, v12, v14, v16] : [Field; 4]
            v19 = call poseidon2_permutation(v18) -> [Field; 4]
            enable_side_effects u1 1
            v21 = if v4 then v19 else (if v5) v10
            return v21
        }
        ");
    }

    /// Another example with array_set -> call -> array_set where the last array_set escapes
    /// the window so the first one cannot be optimized.
    #[test]
    fn does_not_replace_chain_of_array_set_call_array_set_where_last_one_is_used_outside_window() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u32, v2: u1, v3: u1, v4: u1, v5: u1):
            v7 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v9 = call poseidon2_permutation(v0) -> [Field; 4]
            v10 = if v3 then v9 else (if v2) v7
            enable_side_effects v4
            v13 = array_set v10, index u32 0, value Field 6
            v14 = call poseidon2_permutation(v13) -> [Field; 4]
            v17 = array_set v14, index u32 1, value Field 7
            enable_side_effects u1 1
            v19 = if v4 then v14 else (if v5) v10
            return v17, v19
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// The array_set feeds a call whose result escapes the window (not via IfElse).
    /// Even though the array_set itself is only used inside the window, replacing it
    /// would corrupt the call's result when the predicate is false.
    #[test]
    fn does_not_replace_array_set_when_derived_value_escapes() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            enable_side_effects v1
            v2 = array_set v0, index u32 0, value Field 6
            v3 = call poseidon2_permutation(v2) -> [Field; 4]
            enable_side_effects u1 1
            v4 = array_get v3, index u32 0 -> Field
            return v4
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// An array_set outside any conditional window (no enable_side_effects wrapping it)
    /// should not be touched by this pass.
    #[test]
    fn does_not_touch_array_set_outside_conditional_window() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v1 = array_set v0, index u32 0, value Field 5
            return v1
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// Because `v2` is stored in `v10` and loaded outside of the conditional window,
    /// `v2` shouldn't be optimized.
    #[test]
    fn does_not_optimize_when_array_set_result_is_stored() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            v10 = allocate -> &mut [Field; 4]
            enable_side_effects v1
            v2 = array_set v0, index u32 0, value Field 6
            store v2 at v10
            enable_side_effects u1 1
            v3 = load v10 -> [Field; 4]
            v4 = array_get v3, index u32 0 -> Field
            return v4
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// The array_set result is passed to a call whose argument type contains a reference
    /// nested inside an array ([&mut Field; 1]). The callee could store elements of the
    /// array through those nested references, so the optimization must not apply.
    #[test]
    fn does_not_replace_array_set_when_passed_to_call_with_nested_ref_in_array_arg() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            v10 = allocate -> &mut Field
            v11 = make_array [v10] : [&mut Field; 1]
            enable_side_effects v1
            v2 = array_set v0, index u32 0, value Field 6
            call f1(v2, v11)
            enable_side_effects u1 1
            v3 = load v10 -> Field
            return v3
        }
        acir(inline) fn f1 f1 {
          b0(v0: [Field; 4], v1: [&mut Field; 1]):
            return
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// The array_set result is passed to a call that also takes a reference argument.
    /// The callee (f1) may store the array through the reference, causing it to escape
    /// the conditional window. The optimization must not apply.
    #[test]
    fn does_not_replace_array_set_when_passed_to_call_with_ref_arg() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            v10 = allocate -> &mut [Field; 4]
            enable_side_effects v1
            v2 = array_set v0, index u32 0, value Field 6
            call f1(v2, v10)
            enable_side_effects u1 1
            v3 = load v10 -> [Field; 4]
            v4 = array_get v3, index u32 0 -> Field
            return v4
        }
        acir(inline) fn store_in_ref f1 {
          b0(v0: [Field; 4], v1: &mut [Field; 4]):
            store v0 at v1
            return
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// The array_set result appears as the `else_value` of an IfElse whose
    /// `else_condition` matches the window predicate — the symmetric safe pattern.
    /// `if (not v1) then v0 else v2` with else_condition=v1 is equivalent to
    /// `if v1 then v2 else v0`, so the optimization should apply.
    #[test]
    fn replaces_array_set_used_as_safe_else_value() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v3 = array_set v0, index u32 1, value Field 99
            enable_side_effects u1 1
            v4 = if v2 then v0 else (if v1) v3
            return v4
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v4 = array_get v0, index u32 0 -> Field
            v6 = array_get v0, index u32 2 -> Field
            v8 = make_array [v4, Field 99, v6] : [Field; 3]
            enable_side_effects u1 1
            v10 = if v2 then v0 else (if v1) v8
            return v10
        }
        ");
    }

    /// The array_set result appears as the `else_value` of an IfElse whose
    /// `else_condition` does NOT match the window predicate. This is unsafe:
    /// when the predicate is false the else branch could be taken, yielding an
    /// unconditionally-modified make_array instead of the original array.
    #[test]
    fn does_not_replace_array_set_when_else_value_has_wrong_condition() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1, v5: u1):
            v2 = not v5
            enable_side_effects v1
            v3 = array_set v0, index u32 1, value Field 99
            enable_side_effects u1 1
            v4 = if v2 then v0 else (if v5) v3
            return v4
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    #[test]
    fn does_not_replace_array_set_when_dependent_value_is_used_in_enable_side_effects() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [u1; 3], v1: u1, v5: u1):
            v2 = not v5
            enable_side_effects v1
            v3 = array_set v0, index u32 1, value u1 0
            v4 = array_get v3, index u32 1 -> u1
            enable_side_effects v4
            return
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }
}
