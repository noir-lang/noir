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

use crate::{
    brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    ssa::{
        Ssa,
        ir::{
            function::Function,
            instruction::{ArrayGetOffset, Instruction},
            types::{NumericType, Type},
        },
    },
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn brillig_array_gets(mut self) -> Ssa {
        let brillig_functions =
            self.functions.values_mut().filter(|function| function.runtime().is_brillig());
        for function in brillig_functions {
            function.brillig_array_gets();
        }

        self
    }
}

impl Function {
    pub(super) fn brillig_array_gets(&mut self) {
        self.simple_reachable_blocks_optimization(|context| {
            let instruction = context.instruction();
            let Instruction::ArrayGet { array, index, offset } = instruction else {
                return;
            };

            // This pass should run at most once
            assert!(*offset == ArrayGetOffset::None);

            let array = *array;
            let index = *index;
            if !context.dfg.is_constant(index) {
                return;
            }

            let index_constant =
                context.dfg.get_numeric_constant(index).expect("ICE: Expected constant index");
            let offset = if matches!(context.dfg.type_of_value(array), Type::Array(..)) {
                ArrayGetOffset::Array
            } else {
                ArrayGetOffset::Slice
            };
            let index = context.dfg.make_constant(
                index_constant + offset.to_u32().into(),
                NumericType::unsigned(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            );
            let new_instruction = Instruction::ArrayGet { array, index, offset };
            context.replace_current_instruction_with(new_instruction);
        });
    }
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
        let ssa = ssa.brillig_array_gets();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 1 minus 1 -> Field
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
        let ssa = ssa.brillig_array_gets();
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
        let ssa = ssa.brillig_array_gets();
        assert_normalized_ssa_equals(ssa, src);
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
        let ssa = ssa.brillig_array_gets();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            v2 = array_get v0, index u32 3 minus 3 -> Field
            return v2
        }
        ");
    }
}
