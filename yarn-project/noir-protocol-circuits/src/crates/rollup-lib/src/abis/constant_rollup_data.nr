use crate::abis::global_variables::GlobalVariables;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;

struct ConstantRollupData {
    // The very latest roots as at the very beginning of the entire rollup:
    start_historic_blocks_tree_roots_snapshot : AppendOnlyTreeSnapshot,

    // TODO(Sean): Some members of this struct tbd
    private_kernel_vk_tree_root : Field,
    public_kernel_vk_tree_root : Field,
    base_rollup_vk_hash : Field,
    merge_rollup_vk_hash : Field,

    global_variables : GlobalVariables,
}