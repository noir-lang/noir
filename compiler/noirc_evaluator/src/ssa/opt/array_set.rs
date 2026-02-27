//! Optimizes `array_set` instructions by turning then into `make_array` instructions when they
//! set a known index on a previous `make_array` instruction, as long as both instructions
//! are under the same side-effects predicate or the side effects var for the `array_set` is `true`.
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
//! v0 = make_array [Field 2, Field 3] : [Field; 2]
//! enable_side_effects v1
//! v1 = array_set v0, index u32 0, value Field 4
//! ```
//!
//! will remain unchanged.
use std::collections::HashMap;

use acvm::AcirField;
use im::Vector;

use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Replaces `array_get` instructions with `make_array` instructions when possible.
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

                    if let Some(elements) = fold_array_set_into_make_array(
                        context.dfg,
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
                    }
                }
                _ => {}
            }
        });
    }
}

fn fold_array_set_into_make_array(
    dfg: &DataFlowGraph,
    array_id: ValueId,
    value: ValueId,
    index: u32,
    side_effects_var: ValueId,
    make_array_predicates: &HashMap<InstructionId, ValueId>,
) -> Option<Vector<ValueId>> {
    let index = index as usize;

    let (instruction, instruction_id) = dfg.get_local_or_global_instruction_with_id(array_id)?;
    let Instruction::MakeArray { elements, typ: _ } = instruction else {
        return None;
    };

    if index >= elements.len() {
        // The array_set is for an index that is out of bounds for the make_array, so we can't fold it in.
        return None;
    }

    let can_fold = dfg.get_numeric_constant(side_effects_var).is_some_and(|var| var.is_one())
        || make_array_predicates[&instruction_id] == side_effects_var;
    if !can_fold {
        // The array_set and make_array are under different predicates, and the array_set predicate is not `true`,
        // so we can't fold them together.
        return None;
    }

    let elements = elements.update(index, value);
    Some(elements)
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

    /// ArraySet on a constant array must not be folded into MakeArray when the
    /// side-effects predicate is different for both instructions, because the array_set may
    /// not actually execute.
    #[test]
    fn does_not_fold_array_set_when_side_effects_predicate_is_unknown() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                v1 = make_array [Field 10, Field 11] : [Field; 2]
                enable_side_effects v0
                v2 = array_set v1, index u32 0, value Field 99
                enable_side_effects u1 1
                v3 = array_get v2, index u32 0 -> Field
                return v3
            }
        ";

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
          b0(v4: u1, v5: u1):
            enable_side_effects v4
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            v1 = array_set v0, index u32 0, value Field 4
            enable_side_effects v5
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.array_set_optimization();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v4 = make_array [Field 2, Field 3] : [Field; 2]
            v6 = make_array [Field 4, Field 3] : [Field; 2]
            enable_side_effects v1
            return Field 4, Field 3
        }
        ");
    }

    #[test]
    fn folds_conditional_array_set_into_make_array_when_array_set_predicate_is_true() {
        // The array_set here can be folded into the make_array because they are both under the same predicate.
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
}
