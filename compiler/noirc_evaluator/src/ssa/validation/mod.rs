//! Validator that checks whether a function is well formed.
//!
//! It validates:
//! - That the function contains exactly one return block.
/// - That every checked signed addition or subtraction instruction is
///   followed by a corresponding truncate instruction with the expected bit sizes.
use fxhash::FxHashSet as HashSet;

use crate::ssa::ir::instruction::TerminatorInstruction;

use super::ir::{
    function::Function,
    instruction::{BinaryOp, Instruction, InstructionId},
    types::{NumericType, Type},
    value::ValueId,
};

/// Aside the function being validated, the validator maintains internal state
/// during instruction visitation to track patterns that span multiple instructions.
struct Validator<'f> {
    function: &'f Function,
    // State for truncate-after-signed-sub validation
    // Stores: Option<(bit_size, result)>
    signed_binary_op: Option<(u32, ValueId)>,
}

impl<'f> Validator<'f> {
    fn new(function: &'f Function) -> Self {
        Self { function, signed_binary_op: None }
    }

    /// Validates that any checked signed add/sub is followed by the expected truncate.
    fn validate_truncate_after_signed_sub(&mut self, instruction: InstructionId) {
        let dfg = &self.function.dfg;
        match &dfg[instruction] {
            Instruction::Binary(binary) => {
                self.signed_binary_op = None;

                match binary.operator {
                    // Only validating checked addition/subtraction
                    BinaryOp::Add { unchecked: false } | BinaryOp::Sub { unchecked: false } => {}
                    // Otherwise, move onto the next instruction
                    _ => return,
                }

                // Assumes rhs_type is the same as lhs_type
                let lhs_type = dfg.type_of_value(binary.lhs);
                if let Type::Numeric(NumericType::Signed { bit_size }) = lhs_type {
                    let results = dfg.instruction_results(instruction);
                    self.signed_binary_op = Some((bit_size, results[0]));
                }
            }
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                let Some((signed_op_bit_size, signed_op_res)) = self.signed_binary_op.take() else {
                    return;
                };
                assert_eq!(
                    *bit_size, signed_op_bit_size,
                    "ICE: Correct truncate must follow the result of a checked signed add/sub"
                );
                assert_eq!(
                    *max_bit_size,
                    *bit_size + 1,
                    "ICE: Correct truncate must follow the result of a checked signed add/sub"
                );
                assert_eq!(
                    *value, signed_op_res,
                    "ICE: Correct truncate must follow the result of a checked signed add/sub"
                );
            }
            _ => {
                self.signed_binary_op = None;
            }
        }
    }

    // Validates there is exactly one return block
    fn validate_single_return_block(&self) {
        let reachable_blocks = self.function.reachable_blocks();

        let return_blocks: HashSet<_> = reachable_blocks
            .iter()
            .filter(|block| {
                let terminator = self.function.dfg[**block].terminator().unwrap_or_else(|| {
                    panic!("Function {} has no terminator in block {block}", self.function.id())
                });
                matches!(terminator, TerminatorInstruction::Return { .. })
            })
            .collect();

        if return_blocks.len() > 1 {
            panic!("Function {} has multiple return blocks {return_blocks:?}", self.function.id())
        }
    }

    fn run(&self) {
        self.validate_single_return_block();

        for block in self.function.reachable_blocks() {
            for instruction in self.function.dfg[block].instructions() {
                self.validate_truncate_after_signed_sub(*instruction);
            }
        }

        if self.signed_binary_op.is_some() {
            panic!("ICE: Truncate must follow the result of a checked signed add/sub");
        }
    }
}

/// Validates that the [Function] is well formed.
///
/// Panics on malformed functions.
pub(crate) fn validate_function(function: &Function) {
    let validator = Validator::new(function);
    validator.run();
}

#[cfg(test)]
mod tests {
    use crate::ssa::ssa_gen::Ssa;

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_sub_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_sub_brillig() {
        // This matches the test above we just want to make sure it holds in the Brillig runtime as well as ACIR
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_add_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "ICE: Truncate must follow the result of a checked signed add/sub")]
    fn lone_signed_add_brillig() {
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            return v2
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(
        expected = "ICE: Correct truncate must follow the result of a checked signed add/sub"
    )]
    fn signed_sub_bad_truncate_bit_size() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 32 bits, max_bit_size: 33
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(
        expected = "ICE: Correct truncate must follow the result of a checked signed add/sub"
    )]
    fn signed_sub_bad_truncate_max_bit_size() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 18
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_sub_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_sub_brillig() {
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = sub v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_add_acir() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }

    #[test]
    fn truncate_follows_signed_add_brillig() {
        let src = r"
        brillig(inline) pure fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = add v0, v1
            v3 = truncate v2 to 16 bits, max_bit_size: 17
            return v3
        }
        ";

        let _ = Ssa::from_str(src);
    }
}
