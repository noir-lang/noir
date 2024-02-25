pub mod poseidon;
pub mod mimc;
pub mod poseidon2;
pub mod pedersen;

use crate::default::Default;

#[foreign(sha256)]
// docs:start:sha256
pub fn sha256<N>(input: [u8; N]) -> [u8; 32]
// docs:end:sha256
{}

#[foreign(blake2s)]
// docs:start:blake2s
pub fn blake2s<N>(input: [u8; N]) -> [u8; 32]
// docs:end:blake2s
{}

#[foreign(blake3)]
// docs:start:blake3
pub fn blake3<N>(input: [u8; N]) -> [u8; 32]
// docs:end:blake3
{}

// docs:start:pedersen_commitment
struct PedersenPoint {
   x : Field,
   y : Field,
}

pub fn pedersen_commitment<N>(input: [Field; N]) -> PedersenPoint
// docs:end:pedersen_commitment
{
    pedersen_commitment_with_separator(input, 0)
}

#[foreign(pedersen_commitment)]
pub fn __pedersen_commitment_with_separator<N>(input: [Field; N], separator: u32) -> [Field; 2] {}

pub fn pedersen_commitment_with_separator<N>(input: [Field; N], separator: u32) -> PedersenPoint {
    let values = __pedersen_commitment_with_separator(input, separator);
    PedersenPoint { x: values[0], y: values[1] }
}

// docs:start:pedersen_hash
pub fn pedersen_hash<N>(input: [Field; N]) -> Field
// docs:end:pedersen_hash
{
    pedersen_hash_with_separator(input, 0)
}

#[foreign(pedersen_hash)]
pub fn pedersen_hash_with_separator<N>(input: [Field; N], separator: u32) -> Field {}

pub fn hash_to_field<N>(input: [Field; N]) -> Field {
    let mut inputs_as_bytes = [];

    for i in 0..N {
        let input_bytes = input[i].to_le_bytes(32);
        for i in 0..32 {
            inputs_as_bytes = inputs_as_bytes.push_back(input_bytes[i]);
        }
    }

    let hashed_input = blake2s(inputs_as_bytes);
    crate::field::bytes32_to_field(hashed_input)
}

#[foreign(keccak256)]
// docs:start:keccak256
pub fn keccak256<N>(input: [u8; N], message_size: u32) -> [u8; 32]
// docs:end:keccak256
{}

#[foreign(poseidon2_permutation)]
pub fn poseidon2_permutation<N>(_input: [Field; N], _state_length: u32) -> [Field; N] {}

#[foreign(sha256_compression)]
pub fn sha256_compression(_input: [u32; 16], _state: [u32; 8]) -> [u32; 8] {}

// Generic hashing support. 
// Partially ported and impacted by rust.

// Hash trait shall be implemented per type.
trait Hash{
    fn hash<H>(self, state: &mut H) where H: Hasher;
}

// Hasher trait shall be implemented by algorithms to provide hash-agnostic means.
// TODO: consider making the types generic here ([u8], [Field], etc.)
trait Hasher{
    fn finish(self) -> Field;
    
    fn write(&mut self, input: [Field]);
}

// BuildHasher is a factory trait, responsible for production of specific Hasher.
trait BuildHasher<H> where H: Hasher{
    fn build_hasher(self) -> H;
}

struct BuildHasherDefault<H>;

impl<H> BuildHasher<H> for BuildHasherDefault<H>
where 
    H: Hasher + Default
{
    fn build_hasher(_self: Self) -> H{
        H::default()
    }
}

impl<H> Default for BuildHasherDefault<H>
where 
    H: Hasher + Default
{
    fn default() -> Self{
        BuildHasherDefault{}
    }    
}

// TODO: add implementations for the remainder of primitive types.
impl Hash for Field{
    fn hash<H>(self, state: &mut H) where H: Hasher{
        let input: [Field] = [self];
        H::write(state, input);
    }
}
