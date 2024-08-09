use acvm::acir::{
    brillig::{BinaryFieldOp, BitSize, MemoryAddress, Opcode as BrilligOpcode},
    AcirField,
};

use crate::brillig::brillig_ir::artifact::GeneratedBrillig;

/// Generates brillig bytecode which computes the inverse of its input if not null, and zero else.
pub(crate) fn directive_invert<F: AcirField>() -> GeneratedBrillig<F> {
    //  We generate the following code:
    // fn invert(x : Field) -> Field {
    //    1/ x
    // }

    // The input argument, ie the value that will be inverted.
    // We store the result in this register too.
    let input = MemoryAddress::from(0);
    let one_const = MemoryAddress::from(1);
    let zero_const = MemoryAddress::from(2);
    let input_is_zero = MemoryAddress::from(3);
    // Location of the stop opcode
    let stop_location = 6;

    GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::CalldataCopy { destination_address: input, size: 1, offset: 0 },
            // Put value zero in register (2)
            BrilligOpcode::Const {
                destination: zero_const,
                value: F::from(0_usize),
                bit_size: BitSize::Field,
            },
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::Equals,
                lhs: input,
                rhs: zero_const,
                destination: input_is_zero,
            },
            // If the input is zero, then we jump to the stop opcode
            BrilligOpcode::JumpIf { condition: input_is_zero, location: stop_location },
            // Put value one in register (1)
            BrilligOpcode::Const {
                destination: one_const,
                value: F::one(),
                bit_size: BitSize::Field,
            },
            // Divide 1 by the input, and set the result of the division into register (0)
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::Div,
                lhs: one_const,
                rhs: input,
                destination: input,
            },
            BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 1 },
        ],
        assert_messages: Default::default(),
        locations: Default::default(),
    }
}

/// Generates brillig bytecode which computes `a / b` and returns the quotient and remainder.
///
/// This is equivalent to the Noir (pseudo)code
///
/// ```ignore
/// fn quotient<T>(a: T, b: T) -> (T,T) {
///    (a/b, a-a/b*b)
/// }
/// ```
pub(crate) fn directive_quotient<F: AcirField>() -> GeneratedBrillig<F> {
    // `a` is (0) (i.e register index 0)
    // `b` is (1)

    GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::from(0),
                size: 2,
                offset: 0,
            },
            // No cast, since calldata is typed as field by default
            //q = a/b is set into register (2)
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::IntegerDiv, // We want integer division, not field division!
                lhs: MemoryAddress::from(0),
                rhs: MemoryAddress::from(1),
                destination: MemoryAddress::from(2),
            },
            //(1)= q*b
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::Mul,
                lhs: MemoryAddress::from(2),
                rhs: MemoryAddress::from(1),
                destination: MemoryAddress::from(1),
            },
            //(1) = a-q*b
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::Sub,
                lhs: MemoryAddress::from(0),
                rhs: MemoryAddress::from(1),
                destination: MemoryAddress::from(1),
            },
            //(0) = q
            BrilligOpcode::Mov {
                destination: MemoryAddress::from(0),
                source: MemoryAddress::from(2),
            },
            BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 2 },
        ],
        assert_messages: Default::default(),
        locations: Default::default(),
    }
}
