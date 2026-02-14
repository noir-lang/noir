use std::collections::HashMap;

use acvm::{AcirField, FieldElement};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    opt::simple_optimization::SimpleOptimizationContext,
    ssa_gen::Ssa,
};

impl Ssa {
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
                    let Some(result) = try_optimize_array_get_from_previous_instructions(
                        array,
                        index_field,
                        context,
                        &array_set_predicates,
                    ) else {
                        return;
                    };

                    context.remove_current_instruction();

                    match result {
                        OptimizationResult::Value(new_value) => {
                            let [result] = context.dfg.instruction_result(instruction_id);
                            context.replace_value(result, new_value);
                        }
                        OptimizationResult::ArrayGet(array) => {
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

enum OptimizationResult {
    Value(ValueId),
    ArrayGet(ValueId),
}

fn try_optimize_array_get_from_previous_instructions(
    mut array_id: ValueId,
    target_index: FieldElement,
    context: &SimpleOptimizationContext,
    array_set_predicates: &HashMap<InstructionId, ValueId>,
) -> Option<OptimizationResult> {
    let target_index_u32 = target_index.try_to_u32()?;

    // Arbitrary number of maximum tries just to prevent this optimization from taking too long.
    let max_tries = 5;
    for _ in 0..max_tries {
        if let Some((instruction, other_instruction_id)) =
            context.dfg.get_local_or_global_instruction_with_id(array_id)
        {
            match instruction {
                Instruction::ArraySet { array, index, value, .. } => {
                    if let Some(constant) = context.dfg.get_numeric_constant(*index) {
                        if constant == target_index {
                            // If there's an array_set with the same index as the array_get, we
                            // can only apply this optimization if they are under the same predicate.
                            if array_set_predicates
                                .get(&other_instruction_id)
                                .is_none_or(|predicate| predicate != &context.enable_side_effects)
                            {
                                return None;
                            }

                            return Some(OptimizationResult::Value(*value));
                        }

                        // If it's for a different known index, we can safely recur, because
                        // regardless of whether the array_set ends up being executed or not, it
                        // won't modify the value at the array_get index.
                        array_id = *array; // recur
                        continue;
                    }
                }
                Instruction::MakeArray { elements: array, typ: _ } => {
                    let index = target_index_u32 as usize;
                    if index < array.len() {
                        return Some(OptimizationResult::Value(array[index]));
                    }
                }
                _ => (),
            }
        } else if let Value::Param { typ: Type::Array(_, length), .. } = &context.dfg[array_id]
            && target_index_u32 < length.0
        {
            return Some(OptimizationResult::ArrayGet(array_id));
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
            v1 = array_set v0, index u32 1, value Field 1
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
