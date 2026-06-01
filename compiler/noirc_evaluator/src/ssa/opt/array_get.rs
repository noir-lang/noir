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
//! However, in this case:
//!
//! ```text
//! enable_side_effects u1 1
//! v1 = array_set v0, index 0, value: 42
//! enable_side_effects v200
//! v2 = array_get v1, index 0 -> Field
//! ```
//!
//! the optimization can be applied because the `array_set` is unconditional.
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
//! The [pass][Function::array_get_optimization] applies all of the above by scanning the function
//! and caching the known contents of the array values it writes (an [`ArrayView`]). Resolving an
//! `array_get` at a constant index is then a lookup into that view rather than a walk back over
//! previous instructions, so a long chain of `array_set`s no longer makes each read more expensive.
//! Each cached element records the side-effects predicate it was written under, and a read only uses
//! it when that predicate is unconditional or equal to the read's own predicate. Because
//! [`simple_optimization`][Function::simple_optimization] resets the predicate to `1` at the start of
//! each block, this comparison is sound whether the write and the read are in the same block or not,
//! so the cache is kept for the whole function.
//!
//! This module also provides a [`try_optimize_array_get_from_previous_instructions`] function
//! that is used in other SSA-related optimizations. For example, whenever an `array_get` is inserted
//! into a [`DFG`][crate::ssa::ir::dfg::DataFlowGraph]: in this case a previous `array_set` with the
//! same index as the `array_get` cannot be used because we don't know under which side effects var it
//! happens. However, `array_set` with a different known index can be skipped through to eventually
//! reach a `make_array` or param. That helper has no cache to consult, so it does a small bounded
//! walk instead.
use std::collections::HashMap;

use acvm::{AcirField, FieldElement};
use im::OrdMap;

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
        // Caches the known contents of each array value as the function is scanned, so resolving an
        // `array_get` at a constant index is a lookup rather than a walk back through previous
        // instructions.
        //
        // Each cached element records the side-effects predicate of the `array_set` that wrote it,
        // and `resolve` only uses it under a matching or unconditional predicate.
        // [`Function::simple_optimization`] resets the predicate to `1` at each block entry, so a
        // conditionally-written element only ever resolves a read that re-establishes the same
        // predicate value (or an unconditional read). The cache is therefore safe to keep across
        // blocks, which lets a read resolve against an `array_set` chain built in an earlier block.
        let mut views: HashMap<ValueId, ArrayView> = HashMap::new();

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;

            match context.instruction() {
                Instruction::ArraySet { array, index, value, .. } => {
                    let array = *array;
                    let value = *value;
                    let Some(index) = constant_index(context.dfg, *index) else {
                        // A dynamic index may write to any element, so nothing is known about the
                        // result. Leaving it uncached means reads of it resolve as `Unknown`.
                        return;
                    };
                    let predicate = context.enable_side_effects;
                    let [result] = context.dfg.instruction_result(instruction_id);

                    let view = array_view(&views, context.dfg, array)
                        .with_element(index, KnownElement { value, predicate });
                    views.insert(result, view);
                }
                Instruction::ArrayGet { array, index } => {
                    let array = *array;
                    let index = *index;
                    let Some(target_index) = constant_index(context.dfg, index) else {
                        return;
                    };
                    let predicate = context.enable_side_effects;

                    let view = array_view(&views, context.dfg, array);
                    let Some(resolution) =
                        view.resolve(array, target_index, predicate, context.dfg)
                    else {
                        return;
                    };

                    context.remove_current_instruction();
                    let [result] = context.dfg.instruction_result(instruction_id);
                    match resolution {
                        Resolution::Value(value) => {
                            context.replace_value(result, value);
                        }
                        Resolution::ReadFrom(source) => {
                            let array_get = Instruction::ArrayGet { array: source, index };
                            let result_typ = context.dfg.type_of_value(result).into_owned();
                            let new_result = context
                                .insert_instruction(array_get, Some(vec![result_typ]))
                                .first();
                            context.replace_value(result, new_result);
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

/// The known contents of an array value, built up incrementally by
/// [`Function::array_get_optimization`] as it scans a function.
#[derive(Clone)]
struct ArrayView {
    /// Values known to live at specific constant indices, overriding `base`.
    elements: OrdMap<u32, KnownElement>,
    /// Where an index that isn't present in `elements` gets its value from.
    base: ArrayBase,
}

#[derive(Clone, Copy)]
struct KnownElement {
    value: ValueId,
    /// The side-effects predicate under which this element was written by an `array_set`.
    predicate: ValueId,
}

#[derive(Clone)]
enum ArrayBase {
    /// Indices not in `elements` come from this `make_array`'s elements.
    MakeArray(im::Vector<ValueId>),
    /// Indices not in `elements` can be read directly from this array (a function parameter), at
    /// the same index. `length` bounds which indices that is valid for.
    ReadFrom { array: ValueId, length: u32 },
    /// Nothing is known about indices not in `elements`.
    Unknown,
}

/// How an `array_get` at a known index can be resolved against an [`ArrayView`].
enum Resolution {
    /// The `array_get` is equal to this value.
    Value(ValueId),
    /// The `array_get` can read from this array instead, at the same index.
    ReadFrom(ValueId),
}

impl ArrayView {
    fn from_make_array(elements: im::Vector<ValueId>) -> Self {
        ArrayView { elements: OrdMap::new(), base: ArrayBase::MakeArray(elements) }
    }

    fn unknown() -> Self {
        ArrayView { elements: OrdMap::new(), base: ArrayBase::Unknown }
    }

    fn with_element(mut self, index: u32, element: KnownElement) -> Self {
        self.elements.insert(index, element);
        self
    }

    fn resolve(
        &self,
        array: ValueId,
        index: u32,
        predicate: ValueId,
        dfg: &DataFlowGraph,
    ) -> Option<Resolution> {
        if let Some(element) = self.elements.get(&index) {
            // A known element can only be used if it was written unconditionally or under the same
            // predicate as the `array_get`; otherwise the write might not have happened.
            let unconditional =
                dfg.get_numeric_constant(element.predicate).is_some_and(|var| var.is_one());
            return (unconditional || element.predicate == predicate)
                .then_some(Resolution::Value(element.value));
        }

        match self.base {
            ArrayBase::MakeArray(ref elements) => {
                elements.get(index as usize).copied().map(Resolution::Value)
            }
            // Reading directly from `array` itself wouldn't be an improvement.
            ArrayBase::ReadFrom { array: source, length } => {
                (index < length && source != array).then_some(Resolution::ReadFrom(source))
            }
            ArrayBase::Unknown => None,
        }
    }
}

/// Returns the cached view of `array`, seeding one for arrays not written by an `array_set` earlier
/// in the current block: a `make_array` (local or global) exposes its elements directly, a parameter
/// can be read from at the same index, and anything else (including arrays from other blocks) is
/// opaque.
fn array_view(
    views: &HashMap<ValueId, ArrayView>,
    dfg: &DataFlowGraph,
    array: ValueId,
) -> ArrayView {
    if let Some(view) = views.get(&array) {
        return view.clone();
    }

    if let Some((Instruction::MakeArray { elements, .. }, _)) =
        dfg.get_local_or_global_instruction_with_id(array)
    {
        return ArrayView::from_make_array(elements.clone());
    }

    if let Value::Param { typ: Type::Array(_, length), .. } = &dfg[array] {
        return ArrayView {
            elements: OrdMap::new(),
            base: ArrayBase::ReadFrom { array, length: length.0 },
        };
    }

    ArrayView::unknown()
}

fn constant_index(dfg: &DataFlowGraph, index: ValueId) -> Option<u32> {
    dfg.get_numeric_constant(index)?.try_to_u32()
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
    let original_array_id = array_id;
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
                                    let array_set_predicate = array_set_predicates
                                        .get(&other_instruction_id)
                                        .expect("Expected to know the predicate of every array_set preceding this array_get");

                                    let array_set_predicate_is_one = dfg
                                        .get_numeric_constant(*array_set_predicate)
                                        .is_some_and(|var| var.is_one());
                                    let can_optimize = array_set_predicate_is_one
                                        || array_set_predicate == side_effects_var;
                                    if !can_optimize {
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
            // There's no optimization if we end up getting from the original array
            if array_id == original_array_id {
                return None;
            }

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
    fn resolves_array_get_across_blocks() {
        // `v1` is built in `b0` and read in `b1`. The `array_set` is unconditional, so the read of
        // the same index is folded to the set value even though it is in a different block: the
        // cache is kept across blocks and the predicate comparison stays sound (see the module
        // docs).
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v1 = array_set v0, index u32 0, value Field 1
            jmp b1()
          b1():
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
            jmp b1()
          b1():
            return Field 1
        }
        ");
    }

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

    #[test]
    fn optimized_array_get_from_unconditional_array_set() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v10: u1, v11: u1):
            enable_side_effects u1 1
            v1 = array_set v0, index u32 0, value Field 1
            enable_side_effects v11
            v2 = array_get v1, index u32 0 -> Field
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.array_get_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u1, v2: u1):
            enable_side_effects u1 1
            v6 = array_set v0, index u32 0, value Field 1
            enable_side_effects v2
            return Field 1
        }
        ");
    }
}
