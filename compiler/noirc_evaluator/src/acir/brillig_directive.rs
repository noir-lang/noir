use acvm::acir::{
    brillig::{
        BinaryFieldOp, BinaryIntOp, BitSize, HeapVector, IntegerBitSize, MemoryAddress,
        Opcode as BrilligOpcode,
    },
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
    let input = MemoryAddress::direct(0);
    let one_const = MemoryAddress::direct(1);
    let zero_const = MemoryAddress::direct(2);
    let input_is_zero = MemoryAddress::direct(3);
    let zero_usize = MemoryAddress::direct(20);
    let one_usize = MemoryAddress::direct(21);
    // Location of the stop opcode
    let stop_location = 8;

    GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::Const {
                destination: one_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: F::from(1_usize),
            },
            BrilligOpcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: F::from(0_usize),
            },
            BrilligOpcode::CalldataCopy {
                destination_address: input,
                size_address: one_usize,
                offset_address: zero_usize,
            },
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
            BrilligOpcode::Stop {
                return_data: HeapVector { pointer: zero_usize, size: one_usize },
            },
        ],
        name: "directive_invert".to_string(),
        ..Default::default()
    }
}

/// Generates brillig bytecode which computes `a / b` and returns the quotient and remainder.
///
/// This is equivalent to the Noir (pseudo)code
///
/// ```text
/// fn quotient<T>(a: T, b: T) -> (T,T) {
///    (a/b, a-a/b*b)
/// }
/// ```
pub(crate) fn directive_quotient<F: AcirField>() -> GeneratedBrillig<F> {
    // `a` is (0) (i.e register index 0)
    // `b` is (1)

    GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(10),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: F::from(2_usize),
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(11),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: F::from(0_usize),
            },
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(10),
                offset_address: MemoryAddress::direct(11),
            },
            // No cast, since calldata is typed as field by default
            //q = a/b is set into register (2)
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::IntegerDiv, // We want integer division, not field division!
                lhs: MemoryAddress::direct(0),
                rhs: MemoryAddress::direct(1),
                destination: MemoryAddress::direct(2),
            },
            //(1)= q*b
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::Mul,
                lhs: MemoryAddress::direct(2),
                rhs: MemoryAddress::direct(1),
                destination: MemoryAddress::direct(1),
            },
            //(1) = a-q*b
            BrilligOpcode::BinaryFieldOp {
                op: BinaryFieldOp::Sub,
                lhs: MemoryAddress::direct(0),
                rhs: MemoryAddress::direct(1),
                destination: MemoryAddress::direct(1),
            },
            //(0) = q
            BrilligOpcode::Mov {
                destination: MemoryAddress::direct(0),
                source: MemoryAddress::direct(2),
            },
            BrilligOpcode::Stop {
                return_data: HeapVector {
                    pointer: MemoryAddress::direct(11),
                    size: MemoryAddress::direct(10),
                },
            },
        ],
        name: "directive_integer_quotient".to_string(),
        ..Default::default()
    }
}

/// Generates brillig bytecode which performs a radix-base decomposition of `a`
/// The brillig inputs are 'a', the numbers of limbs and the radix
pub(crate) fn directive_to_radix<F: AcirField>() -> GeneratedBrillig<F> {
    let memory_adr_int_size = IntegerBitSize::U32;
    let memory_adr_size = BitSize::Integer(memory_adr_int_size);

    // (0) is the input field `a` to decompose
    // (1) contains the number of limbs (second input)
    let limbs_nb = MemoryAddress::direct(1);
    // (2) contains the radix (third input)
    let radix = MemoryAddress::direct(2);
    // (3) and (4) are intermediate registers
    // (5,6,7) are constants: 0,1,3
    let zero = MemoryAddress::direct(5);
    let one = MemoryAddress::direct(6);
    let three = MemoryAddress::direct(7);
    // (7) is the iteration bound, it is the same register as three because the latter is only used at the start
    let bound = MemoryAddress::direct(7);
    // (8) is the register for storing the loop condition
    let cond = MemoryAddress::direct(8);
    // (9) is the pointer to the result array
    let result_pointer = MemoryAddress::direct(9);
    // address of the result array
    let result_base_adr = 10_usize;

    let result_vector = HeapVector { pointer: result_pointer, size: limbs_nb };

    let byte_code = vec![
        // Initialize registers
        // Constants
        // Zero
        BrilligOpcode::Const { destination: zero, bit_size: memory_adr_size, value: F::zero() },
        // One
        BrilligOpcode::Const {
            destination: one,
            bit_size: memory_adr_size,
            value: F::from(1_usize),
        },
        // Three
        BrilligOpcode::Const {
            destination: three,
            bit_size: memory_adr_size,
            value: F::from(3_usize),
        },
        // Brillig Inputs
        BrilligOpcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: three,
            offset_address: zero,
        },
        // The number of limbs needs to be an integer
        BrilligOpcode::Cast { destination: limbs_nb, source: limbs_nb, bit_size: memory_adr_size },
        // Result_pointer starts at the base address
        BrilligOpcode::Const {
            destination: result_pointer,
            bit_size: memory_adr_size,
            value: F::from(result_base_adr),
        },
        // Loop bound
        BrilligOpcode::BinaryIntOp {
            destination: bound,
            op: BinaryIntOp::Add,
            bit_size: memory_adr_int_size,
            lhs: result_pointer,
            rhs: limbs_nb,
        },
        // loop label: (3) = a / radix (integer division)
        BrilligOpcode::BinaryFieldOp {
            op: BinaryFieldOp::IntegerDiv,
            lhs: MemoryAddress::direct(0),
            rhs: radix,
            destination: MemoryAddress::direct(3),
        },
        //(4) = (3)*256
        BrilligOpcode::BinaryFieldOp {
            op: BinaryFieldOp::Mul,
            lhs: MemoryAddress::direct(3),
            rhs: radix,
            destination: MemoryAddress::direct(4),
        },
        //(4) = a-(3)*256 (remainder)
        BrilligOpcode::BinaryFieldOp {
            op: BinaryFieldOp::Sub,
            lhs: MemoryAddress::direct(0),
            rhs: MemoryAddress::direct(4),
            destination: MemoryAddress::direct(4),
        },
        // Store the remainder in the result array
        BrilligOpcode::Store {
            destination_pointer: result_pointer,
            source: MemoryAddress::direct(4),
        },
        // Increment the result pointer
        BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Add,
            lhs: result_pointer,
            rhs: one,
            destination: result_pointer,
            bit_size: memory_adr_int_size,
        },
        //a := quotient
        BrilligOpcode::Mov {
            destination: MemoryAddress::direct(0),
            source: MemoryAddress::direct(3),
        },
        // loop condition
        BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::LessThan,
            lhs: result_pointer,
            rhs: bound,
            destination: cond,
            bit_size: memory_adr_int_size,
        },
        // loop back
        BrilligOpcode::JumpIf { condition: cond, location: 7 },
        // reset result pointer to the start of the array
        BrilligOpcode::Const {
            destination: result_pointer,
            bit_size: memory_adr_size,
            value: F::from(result_base_adr),
        },
        BrilligOpcode::Stop { return_data: result_vector },
    ];

    GeneratedBrillig { byte_code, name: "directive_to_radix".to_string(), ..Default::default() }
}
