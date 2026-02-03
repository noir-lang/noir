//! This SSA pass removes `truncate` instructions that happen on values that
//! have a `range_check` on them, where the checked range is less or equal than
//! the bits to truncate (the truncate isn't needed then as it won't change the
//! underlying value).

use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{function::Function, instruction::Instruction, value::ValueId},
    ssa_gen::Ssa,
};

impl Ssa {
    /// Removes `truncate` instructions that happen on values that
    /// have a `range_check` on them, where the checked range is less or equal than
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
        // Keeps the minimum bit size a value was range-checked against
        let mut range_checks: HashMap<ValueId, u32> = HashMap::default();

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

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
                // If this is a truncate instruction, check if there's a range check for that same value
                Instruction::Truncate { value, bit_size, .. } => {
                    if let Some(range_check_bit_size) = range_checks.get(value) {
                        if range_check_bit_size <= bit_size {
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
}
