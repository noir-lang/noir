mod poseidon;
mod mimc;
mod poseidon2;

use crate::default::Default;
use crate::uint128::U128;
use crate::sha256::{digest, sha256_var};
use crate::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul};

#[foreign(sha256)]
// docs:start:sha256
pub fn sha256<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:sha256
{}

#[foreign(blake2s)]
// docs:start:blake2s
pub fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:blake2s
{}

#[foreign(blake3)]
// docs:start:blake3
pub fn blake3<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:blake3
{}

#[no_predicates]
// docs:start:pedersen_commitment
pub fn pedersen_commitment<let N: u32>(input: [Field; N]) -> EmbeddedCurvePoint {
    // docs:end:pedersen_commitment
    let value = pedersen_commitment_with_separator(input, 0);
    if (value.x == 0) & (value.y == 0) {
        EmbeddedCurvePoint { x: 0, y: 0, is_infinite: true }
    } else {
        EmbeddedCurvePoint { x: value.x, y: value.y, is_infinite: false }
    }
}

fn pedersen_commitment_with_separator_noir<let N: u32>(input: [Field; N], separator: u32) -> EmbeddedCurvePoint {
    let mut points = [EmbeddedCurveScalar { lo: 0, hi: 0 }; N];
    for i in 0..N {
        points[i] = EmbeddedCurveScalar::from_field(input[i]);
    }
    let generators = derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);
    multi_scalar_mul(generators, points)
}

#[no_predicates]
pub fn pedersen_commitment_with_separator<let N: u32>(input: [Field; N], separator: u32) -> EmbeddedCurvePoint {
    let values = __pedersen_commitment_with_separator(input, separator);
    EmbeddedCurvePoint { x: values[0], y: values[1], is_infinite: false }
}

// docs:start:pedersen_hash
pub fn pedersen_hash<let N: u32>(input: [Field; N]) -> Field
// docs:end:pedersen_hash
{
    pedersen_hash_with_separator(input, 0)
}

#[field(bn254)]
fn derive_generators<let N: u32, let M: u32>(
    domain_separator_bytes: [u8; M],
    starting_index: u32
) -> [EmbeddedCurvePoint; N] {
    crate::assert_constant(domain_separator_bytes);
    crate::assert_constant(starting_index);
    __derive_generators(domain_separator_bytes, starting_index)
}

#[builtin(derive_pedersen_generators)]
#[field(bn254)]
fn __derive_generators<let N: u32, let M: u32>(domain_separator_bytes: [u8; M], starting_index: u32) -> [EmbeddedCurvePoint; N] {}

fn pedersen_hash_with_separator_noir<let N: u32>(input: [Field; N], separator: u32) -> Field {
    let v1 = pedersen_commitment_with_separator(input, separator);
    let length_generator : [EmbeddedCurvePoint; 1] = derive_generators("pedersen_hash_length".as_bytes(), 0);
    multi_scalar_mul(
        [length_generator[0], v1],
        [EmbeddedCurveScalar { lo: N as Field, hi: 0 }, EmbeddedCurveScalar { lo: 1, hi: 0 }]
    ).x
}

#[foreign(pedersen_hash)]
pub fn pedersen_hash_with_separator<let N: u32>(input: [Field; N], separator: u32) -> Field {}

#[foreign(pedersen_commitment)]
fn __pedersen_commitment_with_separator<let N: u32>(input: [Field; N], separator: u32) -> [Field; 2] {}

pub fn hash_to_field(inputs: [Field]) -> Field {
    let mut sum = 0;

    for input in inputs {
        let input_bytes: [u8; 32] = input.to_le_bytes(32).as_array();
        sum += crate::field::bytes32_to_field(blake2s(input_bytes));
    }

    sum
}

#[foreign(keccak256)]
// docs:start:keccak256
pub fn keccak256<let N: u32>(input: [u8; N], message_size: u32) -> [u8; 32]
// docs:end:keccak256
{}

#[foreign(poseidon2_permutation)]
pub fn poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N] {}

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
    
    fn write(&mut self, input: Field);
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

impl Hash for Field {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self);
    }
}

impl Hash for u8 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for u32 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for u64 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for i8 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for i32 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for i64 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for bool {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self as Field);
    }
}

impl Hash for () {
    fn hash<H>(_self: Self, _state: &mut H) where H: Hasher {}
}

impl Hash for U128 {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        H::write(state, self.lo as Field);
        H::write(state, self.hi as Field);
    }
}

impl<T, let N: u32> Hash for [T; N] where T: Hash {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        for elem in self {
            elem.hash(state);
        }
    }
}

impl<T> Hash for [T] where T: Hash {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        self.len().hash(state);
        for elem in self {
            elem.hash(state);
        }
    }
}

impl<A, B> Hash for (A, B) where A: Hash, B: Hash {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<A, B, C> Hash for (A, B, C) where A: Hash, B: Hash, C: Hash {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}

impl<A, B, C, D> Hash for (A, B, C, D) where A: Hash, B: Hash, C: Hash, D: Hash {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
    }
}

impl<A, B, C, D, E> Hash for (A, B, C, D, E) where A: Hash, B: Hash, C: Hash, D: Hash, E: Hash {
    fn hash<H>(self, state: &mut H) where H: Hasher{
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
    }
}

#[test]
fn assert_pedersen_noir() {
    // TODO: make this a fuzzer test once fuzzer supports curve-specific blackbox functions.
    let input = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq(pedersen_hash_with_separator(input, 4), pedersen_hash_with_separator_noir(input, 4));
    assert_eq(pedersen_commitment_with_separator(input, 4), pedersen_commitment_with_separator_noir(input, 4));
}
