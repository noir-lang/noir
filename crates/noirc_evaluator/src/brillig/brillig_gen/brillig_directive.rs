use acvm::acir::brillig_vm::{BinaryFieldOp, Opcode as BrilligOpcode, RegisterIndex, Value};

/// Generates brillig bytecode which computes the inverse of its input if not null, and zero else.
pub(crate) fn directive_invert() -> Vec<BrilligOpcode> {
    vec![
        BrilligOpcode::JumpIfNot { condition: RegisterIndex::from(0), location: 3 },
        BrilligOpcode::Const { destination: RegisterIndex::from(1), value: Value::from(1_usize) },
        BrilligOpcode::BinaryFieldOp {
            op: BinaryFieldOp::Div,
            lhs: RegisterIndex::from(1),
            rhs: RegisterIndex::from(0),
            destination: RegisterIndex::from(0),
        },
        BrilligOpcode::Stop,
    ]
}
