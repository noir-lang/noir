use acvm::acir::brillig_vm::{BinaryFieldOp, Opcode as BrilligOpcode, RegisterIndex, Value};

/// Generates brillig bytecode which computes the inverse of its input if not null, and zero else.
pub(crate) fn directive_invert() -> Vec<BrilligOpcode> {
    //  We generate the following code:
    // fn invert(x : Field) -> Field {
    //    1/ x
    // }

    // The input argument, ie the value that will be inverted.
    let input = RegisterIndex::from(0);
    vec![
        // If the input is zero, then we jump to the stop opcode
        BrilligOpcode::JumpIfNot { condition: input, location: 3 },
        // put value one in register (1)
        BrilligOpcode::Const { destination: RegisterIndex::from(1), value: Value::from(1_usize) },
        // Divide 1 by the input, and set the result of the division into register (0)
        BrilligOpcode::BinaryFieldOp {
            op: BinaryFieldOp::Div,
            lhs: RegisterIndex::from(1),
            rhs: input,
            destination: RegisterIndex::from(0),
        },
        BrilligOpcode::Stop,
    ]
}
