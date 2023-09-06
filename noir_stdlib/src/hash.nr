mod poseidon;

#[foreign(sha256)]
fn sha256<N>(_input : [u8; N]) -> [u8; 32] {}

#[foreign(blake2s)]
fn blake2s<N>(_input : [u8; N]) -> [u8; 32] {}

fn pedersen<N>(input : [Field; N]) -> [Field; 2] {
    pedersen_with_separator(input, 0)
}

#[foreign(pedersen)]
fn pedersen_with_separator<N>(_input : [Field; N], _separator : u32) -> [Field; 2] {}

#[foreign(hash_to_field_128_security)]
fn hash_to_field<N>(_input : [Field; N]) -> Field {}

#[foreign(keccak256)]
fn keccak256<N>(_input : [u8; N], _message_size: u32) -> [u8; 32] {}
