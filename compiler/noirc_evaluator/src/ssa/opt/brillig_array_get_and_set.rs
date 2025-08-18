//! In the Brillig runtime arrays are represented as [RC, ...items],
//! Certain operations such as array gets only utilize the items pointer.
//! Without handling the items pointer offset in SSA, it is left to Brillig generation
//! to offset the array pointer.
//!
//! Slices are represented as Brillig vectors, where the items pointer instead starts at three rather than one.
//! A Brillig vector is represented as [RC, Size, Capacity, ...items].
//!
//! For array operations with constant indices adding an instruction to offset the pointer
//! is unnecessary as we already know the index. This pass looks for such array operations
//! with constant indices and replaces their index with the appropriate offset.

use acvm::FieldElement;

use crate::{
    brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    ssa::{
        Ssa,
        ir::{
            function::Function,
            instruction::{ArrayOffset, Instruction},
            types::{NumericType, Type},
            value::ValueId,
        },
    },
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// See [`brillig_array_get_and_set`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn brillig_array_get_and_set(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.brillig_array_get_and_set();
        }

        self
    }
}

impl Function {
    fn brillig_array_get_and_set(&mut self) {
        if !self.runtime().is_brillig() {
            return;
        }

        self.simple_optimization(|context| {
            let instruction = context.instruction();
            match instruction {
                Instruction::ArrayGet { array, index, offset } => {
                    // This pass should run at most once
                    assert!(*offset == ArrayOffset::None);

                    let array = *array;
                    let index = *index;
                    let Some(index_constant) = context.dfg.get_numeric_constant(index) else {
                        return;
                    };

                    let (index, offset) = compute_index_and_offset(context, array, index_constant);
                    let new_instruction = Instruction::ArrayGet { array, index, offset };
                    context.replace_current_instruction_with(new_instruction);
                }
                Instruction::ArraySet { array, index, value, mutable, offset } => {
                    // This pass should run at most once
                    assert!(*offset == ArrayOffset::None);

                    let array = *array;
                    let index = *index;
                    let value = *value;
                    let mutable = *mutable;
                    let Some(index_constant) = context.dfg.get_numeric_constant(index) else {
                        return;
                    };

                    let (index, offset) = compute_index_and_offset(context, array, index_constant);
                    let new_instruction =
                        Instruction::ArraySet { array, index, value, mutable, offset };
                    context.replace_current_instruction_with(new_instruction);
                }
                _ => (),
            }
        });
    }
}

/// Given an array or slice value and a constant index, returns an offseted index
/// together with which type of [`ArrayOffset`] was used to shift it.
fn compute_index_and_offset(
    context: &mut SimpleOptimizationContext,
    array_or_slice: ValueId,
    index_constant: FieldElement,
) -> (ValueId, ArrayOffset) {
    let offset = if matches!(context.dfg.type_of_value(array_or_slice), Type::Array(..)) {
        ArrayOffset::Array
    } else {
        ArrayOffset::Slice
    };
    let index = context.dfg.make_constant(
        index_constant + offset.to_u32().into(),
        NumericType::unsigned(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
    );
    (index, offset)
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_normalized_ssa_equals};

    use super::Ssa;

    #[test]
    fn offset_array_get_constant_index() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 1 minus 1 -> Field
            return v2
        }
        ");
    }

    #[test]
    fn offset_slice_array_get_constant_index() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            v2 = array_get v0, index u32 3 minus 3 -> Field
            return v2
        }
        ");
    }

    #[test]
    fn do_not_offset_dynamic_array_get() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32):
            v2 = array_get v0, index v1 -> Field
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn do_not_offset_array_get_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn offset_array_set_constant_index() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_set v0, index u32 0, value Field 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_set v0, index u32 1 minus 1, value Field 2
            return v3
        }
        ");
    }

    #[test]
    fn offset_slice_array_set_constant_index() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            v2 = array_set v0, index u32 0, value Field 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            v3 = array_set v0, index u32 3 minus 3, value Field 2
            return v3
        }
        ");
    }

    #[test]
    fn do_not_offset_dynamic_array_set() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32):
            v2 = array_set v0, index v1, value Field 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn do_not_offset_array_set_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_set v0, index u32 0, value Field 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_array_get_and_set();
        assert_normalized_ssa_equals(ssa, src);
    }
}
