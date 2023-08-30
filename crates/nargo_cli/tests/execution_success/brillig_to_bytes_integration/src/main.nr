use dep::std;

unconstrained fn main(x : Field, _y: Field)  {
    // The result of this byte array will be big-endian
    let y: Field = 2040124;
    let be_byte_array = y.to_be_bytes(31);
    // The result of this byte array will be little-endian
    let le_byte_array = x.to_le_bytes(31);
    
    assert(le_byte_array[0] == 60);
    assert(le_byte_array[0] == be_byte_array[30]);
    assert(le_byte_array[1] == be_byte_array[29]);
    assert(le_byte_array[2] == be_byte_array[28]);

    let z = 0 - 1;
    let p_bytes = std::field::modulus_le_bytes();
    let z_bytes = z.to_le_bytes(32);
    assert(p_bytes[10] == z_bytes[10]);
    assert(p_bytes[0] == z_bytes[0] as u8 + 1 as u8);

    let p_bits = std::field::modulus_le_bits();
    let z_bits = z.to_le_bits(std::field::modulus_num_bits() as u32);
    assert(z_bits[0] == 0);
    assert(p_bits[100] == z_bits[100]);

   _y.to_le_bits(std::field::modulus_num_bits() as u32);
}
