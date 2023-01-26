use dep::std;

fn main(bn254_modulus : [u8; 32]) -> pub Field {
    let modulus_size = std::modulus_bits();

    // NOTE: The constraints used in this circuit will only work when testing nargo with the plonk bn254 backend
    constrain modulus_size == 254;

    let modulus_be_byte_array = std::modulus_be_byte_array();
    for i in 0..32 {
        constrain modulus_be_byte_array[i] == bn254_modulus[i];
    }

    modulus_size
}