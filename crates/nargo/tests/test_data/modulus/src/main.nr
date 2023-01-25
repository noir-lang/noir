use dep::std;

fn main() -> pub Field {
    let modulus_size = std::modulus_bits();

    // NOTE: This will only work when testing nargo with the plonk bn254 backend
    constrain modulus_size == 254;

    modulus_size
}