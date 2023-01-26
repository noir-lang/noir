#[builtin(to_bits)]
fn to_bits(_x : Field, _bit_size: u32) -> [u1] {}

fn to_le_bytes(x : Field, byte_size: u32) -> [u8] {
    to_radix(x, 256, byte_size)
}

#[builtin(to_radix)]
//decompose _x into a _result_len vector over the _radix basis
//_radix must be less than 256
fn to_radix(_x : Field, _radix: u32, _result_len: u32) -> [u8] {}

#[builtin(modulus_num_bits)]
fn modulus_num_bits() -> comptime Field {}

#[builtin(modulus_be_bits)]
fn modulus_be_bits() -> [u1] {}

#[builtin(modulus_le_bits)]
fn modulus_le_bits() -> [u1] {}

#[builtin(modulus_be_byte_array)]
fn modulus_be_byte_array() -> [u8] {}

#[builtin(modulus_le_byte_array)]
fn modulus_le_byte_array() -> [u8] {}