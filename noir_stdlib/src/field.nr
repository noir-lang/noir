#[builtin(to_bits_le)]
fn to_bits_le(_x : Field, _bit_size: u32) -> [u1] {}

fn to_bytes_le(x : Field, byte_size: u32) -> [u8] {
    to_radix(x, 256, byte_size)
}

#[builtin(to_radix)]
//decompose _x into a _result_len vector over the _radix basis
//_radix must be less than 256
fn to_radix(_x : Field, _radix: u32, _result_len: u32) -> [u8] {}

#[builtin(modulus_num_bits)]
fn modulus_num_bits() -> comptime Field {}

#[builtin(modulus_bits_be)]
fn modulus_bits_be() -> [u1] {}

#[builtin(modulus_bits_le)]
fn modulus_bits_le() -> [u1] {}

#[builtin(modulus_bytes_be)]
fn modulus_bytes_be() -> [u8] {}

#[builtin(modulus_bytes_le)]
fn modulus_bytes_le() -> [u8] {}