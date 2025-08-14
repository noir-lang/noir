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
        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let (target_func, arguments) = match &instruction {
                Instruction::Call { func, arguments } => (func, arguments),
                _ => return,
            };

            let Value::Intrinsic(Intrinsic::AsSlice) = context.dfg[*target_func] else {
                return;
            };

            let first_argument =
                arguments.first().expect("AsSlice should always have one argument");
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

    #[test]
    fn as_slice_length_multiple_different_arrays() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 5]):
            v3, v4 = call as_slice(v0) -> (u32, [Field])
            v5, v6 = call as_slice(v1) -> (u32, [Field])
            return v3, v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_slice_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 5]):
            v3, v4 = call as_slice(v0) -> (u32, [Field])
            v5, v6 = call as_slice(v1) -> (u32, [Field])
            return u32 3, u32 5
        }
        ");
    }

    /// TODO(https://github.com/noir-lang/noir/issues/9416): This test should be prevented during SSA validation
    /// Once type checking of intrinsic function calls is supported this test will have to be run against non-validated SSA.
    #[test]
    #[should_panic(expected = "AsSlice called with non-array [Field]")]
    fn as_slice_length_on_slice_type() {
        let src = "
        acir(inline) fn main f0 {
            b0():
              v3 = make_array [Field 1, Field 2, Field 3] : [Field] 
              v4 = call f1(v3) -> u32
              return v4
        }

        acir(inline) fn foo f1 {
            b0(v0: [Field]):
              v2, v3 = call as_slice(v0) -> (u32, [Field])
              return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.as_slice_optimization();
    }

    /// TODO(https://github.com/noir-lang/noir/issues/9416): This test should be prevented during SSA validation
    /// Once type checking of intrinsic function calls is supported this test will have to be run against non-validated SSA.
    #[test]
    #[should_panic(expected = "AsSlice called with non-array Field")]
    fn as_slice_length_on_numeric_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2, v3 = call as_slice(v0) -> (u32, [Field])
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.as_slice_optimization();
    }

    /// TODO(https://github.com/noir-lang/noir/issues/9416): This test should be prevented during SSA validation
    /// Once type checking of intrinsic function calls is supported this test will have to be run against non-validated SSA.
    #[test]
    #[should_panic(expected = "AsSlice should always have one argument")]
    fn as_slice_wrong_number_of_arguments() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v1, v2 = call as_slice() -> (u32, [Field])
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.as_slice_optimization();
    }
}
