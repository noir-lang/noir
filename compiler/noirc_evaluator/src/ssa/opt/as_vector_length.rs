//! A simple SSA pass to find any calls to `Intrinsic::AsVector` and replacing any references to the length of the
//! resulting vector with the length of the array from which it was generated.
//!
//! This allows the length of a vector generated from an array to be used in locations where a constant value is
//! necessary when the value of the array is unknown.
//!
//! Note that this pass must be placed before loop unrolling to be useful.

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::{NumericType, Type},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Finds any calls to `Intrinsic::AsVector` and replaces any references to the length of the
    /// resulting vector with the length of the array from which it was generated.
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
    /// Finds any calls to `Intrinsic::AsVector` and replaces any references to the length of the
    /// resulting vector with the length of the array from which it was generated.
    pub(crate) fn as_vector_optimization(&mut self) {
        // If `as_vector` isn't called in this function there's nothing to do
        let Some(as_vector) = self.dfg.get_intrinsic(Intrinsic::AsVector).copied() else {
            return;
        };

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let (target_func, arguments) = match &instruction {
                Instruction::Call { func, arguments } => (func, arguments),
                _ => return,
            };

            if *target_func != as_vector {
                return;
            }

            let first_argument =
                arguments.first().expect("AsVector should always have one argument");
            let array_typ = context.dfg.type_of_value(*first_argument);
            let Type::Array(_, length) = array_typ else {
                unreachable!("AsVector called with non-array {}", array_typ);
            };

            let [original_vector_length, _] = context.dfg.instruction_result(instruction_id);
            let known_length =
                context.dfg.make_constant(length.0.into(), NumericType::length_type());
            context.replace_value(original_vector_length, known_length);
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
