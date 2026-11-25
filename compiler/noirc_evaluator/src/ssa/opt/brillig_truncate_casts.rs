//! This SSA pass arranges the casts of `truncate` results in Brillig functions so that Brillig
//! code generation can lower them with the fewest cast opcodes.
//!
//! A Brillig cast truncates on its own, so going from `Field` to `u128` takes a single opcode.
//! A `truncate` however has to produce a value of its operand's type, which takes a cast down and
//! a cast back up. Brillig code generation avoids the cast back up if the `truncate` result has no
//! use other than a same-width cast immediately following it.
//!
//! This pass brings every `truncate` to that form, by rewriting the casts of its result:
//!
//! - One same-width cast is moved to right after the `truncate`.
//! - Other casts to the same type are removed, in favor of the moved one.
//! - Same-width casts to the other signedness and casts to wider integers now cast the moved one.
//! - If there are only casts to wider integers, an unsigned same-width cast is added for them.
//!
//! For example:
//!
//! ```ssa
//! b0(v0: Field):
//!   v1 = truncate v0 to 64 bits, max_bit_size: 254
//!   v2 = lt ...
//!   jmpif v2 then: b1(), else: b2()
//! b1():
//!   v3 = cast v1 as u64
//!   v4 = cast v1 as u128
//!   ...
//! b2():
//!   v5 = cast v1 as u64
//!   ...
//! ```
//!
//! becomes:
//!
//! ```ssa
//! b0(v0: Field):
//!   v1 = truncate v0 to 64 bits, max_bit_size: 254
//!   v3 = cast v1 as u64
//!   v2 = lt ...
//!   jmpif v2 then: b1(), else: b2()
//! b1():
//!   v4 = cast v3 as u128
//!   ...
//! b2():
//!   ...
//! ```
//!
//! This costs no path more opcodes than before, even though a cast can move out of a branch. The
//! `truncate` lowers to one cast instead of two, or to the same two when its result has other uses,
//! and the moved cast is part of those. Every remaining cast costs one opcode, as it did already.
//!
//! All of this preserves semantics. A cast cannot fail and only depends on its operand, so it can
//! be evaluated anywhere after the `truncate`, which dominates all the casts of its result and
//! thereby all the uses of those casts. The operands are all in range of the casts' target types.

use acvm::acir::brillig::IntegerBitSize;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, InstructionId},
        types::{NumericType, Type},
        value::{ValueId, ValueMapping},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`brillig_truncate_casts`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn brillig_truncate_casts(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            if function.runtime().is_brillig() {
                function.brillig_truncate_casts();
            }
        }
        self
    }
}

/// A `truncate` that a Brillig cast can do, and how its result is used.
struct TruncateUses {
    instruction: InstructionId,
    block: BasicBlockId,
    result: ValueId,
    bit_size: u32,
    /// The casts of the result to an integer of at least `bit_size` bits.
    casts: Vec<(InstructionId, NumericType)>,
    has_other_uses: bool,
}

impl Function {
    fn brillig_truncate_casts(&mut self) {
        let blocks: Vec<_> = self.reachable_blocks().into_iter().collect();
        let truncates = self.collect_truncate_uses(&blocks);

        let mut value_mapping = ValueMapping::default();
        let mut unlinked = HashSet::default();
        let mut cast_after_truncate = HashMap::default();

        for truncate in truncates {
            let Some(chosen_cast) = self.choose_cast(&truncate) else {
                continue;
            };
            let chosen_result = self.dfg.instruction_results(chosen_cast)[0];
            let Instruction::Cast(_, chosen_type) = self.dfg[chosen_cast] else {
                unreachable!("ICE: expected a cast");
            };

            unlinked.insert(chosen_cast);
            cast_after_truncate.insert(truncate.instruction, chosen_cast);

            for (cast, typ) in truncate.casts {
                if cast == chosen_cast {
                    continue;
                }
                if typ == chosen_type {
                    value_mapping.insert(self.dfg.instruction_results(cast)[0], chosen_result);
                    unlinked.insert(cast);
                } else {
                    self.dfg[cast] = Instruction::Cast(chosen_result, typ);
                }
            }
        }

        for block in blocks {
            let instructions = self.dfg[block].take_instructions();
            let target = self.dfg[block].instructions_mut();
            for instruction in instructions {
                if !unlinked.contains(&instruction) {
                    target.push(instruction);
                }
                if let Some(cast) = cast_after_truncate.get(&instruction) {
                    target.push(*cast);
                }
            }
            self.dfg.replace_values_in_block(block, &value_mapping);
        }
        self.dfg.data_bus.replace_values(&value_mapping);
    }

    /// Finds the `truncate` instructions to native integer widths and the uses of their results.
    fn collect_truncate_uses(&self, blocks: &[BasicBlockId]) -> Vec<TruncateUses> {
        let mut truncates = Vec::new();
        let mut truncate_indices = HashMap::default();
        for block in blocks {
            for instruction in self.dfg[*block].instructions() {
                let Instruction::Truncate { value, bit_size, .. } = self.dfg[*instruction] else {
                    continue;
                };
                let source_bit_size = self.dfg.type_of_value(value).bit_size();
                if IntegerBitSize::try_from(bit_size).is_err() || bit_size > source_bit_size {
                    continue;
                }
                let [result] = self.dfg.instruction_result(*instruction);
                truncate_indices.insert(result, truncates.len());
                truncates.push(TruncateUses {
                    instruction: *instruction,
                    block: *block,
                    result,
                    bit_size,
                    casts: Vec::new(),
                    has_other_uses: false,
                });
            }
        }

        for block in blocks {
            for instruction in self.dfg[*block].instructions() {
                if let Instruction::Cast(value, typ) = self.dfg[*instruction]
                    && let Some(index) = truncate_indices.get(&value)
                    && typ != NumericType::NativeField
                    && typ.bit_size::<acvm::FieldElement>() >= truncates[*index].bit_size
                {
                    truncates[*index].casts.push((*instruction, typ));
                    continue;
                }
                self.dfg[*instruction].for_each_value(|value| {
                    if let Some(index) = truncate_indices.get(&value) {
                        truncates[*index].has_other_uses = true;
                    }
                });
            }
            self.dfg[*block].unwrap_terminator().for_each_value(|value| {
                if let Some(index) = truncate_indices.get(&value) {
                    truncates[*index].has_other_uses = true;
                }
            });
        }
        truncates
    }

    /// Picks the same-width cast to go right after the `truncate`, adding one if that saves an
    /// opcode. Returns `None` if the casts of this `truncate` are best left alone.
    fn choose_cast(&mut self, truncate: &TruncateUses) -> Option<InstructionId> {
        let bit_size = truncate.bit_size;
        let unsigned = NumericType::Unsigned { bit_size };
        let same_width = |typ: NumericType| typ.bit_size::<acvm::FieldElement>() == bit_size;

        if let Some((cast, _)) = truncate.casts.iter().find(|(_, typ)| *typ == unsigned) {
            return Some(*cast);
        }

        let signed = truncate.casts.iter().find(|(_, typ)| same_width(*typ));
        let has_wider = truncate.casts.iter().any(|(_, typ)| !same_width(*typ));
        match signed {
            Some((cast, _)) if !has_wider => return Some(*cast),
            // A wider cast from a signed value would have to extend the sign, so it needs an
            // unsigned operand.
            Some(_) => (),
            // With other uses the `truncate` takes two casts anyway, and the wider casts one
            // each, whether or not they go through a same-width one.
            None if !has_wider || truncate.has_other_uses => return None,
            None => (),
        }

        let call_stack = self.dfg.get_instruction_call_stack_id(truncate.instruction);
        let cast = Instruction::Cast(truncate.result, unsigned);
        let _ = self.dfg.insert_instruction_and_results_without_simplification(
            cast,
            truncate.block,
            Some(vec![Type::Numeric(unsigned)]),
            call_stack,
        );
        // The caller moves the new cast from the end of the block to its place.
        self.dfg[truncate.block].instructions().last().copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

    use super::Ssa;

    fn run_pass(src: &str) -> Ssa {
        Ssa::from_str(src).unwrap().brillig_truncate_casts()
    }

    #[test]
    fn moves_cast_next_to_truncate_in_same_block() {
        let ssa = run_pass(
            "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 128 bits, max_bit_size: 254
            v2 = add v0, Field 1
            v3 = cast v1 as u128
            return v2, v3
        }
        ",
        );
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 128 bits, max_bit_size: 254
            v2 = cast v1 as u128
            v4 = add v0, Field 1
            return v4, v2
        }
        ");
    }

    #[test]
    fn moves_cast_from_other_block_and_merges_casts_of_sibling_branches() {
        let ssa = run_pass(
            "
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: u1):
            v2 = truncate v0 to 64 bits, max_bit_size: 254
            jmpif v1 then: b1(), else: b2()
          b1():
            v3 = cast v2 as u64
            jmp b3(v3)
          b2():
            v4 = cast v2 as u64
            v5 = unchecked_add v4, u64 1
            jmp b3(v5)
          b3(v6: u64):
            return v6
        }
        ",
        );
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: u1):
            v3 = truncate v0 to 64 bits, max_bit_size: 254
            v4 = cast v3 as u64
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b3(v4)
          b2():
            v6 = unchecked_add v4, u64 1
            jmp b3(v6)
          b3(v2: u64):
            return v2
        }
        ");
    }

    #[test]
    fn signed_and_wider_casts_go_through_the_unsigned_cast() {
        let ssa = run_pass(
            "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as i32
            v3 = cast v1 as u32
            v4 = cast v1 as u64
            return v2, v3, v4
        }
        ",
        );
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as u32
            v3 = cast v2 as i32
            v4 = cast v2 as u64
            return v3, v2, v4
        }
        ");
    }

    #[test]
    fn signed_cast_alone_is_moved() {
        let ssa = run_pass(
            "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = add v0, Field 1
            v3 = cast v1 as i32
            return v2, v3
        }
        ",
        );
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as i32
            v4 = add v0, Field 1
            return v4, v2
        }
        ");
    }

    #[test]
    fn adds_same_width_cast_for_casts_to_wider_integers() {
        let ssa = run_pass(
            "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as u64
            return v2
        }
        ",
        );
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as u32
            v3 = cast v2 as u64
            return v3
        }
        ");
    }

    #[test]
    fn leaves_wider_cast_alone_if_truncate_result_has_other_uses() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as u64
            return v1, v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::brillig_truncate_casts);
    }

    #[test]
    fn leaves_truncate_without_casts_alone() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = truncate v0 to 8 bits, max_bit_size: 32
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::brillig_truncate_casts);
    }

    #[test]
    fn leaves_truncate_to_non_native_width_alone() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = truncate v0 to 7 bits, max_bit_size: 32
            v2 = cast v1 as u8
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::brillig_truncate_casts);
    }

    #[test]
    fn leaves_acir_functions_alone() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = add v0, Field 1
            v3 = cast v1 as u32
            return v2, v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::brillig_truncate_casts);
    }
}
