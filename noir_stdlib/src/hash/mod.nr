mod poseidon;
mod mimc;
mod poseidon2;
mod keccak;

use crate::default::Default;
use crate::uint128::U128;
use crate::sha256::{digest, sha256_var};
use crate::collections::vec::Vec;
use crate::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul, multi_scalar_mul_slice};

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

// docs:start:pedersen_commitment
pub fn pedersen_commitment<let N: u32>(input: [Field; N]) -> EmbeddedCurvePoint {
    // docs:end:pedersen_commitment
    pedersen_commitment_with_separator(input, 0)
}

fn pedersen_commitment_with_separator<let N: u32>(input: [Field; N], separator: u32) -> EmbeddedCurvePoint {
    let mut points = [EmbeddedCurveScalar { lo: 0, hi: 0 }; N];
    for i in 0..N {
        // we use the unsafe version because the multi_scalar_mul will constraint the scalars.
        points[i] = from_field_unsafe(input[i]);
    }
    let generators = derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);
    let values = multi_scalar_mul(generators, points);
    EmbeddedCurvePoint { x: values[0], y: values[1], is_infinite: values[2] as bool }
}

fn pedersen_hash_with_separator<let N: u32>(input: [Field; N], separator: u32) -> Field {
    let mut scalars: Vec<EmbeddedCurveScalar> = Vec::from_slice([EmbeddedCurveScalar { lo: 0, hi: 0 }; N].as_slice()); //Vec::new();

    for i in 0..N {
        scalars.set(i, from_field_unsafe(input[i]));
    }
    scalars.push(EmbeddedCurveScalar { lo: N as Field, hi: 0 });
    let domain_generators :[EmbeddedCurvePoint; N]= derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);
    let mut vec_generators = Vec::from_slice(domain_generators.as_slice());
    let length_generator : [EmbeddedCurvePoint; 1] = derive_generators("pedersen_hash_length".as_bytes(), 0);
    vec_generators.push(length_generator[0]);
    multi_scalar_mul_slice(vec_generators.slice, scalars.slice)[0]
}

// docs:start:pedersen_hash
pub fn pedersen_hash<let N: u32>(input: [Field; N]) -> Field
// docs:end:pedersen_hash
{
    pedersen_hash_with_separator(input, 0)
}

#[field(bn254)]
fn derive_generators<let N: u32, let M: u32>(domain_separator_bytes: [u8; M], starting_index: u32) -> [EmbeddedCurvePoint; N] {
    crate::assert_constant(domain_separator_bytes);
    crate::assert_constant(starting_index);
    __derive_generators(domain_separator_bytes, starting_index)
}

#[builtin(derive_pedersen_generators)]
#[field(bn254)]
fn __derive_generators<let N: u32, let M: u32>(
    domain_separator_bytes: [u8; M],
    starting_index: u32
) -> [EmbeddedCurvePoint; N] {}

#[field(bn254)]
    // Same as from_field but:
    // does not assert the limbs are 128 bits
    // does not assert the decomposition does not overflow the EmbeddedCurveScalar
    fn from_field_unsafe(scalar: Field) -> EmbeddedCurveScalar {
    let (xlo, xhi) = crate::field::bn254::decompose_hint(scalar);
    // Check that the decomposition is correct
    assert_eq(scalar, xlo + crate::field::bn254::TWO_POW_128 * xhi);
    EmbeddedCurveScalar { lo: xlo, hi: xhi }
}

pub fn hash_to_field(inputs: [Field]) -> Field {
    let mut sum = 0;

    for input in inputs {
        let input_bytes: [u8; 32] = input.to_le_bytes(32).as_array();
        sum += crate::field::bytes32_to_field(blake2s(input_bytes));
    }

    sum
}

// docs:start:keccak256
pub fn keccak256<let N: u32>(input: [u8; N], message_size: u32) -> [u8; 32]
// docs:end:keccak256
{
    crate::hash::keccak::keccak256(input, message_size)
}

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
