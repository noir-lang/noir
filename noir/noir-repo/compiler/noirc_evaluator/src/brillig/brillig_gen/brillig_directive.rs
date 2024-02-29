use acvm::{
    acir::brillig::{BinaryFieldOp, BinaryIntOp, MemoryAddress, Opcode as BrilligOpcode, Value},
    FieldElement,
};

use crate::brillig::brillig_ir::artifact::GeneratedBrillig;

/// Generates brillig bytecode which computes the inverse of its input if not null, and zero else.
pub(crate) fn directive_invert() -> GeneratedBrillig {
    //  We generate the following code:
    // fn invert(x : Field) -> Field {
    //    1/ x
    // }

    // The input argument, ie the value that will be inverted.
    // We store the result in this register too.
    let input = MemoryAddress::from(0);
    let one_const = MemoryAddress::from(1);
    // Location of the stop opcode
    let stop_location = 3;

    GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::CalldataCopy { destination_address: input, size: 1, offset: 0 },
            // If the input is zero, then we jump to the stop opcode
            BrilligOpcode::JumpIfNot { condition: input, location: stop_location },
            // Put value one in register (1)
            BrilligOpcode::Const {
                destination: one_const,
                value: Value::from(1_usize),
                bit_size: FieldElement::max_num_bits(),
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
pub(crate) fn directive_quotient(bit_size: u32) -> GeneratedBrillig {
    // `a` is (0) (i.e register index 0)
    // `b` is (1)
    GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::from(0),
                size: 2,
                offset: 0,
            },
            //q = a/b is set into register (2)
            BrilligOpcode::BinaryIntOp {
                op: BinaryIntOp::UnsignedDiv,
                lhs: MemoryAddress::from(0),
                rhs: MemoryAddress::from(1),
                destination: MemoryAddress::from(2),
                bit_size,
            },
            //(1)= q*b
            BrilligOpcode::BinaryIntOp {
                op: BinaryIntOp::Mul,
                lhs: MemoryAddress::from(2),
                rhs: MemoryAddress::from(1),
                destination: MemoryAddress::from(1),
                bit_size,
            },
            //(1) = a-q*b
            BrilligOpcode::BinaryIntOp {
                op: BinaryIntOp::Sub,
                lhs: MemoryAddress::from(0),
                rhs: MemoryAddress::from(1),
                destination: MemoryAddress::from(1),
                bit_size,
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
