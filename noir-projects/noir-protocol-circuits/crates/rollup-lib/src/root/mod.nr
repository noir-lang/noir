mod root_rollup_inputs;
mod root_rollup_public_inputs;

// Re-exports
use root_rollup_inputs::RootRollupInputs;
use root_rollup_public_inputs::RootRollupPublicInputs;

mod tests {
    use crate::{tests::root_rollup_inputs::default_root_rollup_inputs};
    use dep::types::hash::accumulate_sha256;

    #[test]
    fn check_block_hashes_empty_blocks() {
        let expected_out_hash = accumulate_sha256([1, 2]);

        let inputs = default_root_rollup_inputs();
        let outputs = inputs.root_rollup_circuit();

        // check out hash
        assert_eq(outputs.out_hash, expected_out_hash);
    }

    #[test]
    fn end_state() {
        let inputs = default_root_rollup_inputs();
        let outputs = inputs.root_rollup_circuit();

        assert(
            outputs.previous_archive.eq(inputs.previous_rollup_data[0].block_root_or_block_merge_public_inputs.previous_archive)
        );

        assert(
            outputs.end_archive.eq(inputs.previous_rollup_data[1].block_root_or_block_merge_public_inputs.new_archive)
        );

        assert(
            outputs.end_timestamp.eq(
                inputs.previous_rollup_data[1].block_root_or_block_merge_public_inputs.end_global_variables.timestamp
            )
        );

        assert(
            outputs.end_block_hash.eq(inputs.previous_rollup_data[1].block_root_or_block_merge_public_inputs.end_block_hash)
        );
    }
}
