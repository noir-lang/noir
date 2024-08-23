mod block_root_rollup_inputs;

// Re-exports
use block_root_rollup_inputs::BlockRootRollupInputs;
use crate::abis::block_root_or_block_merge_public_inputs::BlockRootOrBlockMergePublicInputs;

mod tests {
    use crate::tests::block_root_rollup_inputs::default_block_root_rollup_inputs;
    use dep::types::hash::accumulate_sha256;

    #[test]
    fn check_block_hashes_empty_blocks() {
        let expected_out_hash = accumulate_sha256([1, 2]);

        let inputs = default_block_root_rollup_inputs();
        let outputs = inputs.block_root_rollup_circuit();

        // check out hash
        assert_eq(outputs.out_hash, expected_out_hash);
    }

    #[test]
    fn end_constants() {
        let inputs = default_block_root_rollup_inputs();
        let outputs = inputs.block_root_rollup_circuit();

        assert(
            outputs.previous_archive.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.constants.last_archive)
        );
        assert(
            outputs.start_global_variables.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.constants.global_variables)
        );
    }
}
