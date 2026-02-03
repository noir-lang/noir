fn main(x: Field, a: Field) {
    let y: Field = 2040124;
    let be_byte_array: [u8; 31] = y.to_be_bytes();
    let le_byte_array: [u8; 31] = x.to_le_bytes();

    assert(le_byte_array[0] == 60);
    assert(le_byte_array[0] == be_byte_array[30]);
    assert(le_byte_array[1] == be_byte_array[29]);
    assert(le_byte_array[2] == be_byte_array[28]);

    let z = 0 - 1;
    let p_bytes = std::field::modulus_le_bytes();
    let z_bytes: [u8; 32] = z.to_le_bytes();
    assert(p_bytes[10] == z_bytes[10]);
    assert(p_bytes[0] == z_bytes[0] as u8 + 1);

    let p_bits = std::field::modulus_le_bits();
    let z_bits: [u1; 254] = z.to_le_bits();
    assert(z_bits[0] == 0);
    assert(p_bits[100] == z_bits[100]);

    let _: [u1; 254] = a.to_le_bits();
}
