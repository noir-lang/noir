use acvm::acir::brillig_vm::{
    BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, Value,
};

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

/// Generates brillig bytecode which computes the a / b and returns the quotient and remainder.
/// It returns (0,0) if the predicate is null
pub(crate) fn directive_quotient(bit_size: u32) -> Vec<BrilligOpcode> {
    //  We generate the following code:
    // fn quotient(a : Int, b: Int, predicate: bool) -> (Int,Int) {
    //    if predicate != 0 {
    //      (a/b, a-a/b*b)
    //    } else {
    //      (0,0)
    //    }
    // }

    //a is (0) (i.e register index 0)
    //b is (1)
    //predicate is (2)
    vec![
        // If the predicate is zero, we jump to the exit segment
        BrilligOpcode::JumpIfNot { condition: RegisterIndex::from(2), location: 6 },
        //q = a/b is set into register (3)
        BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::UnsignedDiv,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(3),
            bit_size,
        },
        //(1)= q*b
        BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Mul,
            lhs: RegisterIndex::from(3),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(1),
            bit_size,
        },
        //(1) = a-q*b
        BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Sub,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(1),
            bit_size,
        },
        //(0) = q
        BrilligOpcode::Mov { destination: RegisterIndex::from(0), source: RegisterIndex::from(3) },
        BrilligOpcode::Stop,
        // Exit segment: we return 0,0
        BrilligOpcode::Const { destination: RegisterIndex::from(0), value: Value::from(0_usize) },
        BrilligOpcode::Const { destination: RegisterIndex::from(1), value: Value::from(0_usize) },
        BrilligOpcode::Stop,
    ]
}
