use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::{NumericType, Type},
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA pass to find any calls to `Intrinsic::AsSlice` and replacing any references to the length of the
    /// resulting slice with the length of the array from which it was generated.
    ///
    /// This allows the length of a slice generated from an array to be used in locations where a constant value is
    /// necessary when the value of the array is unknown.
    ///
    /// Note that this pass must be placed before loop unrolling to be useful.
    #[expect(clippy::wrong_self_convention)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn as_slice_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.as_slice_optimization();
        }
        self
    }
}

impl Function {
    pub(crate) fn as_slice_optimization(&mut self) {
        self.simple_reachable_blocks_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let (target_func, arguments) = match &instruction {
                Instruction::Call { func, arguments } => (func, arguments),
                _ => return,
            };

            let Value::Intrinsic(Intrinsic::AsSlice) = context.dfg[*target_func] else {
                return;
            };

            let first_argument = arguments.first().unwrap();
            let array_typ = context.dfg.type_of_value(*first_argument);
            let Type::Array(_, length) = array_typ else {
                unreachable!("AsSlice called with non-array {}", array_typ);
            };

            let call_returns = context.dfg.instruction_results(instruction_id);
            let original_slice_length = call_returns[0];
            let known_length = context.dfg.make_constant(length.into(), NumericType::length_type());
            context.replace_value(original_slice_length, known_length);
        });
    }
}

#[cfg(test)]
mod test {
    use crate::assert_ssa_snapshot;

    use super::Ssa;

    #[test]
    fn as_slice_length_optimization() {
        // In this code we expect `return v2` to be replaced with `return u32 3` because
        // that's the length of the v0 array.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 3]):
                v2, v3 = call as_slice(v0) -> (u32, [Field])
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.as_slice_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_slice(v0) -> (u32, [Field])
            return u32 3
        }
        ");
    }
}
