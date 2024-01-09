mod poseidon;
mod mimc;

#[foreign(sha256)]
pub fn sha256<N>(_input: [u8; N]) -> [u8; 32] {}

#[foreign(blake2s)]
pub fn blake2s<N>(_input: [u8; N]) -> [u8; 32] {}

#[foreign(blake3)]
pub fn blake3<N>(_input: [u8; N]) -> [u8; 32] {}

struct PedersenPoint {
   x : Field,
   y : Field,
}

pub fn pedersen_commitment<N>(input: [Field; N]) -> PedersenPoint {
    pedersen_commitment_with_separator(input, 0)
}

#[foreign(pedersen_commitment)]
pub fn __pedersen_commitment_with_separator<N>(_input: [Field; N], _separator: u32) -> [Field; 2] {}

pub fn pedersen_commitment_with_separator<N>(input: [Field; N], separator: u32) -> PedersenPoint {
    let values = __pedersen_commitment_with_separator(input, separator);
    PedersenPoint { x: values[0], y: values[1] }
}

pub fn pedersen_hash<N>(input: [Field; N]) -> Field {
    pedersen_hash_with_separator(input, 0)
}

#[foreign(pedersen_hash)]
pub fn pedersen_hash_with_separator<N>(_input: [Field; N], _separator: u32) -> Field {}

pub fn hash_to_field<N>(_input: [Field; N]) -> Field {
    let mut inputs_as_bytes = [];

    for i in 0..N {
        let input_bytes = _input[i].to_le_bytes(32);
        for i in 0..32 {
            inputs_as_bytes = inputs_as_bytes.push_back(input_bytes[i]);
        }
    }

    let hashed_input = blake2s(inputs_as_bytes);
    crate::field::bytes32_to_field(hashed_input)
}

#[foreign(keccak256)]
pub fn keccak256<N>(_input: [u8; N], _message_size: u32) -> [u8; 32] {}

