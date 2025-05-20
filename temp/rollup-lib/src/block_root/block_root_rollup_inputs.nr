use crate::abis::{
    block_root_rollup_data::{BlockRootRollupBlobData, BlockRootRollupData},
    previous_rollup_data::PreviousRollupData,
};

pub struct BlockRootRollupInputs {
    previous_rollup_data: [PreviousRollupData; 2],
    data: BlockRootRollupData,
    blob_data: BlockRootRollupBlobData,
}
