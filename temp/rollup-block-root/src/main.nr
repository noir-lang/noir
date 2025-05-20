use crate::lib::abis::{
    block_root_rollup_data::{BlockRootRollupBlobData, BlockRootRollupData, FinalBlobBatchingChallenges},
    previous_rollup_data::PreviousRollupData,
};
use crate::types::abis::sponge_blob::SpongeBlob;


use blob::{
    blob_batching::evaluate_blobs_and_batch,
    blob_batching_public_inputs::BlobAccumulatorPublicInputs,
};

pub(crate) mod lib;
pub(crate) mod types;
pub(crate) mod blob;


fn main(start_blob_accumulator: BlobAccumulatorPublicInputs, final_blob_challenges: FinalBlobBatchingChallenges, end_sponge_blob: SpongeBlob, blob_data: BlockRootRollupBlobData) -> pub BlobAccumulatorPublicInputs {
    evaluate_blobs_and_batch(
        blob_data.blobs_fields,
        blob_data.blob_commitments,
        end_sponge_blob,
        final_blob_challenges,
        start_blob_accumulator,
    )
}
