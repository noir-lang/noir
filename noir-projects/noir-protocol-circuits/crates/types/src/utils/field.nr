pub fn field_from_bytes<N>(bytes: [u8; N], big_endian: bool) -> Field {
    assert(bytes.len() < 32, "field_from_bytes: N must be less than 32");
    let mut as_field = 0;
    let mut offset = 1;
    for i in 0..N {
        let mut index = i;
        if big_endian {
            index = N - i - 1;
        }
        as_field += (bytes[index] as Field) * offset;
        offset *= 256;
    }

    as_field
}

// Convert a 32 byte array to a field element by truncating the final byte
pub fn field_from_bytes_32_trunc(bytes32: [u8; 32]) -> Field {
    // Convert it to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..15 {
        // covers bytes 16..30 (31 is truncated and ignored)
        low = low + (bytes32[15 + 15 - i] as Field) * v;
        v = v * 256;
        // covers bytes 0..14
        high = high + (bytes32[14 - i] as Field) * v;
    }
    // covers byte 15
    low = low + (bytes32[15] as Field) * v;

    low + high * v
}

// TODO to radix returns u8, so we cannot use bigger radixes. It'd be ideal to use a radix of the maximum range-constrained integer noir supports
pub fn full_field_less_than(lhs: Field, rhs: Field) -> bool {
    lhs.lt(rhs)
}

pub fn full_field_greater_than(lhs: Field, rhs: Field) -> bool {
    rhs.lt(lhs)
}

#[test]
unconstrained fn bytes_field_test() {
    // Tests correctness of field_from_bytes_32_trunc against existing methods
    // Bytes representing 0x543e0a6642ffeb8039296861765a53407bba62bd1c97ca43374de950bbe0a7
    let inputs = [
        84, 62, 10, 102, 66, 255, 235, 128, 57, 41, 104, 97, 118, 90, 83, 64, 123, 186, 98, 189, 28, 151, 202, 67, 55, 77, 233, 80, 187, 224, 167
    ];
    let field = field_from_bytes(inputs, true);
    let return_bytes = field.to_be_bytes(31);
    for i in 0..31 {
        assert_eq(inputs[i], return_bytes[i]);
    }
    // 32 bytes - we remove the final byte, and check it matches the field
    let inputs2 = [
        84, 62, 10, 102, 66, 255, 235, 128, 57, 41, 104, 97, 118, 90, 83, 64, 123, 186, 98, 189, 28, 151, 202, 67, 55, 77, 233, 80, 187, 224, 167, 158
    ];
    let field2 = field_from_bytes_32_trunc(inputs2);
    let return_bytes2 = field.to_be_bytes(31);

    for i in 0..31 {
        assert_eq(return_bytes2[i], return_bytes[i]);
    }
    assert_eq(field2, field);
}
