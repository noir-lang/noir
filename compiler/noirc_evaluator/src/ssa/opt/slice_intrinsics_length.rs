use acvm::{AcirField, FieldElement};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::{NumericType, Type},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA pass to find any calls to slice intrinsics  and replacing any references to the length of the
    /// resulting slice with the length of the array from which it was generated, or with a relative
    /// length based on the input length.
    ///
    /// This allows the length of a slice generated from an array to be used in locations where a constant value is
    /// necessary when the value of the array is unknown.
    ///
    /// Note that this pass must be placed before loop unrolling to be useful.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn slice_instrinsics_length_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.slice_intrinsics_length_optimization();
        }
        self
    }
}

impl Function {
    pub(crate) fn slice_intrinsics_length_optimization(&mut self) {
        let as_slice = self.dfg.get_intrinsic(Intrinsic::AsSlice).copied();
        let slice_insert = self.dfg.get_intrinsic(Intrinsic::SliceInsert).copied();
        let slice_remove = self.dfg.get_intrinsic(Intrinsic::SliceRemove).copied();
        let slice_push_back = self.dfg.get_intrinsic(Intrinsic::SlicePushBack).copied();
        let slice_push_front = self.dfg.get_intrinsic(Intrinsic::SlicePushFront).copied();
        let slice_pop_back = self.dfg.get_intrinsic(Intrinsic::SlicePopBack).copied();
        let slice_pop_front = self.dfg.get_intrinsic(Intrinsic::SlicePopFront).copied();

        let ops = [
            as_slice,
            slice_insert,
            slice_remove,
            slice_push_back,
            slice_push_front,
            slice_pop_back,
            slice_pop_front,
        ];
        if ops.iter().all(Option::is_none) {
            // No slice intrinsics used in this function
            return;
        }

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let (target_func, arguments) = match &instruction {
                Instruction::Call { func, arguments } => (func, arguments),
                _ => return,
            };

            let replacement = if as_slice.is_some_and(|op| target_func == &op) {
                // For `as_slice(array)` we can replace the resulting length with the length of the array
                let first_argument =
                    arguments.first().expect("AsSlice should always have one argument");
                let array_typ = context.dfg.type_of_value(*first_argument);
                let Type::Array(_, length) = array_typ else {
                    unreachable!("AsSlice called with non-array {}", array_typ);
                };

                let original_slice_length = context.dfg.instruction_results(instruction_id)[0];
                Some((original_slice_length, length.into()))
            } else if slice_insert.is_some_and(|op| target_func == &op)
                || slice_push_front.is_some_and(|op| target_func == &op)
            {
                if let Some(length) = context.dfg.get_numeric_constant(arguments[0]) {
                    // For `slice_insert(length, ...)` we can replace the resulting length with length + 1
                    // Same goes for `slice_push_front(length, ...)`
                    let length = length + FieldElement::one();
                    let new_slice_length = context.dfg.instruction_results(instruction_id)[0];
                    Some((new_slice_length, length))
                } else {
                    None
                }
            } else if slice_remove.is_some_and(|op| target_func == &op) {
                if let Some(length) = context.dfg.get_numeric_constant(arguments[0]) {
                    if !length.is_zero() {
                        // For `slice_remove(length, ...)` we can replace the resulting length with length - 1
                        let length = length - FieldElement::one();
                        let new_slice_length = context.dfg.instruction_results(instruction_id)[0];
                        Some((new_slice_length, length))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((value_to_replace, replacement)) = replacement {
                let known_length =
                    context.dfg.make_constant(replacement, NumericType::length_type());
                context.replace_value(value_to_replace, known_length);
            }
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

        let ssa = ssa.slice_instrinsics_length_optimization();
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
        let ssa = ssa.slice_instrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: [Field; 5]):
            v3, v4 = call as_slice(v0) -> (u32, [Field])
            v5, v6 = call as_slice(v1) -> (u32, [Field])
            return u32 3, u32 5
        }
        ");
    }

    #[test]
    fn slice_insert_optimization() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field]
            v1, v2 = call slice_insert(u32 2, v0, u32 1, Field 4) -> (u32, [Field])
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v1` was replaced with 3 because we know the new length is 2 + 1
        let ssa = ssa.slice_instrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v7, v8 = call slice_insert(u32 2, v2, u32 1, Field 4) -> (u32, [Field])
            return u32 3
        }
        ");
    }

    #[test]
    fn slice_remove_optimization() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field]
            v1, v2 = call slice_remove(u32 2, v0, u32 1) -> (u32, [Field])
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v1` was replaced with 1 because we know the new length is 2 - 1
        let ssa = ssa.slice_instrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v6, v7 = call slice_remove(u32 2, v2, u32 1) -> (u32, [Field])
            return u32 1
        }
        ");
    }

    #[test]
    fn slice_push_front_optimization() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field]
            v1, v2 = call slice_push_front(u32 2, v0, Field 4) -> (u32, [Field])
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v1` was replaced with 1 because we know the new length is 2 + 1
        let ssa = ssa.slice_instrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v6, v7 = call slice_push_front(u32 2, v2, Field 4) -> (u32, [Field])
            return u32 3
        }
        ");
    }
}
