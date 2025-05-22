use crate::Empty;
use bigcurve::{
    BigCurveTrait,
    curves::bls12_381::{BLS12_381, BLS12_381Scalar},
    scalar_field::ScalarFieldTrait,
};
use bigint::{BigNum, BLS12_381_Fq as Q, BLS12_381_Fr as F};
use std::ops::{Add, Mul};

pub(crate) type BLSPoint = BLS12_381;

pub(crate) struct BatchingBlobCommitment {
    pub(crate) point: BLSPoint,
    pub(crate) compressed: [u8; 48], // TODO(MW): get Q::num_bytes from somewhere, rather than hardcoded 48
}

impl BatchingBlobCommitment {
    // The compressed form is a BLS12Fq field encoded as 2 BN fields.
    // The first is the first 31 bytes, and the second is the next 17 bytes:
    pub(crate) fn to_compressed_fields(self) -> [Field; 2] {
        [
            // field 0 is bytes 0..31
            field_from_bytes::<31>(array_splice(self.compressed, 0), true),
            // field 1 is bytes 31..48
            field_from_bytes::<17>(array_splice(self.compressed, 31), true),
        ]
    }
}

impl Empty for BatchingBlobCommitment {
    fn empty() -> Self {
        Self { point: BLSPoint::point_at_infinity(), compressed: [0; 48] }
    }
}

// TODO(MW): get_flags() and compress_to_bytes() will eventually be part of BigCurve.
// Ideally we would impl like:
// impl From<BLSPoint> for BlobCommitment {
//     }
// }
// impl BLSPoint {
//     fn get_flags(point: BLSPoint) -> ([bool; 3], Q) {
//     }
//     fn compress_to_bytes(self) -> [u8; 48] {
//     }
// }
// So, e.g., can just call point.compress_to_bytes() in future. For now:
pub(crate) fn compress_to_blob_commitment(point: BLSPoint) -> (BatchingBlobCommitment) {
    let (flags, x) = get_flags(point);
    // TODO(#13609): use built in .to_be_bytes() in bignum 0.7.0
    let mut compressed = x.to_be_bytes();
    let most_sig_bits = byte_to_bits_be(compressed[0]);
    // TODO(MW): May not need to assert this since Bignum may check it's 381 bits?
    for i in 0..3 {
        assert_eq(most_sig_bits[i], 0, "Invalid BLS12-381 x coordinate given to compress().");
    }
    compressed[0] = set_flags(flags, compressed[0]);
    BatchingBlobCommitment { point, compressed }
}

// TODO: Will be part of bigcurve's compression functionality
// Given a point in BLS12-381, returns [is_compressed, is_infinity, is_greater] and the point's x coordinate
fn get_flags(point: BLSPoint) -> ([bool; 3], Q) {
    let x = point.x;
    let y = point.y;
    let is_compressed = true;
    let is_infinity = point.is_infinity;
    let is_greater = y > -y;
    let flags = [is_compressed, is_infinity, is_greater];
    (flags, x)
}

// TODO: Will be part of bigcurve's compression functionality
// Given [is_compressed, is_infinity, is_greater] and the point's most significant byte, returns that
// byte with the flags set.
fn set_flags(flags: [bool; 3], byte: u8) -> u8 {
    // Flip some bits (NB: this is probably not efficient but just easier to log/visualise for now)
    let mut flip = 0;
    // flags = [is_compressed, is_infinity, is_greater]
    // if (is_compressed) { flip most sig bit in u8 => byte |= 2**7 => byte |= 2 << 6 }
    // if (is_infinity) { flip next most sig bit in u8 => byte |= 2**6 => byte |= 2 << 5 }
    // if (is_greater) { flip next most sig bit in u8 => byte |= 2**5 => byte |= 2 << 4 }
    for i in 0..3 {
        if flags[i] {
            flip += 2 << (6 - (i as u8));
        }
    }
    byte | flip
}

/**
* The outputs we care about from using the barycentric to evaluate blob i at z:
* - z_i = Challenge for one blob (=H(H(blob_i), C_i))
* - y_i = Evaluation for one blob (=p_i(z))
* - c_i = Commitment for one blob (=C_i)
*/
pub(crate) struct BlobAccumulationInputs {
    pub(crate) z_i: Field,
    pub(crate) y_i: F,
    pub(crate) c_i: BatchingBlobCommitment,
}

impl Empty for BlobAccumulationInputs {
    fn empty() -> Self {
        Self { z_i: 0, y_i: BigNum::zero(), c_i: BatchingBlobCommitment::empty() }
    }
}

/**
* Contains all fields required to construct a batched KZG proof of ALL blobs in the epoch.
* Instead of calling the point evaluation precompile on L1 for each blob, we create a multi-opening proof
* with the scheme below, and call it just once:
*   point_evaluation_precompile(b, z, y, C, Q)
* Where b (= kzg_to_versioned_hash(C)) and Q (= KZG proof) are computed outside the circuit. The other params are
* calculated here across the rollup circuits (until root, when .finalize() is called).
* Other notes:
*  - We use crate::blob_commitments_hash to validate the commitments injected here correspond to blobs published on L1.
*  - We use gamma as the challenge for multi opening, so it can be discarded once the rollup is complete.
*  - We already know that the elements in each blob correspond to validated data from the kernels from the use of
*    the blob_sponge and validating blob_sponge.squeeze() vs H(input_elements).
*  - We encompass all the blob elements in challenges (z_i) unique to each blob by using the above H(input_elements)
*    and the blob's commitment (c_i).
*
* TODO(MW): Compress F and Q values to reduce number of public inputs (BLOB_ACCUMULATOR_PUBLIC_INPUTS)
*/
pub(crate) struct BlobAccumulatorPublicInputs {
    pub(crate) blob_commitments_hash: Field, // Hash of Cs (to link to L1 blob hashes) (BN254Fr)
    pub(crate) z: Field, // Challenge at which the batched blob polynomial is evaluated (BN254Fr)
    pub(crate) y: F, // Current state of y's linear combination (sum_i {gamma^i * y_i}) where y_i is blob_i's evaluation y (BLS12Fr)
    pub(crate) c: BLSPoint, // Current state of C's linear combination (sum_i {gamma^i * C_i}) where C_i is blob_i's commitment C (BLS12 point: { x: BLS12Fq, y: BLS12Fq })
    pub(crate) gamma: Field, // Challenge for linear combination of each blob's y and C (BLS12Fr but represented here as BN254Fr, since it is hashed natively)
    pub(crate) gamma_pow: F, // gamma^i for current blob, used above (BLS12Fr)
}

impl BlobAccumulatorPublicInputs {
    pub(crate) fn accumulate(self, other: BlobAccumulationInputs, final_gamma: F) -> Self {
        // TODO(#13608): use a BLS12 based hash? Is using BN based safe - since the output is smaller is there a skew?
        let hashed_y_i = poseidon2_hash(other.y_i.get_limbs().map(|l| l as Field));
        Self {
            blob_commitments_hash: sha256_to_field(self
                .blob_commitments_hash
                .to_be_bytes::<32>()
                .concat(other.c_i.compressed)),
            z: poseidon2_hash([self.z, other.z_i]),
            y: self.y.add(other.y_i.mul(self.gamma_pow)),
            c: self.c.add(other.c_i.point.mul(BLS12_381Scalar::from_bignum(self.gamma_pow))),
            gamma: poseidon2_hash([self.gamma, hashed_y_i]),
            gamma_pow: self.gamma_pow.mul(final_gamma),
        }
    }

    pub(crate) fn init(first_output: BlobAccumulationInputs, final_gamma: F) -> Self {
        // TODO(#13608): use a BLS12 based hash? Is using BN based safe - since the output is smaller is there a skew?
        let hashed_y_0 = poseidon2_hash(first_output.y_i.get_limbs().map(|l| l as Field));
        Self {
            blob_commitments_hash: sha256_to_field(first_output.c_i.compressed),
            z: first_output.z_i,
            y: first_output.y_i, // y_acc_0 = gamma^0 * y_0 = y_0
            c: first_output.c_i.point, // c_acc_0 = gamma^0 * c_0 = c_0
            gamma: hashed_y_0,
            gamma_pow: final_gamma,
        }
    }
}

impl Empty for BlobAccumulatorPublicInputs {
    fn empty() -> Self {
        Self {
            blob_commitments_hash: 0,
            z: 0,
            y: F::zero(),
            c: BLSPoint::point_at_infinity(),
            gamma: 0,
            gamma_pow: F::zero(),
        }
    }
}

impl Eq for BlobAccumulatorPublicInputs {
    fn eq(self, other: Self) -> bool {
        (self.blob_commitments_hash.eq(other.blob_commitments_hash))
            & (self.z.eq(other.z))
            & (self.y.eq(other.y))
            & (self.c.eq(other.c))
            & (self.gamma.eq(other.gamma))
            & (self.gamma_pow.eq(other.gamma_pow))
    }
}

/**
* Final values z and gamma are injected into each block root circuit. We ensure they are correct by:
* - Checking equality in each block merge circuit and propagating up
* - Checking final z_acc == z in root circuit
* - Checking final gamma_acc == gamma in root circuit
*
*  - z = H(...H(H(z_0, z_1) z_2)..z_n)
*    - where z_i = H(H(fields of blob_i), C_i),
*    - used such that p_i(z) = y_i = Blob.evaluationY for all n blob polynomials p_i().
*  - gamma = H(H(...H(H(y_0, y_1) y_2)..y_n), z)
*    - used such that y = sum_i { gamma^i * y_i }, and C = sum_i { gamma^i * C_i }
*      for all blob evaluations y_i (see above) and commitments C_i.
*
* Iteratively calculated by BlobAccumulatorPublicInputs.accumulate() above. See also precomputeBatchedBlobChallenges() in ts.
*/
pub(crate) struct FinalBlobBatchingChallenges {
    pub(crate) z: Field,
    pub(crate) gamma: F,
}

/**
* - start_blob_accumulator: Accumulated opening proofs for all blobs before this block range.
* - end_blob_accumulator: Accumulated opening proofs for all blobs after adding this block range.
* - final_blob_challenges: Final values z and gamma, shared across the epoch.
*/

fn byte_to_bits_be(byte: u8) -> [u1; 8] {
    let mut mut_byte = byte;
    let mut bits: [u1; 8] = [0; 8];
    for i in 0..8 {
        bits[7 - i] = (mut_byte & 1) as u1;
        mut_byte >>= 1;
    }
    bits
}

pub(crate) fn field_from_bytes<let N: u32>(bytes: [u8; N], big_endian: bool) -> Field {
    assert(bytes.len() < 32, "field_from_bytes: N must be less than 32");
    let mut as_field = 0;
    let mut offset = 1;
    for i in 0..N {
        let mut index = i;
        if big_endian {
            index = N - i - 1;
        }
        as_field += (bytes[index] as Field) * offset;
        offset *= 256;
    }

    as_field
}

pub(crate) fn array_splice<T, let N: u32, let M: u32>(array: [T; N], offset: u32) -> [T; M]
where
    T: Empty,
{
    assert(M + offset <= N, "Subarray length larger than array length");
    let mut result: [T; M] = [T::empty(); M];
    for i in 0..M {
        result[i] = array[offset + i];
    }
    result
}

pub(crate) fn poseidon2_hash<let N: u32>(inputs: [Field; N]) -> Field {
    poseidon::poseidon2::Poseidon2::hash(inputs, N)
}

pub(crate) fn sha256_to_field<let N: u32>(bytes_to_hash: [u8; N]) -> Field {
    let sha256_hashed = sha256::digest(bytes_to_hash);
    let hash_in_a_field = field_from_bytes_32_trunc(sha256_hashed);

    hash_in_a_field
}

// Convert a 32 byte array to a field element by truncating the final byte
pub(crate) fn field_from_bytes_32_trunc(bytes32: [u8; 32]) -> Field {
    // Convert it to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..15 {
        // covers bytes 16..30 (31 is truncated and ignored)
        low = low + (bytes32[15 + 15 - i] as Field) * v;
        v = v * 256;
        // covers bytes 0..14
        high = high + (bytes32[14 - i] as Field) * v;
    }
    // covers byte 15
    low = low + (bytes32[15] as Field) * v;

    low + high * v
}
