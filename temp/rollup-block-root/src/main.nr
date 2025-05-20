use crate::lib::abis::{
    block_root_rollup_data::{BlockRootRollupBlobData, BlockRootRollupData},
    previous_rollup_data::PreviousRollupData,
};


use blob::{
    blob_batching::evaluate_blobs_and_batch,
    blob_batching_public_inputs::BlobAccumulatorPublicInputs,
};

pub(crate) mod lib;
pub(crate) mod types;
pub(crate) mod blob;

fn main(inputs: BlockRootRollupInputs) -> pub BlobAccumulatorPublicInputs {
    evaluate_blobs_and_batch(
        inputs.blob_data.blobs_fields,
        inputs.blob_data.blob_commitments,
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_sponge_blob,
        inputs.data.final_blob_challenges,
        inputs.data.start_blob_accumulator,
    )
}

pub(crate)struct BlockRootRollupInputs {
    previous_rollup_data: [PreviousRollupData; 2],
    data: BlockRootRollupData,
    blob_data: BlockRootRollupBlobData,
}
