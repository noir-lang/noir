//! Replaces `array_set` instructions with `make_array` instructions inside "conditional
//! windows" when the result of the `array_set` is only used within that same "conditional window"
//! and eventually in an `IfElse` instruction under the predicate associated with the conditional
//! window.
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
//!
//! Note 1: this optimization applies to both arrays and vectors. For arrays the length is
//! statically known from the type. For vectors the capacity must be determinable via
//! [DataFlowGraph::try_get_vector_capacity] (e.g. the vector traces back to a `make_array`).
//!
//! Note 2: because the optimization expands to multiple `array_get` and a `make_array` instruction,
//! for large arrays this might result in too many `array_get` instructions that slow down SSA optimization.
//! For this reason, only small arrays (up to `MAX_ARRAY_SEMI_FLATTENED_LENGTH` elements) are optimized by this pass.
use std::collections::{HashMap, HashSet};

use acvm::{AcirField, FieldElement, acir::brillig::lengths::ElementTypesLength};
use im::Vector;

use crate::{
    brillig::assert_u32,
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            dfg::DataFlowGraph,
            function::Function,
            instruction::{Instruction, InstructionId},
            types::{NumericType, Type},
            value::ValueId,
        },
        ssa_gen::Ssa,
    },
};

/// The maximum length of arrays that this optimization will apply to.
const MAX_ARRAY_SEMI_FLATTENED_LENGTH: u32 = 64;

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

            let typ = context.dfg.type_of_value(array);
            let element_types = typ.element_types();
            let len = context
                .dfg
                .try_get_vector_capacity(array)
                .expect("candidate ArraySet must have a known capacity");

            let array_constant = context.dfg.get_array_constant(array);
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

#[derive(Debug, Clone)]
struct TrackedValue {
    window: ConditionalWindow,
    /// Indices into the `candidate_values` vec — the set of array_set candidates
    /// this value transitively depends on. Only candidate indices are stored,
    /// not intermediate ValueIds, keeping the sets small (bounded by candidate count).
    candidate_deps: HashSet<usize>,
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
    // Candidates are numbered as discovered. Maps index → (ValueId, InstructionId).
    let mut candidate_values: Vec<(ValueId, InstructionId)> = Vec::new();
    // Maps every "tracked" value (a candidate or a value derived from candidates within the same window).
    let mut tracked: HashMap<ValueId, TrackedValue> = HashMap::new();
    // Candidate indices confirmed used in an `IfElse` instruction under their associated predicate.
    let mut if_else_candidate_indices = HashSet::<usize>::new();
    // Candidate indices disqualified because a derived value escaped its window.
    let mut disqualified_indices = HashSet::<usize>::new();

    let mut current_window: Option<ConditionalWindow> = None;
    let mut window_counter: ConditionalWindowId = ConditionalWindowId(0);

    // Remember all windows by their predicate. In case a new window shows up with a predicate we've
    // already seen in a previous window, we can consider it to be the same window.
    let mut windows_by_predicate = HashMap::<ValueId, ConditionalWindow>::new();

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
            } else if let Some(existing_window) = windows_by_predicate.get(condition) {
                current_window = Some(*existing_window);
            } else {
                window_counter = ConditionalWindowId(window_counter.0 + 1);
                let new_window = ConditionalWindow { id: window_counter, predicate: *condition };
                current_window = Some(new_window);
                windows_by_predicate.insert(*condition, new_window);
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
                // A call with reference arguments might store the value in one of those references
                let escapes_via_call_arguments = matches!(
                    instruction,
                    Instruction::Call { arguments, .. } if arguments.iter().any(|&arg| dfg.type_of_value(arg).contains_reference()),
                );

                // Determine whether this value stays within the window it was defined in.
                let stays_within_window = if escapes_via_store || escapes_via_call_arguments {
                    false
                } else {
                    // Otherwise, if the pair of if "condition/value" matches the tracked value and its window,
                    // then the value might escape the window but through an `if-else` that's going to
                    // be merged, in which case we consider this as not escaping the window.
                    match instruction {
                        // Check the "then-condition/then-value" case.
                        Instruction::IfElse { then_condition, then_value, .. }
                            if *then_value == value && *then_condition == tracked_value.window.predicate =>
                        {
                            if_else_candidate_indices.extend(&tracked_value.candidate_deps);
                            true
                        }
                        // Check the "else-condition/else-value" pair
                        Instruction::IfElse { else_condition, else_value, .. }
                            if *else_value == value && *else_condition == tracked_value.window.predicate =>
                        {
                            if_else_candidate_indices.extend(&tracked_value.candidate_deps);
                            true
                        }
                        _ => {
                            // Otherwise: safe as long as we are inside the tracked window.
                            current_window
                                .is_some_and(|window| window.id == tracked_value.window.id)
                        }
                    }
                };
                if !stays_within_window {
                    let candidate_deps = tracked_value.candidate_deps.clone();
                    tracked.remove(&value);
                    disqualified_indices.extend(&candidate_deps);
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

                    // array_set with a constant in-bound index on a small array or vector
                    let is_candidate = if let Some(index) = dfg.get_numeric_constant(*index) {
                        let semi_flattened_length = match dfg.type_of_value(*array) {
                            Type::Array(elements, len) => {
                                let elements_length =
                                    ElementTypesLength(assert_u32(elements.len()));
                                Some(len * elements_length)
                            }
                            Type::Vector(elements) => {
                                dfg.try_get_vector_capacity(*array).map(|capacity| {
                                    let elements_length =
                                        ElementTypesLength(assert_u32(elements.len()));
                                    capacity * elements_length
                                })
                            }
                            _ => None,
                        };
                        semi_flattened_length.is_some_and(|len| {
                            index.to_u128() < u128::from(len.0)
                                && len.0 <= MAX_ARRAY_SEMI_FLATTENED_LENGTH
                        })
                    } else {
                        false
                    };

                    // Build deps from tracked inputs.
                    let mut candidate_deps = HashSet::new();
                    for input in [array, value] {
                        if let Some(tracked_input) = tracked.get(input) {
                            candidate_deps.extend(&tracked_input.candidate_deps);
                        }
                    }

                    if is_candidate {
                        let idx = candidate_values.len();
                        candidate_values.push((result, instruction_id));
                        candidate_deps.insert(idx);
                    }

                    tracked.insert(result, TrackedValue { window, candidate_deps });
                }
            }
            Instruction::IfElse { .. } => {
                // We already disqualified candidates used in `IfElse` that escape the current
                // window, so there's no need to keep track of `IfElse` dependencies.
            }
            _ => {
                // For any other instruction inside a window: if it consumes tracked values,
                // its results are "derived" and must be tracked with the union of their deps.
                if let Some(current_window) = current_window {
                    let results = dfg.instruction_results(instruction_id);
                    if !results.is_empty() {
                        let mut candidate_deps = HashSet::new();
                        instruction.for_each_value(|value| {
                            if let Some(tracked_value) = tracked.get(&value) {
                                candidate_deps.extend(&tracked_value.candidate_deps);
                            }
                        });
                        if !candidate_deps.is_empty() {
                            for &result in results {
                                tracked.insert(
                                    result,
                                    TrackedValue {
                                        window: current_window,
                                        candidate_deps: candidate_deps.clone(),
                                    },
                                );
                            }
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
                disqualified_indices.extend(&tracked_value.candidate_deps);
                tracked.remove(&value);
            }
        });
    }

    // A candidate is valid if it was seen in an IfElse and is not disqualified.
    candidate_values
        .into_iter()
        .enumerate()
        .filter_map(|(idx, (_value_id, instruction_id))| {
            let in_if_else = if_else_candidate_indices.contains(&idx);
            let is_disqualified = disqualified_indices.contains(&idx);
            (in_if_else && !is_disqualified).then_some(instruction_id)
        })
        .collect()
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
            v2 = not v1
            enable_side_effects v1
            v4 = array_set v0, index u32 0, value Field 7
            enable_side_effects u1 1
            v6 = array_get v4, index u32 0 -> Field
            v10 = if v1 then v4 else (if v2) v0
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
            v2 = not v1
            enable_side_effects v1
            v4 = array_set v0, index u32 0, value Field 1
            v7 = array_set v4, index u32 1, value Field 2
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> Field
            v10 = if v1 then v4 else (if v2) v0
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
            v2 = not v1
            enable_side_effects v1
            v4 = array_set v0, index u32 0, value Field 1
            v7 = array_set v4, index v10, value Field 2
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> Field
            v11 = if v1 then v4 else (if v2) v0
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
            v99 = not v1
            enable_side_effects v1
            v2 = array_set v0, index u32 0, value Field 6
            v3 = call poseidon2_permutation(v2) -> [Field; 4]
            enable_side_effects u1 1
            v4 = array_get v3, index u32 0 -> Field
            v10 = if v1 then v2 else (if v99) v0
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

    #[test]
    fn does_not_touch_array_set_that_is_not_eventually_used_in_an_if_else() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            v2 = not v1
            enable_side_effects v1
            v5 = array_set v0, index u32 1, value Field 99
            enable_side_effects u1 1
            return
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

    #[test]
    fn replaces_array_set_used_only_within_conditional_windows_with_the_same_predicate() {
        // Here v5 is used outside it's original window, but it's used in a second window
        // under the same predicate as the first one, so we consider those to be the same window.
        let src = r#"
        acir(inline) fn main f0 {
        b0(v0: [Field; 3], v1: u1, v2: u1):
            v3 = not v1
            enable_side_effects v1
            v6 = array_set v0, index u32 1, value Field 99
            enable_side_effects v2
            v7 = not v3
            enable_side_effects v1
            v8 = array_get v6, index u32 1 -> Field
            enable_side_effects u1 1
            v10 = if v1 then v6 else (if v3) v0
            return v10
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1, v2: u1):
            v3 = not v1
            enable_side_effects v1
            v5 = array_get v0, index u32 0 -> Field
            v7 = array_get v0, index u32 2 -> Field
            v9 = make_array [v5, Field 99, v7] : [Field; 3]
            enable_side_effects v2
            v10 = not v3
            enable_side_effects v1
            enable_side_effects u1 1
            v12 = if v1 then v9 else (if v3) v0
            return v12
        }
        ");
    }

    /// Vector from `make_array` — capacity is known, optimization applies.
    #[test]
    fn replaces_vector_array_set_from_make_array() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v1: u1):
            v2 = not v1
            v3 = make_array [Field 10, Field 20, Field 30] : [Field]
            enable_side_effects v1
            v5 = array_set v3, index u32 1, value Field 99
            enable_side_effects u1 1
            v7 = if v1 then v5 else (if v2) v3
            return v7
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = not v0
            v5 = make_array [Field 10, Field 20, Field 30] : [Field]
            enable_side_effects v0
            v7 = make_array [Field 10, Field 99, Field 30] : [Field]
            enable_side_effects u1 1
            v9 = if v0 then v7 else (if v1) v5
            return v9
        }
        ");
    }

    /// Vector returned by a call — `try_get_vector_capacity` returns None,
    /// so the optimization should not apply.
    #[test]
    fn does_not_replace_vector_array_set_with_unknown_capacity() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v1: u1):
            v2 = not v1
            v3 = call f1() -> [Field]
            enable_side_effects v1
            v5 = array_set v3, index u32 1, value Field 99
            enable_side_effects u1 1
            v7 = if v1 then v5 else (if v2) v3
            return v7
        }
        acir(inline) fn get_vector f1 {
          b0():
            v0 = make_array [Field 1, Field 2, Field 3] : [Field]
            return v0
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_window_optimization);
    }

    /// Vector from `array_set` on a `make_array` — capacity traced through the chain.
    #[test]
    fn replaces_vector_array_set_through_array_set_chain() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v1: u1, v10: Field):
            v2 = not v1
            v3 = make_array [Field 10, Field 20, Field 30] : [Field]
            v4 = array_set v3, index u32 0, value v10
            enable_side_effects v1
            v6 = array_set v4, index u32 2, value Field 99
            enable_side_effects u1 1
            v8 = if v1 then v6 else (if v2) v4
            return v8
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            v2 = not v0
            v6 = make_array [Field 10, Field 20, Field 30] : [Field]
            v8 = array_set v6, index u32 0, value v1
            enable_side_effects v0
            v9 = array_get v8, index u32 0 -> Field
            v11 = make_array [v9, Field 20, Field 99] : [Field]
            enable_side_effects u1 1
            v13 = if v0 then v11 else (if v2) v8
            return v13
        }
        ");
    }

    /// Two independent candidates A (v4) and B (v5) are connected through a derived
    /// value v7 = array_set(v5, v6_from_A). Candidate A's chain escapes (v6 used
    /// outside the window), but B is correctly used in IfElse. B should still be
    /// optimized because its own uses are all within the window.
    #[test]
    fn optimizes_independent_candidate_when_sibling_chain_escapes() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 3], v2: u1):
            v3 = not v2
            enable_side_effects v2
            v4 = array_set v0, index u32 0, value Field 1
            v5 = array_set v1, index u32 0, value Field 2
            v6 = array_get v4, index u32 0 -> Field
            v7 = array_set v5, index u32 1, value v6
            enable_side_effects u1 1
            v8 = add v6, Field 0
            v9 = if v2 then v5 else (if v3) v1
            return v8, v9
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_window_optimization();
        // v5 (candidate B) should be replaced with make_array — its own uses
        // stay within the window, even though v4 (candidate A) was disqualified.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 3], v2: u1):
            v3 = not v2
            enable_side_effects v2
            v6 = array_set v0, index u32 0, value Field 1
            v8 = array_get v1, index u32 1 -> Field
            v10 = array_get v1, index u32 2 -> Field
            v12 = make_array [Field 2, v8, v10] : [Field; 3]
            v13 = array_get v6, index u32 0 -> Field
            v14 = array_set v12, index u32 1, value v13
            enable_side_effects u1 1
            v17 = add v13, Field 0
            v18 = if v2 then v12 else (if v3) v1
            return v17, v18
        }
        ");
    }
}
