

#pragma once
#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/merge/previous_rollup_data.hpp"
#include "aztec3/constants.hpp"

#include <ostream>

namespace aztec3::circuits::abis {

// TODO: The copy constructor for this struct may throw memory access out of bounds
// Hit when running aztec3-packages/yarn-project/circuits.js/src/rollup/rollup_wasm_wrapper.test.ts."calls
// root_rollup__sim"
template <typename NCT> struct RootRollupInputs {
    using fr = typename NCT::fr;

    // All below are shared between the base and merge rollups
    std::array<PreviousRollupData<NCT>, 2> previous_rollup_data{};

    // inputs required to process l1 to l2 messages
    std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> l1_to_l2_messages{};
    std::array<fr, L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH> new_l1_to_l2_message_tree_root_sibling_path{};

    AppendOnlyTreeSnapshot<NCT> start_l1_to_l2_message_tree_snapshot{};

    // inputs required to add the block hash
    AppendOnlyTreeSnapshot<NCT> start_historic_blocks_tree_snapshot{};
    std::array<fr, HISTORIC_BLOCKS_TREE_HEIGHT> new_historic_blocks_tree_sibling_path{};

    bool operator==(RootRollupInputs<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, RootRollupInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.previous_rollup_data);
    read(it, obj.l1_to_l2_messages);
    read(it, obj.new_l1_to_l2_message_tree_root_sibling_path);
    read(it, obj.start_l1_to_l2_message_tree_snapshot);
    read(it, obj.start_historic_blocks_tree_snapshot);
    read(it, obj.new_historic_blocks_tree_sibling_path);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, RootRollupInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.previous_rollup_data);
    write(buf, obj.l1_to_l2_messages);
    write(buf, obj.new_l1_to_l2_message_tree_root_sibling_path);
    write(buf, obj.start_l1_to_l2_message_tree_snapshot);
    write(buf, obj.start_historic_blocks_tree_snapshot);
    write(buf, obj.new_historic_blocks_tree_sibling_path);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, RootRollupInputs<NCT> const& obj)
{
    return os << "previous_rollup_data: " << obj.previous_rollup_data << "\n"
              << "new_l1_to_l2_messages: " << obj.l1_to_l2_messages << "\n"
              << "start_l1_to_l2_message_tree_snapshot: " << obj.start_l1_to_l2_message_tree_snapshot << "\n"
              << "start_historic_blocks_tree_snapshot: " << obj.start_historic_blocks_tree_snapshot << "\n"
              << "new_historic_blocks_tree_sibling_path: " << obj.new_historic_blocks_tree_sibling_path << "\n";
}

}  // namespace aztec3::circuits::abis