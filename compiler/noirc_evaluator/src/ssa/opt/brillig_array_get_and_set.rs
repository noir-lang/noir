//! This SSA pass adjusts constant indexes of array operations inside Brillig functions
//! to avoid performing an extra binary operation.
//!
//! For example, if we have this SSA:
//!
//! ```ssa
//! b0(v0: [Field; 10]):
//!   v1 = array_get v0, index u32 3 -> Field
//! ```
//!
//! Brillig would have to fetch the element at index 3 of the array. However,
//! in the Brillig runtime arrays are represented as [RC, ...items],
//! where `RC` holds the reference count of the array. That means that the final
//! index that needs to be retrieved is 4, not 3. With the above operation
//! the final Brillig code would have to add 1 to 3 to get the desired element.
//!
//! So, this pass will transform the above SSA into this:
//!
//! ```ssa
//! b0(v0: [Field; 10]):
//!   v1 = array_get v0, index u32 4 minus 1 -> Field
//! ```
//!
//! Now the index to retrieve is 4 and there's no need to offset it in Brillig,
//! avoiding one addition.
//!
//! The "minus 1" part is just there so that readers can understand that the index
//! was offset and that the actual element index is 3. On the Brillig side,
//! array operations with constant indexes are always assumed to have already been
//! shifted.
//!
//! In the case of slices, they are represented as Brillig vectors as [RC, Size, Capacity, ...items],
//! thus the items pointer instead starts at three rather than one. So for a slice
//! this pass will transform this:
//!
//! ```ssa
//! b0(v0: [Field]):
//!   v1 = array_get v0, index u32 3 -> Field
//! ```
//!
//! to this:
//!
//! ```ssa
//! b0(v0: [Field]):
//!   v1 = array_get v0, index u32 6 minus 3 -> Field
//! ```
//!
//! This pass meant to be one of the very last, just before generating ACIR and Brillig opcodes
//! from the SSA. Some of the earlier passes would not expect the offset to be set, and may
//! not take offset indexes correctly into account.
//!
//! The main motivation behind this pass is to make it possible to reuse the same constants across
//! Brillig opcodes without having to re-allocate another register for them every time they appear.
//! For this to happen the constants need to be part of the DFG, where they can be hoisted and
//! deduplicated across instructions. Doing this during Brillig codegen would be too late, as at
//! that time we have a read-only DFG and we would be forced to generate more Brillig opcodes.
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
                    let Some(constant_index) = context.dfg.get_numeric_constant(index) else {
                        return;
                    };

                    let (index, offset) = compute_index_and_offset(context, array, constant_index);
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
                    let Some(constant_index) = context.dfg.get_numeric_constant(index) else {
                        return;
                    };

                    let (index, offset) = compute_index_and_offset(context, array, constant_index);
                    let new_instruction =
                        Instruction::ArraySet { array, index, value, mutable, offset };
                    context.replace_current_instruction_with(new_instruction);
                }
                _ => (),
            }
        });
    }
}

/// Given an array or slice value and a constant index, returns an offset (shifted) index
/// together with which type of [`ArrayOffset`] was used to shift it.
fn compute_index_and_offset(
    context: &mut SimpleOptimizationContext,
    array_or_slice: ValueId,
    constant_index: FieldElement,
) -> (ValueId, ArrayOffset) {
    let offset = if matches!(context.dfg.type_of_value(array_or_slice), Type::Array(..)) {
        ArrayOffset::Array
    } else {
        ArrayOffset::Slice
    };
    let index = context.dfg.make_constant(
        constant_index + offset.to_u32().into(),
        NumericType::unsigned(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
    );
    (index, offset)
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

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
        assert_ssa_does_not_change(src, Ssa::brillig_array_get_and_set);
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
        assert_ssa_does_not_change(src, Ssa::brillig_array_get_and_set);
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
        assert_ssa_does_not_change(src, Ssa::brillig_array_get_and_set);
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
        assert_ssa_does_not_change(src, Ssa::brillig_array_get_and_set);
    }

    #[test]
    #[should_panic(expected = "Array offsets are not supposed to be set yet")]
    fn is_safe_index_unusable_once_offset() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 1]):
            v1 = array_get v0, index u32 0 -> Field
            return v1
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let b0 = &func.dfg[func.entry_block()];
        let instruction = &func.dfg[b0.instructions()[0]];
        assert!(
            !instruction.has_side_effects(&func.dfg),
            "Indexing 1-element array with index 0 should be safe and have no side effects"
        );

        let ssa = ssa.brillig_array_get_and_set();

        let func = ssa.main();
        let b0 = &func.dfg[func.entry_block()];
        let instruction = &func.dfg[b0.instructions()[0]];
        assert!(
            !instruction.has_side_effects(&func.dfg),
            "It should have no side effects, but it panics instead as it's not expected to be used"
        );
    }
}
