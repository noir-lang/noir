use dep::std;

fn main(x : Field, a: Field)  {
    let y: Field = 2040124;
    let be_byte_array = y.to_be_bytes(31);
    let le_byte_array = x.to_le_bytes(31);
    
    std::assert_eq(le_byte_array[0], 60);
    std::assert_eq(le_byte_array[0], be_byte_array[30]);
    std::assert_eq(le_byte_array[1], be_byte_array[29]);
    std::assert_eq(le_byte_array[2], be_byte_array[28]);

    let z = 0 - 1;
    let p_bytes = std::field::modulus_le_bytes();
    let z_bytes = z.to_le_bytes(32);
    std::assert_eq(p_bytes[10], z_bytes[10]);
    std::assert_eq(p_bytes[0], z_bytes[0] as u8 + 1 as u8);

    let p_bits = std::field::modulus_le_bits();
    let z_bits = z.to_le_bits(std::field::modulus_num_bits() as u32);
    std::assert_eq(z_bits[0], 0);
    std::assert_eq(p_bits[100], z_bits[100]);

    a.to_le_bits(std::field::modulus_num_bits() as u32);
}
