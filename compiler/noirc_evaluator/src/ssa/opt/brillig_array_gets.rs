//! In the Brillig runtime arrays are represented as [RC, ...items],
//! Certain operations such as array gets only utilize the items pointer.
//! Without handling the items pointer offset in SSA, it is left to Brillig generation
//! to offset the array pointer.
//! Slices are represented in a similar manner, except the items pointer starts at three rather than one.
//!
//! For array operations with constant indices adding an instruction to offset the pointer
//! is unnecessary as we already know the index. This pass looks for such array operations
//! with constant indices and replaces their index with the appropriate offset.
//!
use fxhash::FxHashMap as HashMap;

use crate::{
    brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    ssa::{
        ir::{function::Function, instruction::Instruction, types::{NumericType, Type}},
        Ssa,
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
        let reachable_blocks = self.reachable_blocks();

        let mut instructions_to_update = HashMap::default();
        for block_id in reachable_blocks.into_iter() {
            for instruction_id in self.dfg[block_id].instructions().to_vec() {
                match self.dfg[instruction_id] {
                    Instruction::ArrayGet { array, index } => {
                        if self.dfg.is_constant(index) {
                            instructions_to_update.insert(
                                instruction_id,
                                (Instruction::ArrayGet { array, index }, block_id),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        for (instruction_id, _) in instructions_to_update {
            let new_instruction = match self.dfg[instruction_id] {
                Instruction::ArrayGet { array, index } => {
                    let index_constant =
                        self.dfg.get_numeric_constant(index).expect("ICE: Expected constant index");
                    let offset = if matches!(self.dfg.type_of_value(array), Type::Array(..)) {
                        1u128
                    } else {
                        3u128
                    };
                    let index = self.dfg.make_constant(
                        index_constant + offset.into(),
                        NumericType::unsigned(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                    );
                    Instruction::ArrayGet { array, index }
                }
                _ => {
                    continue;
                }
            };
            self.dfg[instruction_id] = new_instruction;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::opt::assert_normalized_ssa_equals;

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

        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 1 -> Field
            return v2
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_offset_dynamic_array_get() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32):
            v2 = array_get v0, v1 -> Field
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
}
