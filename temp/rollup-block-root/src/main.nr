use bigint::BigNum;
use blob::{
    blob_batching::evaluate_blob_for_batching,
    blob_batching_public_inputs::{
        BlobAccumulationInputs, BlobAccumulatorPublicInputs, BLSPoint, compress_to_blob_commitment,
        FinalBlobBatchingChallenges,
    },
};

use crate::types::traits::is_empty;

pub(crate) mod types;
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

    // for-loop is required
    for i in 0..1 {
        let c_i = compress_to_blob_commitment(kzg_commitments_point);
        let (z_i, y_i) = evaluate_blob_for_batching(
            blobs_as_fields,
            c_i,
            hashed_blobs_fields,
            final_blob_challenges.z,
        );
        if !(y_i.is_zero()) & !(blobs_as_fields[0] == 0) {
            // Only accumulate if the blob is non empty
            if is_empty(start_blob_accumulator) {
                start_blob_accumulator = BlobAccumulatorPublicInputs::init(
                    BlobAccumulationInputs { z_i, y_i, c_i },
                    final_blob_challenges.gamma,
                );
            } else {
                start_blob_accumulator = start_blob_accumulator.accumulate(
                    BlobAccumulationInputs { z_i, y_i, c_i },
                    final_blob_challenges.gamma,
                );
            }
        }
    }
    start_blob_accumulator
}

use crate::types::poseidon2::Poseidon2Sponge;

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
