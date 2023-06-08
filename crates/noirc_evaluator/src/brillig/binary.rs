use crate::ssa_refactor::ir::{
    instruction::BinaryOp,
    types::{NumericType, Type},
};
use acvm::acir::brillig_vm::{BinaryFieldOp, BinaryIntOp};

/// Type to encapsulate the binary operation types in Brillig
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
}

impl BrilligBinaryOp {
    /// Convert an SSA binary operation into:
    /// - Brillig Binary Integer Op, if it is a integer type
    /// - Brillig Binary Field Op, if it is a field type
    pub(crate) fn convert_ssa_binary_op_to_brillig_binary_op(
        ssa_op: BinaryOp,
        typ: Type,
    ) -> BrilligBinaryOp {
        // First get the bit size and whether its a signed integer, if it is a numeric type
        // if it is not,then we return None, indicating that
        // it is a Field.
        let bit_size_signedness = match typ {
          Type::Numeric(numeric_type) => match numeric_type {
              NumericType::Signed { bit_size } => Some((bit_size, true)),
              NumericType::Unsigned { bit_size } => Some((bit_size, false)),
              NumericType::NativeField => None,
          },
          _ => unreachable!("only numeric types are allowed in binary operations. References are handled separately"),
      };

        fn binary_op_to_field_op(op: BinaryOp) -> BinaryFieldOp {
            match op {
                BinaryOp::Add => BinaryFieldOp::Add,
                BinaryOp::Sub => BinaryFieldOp::Sub,
                BinaryOp::Mul => BinaryFieldOp::Mul,
                BinaryOp::Div => BinaryFieldOp::Div,
                BinaryOp::Eq => BinaryFieldOp::Equals,
                _ => unreachable!(
              "Field type cannot be used with {op}. This should have been caught by the frontend"
          ),
            }
        }
        fn binary_op_to_int_op(op: BinaryOp, is_signed: bool) -> BinaryIntOp {
            match op {
              BinaryOp::Add => BinaryIntOp::Add,
              BinaryOp::Sub => BinaryIntOp::Sub,
              BinaryOp::Mul => BinaryIntOp::Mul,
              BinaryOp::Div => {
                  if is_signed {
                      BinaryIntOp::SignedDiv
                  } else {
                      BinaryIntOp::UnsignedDiv
                  }
              },
              BinaryOp::Mod => todo!("This is not supported by Brillig. It should either be added into Brillig or legalized by the SSA IR"),
              BinaryOp::Eq => BinaryIntOp::Equals,
              BinaryOp::Lt => BinaryIntOp::LessThan,
              BinaryOp::And => BinaryIntOp::And,
              BinaryOp::Or => BinaryIntOp::Or,
              BinaryOp::Xor => BinaryIntOp::Xor,
              BinaryOp::Shl => BinaryIntOp::Shl,
              BinaryOp::Shr => BinaryIntOp::Shr,
          }
        }
        // If bit size is available then it is a binary integer operation
        match bit_size_signedness {
            Some((bit_size, is_signed)) => {
                let binary_int_op = binary_op_to_int_op(ssa_op, is_signed);
                BrilligBinaryOp::Integer { op: binary_int_op, bit_size }
            }
            None => {
                let binary_field_op = binary_op_to_field_op(ssa_op);
                BrilligBinaryOp::Field { op: binary_field_op }
            }
        }
    }
}

/// Operands in a binary operation are checked to have the same type.
///
/// In Noir, binary operands should have the same type due to the language
/// semantics.
///
/// There are some edge cases to consider:
/// - Constants are not explicitly type casted, so we need to check for this and
/// return the type of the other operand, if we have a constant.
/// - 0 is not seen as `Field 0` but instead as `Unit 0`
/// TODO: The latter seems like a bug, if we cannot differentiate between a function returning
/// TODO nothing and a 0.
///
/// TODO: This constant coercion should ideally be done in the type checker.
pub(crate) fn type_of_binary_operation(lhs_type: Type, rhs_type: Type) -> Type {
    match (lhs_type, rhs_type) {
        // Function type should not be possible, since all functions
        // have been inlined.
        (Type::Function, _) | (_, Type::Function) => {
            unreachable!("ICE: Function type not allowed in binary operations")
        }
        (_, Type::Reference) | (Type::Reference, _) => {
            unreachable!("ICE: Reference reached a binary operation")
        }
        // Unit type currently can mean a 0 constant, so we return the
        // other type.
        (typ, Type::Unit) | (Type::Unit, typ) => typ,
        // If either side is a Field constant then, we coerce into the type
        // of the other operand
        (Type::Numeric(NumericType::NativeField), typ)
        | (typ, Type::Numeric(NumericType::NativeField)) => typ,
        // If either side is a numeric type, then we expect their types to be
        // the same.
        (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
            assert_eq!(
                lhs_type, rhs_type,
                "lhs and rhs types in a binary operation are always the same"
            );
            Type::Numeric(lhs_type)
        }
    }
}
