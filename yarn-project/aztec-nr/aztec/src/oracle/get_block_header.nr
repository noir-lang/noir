use dep::std::merkle::compute_merkle_root;
use crate::{
    abi::BlockHeader,
    constants_gen::{
        BLOCK_HEADER_LENGTH,
        BLOCKS_TREE_HEIGHT,
    },
    context::PrivateContext,
    oracle::get_membership_witness::{
        get_membership_witness,
        MembershipWitness,
    },
};

#[oracle(getBlockHeader)]
fn get_block_header_oracle(_block_number: Field) -> [Field; BLOCK_HEADER_LENGTH] {}

unconstrained pub fn get_block_header_internal(block_number: Field) -> BlockHeader {
    let block_header = get_block_header_oracle(block_number);
    BlockHeader::deserialize(block_header)
}

pub fn get_block_header(block_number: Field, context: PrivateContext) -> BlockHeader {
    // 1) Get block header of a given block from oracle
    let block_header = get_block_header_internal(block_number);

    // 2) Compute the block hash from the block header
    let block_hash = block_header.block_hash();

    // 3) Get the membership wintess of the block in the blocks tree
    let blocks_tree_id = 5; // TODO(#3443)
    let witness: MembershipWitness<BLOCKS_TREE_HEIGHT, BLOCKS_TREE_HEIGHT + 1> = get_membership_witness(block_number, blocks_tree_id, block_hash);

    // 4) Check that the block is in the blocks tree (i.e. the witness is valid)
    assert(context.block_header.blocks_tree_root == compute_merkle_root(block_hash, witness.index, witness.path), "Proving membership of a block in blocks tree failed");

    // 5) Return the block header
    block_header
}
