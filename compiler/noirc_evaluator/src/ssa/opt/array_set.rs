use acvm::AcirField;
use im::Vector;

use crate::ssa::{
    ir::{dfg::DataFlowGraph, function::Function, instruction::Instruction, value::ValueId},
    ssa_gen::Ssa,
};

impl Ssa {
    /// Replaces `array_get` instructions with known indices with known values from
    /// previous instructions. See the [`array_get`][self] module for more information.
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
/// Only applies to ACIR functions. Panics if:
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
        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;

            // We do not want to fold ArraySet into MakeArray if the side effects var is not known to be 1,
            // because that means the array_set might not actually execute and so future array_gets may read
            // from the original array instead of the updated one, which would make the optimization incorrect.
            if !context
                .dfg
                .get_numeric_constant(context.enable_side_effects)
                .is_some_and(|side_effects| side_effects.is_one())
            {
                return;
            }

            if let Instruction::ArraySet { array, index, value, mutable: false } =
                context.instruction()
            {
                let Some(index) =
                    context.dfg.get_numeric_constant(*index).and_then(|index| index.try_to_u32())
                else {
                    return;
                };

                let array = *array;
                let value = *value;

                if let Some(elements) =
                    fold_array_set_into_make_array(context.dfg, array, value, index)
                {
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
        });
    }
}

fn fold_array_set_into_make_array(
    dfg: &DataFlowGraph,
    array_id: ValueId,
    value: ValueId,
    index: u32,
) -> Option<Vector<ValueId>> {
    let index = index as usize;

    let (array, _) = dfg.get_array_constant(array_id)?;
    if index >= array.len() {
        // The array_set is for an index that is out of bounds for the make_array, so we can't fold it in.
        return None;
    }

    let elements = array.update(index, value);
    Some(elements)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::Ssa;

    #[test_case("acir")]
    #[test_case("brillig")]

    fn folds_unconditional_array_set_into_make_array(runtime: &str) {
        // For ACIR functions, ArraySet must not be folded to MakeArray because
        // ArraySet is predicate-dependent and the simplifier lacks predicate context.
        let src = format!(
            r#"
        {runtime}(inline) predicate_pure fn main f0 {{
          b0():
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            v1 = array_set v0, index u32 0, value Field 4
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }}
        "#
        );
        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.array_set_optimization();

        let expected = format!(
            "\
{runtime}(inline) predicate_pure fn main f0 {{
  b0():
    v2 = make_array [Field 2, Field 3] : [Field; 2]
    v10 = make_array [Field 4, Field 3] : [Field; 2]
    return Field 4, Field 3
}}"
        );
        assert_eq!(ssa.to_string().trim(), expected);
    }
}
