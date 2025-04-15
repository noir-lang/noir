use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::{NumericType, Type},
        value::Value,
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// A simple SSA pass to find any calls to `Intrinsic::AsSlice` and replacing any references to the length of the
    /// resulting slice with the length of the array from which it was generated.
    ///
    /// This allows the length of a slice generated from an array to be used in locations where a constant value is
    /// necessary when the value of the array is unknown.
    ///
    /// Note that this pass must be placed before loop unrolling to be useful.
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
        let mut values_to_replace = HashMap::default();

        for block in self.reachable_blocks() {
            let instruction_ids = self.dfg[block].take_instructions();
            for instruction_id in &instruction_ids {
                let (target_func, first_argument) = {
                    let instruction = &mut self.dfg[*instruction_id];
                    instruction.replace_values(&values_to_replace);

                    let (target_func, arguments) = match &instruction {
                        Instruction::Call { func, arguments } => (func, arguments),
                        _ => continue,
                    };

                    (*target_func, arguments.first().copied())
                };

                match &self.dfg[target_func] {
                    Value::Intrinsic(Intrinsic::AsSlice) => {
                        let first_argument = first_argument.unwrap();
                        let array_typ = self.dfg.type_of_value(first_argument);
                        if let Type::Array(_, length) = array_typ {
                            let call_returns = self.dfg.instruction_results(*instruction_id);
                            let original_slice_length = call_returns[0];
                            let known_length =
                                self.dfg.make_constant(length.into(), NumericType::length_type());
                            values_to_replace.insert(original_slice_length, known_length);
                        } else {
                            unreachable!("AsSlice called with non-array {}", array_typ);
                        }
                    }
                    _ => continue,
                };
            }

            *self.dfg[block].instructions_mut() = instruction_ids;
            self.dfg.replace_values_in_block_terminator(block, &values_to_replace);
        }
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
