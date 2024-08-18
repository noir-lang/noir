use dep::protocol_types::merkle_tree::root::root_from_sibling_path;
use dep::protocol_types::{constants::HEADER_LENGTH, header::Header};

use crate::{context::PrivateContext, oracle::get_membership_witness::get_archive_membership_witness};

use crate::test::helpers::test_environment::TestEnvironment;

#[oracle(getHeader)]
unconstrained fn get_header_at_oracle(_block_number: u32) -> [Field; HEADER_LENGTH] {}

unconstrained pub fn get_header_at_internal(block_number: u32) -> Header {
    let header = get_header_at_oracle(block_number);
    Header::deserialize(header)
}

pub fn get_header_at(block_number: u32, context: PrivateContext) -> Header {
    let header = context.historical_header;
    let current_block_number = header.global_variables.block_number as u32;

    if (block_number == current_block_number) {
        // If the block number we want to prove against is the same as the block number in the historical header we
        // skip the inclusion proofs and just return the historical header from context.
        header
    } else {
        // 1) Get block number corresponding to the last_archive root in the header
        // Note: We subtract 1 because the last_archive root is the root of the archive after applying the previous block
        let last_archive_block_number = current_block_number - 1;

        // 2) Check that the last archive block number is more than or equal to the block number we want to prove against
        // We could not perform the proof otherwise because the last archive root from the header would not "contain"
        // the header we want to prove against
        assert(
            last_archive_block_number >= block_number, "Last archive block number is smaller than the block number we want to prove against"
        );

        // 3) Get the header hint of a given block from an oracle
        let historical = get_header_at_internal(block_number);

        // 4) We make sure that the header hint we received from the oracle exists in the state tree and is the actual header
        // at the desired block number
        constrain_get_header_at_internal(
            historical,
            block_number,
            last_archive_block_number,
            header.last_archive.root
        );

        // 5) Return the block header
        historical
    }
}

fn constrain_get_header_at_internal(
    header_hint: Header,
    block_number: u32,
    last_archive_block_number: u32,
    last_archive_root: Field
) {
    // 1) Compute the block hash from the block header
    let block_hash = header_hint.hash();

    // 2) Get the membership witness of the block in the archive tree
    let witness = get_archive_membership_witness(last_archive_block_number, block_hash);

    // 3) Check that the block is in the archive (i.e. the witness is valid)
    assert(
        last_archive_root == root_from_sibling_path(block_hash, witness.index, witness.path), "Proving membership of a block in archive failed"
    );

    // 4) Check that the header hint has the same block number as the block number we are looking for, ensuring we are actually grabbing the header we specify
    assert(
        header_hint.global_variables.block_number as u32 == block_number, "Block number provided is not the same as the block number from the header hint"
    );
}

#[test(should_fail_with = "Block number provided is not the same as the block number from the header hint")]
fn fetching_a_valid_but_different_header_should_fail() {
    let mut env = TestEnvironment::new();

    env.advance_block_to(3);

    // We get our current header for the last archive values.
    let current_header = env.private().historical_header;

    let target_block_number = 2;
    let bad_header = get_header_at_internal(target_block_number - 1);

    // We pass in a different block number than the header received
    constrain_get_header_at_internal(
        bad_header,
        2,
        current_header.global_variables.block_number as u32 - 1,
        current_header.last_archive.root
    );
}
