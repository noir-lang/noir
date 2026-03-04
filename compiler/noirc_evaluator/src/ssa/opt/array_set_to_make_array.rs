//! Replaces `array_set` instructions with `make_array` instructions inside conditional
//! blocks when the result of the `array_set` is only used within that same conditional block.
//!
//! A "conditional block" is the sequence of instructions between
//! `enable_side_effects v_cond` and the matching `enable_side_effects u1 1`.
//!
//! The optimization is correct only within the block because `array_set` under a conditional
//! predicate is a conditional operation: when the predicate is false the result equals the
//! original array, not the modified one. If the result were used outside the block (where
//! the predicate might be false) a plain `make_array` would produce the wrong value.
//!
//! When the result *is* exclusive to the block the replacement is safe, and it is
//! beneficial: it exposes the constant set-value directly as a `make_array` element so
//! that `remove_if_else`'s `ValueMerger` can short-circuit the conditional multiply for
//! that element rather than emitting a conditional `array_get`.
//!
//! Example – this SSA:
//! ```ssa
//! enable_side_effects v_cond
//! v_set = array_set v_arr, index u32 2, value Field 99
//! enable_side_effects u1 1
//! v_out = if v_cond then v_set else (if v_not_cond) v_arr
//! ```
//!
//! becomes:
//! ```ssa
//! enable_side_effects v_cond
//! v0 = array_get v_arr, index u32 0 -> Field
//! v1 = array_get v_arr, index u32 1 -> Field
//! v_set = make_array [v0, v1, Field 99] : [Field; 3]
//! enable_side_effects u1 1
//! v_out = if v_cond then v_set else (if v_not_cond) v_arr
//! ```

use std::collections::{HashMap, HashSet};

use acvm::{AcirField, FieldElement};
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
    pub(crate) fn array_set_to_make_array(mut self) -> Self {
        for func in self.functions.values_mut() {
            if func.runtime().is_brillig() {
                continue;
            }
            func.array_set_to_make_array();
        }
        self
    }
}

impl Function {
    fn array_set_to_make_array(&mut self) {
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

            // Clone needed values out of the instruction before mutating.
            let Instruction::ArraySet { array, index, value, .. } = *context.instruction() else {
                return;
            };

            let Some(const_index) =
                context.dfg.get_numeric_constant(index).and_then(|v| v.try_to_u32())
            else {
                return;
            };

            let Type::Array(ref element_types, len) = context.dfg.type_of_value(array) else {
                return;
            };

            let element_types = element_types.clone();
            let element_count = element_types.len() as u32;
            let total_elements = len.0 * element_count;

            // Remove the array_set; we will emit replacement instructions instead.
            context.remove_current_instruction();
            let [old_result] = context.dfg.instruction_result(inst_id);

            // Build the element list for the make_array.
            let mut elements = Vector::new();
            for flat_i in 0..total_elements {
                if flat_i == const_index {
                    elements.push_back(value);
                } else {
                    let element_index = (flat_i % element_count) as usize;
                    let element_type = element_types[element_index].clone();
                    let idx = context.dfg.make_constant(
                        FieldElement::from(u128::from(flat_i)),
                        NumericType::length_type(),
                    );
                    let get = Instruction::ArrayGet { array, index: idx };
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

/// Returns the set of `InstructionId`s for `array_set` instructions that are safe
/// to replace with `make_array + array_get` — i.e. those whose result (and any value
/// transitively derived from it within the same window) is used only within that window
/// or in an `IfElse` whose `then_condition` matches the window's predicate.
///
/// A "derived value" is the result of any instruction inside the window that takes a
/// candidate (or another derived value) as an input — e.g. a `call` or another
/// `array_set`. If a derived value escapes unsafely, the candidates it was derived
/// from are disqualified: replacing them with unconditional `make_array`s would corrupt
/// the derived value when the predicate is false.
fn find_candidates(dfg: &DataFlowGraph, block_id: BasicBlockId) -> HashSet<InstructionId> {
    // Maps array_set candidate results → (instruction id, window id, window predicate).
    let mut candidates: HashMap<ValueId, (InstructionId, u32, ValueId)> = HashMap::new();
    // Maps every "tracked" value (a candidate or a value derived from candidates within the
    // same window) → (window_id, window_predicate, candidate_deps).
    // `candidate_deps` is the set of original array_set candidate results that this value
    // transitively depends on; escaping a tracked value disqualifies all its deps.
    let mut tracked: HashMap<ValueId, (u32, ValueId, im::HashSet<ValueId>)> = HashMap::new();
    let mut disqualified: HashSet<ValueId> = HashSet::new();

    let mut current_window: Option<(u32, ValueId)> = None;
    let mut window_counter: u32 = 0;

    let instructions = dfg[block_id].instructions();

    for &inst_id in instructions {
        let instruction = &dfg[inst_id];

        // Check whether any input to this instruction is a tracked value being used in
        // a context that lets it escape the conditional window.
        instruction.for_each_value(|value| {
            if let Some(&(tracked_window, window_predicate, ref deps)) = tracked.get(&value) {
                let current_window_id = current_window.map(|(wid, _)| wid);
                let used_outside_window = current_window_id != Some(tracked_window);
                // A `store` instruction writes the value into memory from which it can be
                // loaded after the window closes — treat this as an escape even when the
                // store is inside the same window.
                let escapes_via_store = matches!(
                    instruction,
                    Instruction::Store { value: stored, .. } if *stored == value
                );
                if used_outside_window || escapes_via_store {
                    let is_safe = !escapes_via_store
                        && matches!(
                            instruction,
                            Instruction::IfElse { then_condition, then_value, .. }
                            if *then_condition == window_predicate && *then_value == value
                        );
                    if !is_safe {
                        disqualified.extend(deps.iter().copied());
                    }
                }
            }
        });

        match instruction {
            Instruction::EnableSideEffectsIf { condition } => {
                let is_unconditional =
                    dfg.get_numeric_constant(*condition).is_some_and(|v| v.is_one());
                if is_unconditional {
                    current_window = None;
                } else {
                    window_counter += 1;
                    current_window = Some((window_counter, *condition));
                }
            }
            Instruction::ArraySet { array, index, mutable: false, .. } => {
                if let Some((window_id, predicate)) = current_window
                    && dfg.is_safe_index(*index, *array)
                {
                    let [result] = dfg.instruction_result(inst_id);
                    candidates.insert(result, (inst_id, window_id, predicate));
                    // Deps = {self} ∪ deps_of(array input).
                    // Only the array input matters: when the predicate is false the
                    // array_set result *equals* the array input, so a wrong array input
                    // (one replaced by an unconditional make_array) would corrupt this
                    // instruction's false-predicate value too.
                    let mut deps = im::HashSet::new();
                    deps.insert(result);
                    if let Some((_, _, array_deps)) = tracked.get(array) {
                        for dep in array_deps {
                            deps.insert(*dep);
                        }
                    }
                    tracked.insert(result, (window_id, predicate, deps));
                }
            }
            _ => {
                // For any other instruction inside a window: if it consumes tracked values,
                // its results are "derived" and must be tracked with the union of their deps.
                //
                // Exception: a `call` with reference-typed arguments may store any of its
                // arguments through those references, causing them to escape the window
                // through memory. Treat all tracked arguments as escaping in that case.
                if let Some((window_id, predicate)) = current_window {
                    let is_call_with_ref_args =
                        if let Instruction::Call { arguments, .. } = instruction {
                            arguments.iter().any(|&arg| dfg.type_of_value(arg).contains_reference())
                        } else {
                            false
                        };

                    let mut deps: im::HashSet<ValueId> = im::HashSet::new();
                    instruction.for_each_value(|value| {
                        if let Some((_, _, value_deps)) = tracked.get(&value) {
                            if is_call_with_ref_args {
                                disqualified.extend(value_deps.iter().copied());
                            } else {
                                for dep in value_deps {
                                    deps.insert(*dep);
                                }
                            }
                        }
                    });
                    if !is_call_with_ref_args && !deps.is_empty() {
                        for &result in dfg.instruction_results(inst_id) {
                            tracked.insert(result, (window_id, predicate, deps.clone()));
                        }
                    }
                }
            }
        }
    }

    // Also check the block terminator: if a tracked value is used there it escapes.
    if let Some(terminator) = dfg[block_id].terminator() {
        terminator.for_each_value(|value| {
            if let Some((_, _, deps)) = tracked.get(&value) {
                disqualified.extend(deps.iter().copied());
            }
        });
    }

    candidates
        .into_iter()
        .filter(|(result, _)| !disqualified.contains(result))
        .map(|(_, (inst_id, _, _))| inst_id)
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
            v3 = array_set v0, index u32 1, value Field 99
            enable_side_effects u1 1
            v4 = if v1 then v3 else (if v2) v0
            return v4
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_to_make_array();
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
            v2 = array_set v0, index u32 0, value Field 7
            enable_side_effects u1 1
            v3 = array_get v2, index u32 0 -> Field
            return v3
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
    }

    /// A chain of array_sets where each result feeds into the next, and the final
    /// result escapes the window. Neither the final nor the intermediate one can be
    /// replaced: replacing the intermediate `v2` would corrupt `v3`'s value when the
    /// predicate is false (its false-predicate result is `v2`, which would then be the
    /// wrong always-modified make_array instead of the original array).
    #[test]
    fn does_not_replace_array_set_chain_when_final_escapes() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1):
            enable_side_effects v1
            v2 = array_set v0, index u32 0, value Field 1
            v3 = array_set v2, index u32 1, value Field 2
            enable_side_effects u1 1
            v4 = array_get v3, index u32 0 -> Field
            return v4
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
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
        let ssa = ssa.array_set_to_make_array();
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

    /// Mirrors the motivating example from the `remove_if_else` test:
    /// the array_set feeds a poseidon2 call whose result is merged via IfElse.
    /// The array_set is only used within the window so it should be replaced.
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
        let ssa = ssa.array_set_to_make_array();
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
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
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
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
    }

    /// Brillig functions should not be touched.
    #[test]
    fn does_not_touch_brillig_functions() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v1 = array_set v0, index u32 0, value Field 5
            return v1
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
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
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
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
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
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
        assert_ssa_does_not_change(src, Ssa::array_set_to_make_array);
    }
}
