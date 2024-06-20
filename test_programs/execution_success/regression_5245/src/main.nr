use dep::std::field::modulus_be_bytes;

fn main() -> pub [u8; 32] {
    let bytes = dep::std::field::modulus_be_bytes();
    bytes.as_array()
}
