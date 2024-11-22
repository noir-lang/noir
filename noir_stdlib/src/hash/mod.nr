pub mod poseidon;
pub mod poseidon2;
pub mod keccak;
pub mod sha256;
pub mod sha512;

use crate::default::Default;
use crate::embedded_curve_ops::{
    EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul, multi_scalar_mul_array_return,
};
use crate::meta::derive_via;
use crate::uint128::U128;

// Kept for backwards compatibility
pub use sha256::{digest, sha256, sha256_compression, sha256_var};

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

#[inline_always]
pub fn pedersen_commitment_with_separator<let N: u32>(
    input: [Field; N],
    separator: u32,
) -> EmbeddedCurvePoint {
    let mut points = [EmbeddedCurveScalar { lo: 0, hi: 0 }; N];
    for i in 0..N {
        // we use the unsafe version because the multi_scalar_mul will constrain the scalars.
        points[i] = from_field_unsafe(input[i]);
    }
    let generators = derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);
    multi_scalar_mul(generators, points)
}

// docs:start:pedersen_hash
pub fn pedersen_hash<let N: u32>(input: [Field; N]) -> Field
// docs:end:pedersen_hash
{
    pedersen_hash_with_separator(input, 0)
}

#[no_predicates]
pub fn pedersen_hash_with_separator<let N: u32>(input: [Field; N], separator: u32) -> Field {
    let mut scalars: [EmbeddedCurveScalar; N + 1] = [EmbeddedCurveScalar { lo: 0, hi: 0 }; N + 1];
    let mut generators: [EmbeddedCurvePoint; N + 1] =
        [EmbeddedCurvePoint::point_at_infinity(); N + 1];
    let domain_generators: [EmbeddedCurvePoint; N] =
        derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);

    for i in 0..N {
        scalars[i] = from_field_unsafe(input[i]);
        generators[i] = domain_generators[i];
    }
    scalars[N] = EmbeddedCurveScalar { lo: N as Field, hi: 0 as Field };

    let length_generator: [EmbeddedCurvePoint; 1] =
        derive_generators("pedersen_hash_length".as_bytes(), 0);
    generators[N] = length_generator[0];
    multi_scalar_mul_array_return(generators, scalars)[0]
}

#[field(bn254)]
#[inline_always]
pub fn derive_generators<let N: u32, let M: u32>(
    domain_separator_bytes: [u8; M],
    starting_index: u32,
) -> [EmbeddedCurvePoint; N] {
    crate::assert_constant(domain_separator_bytes);
    // TODO(https://github.com/noir-lang/noir/issues/5672): Add back assert_constant on starting_index
    __derive_generators(domain_separator_bytes, starting_index)
}

#[builtin(derive_pedersen_generators)]
#[field(bn254)]
fn __derive_generators<let N: u32, let M: u32>(
    domain_separator_bytes: [u8; M],
    starting_index: u32,
) -> [EmbeddedCurvePoint; N] {}

#[field(bn254)]
// Same as from_field but:
// does not assert the limbs are 128 bits
// does not assert the decomposition does not overflow the EmbeddedCurveScalar
fn from_field_unsafe(scalar: Field) -> EmbeddedCurveScalar {
    let (xlo, xhi) = unsafe { crate::field::bn254::decompose_hint(scalar) };
    // Check that the decomposition is correct
    assert_eq(scalar, xlo + crate::field::bn254::TWO_POW_128 * xhi);
    EmbeddedCurveScalar { lo: xlo, hi: xhi }
}

pub fn hash_to_field(inputs: [Field]) -> Field {
    let mut sum = 0;

    for input in inputs {
        let input_bytes: [u8; 32] = input.to_le_bytes();
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

// Generic hashing support.
// Partially ported and impacted by rust.

// Hash trait shall be implemented per type.
#[derive_via(derive_hash)]
pub trait Hash {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher;
}

// docs:start:derive_hash
comptime fn derive_hash(s: StructDefinition) -> Quoted {
    let name = quote { Hash };
    let signature = quote { fn hash<H>(_self: Self, _state: &mut H) where H: std::hash::Hasher };
    let for_each_field = |name| quote { _self.$name.hash(_state); };
    crate::meta::make_trait_impl(
        s,
        name,
        signature,
        for_each_field,
        quote {},
        |fields| fields,
    )
}
// docs:end:derive_hash

// Hasher trait shall be implemented by algorithms to provide hash-agnostic means.
// TODO: consider making the types generic here ([u8], [Field], etc.)
pub trait Hasher {
    fn finish(self) -> Field;

    fn write(&mut self, input: Field);
}

// BuildHasher is a factory trait, responsible for production of specific Hasher.
pub trait BuildHasher<H>
where
    H: Hasher,
{
    fn build_hasher(self) -> H;
}

pub struct BuildHasherDefault<H>;

impl<H> BuildHasher<H> for BuildHasherDefault<H>
where
    H: Hasher + Default,
{
    fn build_hasher(_self: Self) -> H {
        H::default()
    }
}

impl<H> Default for BuildHasherDefault<H>
where
    H: Hasher + Default,
{
    fn default() -> Self {
        BuildHasherDefault {}
    }
}

impl Hash for Field {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self);
    }
}

impl Hash for u1 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for u8 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for u16 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for u32 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for u64 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for i8 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for i16 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for i32 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for i64 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for bool {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as Field);
    }
}

impl Hash for () {
    fn hash<H>(_self: Self, _state: &mut H)
    where
        H: Hasher,
    {}
}

impl Hash for U128 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self.lo as Field);
        H::write(state, self.hi as Field);
    }
}

impl<T, let N: u32> Hash for [T; N]
where
    T: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        for elem in self {
            elem.hash(state);
        }
    }
}

impl<T> Hash for [T]
where
    T: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self.len().hash(state);
        for elem in self {
            elem.hash(state);
        }
    }
}

impl<A, B> Hash for (A, B)
where
    A: Hash,
    B: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<A, B, C> Hash for (A, B, C)
where
    A: Hash,
    B: Hash,
    C: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}

impl<A, B, C, D> Hash for (A, B, C, D)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
    }
}

impl<A, B, C, D, E> Hash for (A, B, C, D, E)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
    }
}

// Some test vectors for Pedersen hash and Pedersen Commitment.
// They have been generated using the same functions so the tests are for now useless
// but they will be useful when we switch to Noir implementation.
#[test]
fn assert_pedersen() {
    assert_eq(
        pedersen_hash_with_separator([1], 1),
        0x1b3f4b1a83092a13d8d1a59f7acb62aba15e7002f4440f2275edb99ebbc2305f,
    );
    assert_eq(
        pedersen_commitment_with_separator([1], 1),
        EmbeddedCurvePoint {
            x: 0x054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402,
            y: 0x209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126,
            is_infinite: false,
        },
    );

    assert_eq(
        pedersen_hash_with_separator([1, 2], 2),
        0x26691c129448e9ace0c66d11f0a16d9014a9e8498ee78f4d69f0083168188255,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2], 2),
        EmbeddedCurvePoint {
            x: 0x2e2b3b191e49541fe468ec6877721d445dcaffe41728df0a0eafeb15e87b0753,
            y: 0x2ff4482400ad3a6228be17a2af33e2bcdf41be04795f9782bd96efe7e24f8778,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3], 3),
        0x0bc694b7a1f8d10d2d8987d07433f26bd616a2d351bc79a3c540d85b6206dbe4,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3], 3),
        EmbeddedCurvePoint {
            x: 0x1fee4e8cf8d2f527caa2684236b07c4b1bad7342c01b0f75e9a877a71827dc85,
            y: 0x2f9fedb9a090697ab69bf04c8bc15f7385b3e4b68c849c1536e5ae15ff138fd1,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4], 4),
        0xdae10fb32a8408521803905981a2b300d6a35e40e798743e9322b223a5eddc,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4], 4),
        EmbeddedCurvePoint {
            x: 0x07ae3e202811e1fca39c2d81eabe6f79183978e6f12be0d3b8eda095b79bdbc9,
            y: 0x0afc6f892593db6fbba60f2da558517e279e0ae04f95758587760ba193145014,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5], 5),
        0xfc375b062c4f4f0150f7100dfb8d9b72a6d28582dd9512390b0497cdad9c22,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5], 5),
        EmbeddedCurvePoint {
            x: 0x1754b12bd475a6984a1094b5109eeca9838f4f81ac89c5f0a41dbce53189bb29,
            y: 0x2da030e3cfcdc7ddad80eaf2599df6692cae0717d4e9f7bfbee8d073d5d278f7,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6], 6),
        0x1696ed13dc2730062a98ac9d8f9de0661bb98829c7582f699d0273b18c86a572,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6], 6),
        EmbeddedCurvePoint {
            x: 0x190f6c0e97ad83e1e28da22a98aae156da083c5a4100e929b77e750d3106a697,
            y: 0x1f4b60f34ef91221a0b49756fa0705da93311a61af73d37a0c458877706616fb,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7], 7),
        0x128c0ff144fc66b6cb60eeac8a38e23da52992fc427b92397a7dffd71c45ede3,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7], 7),
        EmbeddedCurvePoint {
            x: 0x015441e9d29491b06563fac16fc76abf7a9534c715421d0de85d20dbe2965939,
            y: 0x1d2575b0276f4e9087e6e07c2cb75aa1baafad127af4be5918ef8a2ef2fea8fc,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7, 8], 8),
        0x2f960e117482044dfc99d12fece2ef6862fba9242be4846c7c9a3e854325a55c,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7, 8], 8),
        EmbeddedCurvePoint {
            x: 0x1657737676968887fceb6dd516382ea13b3a2c557f509811cd86d5d1199bc443,
            y: 0x1f39f0cb569040105fa1e2f156521e8b8e08261e635a2b210bdc94e8d6d65f77,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9], 9),
        0x0c96db0790602dcb166cc4699e2d306c479a76926b81c2cb2aaa92d249ec7be7,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9], 9),
        EmbeddedCurvePoint {
            x: 0x0a3ceae42d14914a432aa60ec7fded4af7dad7dd4acdbf2908452675ec67e06d,
            y: 0xfc19761eaaf621ad4aec9a8b2e84a4eceffdba78f60f8b9391b0bd9345a2f2,
            is_infinite: false,
        },
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 10),
        0x2cd37505871bc460a62ea1e63c7fe51149df5d0801302cf1cbc48beb8dff7e94,
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 10),
        EmbeddedCurvePoint {
            x: 0x2fb3f8b3d41ddde007c8c3c62550f9a9380ee546fcc639ffbb3fd30c8d8de30c,
            y: 0x300783be23c446b11a4c0fabf6c91af148937cea15fcf5fb054abf7f752ee245,
            is_infinite: false,
        },
    );
}
