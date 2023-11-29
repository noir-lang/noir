use dep::std::merkle::compute_merkle_root;
use crate::{
    abi::HistoricBlockData,
    constants_gen::{
        HISTORIC_BLOCK_DATA_LENGTH,
        HISTORIC_BLOCKS_TREE_HEIGHT,
    },
    context::PrivateContext,
    oracle::get_membership_witness::{
        get_membership_witness,
        MembershipWitness,
    },
};

#[oracle(getBlockData)]
fn get_block_data_oracle(_block_number: Field) -> [Field; HISTORIC_BLOCK_DATA_LENGTH] {}

unconstrained pub fn get_block_data_internal(block_number: Field) -> HistoricBlockData {
    let block_data = get_block_data_oracle(block_number);
    HistoricBlockData::deserialize(block_data)
}

pub fn get_block_data(block_number: Field, context: PrivateContext) -> HistoricBlockData {
    // 1) Get historic block data from oracle at the given block
    let block_data = get_block_data_internal(block_number);

    // 2) Compute the block hash from the block data
    let block_hash = block_data.block_hash();

    // 3) Get the membership wintess of the block in the blocks tree
    let blocks_tree_id = 5; // TODO(#3443)
    let witness: MembershipWitness<HISTORIC_BLOCKS_TREE_HEIGHT, HISTORIC_BLOCKS_TREE_HEIGHT + 1> = get_membership_witness(block_number, blocks_tree_id, block_hash);

    // 4) Check that the block is in the blocks tree (i.e. the witness is valid)
    assert(context.block_data.blocks_tree_root == compute_merkle_root(block_hash, witness.index, witness.path), "Proving membership of a block in blocks tree failed");

    // 5) Return the block data
    block_data
}
