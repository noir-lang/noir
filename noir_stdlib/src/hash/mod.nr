// Exposed only for usage in `std::meta`
pub(crate) mod poseidon2;

use crate::default::Default;
use crate::embedded_curve_ops::{
    EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul, multi_scalar_mul_array_return,
};
use crate::meta::derive_via;
use crate::static_assert;

/// The size of the state accepted by the backend in `poseidon2_permutation`.
global POSEIDON2_CONFIG_STATE_SIZE: u32 = poseidon2_config_state_size();

/// Applies the SHA-256 compression function to one 512-bit message block.
///
/// `input` is the 16-word message block and `state` is the 8-word hash state; the result is the
/// next hash state. Words are big-endian, as in FIPS 180-4.
///
/// This is only the compression function, not the SHA-256 hash: it does no padding and no length
/// encoding, and processes exactly one block. To hash arbitrary bytes, use the
/// [`sha256`](https://github.com/noir-lang/sha256) library, which is built on this function.
///
/// Example:
///
/// ```noir
/// // The single padded block for the message "abc".
/// let mut block = [0; 16];
/// block[0] = 0x61626380; // "abc" followed by the 0x80 padding byte
/// block[15] = 24; // message length in bits
///
/// // The SHA-256 initial hash state.
/// let iv = [
///     0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
///     0x5be0cd19,
/// ];
///
/// // For a one-block message, compressing the padded block gives the SHA-256 digest.
/// let digest = std::hash::sha256_compression(block, iv);
/// assert_eq(
///     digest,
///     [
///         0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61,
///         0xf20015ad,
///     ],
/// );
/// ```
#[foreign(sha256_compression)]
// docs:start:sha256_compression
pub fn sha256_compression(input: [u32; 16], state: [u32; 8]) -> [u32; 8] {}
// docs:end:sha256_compression

/// Applies the Keccak-f[1600] permutation to a 25-lane state.
///
/// Each lane is a 64-bit word, and lane `x + 5 * y` holds the state word at column `x`, row `y`.
///
/// This is only the permutation, not a Keccak or SHA-3 hash: it does no absorbing, padding or
/// squeezing. To hash bytes with Keccak-256, use the
/// [`keccak256`](https://github.com/noir-lang/keccak256) library, which is built on this function.
///
/// Example:
///
/// ```noir
/// let state = std::hash::keccakf1600([0; 25]);
/// // The first lane of Keccak-f[1600] applied to the all-zero state.
/// assert_eq(state[0], 0xF1258F7940E1DDE7);
/// ```
#[foreign(keccakf1600)]
// docs:start:keccakf1600
pub fn keccakf1600(input: [u64; 25]) -> [u64; 25] {}
// docs:end:keccakf1600

/// Returns the BLAKE2s-256 hash of `input`.
///
/// The hash is unkeyed and covers exactly the `N` bytes of `input`.
///
/// Example:
///
/// ```noir
/// let digest = std::hash::blake2s("abc".as_bytes());
/// assert_eq(
///     digest,
///     [
///         0x50, 0x8c, 0x5e, 0x8c, 0x32, 0x7c, 0x14, 0xe2, 0xe1, 0xa7, 0x2b, 0xa3, 0x4e, 0xeb,
///         0x45, 0x2f, 0x37, 0x45, 0x8b, 0x20, 0x9e, 0xd6, 0x3a, 0x29, 0x4d, 0x99, 0x9b, 0x4c,
///         0x86, 0x67, 0x59, 0x82,
///     ],
/// );
/// ```
#[foreign(blake2s)]
// docs:start:blake2s
pub fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:blake2s
{}

/// Returns the BLAKE3 hash of `input`, truncated to the default 32-byte output.
///
/// The hash is unkeyed and covers exactly the `N` bytes of `input`. Barretenberg can't prove
/// BLAKE3 hashes of inputs longer than 1024 bytes.
///
/// Example:
///
/// ```noir
/// let digest = std::hash::blake3("abc".as_bytes());
/// assert_eq(
///     digest,
///     [
///         0x64, 0x37, 0xb3, 0xac, 0x38, 0x46, 0x51, 0x33, 0xff, 0xb6, 0x3b, 0x75, 0x27, 0x3a,
///         0x8d, 0xb5, 0x48, 0xc5, 0x58, 0x46, 0x5d, 0x79, 0xdb, 0x03, 0xfd, 0x35, 0x9c, 0x6c,
///         0xd5, 0xbd, 0x9d, 0x85,
///     ],
/// );
/// ```
// docs:start:blake3
pub fn blake3<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:blake3
{
    if crate::runtime::is_unconstrained() {
        // Temporary measure while Barretenberg is main proving system.
        // Please open an issue if you're working on another proving system and running into problems due to this.
        crate::static_assert(
            N <= 1024,
            "Barretenberg cannot prove blake3 hashes with inputs larger than 1024 bytes",
        );
    }
    __blake3(input)
}

#[foreign(blake3)]
fn __blake3<let N: u32>(input: [u8; N]) -> [u8; 32] {}

/// Returns the Pedersen commitment to `input`, a point on the embedded curve.
///
/// The commitment is `input[0] * G[0] + ... + input[N - 1] * G[N - 1]`, where the generators
/// `G[i]` are fixed points derived by [`derive_generators`] from the domain separator
/// `"DEFAULT_DOMAIN_SEPARATOR"`. Nobody knows a discrete-log relation between the generators, so
/// the commitment is binding. It is not hiding unless one of the inputs is a random blinding value.
///
/// The commitment does not include the input length, and a zero input adds nothing to the sum,
/// so inputs that differ only by trailing zeros have the same commitment. Use [`pedersen_hash`],
/// which does include the length, when inputs of different lengths must not collide.
///
/// Only available when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// use std::hash::pedersen_commitment;
///
/// let commitment = pedersen_commitment([1, 2]);
/// assert(commitment != pedersen_commitment([2, 1]));
///
/// // Trailing zeros don't change the commitment.
/// assert_eq(pedersen_commitment([1, 2]), pedersen_commitment([1, 2, 0]));
/// ```
// docs:start:pedersen_commitment
pub fn pedersen_commitment<let N: u32>(input: [Field; N]) -> EmbeddedCurvePoint {
    // docs:end:pedersen_commitment
    pedersen_commitment_with_separator(input, 0)
}

/// Returns the Pedersen commitment to `input`, using the generators starting at index `separator`.
///
/// The commitment is `input[0] * G[separator] + ... + input[N - 1] * G[separator + N - 1]`, and
/// [`pedersen_commitment`] is this function with a `separator` of 0. All the generators come from
/// one sequence derived from `"DEFAULT_DOMAIN_SEPARATOR"`, so `separator` is a generator offset,
/// not domain separation: commitments with different separators share generators and can be equal
/// for different inputs. A commitment with separator 1 equals a commitment with separator 0 whose
/// input has an extra leading zero. Like [`pedersen_commitment`], the commitment does not include
/// the input length.
///
/// Only available when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// use std::hash::{pedersen_commitment, pedersen_commitment_with_separator};
///
/// let x = 42;
/// assert_eq(pedersen_commitment_with_separator([x], 0), pedersen_commitment([x]));
///
/// // Separator 1 starts one generator later, so it matches a leading zero at separator 0.
/// assert_eq(pedersen_commitment_with_separator([x], 1), pedersen_commitment([0, x]));
/// ```
///
/// # Failures
///
/// Fails to compile if `separator` is not a compile-time constant.
#[inline_always]
pub fn pedersen_commitment_with_separator<let N: u32>(
    input: [Field; N],
    separator: u32,
) -> EmbeddedCurvePoint {
    let mut points = [EmbeddedCurveScalar { lo: 0, hi: 0 }; N];
    for i in 0..N {
        points[i] = EmbeddedCurveScalar::from_field(input[i]);
    }
    let generators = derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);
    multi_scalar_mul(generators, points)
}

/// Returns the Pedersen hash of `input`.
///
/// The hash is the x-coordinate of `input[0] * G[0] + ... + input[N - 1] * G[N - 1] + N * L`, where
/// the generators `G[i]` are those of [`pedersen_commitment`] and `L` is a separate generator for
/// the length. Because the length is included, inputs that differ only by trailing zeros hash to
/// different values.
///
/// Only available when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// use std::hash::pedersen_hash;
///
/// let hash = pedersen_hash([1, 2]);
/// assert(hash != pedersen_hash([2, 1]));
///
/// // The length is part of the hash.
/// assert(pedersen_hash([1, 2]) != pedersen_hash([1, 2, 0]));
/// ```
// docs:start:pedersen_hash
pub fn pedersen_hash<let N: u32>(input: [Field; N]) -> Field
// docs:end:pedersen_hash
{
    pedersen_hash_with_separator(input, 0)
}

/// Returns the Pedersen hash of `input`, using the generators starting at index `separator`.
///
/// The hash is the x-coordinate of `input[0] * G[separator] + ... + input[N - 1] * G[separator + N - 1] + N * L`,
/// with the generators `G[i]` and `L` of [`pedersen_hash`], which is this function with a
/// `separator` of 0. All the generators come from one sequence derived from
/// `"DEFAULT_DOMAIN_SEPARATOR"`, so `separator` is a generator offset, not domain separation, and
/// hashes with different separators can collide: `[0, x]` with separator 0 and `[x, 0]` with
/// separator 1 have the same hash, because both compute `x * G[1] + 2 * L`. Don't use different
/// separators to keep hashes of different kinds of data apart; include a distinct tag in the input
/// instead, at the same position for every kind.
///
/// Only available when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// use std::hash::{pedersen_hash, pedersen_hash_with_separator};
///
/// let x = 42;
/// assert_eq(pedersen_hash_with_separator([x], 0), pedersen_hash([x]));
///
/// // Different separators are not independent hash functions.
/// assert_eq(pedersen_hash_with_separator([0, x], 0), pedersen_hash_with_separator([x, 0], 1));
/// ```
///
/// # Failures
///
/// Fails to compile if `separator` is not a compile-time constant.
#[no_predicates]
pub fn pedersen_hash_with_separator<let N: u32>(input: [Field; N], separator: u32) -> Field {
    let mut scalars: [EmbeddedCurveScalar; N + 1] = [EmbeddedCurveScalar { lo: 0, hi: 0 }; N + 1];
    let mut generators: [EmbeddedCurvePoint; N + 1] =
        [EmbeddedCurvePoint::point_at_infinity(); N + 1];
    crate::assert_constant(separator);
    let domain_generators: [EmbeddedCurvePoint; N] =
        derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), separator);

    for i in 0..N {
        scalars[i] = EmbeddedCurveScalar::from_field(input[i]);
        generators[i] = domain_generators[i];
    }
    scalars[N] = EmbeddedCurveScalar { lo: N as Field, hi: 0 as Field };

    let length_generator: [EmbeddedCurvePoint; 1] =
        derive_generators("pedersen_hash_length".as_bytes(), 0);
    generators[N] = length_generator[0];
    multi_scalar_mul_array_return(generators, scalars, true)[0].x
}

/// Derives `N` generator points on the embedded curve from a domain separator.
///
/// The points are generators `starting_index` to `starting_index + N - 1` of the sequence for
/// `domain_separator_bytes`. Generator `i` is obtained by hashing the BLAKE3 hash of `domain_separator_bytes` together with
/// the index `i` onto the curve, so nobody knows a discrete-log relation between any two
/// generators. The same domain separator and index always give the same point, and the
/// generators for different domain separators are independent. [`pedersen_commitment`] and
/// [`pedersen_hash`] use the domain separator `"DEFAULT_DOMAIN_SEPARATOR"`.
///
/// Only available when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// use std::embedded_curve_ops::EmbeddedCurvePoint;
/// use std::hash::derive_generators;
///
/// let generators: [EmbeddedCurvePoint; 2] = derive_generators("my_domain".as_bytes(), 0);
/// assert(generators[0] != generators[1]);
///
/// // Starting one index later gives the same sequence shifted by one.
/// let shifted: [EmbeddedCurvePoint; 1] = derive_generators("my_domain".as_bytes(), 1);
/// assert_eq(shifted[0], generators[1]);
/// ```
///
/// # Failures
///
/// Fails to compile if `domain_separator_bytes` or `starting_index` is not a compile-time
/// constant.
#[field(bn254)]
#[inline_always]
pub fn derive_generators<let N: u32, let M: u32>(
    domain_separator_bytes: [u8; M],
    starting_index: u32,
) -> [EmbeddedCurvePoint; N] {
    crate::assert_constant(domain_separator_bytes);
    crate::assert_constant(starting_index);
    __derive_generators(domain_separator_bytes, starting_index)
}

#[builtin(derive_pedersen_generators)]
#[field(bn254)]
fn __derive_generators<let N: u32, let M: u32>(
    domain_separator_bytes: [u8; M],
    starting_index: u32,
) -> [EmbeddedCurvePoint; N] {}

/// Applies the Poseidon2 permutation to `input`.
///
/// The state size `N` must equal the state size of the backend's Poseidon2 configuration, which
/// is 4 for BN254. This is only the permutation, not a Poseidon2 hash: to hash a sequence of
/// fields, use the [`poseidon`](https://github.com/noir-lang/poseidon) library, which builds a
/// sponge on this function.
///
/// Example:
///
/// ```noir
/// let state = std::hash::poseidon2_permutation([0, 1, 2, 3]);
/// assert_eq(state[0], 0x01BD538C2EE014ED5141B29E9AE240BF8DB3FE5B9A38629A9647CF8D76C01737);
/// ```
///
/// # Failures
///
/// Fails to compile if `N` is not the configured state size.
pub fn poseidon2_permutation<let N: u32>(input: [Field; N]) -> [Field; N] {
    static_assert(
        N == POSEIDON2_CONFIG_STATE_SIZE,
        f"the input length must equal the state size in the Poseidon2 config; expected {POSEIDON2_CONFIG_STATE_SIZE}, got {N}",
    );
    poseidon2_permutation_internal(input)
}

#[foreign(poseidon2_permutation)]
fn poseidon2_permutation_internal<let N: u32>(input: [Field; N]) -> [Field; N] {}

#[foreign(poseidon2_config_state_size)]
comptime fn poseidon2_config_state_size() -> u32 {}

// Generic hashing support.
// Partially ported and impacted by rust.

/// A type that can be fed into a [`Hasher`].
///
/// `hash` writes the value to the hasher as a sequence of `Field`s. Two values that are equal
/// must write the same sequence. The implementations in the standard library write integers and
/// `bool`s as a single `Field`, arrays and tuples as their elements in order, and vectors as
/// their length followed by their elements.
///
/// The trait can be derived with `#[derive(Hash)]`, which hashes each field of a struct in order.
///
/// Example:
///
/// ```noir
/// use std::hash::{Hash, Hasher};
///
/// // A hasher that sums its inputs, to show what `hash` writes.
/// struct Sum {
///     total: Field,
/// }
///
/// impl Hasher for Sum {
///     fn finish(self) -> Field {
///         self.total
///     }
///
///     fn write(&mut self, input: Field) {
///         self.total += input;
///     }
/// }
///
/// #[derive(Hash)]
/// struct Point {
///     x: Field,
///     y: Field,
/// }
///
/// fn main() {
///     let mut hasher = Sum { total: 0 };
///     Point { x: 3, y: 4 }.hash(&mut hasher);
///     assert_eq(hasher.finish(), 7);
/// }
/// ```
#[derive_via(derive_hash)]
pub trait Hash {
    /// Writes `self` to `state`.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::hash::{Hash, Hasher};
    ///
    /// struct Sum {
    ///     total: Field,
    /// }
    ///
    /// impl Hasher for Sum {
    ///     fn finish(self) -> Field {
    ///         self.total
    ///     }
    ///
    ///     fn write(&mut self, input: Field) {
    ///         self.total += input;
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut hasher = Sum { total: 0 };
    ///     (1, 2).hash(&mut hasher);
    ///     assert_eq(hasher.finish(), 3);
    /// }
    /// ```
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher;
}

// docs:start:derive_hash
comptime fn derive_hash(s: TypeDefinition) -> Quoted {
    let name = quote { $crate::hash::Hash };
    let signature = quote { fn hash<H>(_self: Self, _state: &mut H) where H: $crate::hash::Hasher };
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

// TODO: consider making the types generic here ([u8], [Field], etc.)
/// A hash function that accepts input one `Field` at a time.
///
/// Values that implement [`Hash`] write themselves to a `Hasher` with [`Hasher::write`], and
/// [`Hasher::finish`] returns the hash of everything written. Hashers are used by hash-based
/// collections and by generic code that needs to hash values of any type.
///
/// Example:
///
/// ```noir
/// use std::hash::Hasher;
///
/// struct Sum {
///     total: Field,
/// }
///
/// impl Hasher for Sum {
///     fn finish(self) -> Field {
///         self.total
///     }
///
///     fn write(&mut self, input: Field) {
///         self.total += input;
///     }
/// }
///
/// fn main() {
///     let mut hasher = Sum { total: 0 };
///     hasher.write(1);
///     hasher.write(2);
///     assert_eq(hasher.finish_ref(), 3);
///     assert_eq(hasher.finish(), 3);
/// }
/// ```
pub trait Hasher {
    /// Returns the hash of everything written so far, consuming the hasher.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::hash::Hasher;
    ///
    /// struct Sum {
    ///     total: Field,
    /// }
    ///
    /// impl Hasher for Sum {
    ///     fn finish(self) -> Field {
    ///         self.total
    ///     }
    ///
    ///     fn write(&mut self, input: Field) {
    ///         self.total += input;
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut hasher = Sum { total: 0 };
    ///     hasher.write(4);
    ///     assert_eq(hasher.finish(), 4);
    /// }
    /// ```
    fn finish(self) -> Field;

    // TODO: deprecate finish() and replace it
    /// Returns the hash value without consuming the hasher.
    /// Override this for more efficient implementations that avoid copying.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::hash::Hasher;
    ///
    /// struct Sum {
    ///     total: Field,
    /// }
    ///
    /// impl Hasher for Sum {
    ///     fn finish(self) -> Field {
    ///         self.total
    ///     }
    ///
    ///     fn write(&mut self, input: Field) {
    ///         self.total += input;
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut hasher = Sum { total: 0 };
    ///     hasher.write(4);
    ///     assert_eq(hasher.finish_ref(), 4);
    ///
    ///     // The hasher can still be written to.
    ///     hasher.write(1);
    ///     assert_eq(hasher.finish_ref(), 5);
    /// }
    /// ```
    fn finish_ref(&self) -> Field {
        (*self).finish()
    }

    /// Adds `input` to the data being hashed.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::hash::Hasher;
    ///
    /// struct Sum {
    ///     total: Field,
    /// }
    ///
    /// impl Hasher for Sum {
    ///     fn finish(self) -> Field {
    ///         self.total
    ///     }
    ///
    ///     fn write(&mut self, input: Field) {
    ///         self.total += input;
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut hasher = Sum { total: 0 };
    ///     hasher.write(1);
    ///     hasher.write(2);
    ///     assert_eq(hasher.finish(), 3);
    /// }
    /// ```
    fn write(&mut self, input: Field);
}

/// A factory for [`Hasher`]s.
///
/// Hash-based collections take a `BuildHasher` so that each one can create fresh hashers of a
/// chosen type. [`BuildHasherDefault`] builds any hasher that implements `Default`; implement this
/// trait directly when building a hasher needs some state, such as a starting value.
///
/// Example:
///
/// ```noir
/// use std::hash::{BuildHasher, Hasher};
///
/// pub struct Sum {
///     total: Field,
/// }
///
/// impl Hasher for Sum {
///     fn finish(self) -> Field {
///         self.total
///     }
///
///     fn write(&mut self, input: Field) {
///         self.total += input;
///     }
/// }
///
/// // Builds `Sum` hashers that start from a fixed offset.
/// struct SumFrom {
///     start: Field,
/// }
///
/// impl BuildHasher for SumFrom {
///     type H = Sum;
///
///     fn build_hasher(self) -> Sum {
///         Sum { total: self.start }
///     }
/// }
///
/// fn main() {
///     let builder = SumFrom { start: 10 };
///     let mut hasher = builder.build_hasher();
///     hasher.write(5);
///     assert_eq(hasher.finish(), 15);
/// }
/// ```
pub trait BuildHasher {
    /// The type of hasher this builds.
    type H: Hasher;

    /// Returns a new hasher with no input written to it.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
    ///
    /// struct Sum {
    ///     total: Field,
    /// }
    ///
    /// impl Hasher for Sum {
    ///     fn finish(self) -> Field {
    ///         self.total
    ///     }
    ///
    ///     fn write(&mut self, input: Field) {
    ///         self.total += input;
    ///     }
    /// }
    ///
    /// impl Default for Sum {
    ///     fn default() -> Self {
    ///         Sum { total: 0 }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let builder: BuildHasherDefault<Sum> = BuildHasherDefault::default();
    ///     let hasher = builder.build_hasher();
    ///     assert_eq(hasher.finish(), 0);
    /// }
    /// ```
    fn build_hasher(self) -> H;
}

/// A [`BuildHasher`] that creates hashers of type `H` with `H::default()`.
///
/// Example:
///
/// ```noir
/// use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
///
/// struct Sum {
///     total: Field,
/// }
///
/// impl Hasher for Sum {
///     fn finish(self) -> Field {
///         self.total
///     }
///
///     fn write(&mut self, input: Field) {
///         self.total += input;
///     }
/// }
///
/// impl Default for Sum {
///     fn default() -> Self {
///         Sum { total: 0 }
///     }
/// }
///
/// fn main() {
///     let builder: BuildHasherDefault<Sum> = BuildHasherDefault::default();
///     let mut hasher = builder.build_hasher();
///     hasher.write(5);
///     assert_eq(hasher.finish(), 5);
/// }
/// ```
pub struct BuildHasherDefault<H>;

impl<H> BuildHasher for BuildHasherDefault<H>
where
    H: Hasher + Default,
{
    type H = H;

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

impl Hash for u128 {
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
        H::write(state, self as u8 as Field);
    }
}

impl Hash for i16 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as u16 as Field);
    }
}

impl Hash for i32 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as u32 as Field);
    }
}

impl Hash for i64 {
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        H::write(state, self as u64 as Field);
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

impl<A> Hash for (A,)
where
    A: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state);
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

impl<A, B, C, D, E, F> Hash for (A, B, C, D, E, F)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
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
        self.5.hash(state);
    }
}

impl<A, B, C, D, E, F, G> Hash for (A, B, C, D, E, F, G)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
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
        self.5.hash(state);
        self.6.hash(state);
    }
}

impl<A, B, C, D, E, F, G, H_> Hash for (A, B, C, D, E, F, G, H_)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
    H_: Hash,
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
        self.5.hash(state);
        self.6.hash(state);
        self.7.hash(state);
    }
}

impl<A, B, C, D, E, F, G, H_, I> Hash for (A, B, C, D, E, F, G, H_, I)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
    H_: Hash,
    I: Hash,
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
        self.5.hash(state);
        self.6.hash(state);
        self.7.hash(state);
        self.8.hash(state);
    }
}

impl<A, B, C, D, E, F, G, H_, I, J> Hash for (A, B, C, D, E, F, G, H_, I, J)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
    H_: Hash,
    I: Hash,
    J: Hash,
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
        self.5.hash(state);
        self.6.hash(state);
        self.7.hash(state);
        self.8.hash(state);
        self.9.hash(state);
    }
}

impl<A, B, C, D, E, F, G, H_, I, J, K> Hash for (A, B, C, D, E, F, G, H_, I, J, K)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
    H_: Hash,
    I: Hash,
    J: Hash,
    K: Hash,
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
        self.5.hash(state);
        self.6.hash(state);
        self.7.hash(state);
        self.8.hash(state);
        self.9.hash(state);
        self.10.hash(state);
    }
}

impl<A, B, C, D, E, F, G, H_, I, J, K, L> Hash for (A, B, C, D, E, F, G, H_, I, J, K, L)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
    H_: Hash,
    I: Hash,
    J: Hash,
    K: Hash,
    L: Hash,
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
        self.5.hash(state);
        self.6.hash(state);
        self.7.hash(state);
        self.8.hash(state);
        self.9.hash(state);
        self.10.hash(state);
        self.11.hash(state);
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
        },
    );
}
