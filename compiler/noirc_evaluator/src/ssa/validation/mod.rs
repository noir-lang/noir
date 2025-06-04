//! Validator that checks whether a function is well formed.
//!
//! It validates:
//!
//! SSA form
//!
//! - That the function contains exactly one return block.
//! - That every checked signed addition or subtraction instruction is
//!   followed by a corresponding truncate instruction with the expected bit sizes.
//!
//! Type checking
//! - Check that the input values of certain instructions matches that instruction's constraint
//!   At the moment, only [Instruction::Binary], [Instruction::ArrayGet], and [Instruction::ArraySet]
//!   are type checked.
use fxhash::FxHashSet as HashSet;

use crate::ssa::ir::instruction::TerminatorInstruction;

use super::ir::{
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, Intrinsic},
    types::{NumericType, Type},
    value::{Value, ValueId},
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

    /// Validates that the instruction has the expected types associated with the values in each instruction
    fn type_check_instruction(&self, instruction: InstructionId) {
        let dfg = &self.function.dfg;
        match &dfg[instruction] {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let lhs_type = dfg.type_of_value(*lhs);
                let rhs_type = dfg.type_of_value(*rhs);
                match operator {
                    BinaryOp::Lt => {
                        if lhs_type != rhs_type {
                            panic!(
                                "Left-hand side and right-hand side of `lt` must have the same type"
                            );
                        }

                        if matches!(lhs_type, Type::Numeric(NumericType::NativeField)) {
                            panic!("Cannot use `lt` with field elements");
                        }
                    }
                    BinaryOp::Shl => {
                        if !matches!(rhs_type, Type::Numeric(NumericType::Unsigned { bit_size: 8 }))
                        {
                            panic!("Right-hand side of `shl` must be u8");
                        }
                    }
                    BinaryOp::Shr => {
                        if !matches!(rhs_type, Type::Numeric(NumericType::Unsigned { bit_size: 8 }))
                        {
                            panic!("Right-hand side of `shr` must be u8");
                        }
                    }
                    _ => {
                        if lhs_type != rhs_type {
                            panic!(
                                "Left-hand side and right-hand side of `{}` must have the same type",
                                operator
                            );
                        }
                    }
                }
            }
            Instruction::ArrayGet { index, .. } | Instruction::ArraySet { index, .. } => {
                let index_type = dfg.type_of_value(*index);
                if !matches!(index_type, Type::Numeric(NumericType::Unsigned { bit_size: 32 })) {
                    panic!("ArrayGet/ArraySet index must be u32");
                }
            }
            Instruction::Call { func, arguments } => {
                if let Value::Intrinsic(Intrinsic::ToRadix(_) | Intrinsic::ToBits(_)) = &dfg[*func]
                {
                    assert_eq!(arguments.len(), 2);
                    let value_typ = dfg.type_of_value(arguments[0]);
                    assert!(matches!(value_typ, Type::Numeric(NumericType::NativeField)));
                    let radix_typ = dfg.type_of_value(arguments[1]);
                    assert!(matches!(
                        radix_typ,
                        Type::Numeric(NumericType::Unsigned { bit_size: 32 })
                    ));
                }
            }
            _ => (),
        }
    }

    fn run(&mut self) {
        self.validate_single_return_block();

        for block in self.function.reachable_blocks() {
            for instruction in self.function.dfg[block].instructions() {
                self.validate_truncate_after_signed_sub(*instruction);
                self.type_check_instruction(*instruction);
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
    let mut validator = Validator::new(function);
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

    #[test]
    #[should_panic(expected = "Cannot use `lt` with field elements")]
    fn disallows_comparing_fields_with_lt() {
        let src = "
        acir(inline) impure fn main f0 {
          b0():
            v2 = lt Field 1, Field 2
            return
        }
        ";
        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(
        expected = "Left-hand side and right-hand side of `add` must have the same type"
    )]
    fn disallows_binary_add_with_different_types() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = add Field 1, i32 2
            return
        }
        ";
        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "Right-hand side of `shr` must be u8")]
    fn disallows_shr_with_non_u8() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = shr u32 1, u16 1
            return
        }
        ";
        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(expected = "Right-hand side of `shl` must be u8")]
    fn disallows_shl_with_non_u8() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = shl u32 1, u16 1
            return
        }
        ";
        let _ = Ssa::from_str(src);
    }

    #[test]
    #[should_panic(
        expected = "assertion failed: matches!(value_typ, Type::Numeric(NumericType::NativeField))"
    )]
    fn to_le_radix_on_non_field_value() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            call f1(u1 1, v0)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u1, v1: [u4; 2]):
            v2 = call to_le_radix(v0, u32 256) -> [u7; 1]
            return
        }
        ";
        let _ = Ssa::from_str(src);
    }
}
