use crate::blob::{
    blob::{barycentric_evaluate_blob_at_z, BlobCommitment, compute_challenge, convert_blob_fields},
    blob_batching_public_inputs::{
        BatchingBlobCommitment, BlobAccumulationInputs, BlobAccumulatorPublicInputs, BLSPoint,
        compress_to_blob_commitment, FinalBlobBatchingChallenges,
    },
};

use bigint::{BLS12_381_Fr as F};

pub(crate) mod blob;

global FIELDS_PER_BLOB: u32 = 4096;

fn main(
    mut start_blob_accumulator: BlobAccumulatorPublicInputs,
    final_blob_challenges: FinalBlobBatchingChallenges,
    mut end_sponge_blob: SpongeBlob,
    blobs_as_fields: [Field; FIELDS_PER_BLOB],
    kzg_commitments_point: BLSPoint,
) -> pub BlobAccumulatorPublicInputs {
    let hashed_blobs_fields = end_sponge_blob.squeeze();

    let c_i = compress_to_blob_commitment(kzg_commitments_point);
    let (z_i, y_i) = evaluate_blob_for_batching(
        blobs_as_fields,
        c_i,
        hashed_blobs_fields,
        final_blob_challenges.z,
    );

    if is_empty(start_blob_accumulator) {
        BlobAccumulatorPublicInputs::init(
            BlobAccumulationInputs { z_i, y_i, c_i },
            final_blob_challenges.gamma,
        )
    } else {
        start_blob_accumulator.accumulate(
            BlobAccumulationInputs { z_i, y_i, c_i },
            final_blob_challenges.gamma,
        )
    }

}

pub(crate) struct SpongeBlob {
    pub(crate) sponge: Poseidon2Sponge,
    pub(crate) fields: u32,
    pub(crate) expected_fields: u32, // The hinted number of tx effects this will absorb
}

impl SpongeBlob {
    // Finalise the sponge and output poseidon2 hash of all fields absorbed
    pub(crate) fn squeeze(&mut self) -> Field {
        // If the blob sponge is not 'full', we append 1 to match Poseidon2::hash_internal()
        // NB: There is currently no use case in which we don't 'fill' a blob sponge, but adding for completeness
        if self.fields != self.expected_fields {
            self.sponge.absorb(1);
        }
        self.sponge.squeeze()
    }
}

// Evaluates a single blob:
// - Evaluates the blob at shared challenge z and returns result y_i
// - Calculates this blob's challenge z_i (= H(H(blob_i), C_i))
fn evaluate_blob_for_batching(
    blob_as_fields: [Field; 4096],
    kzg_commitment: BatchingBlobCommitment,
    hashed_blobs_fields: Field,
    challenge_z: Field,
) -> (Field, F) {
    let challenge_z_as_bignum = F::from(challenge_z);
    let blob = convert_blob_fields(blob_as_fields);

    let y_i: F = barycentric_evaluate_blob_at_z(challenge_z_as_bignum, blob);
    let z_i: Field = compute_challenge(
        hashed_blobs_fields,
        // TODO(MW): At some point BatchingBlobCommitment will replace BlobCommitment and we won't need this silly conversion
        BlobCommitment { inner: kzg_commitment.to_compressed_fields() },
    );

    (z_i, y_i)
}

global TWO_POW_64: Field = 18446744073709551616;

// NB: This is a clone of noir/noir-repo/noir_stdlib/src/hash/poseidon2.nr
// It exists as we sometimes need to perform custom absorption, but the stdlib version
// has a private absorb() method (it's also designed to just be a hasher)
// Can be removed when standalone noir poseidon lib exists: See noir#6679

comptime global RATE: u32 = 3;

pub(crate) struct Poseidon2Sponge {
    pub(crate) cache: [Field; 3],
    pub(crate) state: [Field; 4],
    pub(crate) cache_size: u32,
    pub(crate) squeeze_mode: bool, // 0 => absorb, 1 => squeeze
}

impl Poseidon2Sponge {
    #[no_predicates]
    pub(crate) fn hash<let N: u32>(input: [Field; N], message_size: u32) -> Field {
        Poseidon2Sponge::hash_internal(input, message_size, message_size != N)
    }

    pub(crate) fn new(iv: Field) -> Poseidon2Sponge {
        let mut result =
            Poseidon2Sponge { cache: [0; 3], state: [0; 4], cache_size: 0, squeeze_mode: false };
        result.state[RATE] = iv;
        result
    }

    fn perform_duplex(&mut self) {
        // add the cache into sponge state
        for i in 0..RATE {
            // We effectively zero-pad the cache by only adding to the state
            // cache that is less than the specified `cache_size`
            if i < self.cache_size {
                self.state[i] += self.cache[i];
            }
        }
        self.state = std::hash::poseidon2_permutation(self.state, 4);
    }

    pub(crate) fn absorb(&mut self, input: Field) {
        assert(!self.squeeze_mode);
        if self.cache_size == RATE {
            // If we're absorbing, and the cache is full, apply the sponge permutation to compress the cache
            self.perform_duplex();
            self.cache[0] = input;
            self.cache_size = 1;
        } else {
            // If we're absorbing, and the cache is not full, add the input into the cache
            self.cache[self.cache_size] = input;
            self.cache_size += 1;
        }
    }

    pub(crate) fn squeeze(&mut self) -> Field {
        assert(!self.squeeze_mode);
        // If we're in absorb mode, apply sponge permutation to compress the cache.
        self.perform_duplex();
        self.squeeze_mode = true;

        // Pop one item off the top of the permutation and return it.
        self.state[0]
    }

    fn hash_internal<let N: u32>(
        input: [Field; N],
        in_len: u32,
        is_variable_length: bool,
    ) -> Field {
        let iv: Field = (in_len as Field) * TWO_POW_64;
        let mut sponge = Poseidon2Sponge::new(iv);
        for i in 0..input.len() {
            if i < in_len {
                sponge.absorb(input[i]);
            }
        }

        // In the case where the hash preimage is variable-length, we append `1` to the end of the input, to distinguish
        // from fixed-length hashes. (the combination of this additional field element + the hash IV ensures
        // fixed-length and variable-length hashes do not collide)
        if is_variable_length {
            sponge.absorb(1);
        }
        sponge.squeeze()
    }
}

pub(crate) trait Empty {
    fn empty() -> Self;
}

impl Empty for u8 {
    fn empty() -> Self {
        0
    }
}

pub(crate) fn is_empty<T>(item: T) -> bool
where
    T: Empty + Eq,
{
    item.eq(T::empty())
}
