//! Optimizes `array_set` instructions by turning then into `make_array` instructions when they
//! set a known index on a previous `make_array` instruction, as long as both instructions
//! are under the same side-effects predicate or the side effects var for the `array_set` is `true`.
//!
//! If the predicates are different, but the element type is numeric, it will use the `ValueMerger`
//! to merge the items.
//!
//! For example, this:
//!
//! ```text
//! v0 = make_array [Field 2, Field 3] : [Field; 2]
//! v1 = array_set v0, index u32 0, value Field 4
//! ```
//!
//! will change into this:
//!
//! ```text
//! v0 = make_array [Field 2, Field 3] : [Field; 2]
//! v1 = make_array [Field 4, Field 3] : [Field; 2]
//! ```
//!
//! but this:
//!
//! ```text
//! enable_side_effects v0
//! v1 = make_array [Field 2, Field 3] : [Field; 2]
//! enable_side_effects v2
//! v3 = array_set v0, index u32 0, value Field 4
//! ```
//!
//! will change into this:
//!
//! ```text
//! enable_side_effects v0
//! v1 = make_array [Field 2, Field 3] : [Field; 2]
//! enable_side_effects v2
//! v3 = not v2
//! v4 = cast v2 as Field
//! v5 = cast v3 as Field
//! v6 = mul v4, Field 4
//! v7 = mul v5, Field 2
//! v8 = add v6, v7
//! v9 = make_array [v8, Field 3] : [Field; 2]
//! ```
//!
//! and this:
//!
//! ```text
//! enable_side_effects v0
//! v1 = make_array [Field 1, Field 2] : [Field; 2]
//! v2 = make_array [Field 3, Field 4] : [Field; 2]
//! v3 = make_array [v1, v2] : [[Field; 2]; 2]
//! enable_side_effects v4
//! v5 = make_array [Field 5, Field 6] : [Field; 2]
//! v6 = array_set v3, index u32 0, value v5
//! ```
//!
//! will remain unchanged.
use std::collections::HashMap;

use acvm::AcirField;
use im::Vector;
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{DataFlowGraph, simplify::value_merger::ValueMerger},
        function::Function,
        instruction::{Instruction, InstructionId},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Replaces `array_set` instructions with `make_array` instructions when possible.
    /// See the [`array_set`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_set_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            #[cfg(debug_assertions)]
            array_set_optimization_pre_check(func);

            func.array_set_optimization();
        }
        self
    }
}

/// Pre-check condition for [Function::array_set_optimization].
///
/// Panics if:
///   - There already exists a mutable array set instruction.
#[cfg(debug_assertions)]
fn array_set_optimization_pre_check(func: &Function) {
    super::checks::for_each_instruction(func, |instruction, _dfg| {
        // No mutable array sets should exist yet
        super::checks::assert_not_mutable_array_set(instruction);
    });
}

impl Function {
    fn array_set_optimization(&mut self) {
        // Keeps track of side effect vars associated to each `make_array` instruction.
        let mut make_array_predicates = HashMap::new();

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;

            match context.instruction() {
                Instruction::MakeArray { .. } => {
                    make_array_predicates.insert(instruction_id, context.enable_side_effects);
                }
                Instruction::ArraySet { array, index, value, mutable: false } => {
                    let Some(index) = context
                        .dfg
                        .get_numeric_constant(*index)
                        .and_then(|index| index.try_to_u32())
                    else {
                        return;
                    };

                    let array = *array;
                    let value = *value;

                    if let Some((elements, insert_predicate)) = fold_array_set_into_make_array(
                        context.dfg,
                        context.block_id,
                        context.call_stack_id,
                        array,
                        value,
                        index,
                        context.enable_side_effects,
                        &make_array_predicates,
                    ) {
                        context.remove_current_instruction();

                        let typ = context.dfg.type_of_value(array);
                        let make_array = Instruction::MakeArray { elements, typ: typ.clone() };
                        let [result] = context.dfg.instruction_result(instruction_id);
                        let ctrl_typevars = Some(vec![typ]);
                        let new_result = context.insert_instruction(make_array, ctrl_typevars);
                        let new_result = new_result.first();
                        context.replace_value(result, new_result);

                        // Keep track of the predicate of the newly inserted make_array instruction
                        if insert_predicate {
                            let Value::Instruction { instruction: new_instruction_id, .. } = context.dfg[new_result] else {
                                unreachable!("Expected the last make_array insertion to be an instruction value");
                            };
                            make_array_predicates.insert(new_instruction_id, context.enable_side_effects);
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

/// Decide whether we can turn an `array_set` into a `make_array`, returning:
/// * `None` to keep the `array_set`
/// * `Some(elements, insert_predicate)` to replace it with a `make_array` with the updated `elements`;
///
/// If `insert_predicate` is true then we should keep tracking the side effect variable of the new `make_array`.
/// Otherwise the `elements` are a result of merging the items at the `index` under different predicates,
/// and the items in the `make_array` can be a mix of various side effects, and tracking must be stopped for it.
#[allow(clippy::too_many_arguments)]
fn fold_array_set_into_make_array(
    dfg: &mut DataFlowGraph,
    block_id: BasicBlockId,
    call_stack_id: CallStackId,
    array_id: ValueId,
    value: ValueId,
    index: u32,
    side_effects_var: ValueId,
    make_array_predicates: &HashMap<InstructionId, ValueId>,
) -> Option<(Vector<ValueId>, bool)> {
    let index = index as usize;

    let (instruction, instruction_id) = dfg.get_local_or_global_instruction_with_id(array_id)?;
    let Instruction::MakeArray { elements, typ: _ } = instruction else {
        return None;
    };

    if index >= elements.len() {
        // The array_set is for an index that is out of bounds for the make_array, so we can't fold it in.
        return None;
    }

    let side_effects_var_value = dfg.get_numeric_constant(side_effects_var);
    let always_executes = side_effects_var_value.is_some_and(|var| var.is_one());
    let never_executes = side_effects_var_value.is_some_and(|var| var.is_zero());

    // If the current side effects var is `u1 0`, the `array_set` will never execute.
    // In that case we can make its return value be the original `make_array` it's (not) modifying.
    if never_executes {
        return Some((elements.clone(), true));
    }

    // If the current side effects var is `u1 1`, then the `array_set` will always execute, and we can safely replace the element.
    // This is also true if the `make_array` was under the same predicate as the `array_set`, however we might not have this
    // information, if the `make_array` is the result of merging an earlier `make_array` with an `array_set` below;
    // in that case different items may be under different predicates, and we must keep using merge.
    let can_fold = always_executes
        || make_array_predicates.get(&instruction_id).is_some_and(|p| *p == side_effects_var);

    if can_fold {
        return Some((elements.update(index, value), true));
    }

    // If we are dealing with a simple numeric value, we can merge it, so the new value will be:
    // elements[index] = side_effects * value + (1 - side_effects) * elements[index]
    if !dfg.type_of_value(value).is_numeric() {
        return None;
    }

    // The array_set and make_array are under different predicates, and the array_set predicate is not `true`,
    // so we can't fold them together. We can either keep the array_set, or merge the items at that index.
    let mut elements = elements.clone();

    let negated_side_effects_var = dfg
        .insert_instruction_and_results(
            Instruction::Not(side_effects_var),
            block_id,
            None,
            call_stack_id,
        )
        .first();

    let merged_value = ValueMerger::merge_numeric_values(
        dfg,
        block_id,
        side_effects_var,
        negated_side_effects_var,
        value,
        elements[index],
    );

    elements[index] = merged_value;

    Some((elements, false))
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

    use super::Ssa;

    #[test]
    fn folds_unconditional_array_set_into_make_array_in_acir() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            v1 = array_set v0, index u32 0, value Field 4
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            v4 = make_array [Field 4, Field 3] : [Field; 2]
            return Field 4, Field 3
        }
        ");
    }

    #[test]
    fn folds_unconditional_array_set_into_make_array_in_brillig() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            v1 = array_set v0, index u32 0, value Field 4
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            v4 = make_array [Field 4, Field 3] : [Field; 2]
            return Field 4, Field 3
        }
        ");
    }

    /// ArraySet on a constant array must use merging with the MakeArray when the
    /// side-effects predicate is different for both instructions, because the array_set may
    /// not actually execute.
    #[test]
    fn merge_folds_array_set_chain_when_side_effects_predicate_is_unknown() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                v1 = make_array [Field 10, Field 11] : [Field; 2]
                enable_side_effects v0
                v2 = array_set v1, index u32 0, value Field 99
                v3 = array_set v2, index u32 0, value Field 100
                enable_side_effects u1 1
                v4 = array_get v3, index u32 0 -> Field
                return v4
            }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v3 = make_array [Field 10, Field 11] : [Field; 2]
            enable_side_effects v0
            v4 = not v0
            v5 = cast v0 as Field
            v6 = cast v4 as Field
            v8 = mul v5, Field 99
            v9 = mul v6, Field 10
            v10 = add v8, v9
            v11 = make_array [v10, Field 11] : [Field; 2]
            v12 = not v0
            v13 = cast v0 as Field
            v14 = cast v12 as Field
            v16 = mul v13, Field 100
            v17 = mul v14, v10
            v18 = add v16, v17
            v19 = make_array [v18, Field 11] : [Field; 2]
            enable_side_effects u1 1
            return v18
        }
        ");
    }

    #[test]
    fn does_not_fold_array_set_on_complex_array() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: u1, v4: u1):
                enable_side_effects v0
                v1 = make_array [Field 1, Field 2] : [Field; 2]
                v2 = make_array [Field 3, Field 4] : [Field; 2]
                v3 = make_array [v1, v2] : [[Field; 2]; 2]
                enable_side_effects v4
                v5 = make_array [Field 5, Field 6] : [Field; 2]
                v6 = array_set v3, index u32 0, value v5
                return v6
            }
        "#;

        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    /// ArraySet cannot fold into a param, only into a MakeArray
    #[test]
    fn does_not_fold_array_set_on_non_constant_array() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: [Field; 1], v1: Field, v2: u1):
                enable_side_effects v2
                v3 = array_set v0, index u32 0, value v1
                enable_side_effects u1 1
                v4 = array_get v3, index u32 0 -> Field
                return v4
            }
        "#;

        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    #[test]
    fn folds_conditional_array_set_into_make_array_when_predicate_matches() {
        // The array_set here can be folded into the make_array because they are both under the same predicate.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            enable_side_effects v1
            v3 = array_get v2, index u32 0 -> Field
            enable_side_effects v0
            v4 = array_set v2, index u32 0, value Field 4
            enable_side_effects v1
            v5 = array_get v4, index u32 0 -> Field
            v6 = array_get v4, index u32 1 -> Field
            return v5, v6
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v4 = make_array [Field 2, Field 3] : [Field; 2]
            enable_side_effects v1
            v6 = array_get v4, index u32 0 -> Field
            enable_side_effects v0
            v8 = make_array [Field 4, Field 3] : [Field; 2]
            enable_side_effects v1
            return Field 4, Field 3
        }
        ");
    }

    #[test]
    fn folds_conditional_array_set_into_make_array_when_array_set_predicate_is_true() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v4: u1):
            enable_side_effects v4
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            enable_side_effects u1 1
            v1 = array_set v0, index u32 0, value Field 4
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = make_array [Field 2, Field 3] : [Field; 2]
            enable_side_effects u1 1
            v6 = make_array [Field 4, Field 3] : [Field; 2]
            return Field 4, Field 3
        }
        ");
    }

    #[test]
    fn replaces_turned_off_array_set_with_the_make_array_it_is_modifying() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v4: u1):
            enable_side_effects v4
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            enable_side_effects u1 0
            v1 = array_set v0, index u32 0, value Field 4
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = make_array [Field 2, Field 3] : [Field; 2]
            enable_side_effects u1 0
            v5 = make_array [Field 2, Field 3] : [Field; 2]
            return v5
        }
        ");
    }

    #[test]
    fn optimizes_multiple_array_set_under_the_same_predicate() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v10: u1):
            enable_side_effects v10
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            v1 = array_set v0, index u32 0, value Field 4
            v2 = array_set v1, index u32 1, value Field 5
            v3 = array_get v2, index u32 0 -> Field
            v4 = array_get v2, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = make_array [Field 2, Field 3] : [Field; 2]
            v5 = make_array [Field 4, Field 3] : [Field; 2]
            v7 = make_array [Field 4, Field 5] : [Field; 2]
            return v7, Field 4
        }
        ");
    }
}
