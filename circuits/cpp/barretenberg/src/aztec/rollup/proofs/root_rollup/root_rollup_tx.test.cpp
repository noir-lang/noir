#include "root_rollup_tx.hpp"
#include "../../constants.hpp"
#include <gtest/gtest.h>

using namespace rollup;
using namespace rollup::proofs::root_rollup;
using namespace rollup::proofs::notes;
using namespace barretenberg;

TEST(root_rollup_tx, test_serialization)
{
    auto random_pair = std::make_pair(fr::random_element(), fr::random_element());

    root_rollup_tx rollup;

    rollup.rollup_id = 5;
    rollup.num_inner_proofs = 2;
    rollup.rollups = std::vector(2, std::vector<uint8_t>(123, 0x80));

    rollup.old_data_roots_root = fr::random_element();
    rollup.new_data_roots_root = fr::random_element();
    rollup.old_data_roots_path = fr_hash_path(ROOT_TREE_DEPTH, random_pair);

    rollup.new_defi_root = fr::random_element();
    rollup.old_defi_root = fr::random_element();
    rollup.old_defi_path = fr_hash_path(DEFI_TREE_DEPTH, random_pair);

    rollup.bridge_ids = { 1, 2, 3, 4 };
    rollup.asset_ids = { 5, 6, 7 };

    native::defi_interaction::note defi_native_note = { 0, 0, 0, 0, 0, false };
    rollup.defi_interaction_notes = { 4, defi_native_note };

    auto buf = to_buffer(rollup);
    auto result = from_buffer<root_rollup_tx>(buf);

    EXPECT_EQ(result.rollup_id, rollup.rollup_id);
    EXPECT_EQ(result.num_inner_proofs, rollup.num_inner_proofs);
    EXPECT_EQ(result.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(result.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(result.old_data_roots_path, rollup.old_data_roots_path);
    EXPECT_EQ(result.old_defi_root, rollup.old_defi_root);
    EXPECT_EQ(result.new_defi_root, rollup.new_defi_root);
    EXPECT_EQ(result.old_defi_path, rollup.old_defi_path);
    EXPECT_EQ(result.bridge_ids, rollup.bridge_ids);
    EXPECT_EQ(result.asset_ids, rollup.asset_ids);
    EXPECT_EQ(result.defi_interaction_notes, rollup.defi_interaction_notes);
}
