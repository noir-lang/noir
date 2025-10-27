use acvm::{AcirField as _, FieldElement};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::NumericType,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA optimization that replaces length values returned from slice intrinsics
    /// with known constants if the input length to those intrinsics are constants.
    ///
    /// For example, if we have:
    ///
    /// ```ssa
    /// v1, v2 = slice_insert(u32 10, v0, u32 5, Field 42) -> (u32, [Field])
    /// ```
    ///
    /// where `v1` is the returned length, we can replace `v1` with `u32 11` since we know
    /// the returned length will be one more than the input length `u32 10`.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn slice_intrinsics_length_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.slice_intrinsics_length_optimization();
        }
        self
    }
}

impl Function {
    pub(crate) fn slice_intrinsics_length_optimization(&mut self) {
        let slice_insert = self.dfg.get_intrinsic(Intrinsic::SliceInsert).copied();
        let slice_remove = self.dfg.get_intrinsic(Intrinsic::SliceRemove).copied();
        let slice_push_back = self.dfg.get_intrinsic(Intrinsic::SlicePushBack).copied();
        let slice_push_front = self.dfg.get_intrinsic(Intrinsic::SlicePushFront).copied();
        let slice_pop_back = self.dfg.get_intrinsic(Intrinsic::SlicePopBack).copied();
        let slice_pop_front = self.dfg.get_intrinsic(Intrinsic::SlicePopFront).copied();

        let ops = [
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

            let replacement = if slice_insert.is_some_and(|op| target_func == &op)
                || slice_push_front.is_some_and(|op| target_func == &op)
                || slice_push_back.is_some_and(|op| target_func == &op)
            {
                context.dfg.get_numeric_constant(arguments[0]).map(|length| {
                    // For `slice_insert(length, ...)` we can replace the resulting length with length + 1.
                    // Same goes for `slice_push_front` and `slice_push_back`.
                    let length = length + FieldElement::one();
                    let new_slice_length = context.dfg.instruction_results(instruction_id)[0];
                    (new_slice_length, length)
                })
            } else if slice_remove.is_some_and(|op| target_func == &op)
                || slice_pop_back.is_some_and(|op| target_func == &op)
            {
                context.dfg.get_numeric_constant(arguments[0]).and_then(|length| {
                    if !length.is_zero() {
                        // For `slice_remove(length, ...)` we can replace the resulting length with length - 1.
                        // Same goes for `slice_pop_back`.
                        let length = length - FieldElement::one();
                        let new_slice_length = context.dfg.instruction_results(instruction_id)[0];
                        Some((new_slice_length, length))
                    } else {
                        None
                    }
                })
            } else if slice_pop_front.is_some_and(|op| target_func == &op) {
                context.dfg.get_numeric_constant(arguments[0]).and_then(|length| {
                    if !length.is_zero() {
                        // For `slice_pop_front(length, ...)` we can replace the resulting length with length - 1.
                        let length = length - FieldElement::one();
                        // Note that `(popped_element, new_slice)` is returned so the new length is
                        // the before last result.
                        let results = context.dfg.instruction_results(instruction_id);
                        let new_slice_length = results[results.len() - 2];
                        Some((new_slice_length, length))
                    } else {
                        None
                    }
                })
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
        let ssa = ssa.slice_intrinsics_length_optimization();
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
            v1, v2, v3 = call slice_remove(u32 2, v0, u32 1) -> (u32, [Field], Field)
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v1` was replaced with 1 because we know the new length is 2 - 1
        let ssa = ssa.slice_intrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v6, v7, v8 = call slice_remove(u32 2, v2, u32 1) -> (u32, [Field], Field)
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
        let ssa = ssa.slice_intrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v6, v7 = call slice_push_front(u32 2, v2, Field 4) -> (u32, [Field])
            return u32 3
        }
        ");
    }

    #[test]
    fn slice_push_back_optimization() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field]
            v1, v2 = call slice_push_back(u32 2, v0, Field 4) -> (u32, [Field])
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v1` was replaced with 1 because we know the new length is 2 + 1
        let ssa = ssa.slice_intrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v6, v7 = call slice_push_back(u32 2, v2, Field 4) -> (u32, [Field])
            return u32 3
        }
        ");
    }

    #[test]
    fn slice_pop_back_optimization() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field]
            v1, v2, v3 = call slice_pop_back(u32 2, v0) -> (u32, [Field], Field)
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v1` was replaced with 1 because we know the new length is 2 - 1
        let ssa = ssa.slice_intrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v5, v6, v7 = call slice_pop_back(u32 2, v2) -> (u32, [Field], Field)
            return u32 1
        }
        ");
    }

    #[test]
    fn slice_pop_front_optimization() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field]
            v1, v2, v3 = call slice_pop_front(u32 2, v0) -> (Field, u32, [Field])
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Here `v2` was replaced with 1 because we know the new length is 2 - 1
        let ssa = ssa.slice_intrinsics_length_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field]
            v5, v6, v7 = call slice_pop_front(u32 2, v2) -> (Field, u32, [Field])
            return u32 1
        }
        ");
    }
}
