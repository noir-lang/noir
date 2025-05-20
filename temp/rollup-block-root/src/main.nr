use crate::lib::abis::{
    block_root_rollup_data::BlockRootRollupBlobData,
};

use blob::{
    blob::{
        barycentric_evaluate_blob_at_z, check_block_blob_sponge, compute_challenge,
        convert_blob_fields,
    },
    blob_batching::{evaluate_blobs_and_batch, evaluate_blob_for_batching},
    blob_batching_public_inputs::{
        BatchingBlobCommitment, BlobAccumulationInputs, BlobAccumulatorPublicInputs, BLSPoint,
        compress_to_blob_commitment, FinalBlobBatchingChallenges,
    },
    blob_public_inputs::BlobCommitment,
};
use bigint::BigNum;


use crate::types::{
    abis::sponge_blob::SpongeBlob,
    constants::{BLOBS_PER_BLOCK, FIELDS_PER_BLOB},
    traits::is_empty,
    utils::arrays::array_splice,
};

pub(crate) mod lib;
pub(crate) mod types;
pub(crate) mod blob;


fn main(start_blob_accumulator: BlobAccumulatorPublicInputs, final_blob_challenges: FinalBlobBatchingChallenges, end_sponge_blob: SpongeBlob, blob_data: BlockRootRollupBlobData) -> pub BlobAccumulatorPublicInputs {
    let blobs_as_fields = blob_data.blobs_fields;
    let kzg_commitments_points = blob_data.blob_commitments;
    let mut sponge_blob = end_sponge_blob;

     // See components.nr out_sponge definition as to why we copy here:
    let mut end_accumulator = start_blob_accumulator;
    // Note that with multiple blobs per block, each blob uses the same hashed_blobs_fields in:
    // z_i = H(hashed_blobs_fields, kzg_commitment[0], kzg_commitment[1])
    // This is ok, because each commitment is unique to the blob, and we need hashed_blobs_fields to encompass
    // all fields in the blob, which it does.
    let hashed_blobs_fields = check_block_blob_sponge(blobs_as_fields, sponge_blob);
    for i in 0..BLOBS_PER_BLOCK {
        let single_blob_fields = array_splice(blobs_as_fields, i * FIELDS_PER_BLOB);
        let c_i = compress_to_blob_commitment(kzg_commitments_points[i]);
        let (z_i, y_i) = evaluate_blob_for_batching(
            single_blob_fields,
            c_i,
            hashed_blobs_fields,
            final_blob_challenges.z,
        );
        if !(y_i.is_zero()) & !(single_blob_fields[0] == 0) {
            // Only accumulate if the blob is non empty
            if is_empty(end_accumulator) {
                // TODO(MW): move this if is_empty() statement to .accumulate()?
                // Init only if accumulator is empty:
                //  - This will be checked in root, where the left input's start acc will be constrained to be zero
                //  - No other accs can be zero since each block_merge checks left's end acc == right's start acc
                end_accumulator = BlobAccumulatorPublicInputs::init(
                    BlobAccumulationInputs { z_i, y_i, c_i },
                    final_blob_challenges.gamma,
                );
            } else {
                end_accumulator = end_accumulator.accumulate(
                    BlobAccumulationInputs { z_i, y_i, c_i },
                    final_blob_challenges.gamma,
                );
            }
        }
    }
    end_accumulator
}
