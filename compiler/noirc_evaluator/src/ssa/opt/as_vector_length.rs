use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Hint, Instruction, Intrinsic},
        types::{NumericType, Type},
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA pass to:
    /// 1. Find any calls to `Intrinsic::AsVector` and replace references to the length of the
    ///    resulting vector with the length of the array from which it was generated.
    /// 2. For `BlackBox::Hint` calls with vector-typed results in ACIR functions, replace
    ///    the results with their corresponding inputs.
    ///
    /// This allows the length of a vector generated from an array or from `BlackBox::Hint` to be
    /// used in locations where a constant value is necessary when the value of the array is unknown.
    ///
    /// Note that this pass must be placed before loop unrolling and `remove_if_else` to be useful.
    #[expect(clippy::wrong_self_convention)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn as_vector_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.as_vector_optimization();
        }
        self
    }
}

impl Function {
    pub(crate) fn as_vector_optimization(&mut self) {
        let as_vector = self.dfg.get_intrinsic(Intrinsic::AsVector).copied();
        let is_acir = !self.runtime().is_brillig();

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let (target_func, arguments) = match &instruction {
                Instruction::Call { func, arguments } => (func, arguments),
                _ => return,
            };

            // Handle `as_vector` optimization
            if let Some(as_vector_func) = as_vector {
                if *target_func == as_vector_func {
                    let first_argument =
                        arguments.first().expect("AsVector should always have one argument");
                    let array_typ = context.dfg.type_of_value(*first_argument);
                    let Type::Array(_, length) = array_typ else {
                        unreachable!("AsVector called with non-array {}", array_typ);
                    };

                    let [original_vector_length, _] =
                        context.dfg.instruction_result(instruction_id);
                    let known_length =
                        context.dfg.make_constant(length.into(), NumericType::length_type());
                    context.replace_value(original_vector_length, known_length);
                    return;
                }
            }

            // Handle black_box hint optimization for ACIR functions
            // Since black_box is an identity function, we can replace its results with inputs
            // when there are vector-typed results, allowing vector sizes to be traced.
            if is_acir {
                if let Value::Intrinsic(Intrinsic::Hint(Hint::BlackBox)) =
                    &context.dfg[*target_func]
                {
                    let results = context.dfg.instruction_results(instruction_id).to_vec();
                    let has_vector_result = results
                        .iter()
                        .any(|r| matches!(context.dfg.type_of_value(*r), Type::Vector(_)));

                    if has_vector_result {
                        // Map each result to its corresponding argument
                        let replacements: Vec<_> = results
                            .iter()
                            .zip(arguments.iter())
                            .map(|(&result, &argument)| (result, argument))
                            .collect();

                        for (result, argument) in replacements {
                            context.replace_value(result, argument);
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use super::Ssa;

    #[test]
    fn as_vector_length_optimization() {
        // In this code we expect `return v2` to be replaced with `return u32 3` because
        // that's the length of the v0 array.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.as_vector_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            return u32 3
        }
        ");
    }

    #[test]
    fn as_vector_length_multiple_different_arrays() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 5]):
            v3, v4 = call as_vector(v0) -> (u32, [Field])
            v5, v6 = call as_vector(v1) -> (u32, [Field])
            return v3, v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_vector_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 5]):
            v3, v4 = call as_vector(v0) -> (u32, [Field])
            v5, v6 = call as_vector(v1) -> (u32, [Field])
            return u32 3, u32 5
        }
        ");
    }
}
