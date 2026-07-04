//! This SSA pass removes `truncate` instructions that happen on values that
//! have a `range_check` on them, where the checked range is less or equal than
//! the bits to truncate (the truncate isn't needed then as it won't change the
//! underlying value).
//!
//! The same bound is also learned from a bitwise `and` with a constant mask:
//! every bit set in the result must also be set in the mask, so the result fits
//! in the mask's bit length (`x & 123` fits in 7 bits, since `123 < 2^7`).

use acvm::AcirField;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        cfg::ControlFlowGraph,
        dom::DominatorTree,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        post_order::PostOrder,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Removes `truncate` instructions that happen on values that
    /// have a `range_check` on them (or are the result of a bitwise `and` with
    /// a constant mask), where the known bound is less or equal than
    /// the bits to truncate.
    pub(crate) fn remove_truncate_after_range_check(mut self) -> Self {
        for function in self.functions.values_mut() {
            function.remove_truncate_after_range_check();
        }
        self
    }
}

impl Function {
    fn remove_truncate_after_range_check(&mut self) {
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        // Keeps the minimum bit size a value was range-checked against
        let mut range_checks: HashMap<ValueId, u32> = HashMap::default();
        let mut previous_block = None;
        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();
            if previous_block != Some(context.block_id) {
                // Clear the range checks when the new block is not dominated by
                // the previous one
                if let Some(prev) = previous_block
                    && !dom_tree.dominates(prev, context.block_id)
                {
                    range_checks.clear();
                }
                previous_block = Some(context.block_id);
            }
            match instruction {
                // If this is a range_check instruction, associate the max bit size with the value
                Instruction::RangeCheck { value, max_bit_size, .. } => {
                    range_checks
                        .entry(*value)
                        .and_modify(|current_max| {
                            if *max_bit_size < *current_max {
                                *current_max = *max_bit_size;
                            }
                        })
                        .or_insert(*max_bit_size);
                }
                // A bitwise `and` with a constant mask bounds the result to the mask's
                // bit length: every bit set in the result must also be set in the mask,
                // so `result < 2^bits(mask)`. This holds for non-contiguous masks too
                // (`x & 0b101` is bounded by 3 bits, the mask's highest set bit).
                // Masks of the form `2^n - 1` are usually canonicalized to `truncate`
                // by `simplify_binary` before this pass runs, but other constant masks
                // (e.g. `x & 123`) reach here as `and` instructions (see issue #8628).
                Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::And }) => {
                    let lhs_constant = context.dfg.get_numeric_constant(*lhs);
                    let rhs_constant = context.dfg.get_numeric_constant(*rhs);
                    // A fully-constant `and` is folded elsewhere; a non-constant mask
                    // gives no bound.
                    if let (Some(mask), None) | (None, Some(mask)) = (lhs_constant, rhs_constant) {
                        let mask_bit_size = mask.num_bits();
                        let [result] = context.dfg.instruction_result(instruction_id);
                        range_checks
                            .entry(result)
                            .and_modify(|current_max| {
                                if mask_bit_size < *current_max {
                                    *current_max = mask_bit_size;
                                }
                            })
                            .or_insert(mask_bit_size);
                    }
                }
                // If this is a truncate instruction, check if there's a range check for that same value
                Instruction::Truncate { value, bit_size, .. } => {
                    if let Some(range_check_bit_size) = range_checks.get(value)
                        && range_check_bit_size <= bit_size
                    {
                        // We need to replace the truncated value with the original one. That is, in:
                        //
                        // range_check v0 to 32 bits
                        // v1 = truncate v0 to 32 bits, max_bit_size: 254
                        //
                        // we need to remove the `truncate` and all references to `v1` should now be `v0`.
                        let [result] = context.dfg.instruction_result(instruction_id);
                        context.replace_value(result, *value);
                        context.remove_current_instruction();
                    }
                }
                _ => (),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn removes_truncate_after_range_check_with_same_bit_size() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 64 bits // This is to make sure we keep the smallest one
            range_check v0 to 32 bits
            jmp b1() // Make sure the optimization is applied across blocks
          b1():
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            return v1
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_truncate_after_range_check();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 64 bits
            range_check v0 to 32 bits
            jmp b1()
          b1():
            return v0
        }
        ");
    }

    #[test]
    fn removes_truncate_after_range_check_with_smaller_bit_size() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 16 bits
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            return v1
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_truncate_after_range_check();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 16 bits
            return v0
        }
        ");
    }

    #[test]
    fn does_not_remove_truncate_after_range_check_with_larger_bit_size() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            range_check v0 to 64 bits
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
    }

    #[test]
    fn removes_truncate_after_and_with_constant_mask() {
        // 123 = 0b1111011 fits in 7 bits, so truncating the masked value
        // to 8 bits cannot change it.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 123
            jmp b1() // Make sure the optimization is applied across dominated blocks
          b1():
            v3 = truncate v2 to 8 bits, max_bit_size: 64
            v4 = cast v3 as u8
            return v4
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_truncate_after_range_check();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 123
            jmp b1()
          b1():
            v3 = cast v2 as u8
            return v3
        }
        ");
    }

    #[test]
    fn removes_truncate_after_and_with_non_contiguous_mask_up_to_highest_set_bit() {
        // 5 = 0b101: the bound is the mask's bit *length* (3), not its
        // number of set bits, so truncating to 3 bits is removable.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 5
            v3 = truncate v2 to 3 bits, max_bit_size: 64
            return v3
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_truncate_after_range_check();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 5
            return v2
        }
        ");
    }

    #[test]
    fn does_not_remove_truncate_below_the_masks_highest_set_bit() {
        // 5 = 0b101 has only two bits set but is *not* bounded by 2 bits
        // (x & 5 can be 4 or 5), so truncating to 2 bits must be kept.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 5
            v3 = truncate v2 to 2 bits, max_bit_size: 64
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
    }

    #[test]
    fn does_not_remove_truncate_after_and_with_non_constant_mask() {
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64, v1: u64):
            v2 = and v0, v1
            v3 = truncate v2 to 8 bits, max_bit_size: 64
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
    }

    #[test]
    fn does_not_remove_truncate_after_and_with_mask_wider_than_truncation() {
        // 0x1fb needs 9 bits, so an 8-bit truncation of the masked value
        // can still change it.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 507
            v3 = truncate v2 to 8 bits, max_bit_size: 64
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
    }

    #[test]
    fn does_not_remove_truncate_after_and_with_mask_spanning_the_type_width() {
        // 0x8000000000000075 needs 64 bits (and is not of the form 2^n - 1,
        // so it is not canonicalized to a truncation), giving no useful bound
        // for a 32-bit truncation.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64):
            v2 = and v0, u64 9223372036854775925
            v3 = truncate v2 to 32 bits, max_bit_size: 64
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
    }

    #[test]
    fn does_not_remove_truncate_in_block_not_dominated_by_the_previous_block() {
        // The pass's known-bounds map is cleared when the traversal moves to a
        // block that is not dominated by the previously visited one (b1 does
        // not dominate b2 here), so the bound learned for v3 in b0 is
        // conservatively dropped and the truncate in b2 is kept, mirroring the
        // pass's existing dominance discipline for range checks.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u64, v1: u1):
            v3 = and v0, u64 123
            jmpif v1 then: b1(), else: b2()
          b1():
            v5 = and v0, u64 42
            jmp b2()
          b2():
            v6 = truncate v3 to 8 bits, max_bit_size: 64
            return v6
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
    }
}
