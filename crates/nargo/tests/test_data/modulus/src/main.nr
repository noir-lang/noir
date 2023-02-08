use dep::std;

fn main(bn254_modulus_be_bytes : [u8; 32], bn254_modulus_be_bits : [u1; 254]) -> pub Field {
    let modulus_size = std::field::modulus_num_bits();
    // NOTE: The constraints used in this circuit will only work when testing nargo with the plonk bn254 backend
    constrain modulus_size == 254;
 
    let modulus_be_byte_array = std::field::modulus_be_bytes();
    for i in 0..32 {
        constrain modulus_be_byte_array[i] == bn254_modulus_be_bytes[i];
    }
    let modulus_le_byte_array = std::field::modulus_le_bytes();
    for i in 0..32 {
        constrain modulus_le_byte_array[i] == bn254_modulus_be_bytes[31-i];
    }

    let modulus_be_bits = std::field::modulus_be_bits();
    for i in 0..254 {
        constrain modulus_be_bits[i] == bn254_modulus_be_bits[i];
    }
    let modulus_le_bits = std::field::modulus_le_bits();
    for i in 0..254 {
        constrain modulus_le_bits[i] == bn254_modulus_be_bits[253-i];
    }

    modulus_size
}