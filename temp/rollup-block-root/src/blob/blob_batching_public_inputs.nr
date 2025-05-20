use crate::types::{
    constants::{BLOB_ACCUMULATOR_PUBLIC_INPUTS, BLOCK_BLOB_PUBLIC_INPUTS, BLS12_FR_LIMBS},
    hash::{poseidon2_hash, sha256_to_field},
    traits::{Deserialize, Empty, Serialize},
    utils::{arrays::array_splice, field::{byte_to_bits_be, field_from_bytes}, reader::Reader},
};
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

    // Helper fn largely used in testing to simplify fixtures
    pub(crate) fn from_limbs(x_limbs: [u128; 4], y_limbs: [u128; 4]) -> Self {
        let x = Q::from_limbs(x_limbs);
        let y = Q::from_limbs(y_limbs);
        let point = BLSPoint { x, y, is_infinity: (x.is_zero()) & (y.is_zero()) };
        compress_to_blob_commitment(point)
    }

    // The commitment for an empty blob is (0, 0), which is compressed to [192, 0, ..., 0]
    // NOT [0, 0, ..., 0] due to the is_compressed flag:
    pub(crate) fn empty_blob() -> Self {
        Self {
            point: BLSPoint { x: Q::zero(), y: Q::zero(), is_infinity: true },
            compressed: [
                192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        }
    }
}

impl Eq for BatchingBlobCommitment {
    fn eq(self, other: Self) -> bool {
        (self.point.eq(other.point)) & (self.compressed.eq(other.compressed))
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
    /**
    * LHS Accumulator: Current state of param accumulation from blob 0 to i-1
    * RHS Accumulator: Outputs from evaluation of blob i
    *
    * NB: blob_commitments_hash is written as v below
    *
    * Each accumulation:
    * - v_acc := sha256(v_acc, C_i)
    * - z_acc := poseidon2(z_acc, z_i)
    * - y_acc := y_acc + (gamma^i * y_i)
    * - c_acc := c_acc + (gamma^i * c_i)
    * - gamma_acc := poseidon2(gamma_acc, poseidon2(y_i.limbs))
    * - gamma^(i + 1) = gamma^i * final_gamma // denoted gamma_pow
    *
    * Final accumulated values (from last blob of last block of epoch):
    * - v := v_acc (hash of all commitments (C_i s) to be checked on L1)
    * - z := z_acc (final challenge, at which all blobs are evaluated)
    * - y := y_acc (final opening to be checked on L1)
    * - c := c_acc (final commitment to be checked on L1)
    * - gamma := poseidon2(gamma_acc, z) (challenge for linear combination of y and C, above)
    *
    * Final values z and gamma are injected into each block root circuit. We ensure they are correct by:
    * - Checking equality in each block merge circuit and propagating up
    * - Checking final z_acc == z in root circuit
    * - Checking final gamma_acc == gamma in root circuit
    *
    */
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

    // Completes the final checks for the last accumulator state. This is called in the root rollup and
    // exists here to avoid importing bignum's to_field in the rollup_lib.
    pub(crate) fn finalize_and_validate(
        self,
        injected: FinalBlobBatchingChallenges,
    ) -> FinalBlobAccumulatorPublicInputs {
        assert(self.z == injected.z, "Final blob challenge z mismatch.");
        let gamma = poseidon2_hash([self.gamma, self.z]);
        assert(
            gamma == bigint::bignum::to_field(injected.gamma),
            "Final blob challenge gamma mismatch.",
        );
        // TODO(MW): Refactor BlobBatchingCommitment once BlobCommitment can be safely deleted
        let c = compress_to_blob_commitment(self.c).to_compressed_fields();
        FinalBlobAccumulatorPublicInputs {
            blob_commitments_hash: self.blob_commitments_hash,
            z: self.z,
            y: self.y,
            c,
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

impl Serialize<BLOB_ACCUMULATOR_PUBLIC_INPUTS> for BlobAccumulatorPublicInputs {
    fn serialize(self) -> [Field; BLOB_ACCUMULATOR_PUBLIC_INPUTS] {
        let mut fields: BoundedVec<Field, BLOB_ACCUMULATOR_PUBLIC_INPUTS> = BoundedVec::new();
        fields.push(self.blob_commitments_hash);
        fields.push(self.z);
        fields.extend_from_array(self.y.get_limbs().map(|l| l as Field));
        fields.extend_from_array(self.c.x.get_limbs().map(|l| l as Field));
        fields.extend_from_array(self.c.y.get_limbs().map(|l| l as Field));
        fields.push(self.gamma);
        fields.extend_from_array(self.gamma_pow.get_limbs().map(|l| l as Field));
        fields.storage()
    }
}

impl Deserialize<BLOB_ACCUMULATOR_PUBLIC_INPUTS> for BlobAccumulatorPublicInputs {
    fn deserialize(fields: [Field; BLOB_ACCUMULATOR_PUBLIC_INPUTS]) -> Self {
        let mut reader = Reader::new(fields);
        let mut item = Self {
            blob_commitments_hash: reader.read(),
            z: reader.read(),
            y: F::from_limbs(reader.read_array().map(|e| e as u128)),
            c: BLSPoint {
                x: Q::from_limbs(reader.read_array().map(|e| e as u128)),
                y: Q::from_limbs(reader.read_array().map(|e| e as u128)),
                is_infinity: false,
            },
            gamma: reader.read(),
            gamma_pow: F::from_limbs(reader.read_array().map(|e| e as u128)),
        };
        if (item.c.x.is_zero()) & (item.c.y.is_zero()) {
            item.c.is_infinity = true;
        }
        item
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

impl Empty for FinalBlobBatchingChallenges {
    fn empty() -> Self {
        Self { z: 0, gamma: F::zero() }
    }
}

impl Eq for FinalBlobBatchingChallenges {
    fn eq(self, other: Self) -> bool {
        (self.z.eq(other.z)) & (self.gamma.eq(other.gamma))
    }
}

impl Serialize<BLS12_FR_LIMBS + 1> for FinalBlobBatchingChallenges {
    fn serialize(self) -> [Field; BLS12_FR_LIMBS + 1] {
        let mut fields: BoundedVec<Field, BLS12_FR_LIMBS + 1> = BoundedVec::new();
        fields.push(self.z);
        fields.extend_from_array(self.gamma.get_limbs().map(|l| l as Field));
        fields.storage()
    }
}

impl Deserialize<BLS12_FR_LIMBS + 1> for FinalBlobBatchingChallenges {
    fn deserialize(fields: [Field; BLS12_FR_LIMBS + 1]) -> Self {
        let mut reader = Reader::new(fields);
        let mut item =
            Self { z: reader.read(), gamma: F::from_limbs(reader.read_array().map(|e| e as u128)) };
        item
    }
}

/**
* Final values of a batched blob. Output in the root rollup circuit and validated on L1.
*
* - blob_commitments_hash = sha256( ...sha256(sha256(C_0), C_1) ... C_n)
* - z = poseidon2( ...poseidon2(poseidon2(z_0, z_1), z_2) ... z_n)
* - y = y_0 + gamma * y_1 + gamma^2 * y_2 + ... + gamma^n * y_n
* - c = C_0 + gamma * C_1 + gamma^2 * C_2 + ... + gamma^n * C_n
*
* Gamma has served its purpose in the 'random' linear combination and is not required as a public input.
*/
pub(crate) struct FinalBlobAccumulatorPublicInputs {
    pub(crate) blob_commitments_hash: Field, // Hash of Cs (to link to L1 blob hashes) (BN254Fr)
    pub(crate) z: Field, // Challenge at which the batched blob polynomial is evaluated (BN254Fr)
    pub(crate) y: F, // Batched blob's evaluation y (BLS12Fr)
    pub(crate) c: [Field; 2], // Batched blob's commitment C (BLS12 point: { x: BLS12Fq, y: BLS12Fq }) compressed to 48 bytes, stored in 2 fields of 31 and 17 bytes
}

/**
* - start_blob_accumulator: Accumulated opening proofs for all blobs before this block range.
* - end_blob_accumulator: Accumulated opening proofs for all blobs after adding this block range.
* - final_blob_challenges: Final values z and gamma, shared across the epoch.
*/
pub(crate) struct BlockBlobPublicInputs {
    pub(crate) start_blob_accumulator: BlobAccumulatorPublicInputs,
    pub(crate) end_blob_accumulator: BlobAccumulatorPublicInputs,
    pub(crate) final_blob_challenges: FinalBlobBatchingChallenges,
}

impl Empty for BlockBlobPublicInputs {
    fn empty() -> Self {
        Self {
            start_blob_accumulator: BlobAccumulatorPublicInputs::empty(),
            end_blob_accumulator: BlobAccumulatorPublicInputs::empty(),
            final_blob_challenges: FinalBlobBatchingChallenges::empty(),
        }
    }
}

impl Eq for BlockBlobPublicInputs {
    fn eq(self, other: Self) -> bool {
        (self.start_blob_accumulator.eq(other.start_blob_accumulator))
            & (self.end_blob_accumulator.eq(other.end_blob_accumulator))
            & (self.final_blob_challenges.eq(other.final_blob_challenges))
    }
}

impl Serialize<BLOCK_BLOB_PUBLIC_INPUTS> for BlockBlobPublicInputs {
    fn serialize(self) -> [Field; BLOCK_BLOB_PUBLIC_INPUTS] {
        let mut fields: BoundedVec<Field, BLOCK_BLOB_PUBLIC_INPUTS> = BoundedVec::new();
        fields.extend_from_array(self.start_blob_accumulator.serialize());
        fields.extend_from_array(self.end_blob_accumulator.serialize());
        fields.extend_from_array(self.final_blob_challenges.serialize());
        fields.storage()
    }
}

impl Deserialize<BLOCK_BLOB_PUBLIC_INPUTS> for BlockBlobPublicInputs {
    fn deserialize(fields: [Field; BLOCK_BLOB_PUBLIC_INPUTS]) -> Self {
        let mut reader = Reader::new(fields);
        let mut item = Self {
            start_blob_accumulator: reader.read_struct(BlobAccumulatorPublicInputs::deserialize),
            end_blob_accumulator: reader.read_struct(BlobAccumulatorPublicInputs::deserialize),
            final_blob_challenges: reader.read_struct(FinalBlobBatchingChallenges::deserialize),
        };
        item
    }
}

mod tests {
    use crate::blob::blob_batching_public_inputs::{
        BatchingBlobCommitment, BlockBlobPublicInputs, BLSPoint, compress_to_blob_commitment,
        get_flags,
    };
    use crate::types::traits::{Deserialize, Empty, Serialize};
    use bigcurve::{
        BigCurveTrait,
        CurveParamsTrait,
        curves::bls12_381::{BLS12_381 as Point, BLS12_381_Params},
    };
    use bigint::{BigNum, BLS12_381_Fq as Q};
    use std::ops::{Add, Mul};

    #[test]
    unconstrained fn point_compression() {
        let point = Point::offset_generator();
        let (flags, x) = get_flags(point);
        // is_compressed = true
        assert_eq(flags[0], true);
        // is_infinity = false
        assert_eq(flags[1], false);
        // is_greater = false (point.y < -point.y for G)
        assert_eq(flags[2], false);
        // Decompress back to the same point:
        let mut bytes = compress_to_blob_commitment(point).compressed;
        // Same as &= 0b0001_1111 - clear first three bits of our flags
        bytes[0] &= 31;
        let reconstructed_x = Q::from_be_bytes(bytes);
        assert_eq(reconstructed_x, x);
        let (a, b) = (BLS12_381_Params::a(), BLS12_381_Params::b());
        // y^2 = x^3 + ax + b
        let reconstructed_y_squared =
            reconstructed_x.__pow(Q::from(3)).add(a.mul(reconstructed_x)).add(b);
        let mut reconstructed_y = reconstructed_y_squared.__tonelli_shanks_sqrt().unwrap();
        // If the sqrt returned is the 'greater' one, negate it (since here is_greater = false):
        reconstructed_y = if reconstructed_y > -reconstructed_y {
            -reconstructed_y
        } else {
            reconstructed_y
        };
        assert_eq(reconstructed_y, point.y);
    }

    #[test]
    unconstrained fn empty_blob_commitment() {
        let expected = compress_to_blob_commitment(BLSPoint::point_at_infinity());
        assert_eq(expected, BatchingBlobCommitment::empty_blob());
    }

    #[test]
    unconstrained fn serialization_of_empty() {
        let item = BlockBlobPublicInputs::empty();
        let serialized = item.serialize();
        let deserialized = BlockBlobPublicInputs::deserialize(serialized);
        assert(item.eq(deserialized));
    }
}
