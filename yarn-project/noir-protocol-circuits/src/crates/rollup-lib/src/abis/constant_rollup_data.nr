use crate::abis::global_variables::GlobalVariables;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use dep::std::ops::Eq;

struct ConstantRollupData {
    // The very latest roots as at the very beginning of the entire rollup:
    archive_snapshot : AppendOnlyTreeSnapshot,

    // TODO(Sean): Some members of this struct tbd
    private_kernel_vk_tree_root : Field,
    public_kernel_vk_tree_root : Field,
    base_rollup_vk_hash : Field,
    merge_rollup_vk_hash : Field,

    global_variables : GlobalVariables,
}

impl Eq for ConstantRollupData {
    fn eq(self, other : ConstantRollupData) -> bool {
        self.archive_snapshot.eq(other.archive_snapshot) &
        self.global_variables.eq(other.global_variables) &
        (self.private_kernel_vk_tree_root == other.private_kernel_vk_tree_root) &
        (self.public_kernel_vk_tree_root == other.public_kernel_vk_tree_root) &
        (self.base_rollup_vk_hash == other.base_rollup_vk_hash) &
        (self.merge_rollup_vk_hash == other.merge_rollup_vk_hash)
    }
}
