use acvm::acir::brillig::{BinaryFieldOp, Opcode as BrilligOpcode, RegisterIndex, Value};

/// Generates brillig bytecode which computes the inverse of its input if not null, and zero else.
pub(crate) fn directive_invert() -> Vec<BrilligOpcode> {
    //  We generate the following code:
    // fn invert(x : Field) -> Field {
    //    1/ x
    // }

    // The input argument, ie the value that will be inverted.
    // We store the result in this register too.
    let input = RegisterIndex::from(0);
    let one_const = RegisterIndex::from(1);
    // Location of the stop opcode
    let stop_location = 3;

    vec![
        // If the input is zero, then we jump to the stop opcode
        BrilligOpcode::JumpIfNot { condition: input, location: stop_location },
        // Put value one in register (1)
        BrilligOpcode::Const { destination: one_const, value: Value::from(1_usize) },
        // Divide 1 by the input, and set the result of the division into register (0)
        BrilligOpcode::BinaryFieldOp {
            op: BinaryFieldOp::Div,
            lhs: one_const,
            rhs: input,
            destination: input,
        },
        BrilligOpcode::Stop,
    ]
}
