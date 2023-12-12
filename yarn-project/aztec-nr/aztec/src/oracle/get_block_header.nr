use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::constants::{
    BLOCK_HEADER_LENGTH,
    ARCHIVE_HEIGHT,
};

use crate::{
    abi::BlockHeader,
    context::PrivateContext,
    oracle::get_membership_witness::{
        get_membership_witness,
        MembershipWitness,
    },
};

// TODO(#3564) - Nuke this oracle and Inject the number directly to context
#[oracle(getNullifierRootBlockNumber)]
fn get_nullifier_root_block_number_oracle(_nullifier_tree_root: Field) -> Field {}

unconstrained pub fn get_nullifier_root_block_number(nullifier_tree_root: Field) -> u32 {
    get_nullifier_root_block_number_oracle(nullifier_tree_root) as u32
}

#[oracle(getBlockHeader)]
fn get_block_header_oracle(_block_number: u32) -> [Field; BLOCK_HEADER_LENGTH] {}

unconstrained pub fn get_block_header_internal(block_number: u32) -> BlockHeader {
    let block_header = get_block_header_oracle(block_number);
    BlockHeader::deserialize(block_header)
}

pub fn get_block_header(block_number: u32, context: PrivateContext) -> BlockHeader {
    // 1) Get block number corresponding to block header inside context
    // Using nullifier tree root to get the block header block number because that changes in every block (every tx emits a nullifier).
    let block_header_block_number = get_nullifier_root_block_number(context.block_header.nullifier_tree_root);

    // 2) Check that the block header block number is more than or equal to the block number we want to prove against
    // We could not perform the proof otherwise because the archive root from the header would not "contain" the block we want to prove against
    assert(block_header_block_number >= block_number, "Block header block number is smaller than the block number we want to prove against");

    // 3) Get block header of a given block from oracle
    let block_header = get_block_header_internal(block_number);

    // 4) Compute the block hash from the block header
    let block_hash = block_header.block_hash();

    // 5) Get the membership witness of the block in the archive
    let archive_id = 5; // TODO(#3443)
    let witness: MembershipWitness<ARCHIVE_HEIGHT, ARCHIVE_HEIGHT + 1> = get_membership_witness(block_header_block_number, archive_id, block_hash);

    // 6) Check that the block is in the archive (i.e. the witness is valid)
    assert(context.block_header.archive_root == compute_merkle_root(block_hash, witness.index, witness.path), "Proving membership of a block in archive failed");

    // 7) Return the block header
    block_header
}
