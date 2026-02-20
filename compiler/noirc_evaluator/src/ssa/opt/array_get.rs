//! Replaces `array_get` with known indices with values from previous instructions
//! such as `array_set` or `make_array`.
//!
//! Given these two instructions:
//!
//! ```text
//! v1 = array_set v0, index 0, value: 42
//! v2 = array_get v1, index 0 -> Field
//! ```
//!
//! because we get from `v1` at `index 0`, but `v1` is the result of setting the value "42"
//! at `index 0`, we can conclude that `v2` will be "42", and so this SSA pass will do that.
//! However, this is only safe to do if the `array_set` happened under the same side effects
//! variable as the `array_get`. For example, in this case:
//!
//! ```text
//! enable_side_effects v100
//! v1 = array_set v0, index 0, value: 42
//! enable_side_effects v200
//! v2 = array_get v1, index 0 -> Field
//! ```
//!
//! it would be wrong to replace `v2` with "42" as the previous array_set might not have
//! been executed.
//!
//! In this case:
//!
//! ```text
//! v1 = array_set v0, index 0, value: 42
//! v2 = array_set v1, index 1, value: 15
//! v3 = array_get v2, index 0 -> Field
//! ```
//!
//! for `v3` the optimization will try to find a previous `array_set` with the same index (`index 0`).
//! It will first find `v2`. Because it's an `array_set` of a different **known** index, it will
//! then find `v1` and apply the same optimization as before. Note that it's safe to skip `v2` and
//! look at `v1` even if `v2` was under a different side effects var because it doesn't affect
//! the index used in `v3`.
//!
//! In this case:
//!
//! ```text
//! v1 = array_set v0, index 0, value: 42
//! v2 = array_set v1, index v88, value: 15
//! v3 = array_get v2, index 0 -> Field
//! ```
//!
//! for `v3` the optimization will find `v2`. Because the set index is unknown, and it might be zero,
//! the optimization can't deduce anything so it won't do anything.
//!
//! Another case where the optimization applies is when it finds a `make_array`:
//!
//! ```text
//! v1 = make_array [Field 10, Field 20] : [Field; 2]
//! v2 = array_get v1, index 0 -> Field
//! ```
//!
//! In this case `v2` will be replaced with `Field 10`. A `make_array` could also be reached
//! after passing through other `array_set` with a different index, as previously shown.
//!
//! Finally, the optimization might also reach to params:
//!
//! ```text
//! b0(v1: [Field; 2]):
//!   v2 = array_set v1, index 1, value: 42
//!   v3 = array_get v2, index 0 -> Field
//! ```
//!
//! In this case `v3` will be replaced with `array_get v1, index 0`, directly getting from `v1`
//! instead of from `v2`, because `v2` is the same as `v1` except for what's in index 1, but
//! `v3` is getting from index 0.
//!
//! This module also provides a [`try_optimize_array_get_from_previous_instructions`] function
//! that is used in other SSA-related optimizations. For example, whenever an `array_get` is inserted
//! into a [`DFG`][crate::ssa::ir::dfg::DataFlowGraph]: in this case a previous `array_set` with the
//! same index as the `array_get` cannot be used because we don't know under which side effects var it
//! happens. However, `array_set` with a different known index can be skipped through to eventually
//! reach a `make_array` or param.
use std::collections::HashMap;

use acvm::{AcirField, FieldElement};

use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Replaces `array_get` instructions with known indices with known values from
    /// previous instructions. See the [`array_get`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_get_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.array_get_optimization();
        }
        self
    }
}

impl Function {
    fn array_get_optimization(&mut self) {
        // Keeps track of side effect vars associated to each `array_set` instruction.
        let mut array_set_predicates = HashMap::new();

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;

            match context.instruction() {
                Instruction::ArraySet { .. } => {
                    array_set_predicates.insert(instruction_id, context.enable_side_effects);
                }
                Instruction::ArrayGet { array, index } => {
                    let Some(index_field) = context.dfg.get_numeric_constant(*index) else {
                        return;
                    };
                    let array = *array;
                    let index = *index;
                    let side_effects = Some(&ArrayGetOptimizationSideEffects {
                        side_effects_var: context.enable_side_effects,
                        array_set_predicates: &array_set_predicates,
                    });
                    let Some(result) = try_optimize_array_get_from_previous_instructions(
                        array,
                        index_field,
                        context.dfg,
                        side_effects,
                    ) else {
                        return;
                    };

                    context.remove_current_instruction();

                    match result {
                        ArrayGetOptimizationResult::Value(new_value) => {
                            let [result] = context.dfg.instruction_result(instruction_id);
                            context.replace_value(result, new_value);
                        }
                        ArrayGetOptimizationResult::ArrayGet(array) => {
                            let array_get = Instruction::ArrayGet { array, index };
                            let [result] = context.dfg.instruction_result(instruction_id);
                            let result_typ = context.dfg.type_of_value(result);
                            let ctrl_typevars = Some(vec![result_typ]);
                            let new_result = context.insert_instruction(array_get, ctrl_typevars);
                            let new_result = new_result.first();
                            context.replace_value(result, new_result);
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

/// The result of the array_get optimization.
pub(crate) enum ArrayGetOptimizationResult {
    /// The `array_get` can be replaced with the given value.
    Value(ValueId),
    /// The `array_get` can be replaced by fetching from the given array at the same index as
    /// the `array_get`'s index.
    ArrayGet(ValueId),
}

/// Side effects information to be able to optimize `array_get` more efficiently.
pub(crate) struct ArrayGetOptimizationSideEffects<'a> {
    /// The current value of the side effects var.
    pub(crate) side_effects_var: ValueId,
    /// The side effects var applied to each known `array_set` instruction.
    pub(crate) array_set_predicates: &'a HashMap<InstructionId, ValueId>,
}

/// Tries to replace an `array_get` instructions with values from previous instructions.
/// See the [`array_get`][self] module for more information.
pub(crate) fn try_optimize_array_get_from_previous_instructions(
    mut array_id: ValueId,
    target_index: FieldElement,
    dfg: &DataFlowGraph,
    side_effects: Option<&ArrayGetOptimizationSideEffects>,
) -> Option<ArrayGetOptimizationResult> {
    let target_index_u32 = target_index.try_to_u32()?;

    // Arbitrary number of maximum tries just to prevent this optimization from taking too long.
    let max_tries = 5;
    for _ in 0..max_tries {
        if let Some((instruction, other_instruction_id)) =
            dfg.get_local_or_global_instruction_with_id(array_id)
        {
            match instruction {
                Instruction::ArraySet { array, index, value, .. } => {
                    if let Some(constant) = dfg.get_numeric_constant(*index) {
                        if constant == target_index {
                            match side_effects {
                                None => {
                                    // If it's an array_set with the same index as the array_get, we don't
                                    // use the value at that index. The reason is that the array_set might
                                    // be under a different predicate than the array_get, so the set value
                                    // might not be the correct one in the end.
                                    return None;
                                }
                                Some(ArrayGetOptimizationSideEffects {
                                    side_effects_var,
                                    array_set_predicates,
                                }) => {
                                    // If there's an array_set with the same index as the array_get, we
                                    // can only apply this optimization if they are under the same predicate.
                                    if array_set_predicates
                                        .get(&other_instruction_id)
                                        .is_none_or(|predicate| predicate != side_effects_var)
                                    {
                                        return None;
                                    }
                                }
                            }

                            return Some(ArrayGetOptimizationResult::Value(*value));
                        }

                        // If it's for a different known index, we can safely recur, because
                        // regardless of whether the array_set ends up being executed or not, it
                        // won't modify the value at the array_get index.
                        array_id = *array;
                        continue;
                    }
                }
                Instruction::MakeArray { elements: array, typ: _ } => {
                    let index = target_index_u32 as usize;
                    if index < array.len() {
                        return Some(ArrayGetOptimizationResult::Value(array[index]));
                    }
                }
                _ => (),
            }
        } else if let Value::Param { typ: Type::Array(_, length), .. } = &dfg[array_id]
            && target_index_u32 < length.0
        {
            return Some(ArrayGetOptimizationResult::ArrayGet(array_id));
        }

        break;
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

    use super::Ssa;

    #[test]
    fn optimizes_array_get_from_array_set_to_set_value_under_default_predicate() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v1 = array_set v0, index u32 0, value Field 1
            v2 = array_get v1, index u32 0 -> Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.array_get_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_set v0, index u32 0, value Field 1
            return Field 1
        }
        ");
    }

    #[test]
    fn optimizes_array_get_from_array_set_to_array_get_from_param() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v1 = array_set v0, index u32 1, value Field 1
            v2 = array_get v1, index u32 0 -> Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.array_get_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_set v0, index u32 1, value Field 1
            v5 = array_get v0, index u32 0 -> Field
            return v5
        }
        ");
    }

    #[test]
    fn optimizes_array_get_from_array_set_to_make_array_value() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 4, Field 8] : [Field; 3]
            v2 = array_get v0, index u32 1 -> Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.array_get_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 2, Field 4, Field 8] : [Field; 3]
            return Field 4
        }
        ");
    }

    #[test]
    fn does_not_optimize_array_get_from_array_set_with_different_predicate() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v10: u1, v11: u1):
            enable_side_effects v10
            v1 = array_set v0, index u32 0, value Field 1
            enable_side_effects v11
            v2 = array_get v1, index u32 0 -> Field
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::array_get_optimization);
    }

    #[test]
    fn optimizes_array_get_from_array_set_to_set_value_under_predicate() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v10: u1, v11: u1):
            enable_side_effects v10
            v1 = array_set v0, index u32 0, value Field 1
            enable_side_effects v11
            v12 = not v10
            enable_side_effects v10
            v3 = array_get v1, index u32 0 -> Field
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.array_get_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1, v2: u1):
            enable_side_effects v1
            v5 = array_set v0, index u32 0, value Field 1
            enable_side_effects v2
            v6 = not v1
            enable_side_effects v1
            return Field 1
        }
        ");
    }
}
