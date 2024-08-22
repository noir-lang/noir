#include "append_only_tree.hpp"
#include "../fixtures.hpp"
#include "../memory_tree.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/common/thread_pool.hpp"
#include "barretenberg/crypto/merkle_tree/hash.hpp"
#include "barretenberg/crypto/merkle_tree/hash_path.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_tree.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_environment.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_store.hpp"
#include "barretenberg/crypto/merkle_tree/node_store/array_store.hpp"
#include "barretenberg/crypto/merkle_tree/node_store/cached_tree_store.hpp"
#include "barretenberg/crypto/merkle_tree/response.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "gtest/gtest.h"
#include <cstdint>
#include <filesystem>
#include <gtest/gtest.h>
#include <vector>

using namespace bb;
using namespace bb::crypto::merkle_tree;

using Store = CachedTreeStore<LMDBStore, bb::fr>;
using TreeType = AppendOnlyTree<Store, Poseidon2HashPolicy>;

class PersistedAppendOnlyTreeTest : public testing::Test {
  protected:
    void SetUp() override
    {
        // setup with 1MB max db size, 1 max database and 2 maximum concurrent readers
        _directory = random_temp_directory();
        std::filesystem::create_directories(_directory);
        _environment = std::make_unique<LMDBEnvironment>(_directory, 1024, 2, 2);
    }

    void TearDown() override { std::filesystem::remove_all(_directory); }

    static std::string _directory;

    std::unique_ptr<LMDBEnvironment> _environment;
};

std::string PersistedAppendOnlyTreeTest::_directory;

void check_size(TreeType& tree, index_t expected_size, bool includeUncommitted = true)
{
    Signal signal;
    auto completion = [&](const TypedResponse<TreeMetaResponse>& response) -> void {
        EXPECT_EQ(response.success, true);
        EXPECT_EQ(response.inner.size, expected_size);
        signal.signal_level();
    };
    tree.get_meta_data(includeUncommitted, completion);
    signal.wait_for_level();
}

void check_root(TreeType& tree, fr expected_root, bool includeUncommitted = true)
{
    Signal signal;
    auto completion = [&](const TypedResponse<TreeMetaResponse>& response) -> void {
        EXPECT_EQ(response.success, true);
        EXPECT_EQ(response.inner.root, expected_root);
        signal.signal_level();
    };
    tree.get_meta_data(includeUncommitted, completion);
    signal.wait_for_level();
}

void check_sibling_path(TreeType& tree,
                        index_t index,
                        fr_sibling_path expected_sibling_path,
                        bool includeUncommitted = true)
{
    Signal signal;
    auto completion = [&](const TypedResponse<GetSiblingPathResponse>& response) -> void {
        EXPECT_EQ(response.success, true);
        EXPECT_EQ(response.inner.path, expected_sibling_path);
        signal.signal_level();
    };
    tree.get_sibling_path(index, completion, includeUncommitted);
    signal.wait_for_level();
}

void commit_tree(TreeType& tree)
{
    Signal signal;
    auto completion = [&](const Response& response) -> void {
        EXPECT_EQ(response.success, true);
        signal.signal_level();
    };
    tree.commit(completion);
    signal.wait_for_level();
}

void rollback_tree(TreeType& tree)
{
    Signal signal;
    auto completion = [&](const Response& response) -> void {
        EXPECT_EQ(response.success, true);
        signal.signal_level();
    };
    tree.rollback(completion);
    signal.wait_for_level();
}

void add_value(TreeType& tree, const fr& value)
{
    Signal signal;
    auto completion = [&](const TypedResponse<AddDataResponse>& response) -> void {
        EXPECT_EQ(response.success, true);
        signal.signal_level();
    };

    tree.add_value(value, completion);
    signal.wait_for_level();
}

void add_values(TreeType& tree, const std::vector<fr>& values)
{
    Signal signal;
    auto completion = [&](const TypedResponse<AddDataResponse>& response) -> void {
        EXPECT_EQ(response.success, true);
        signal.signal_level();
    };

    tree.add_values(values, completion);
    signal.wait_for_level();
}

void check_find_leaf_index(
    TreeType& tree, const fr& leaf, index_t expected_index, bool expected_success, bool includeUncommitted = true)
{
    Signal signal;
    auto completion = [&](const TypedResponse<FindLeafIndexResponse>& response) -> void {
        EXPECT_EQ(response.success, expected_success);
        if (expected_success) {
            EXPECT_EQ(response.inner.leaf_index, expected_index);
        }
        signal.signal_level();
    };

    tree.find_leaf_index(leaf, includeUncommitted, completion);
    signal.wait_for_level();
}

void check_find_leaf_index_from(TreeType& tree,
                                const fr& leaf,
                                index_t start_index,
                                index_t expected_index,
                                bool expected_success,
                                bool includeUncommitted = true)
{
    Signal signal;
    auto completion = [&](const TypedResponse<FindLeafIndexResponse>& response) -> void {
        EXPECT_EQ(response.success, expected_success);
        if (expected_success) {
            EXPECT_EQ(response.inner.leaf_index, expected_index);
        }
        signal.signal_level();
    };

    tree.find_leaf_index_from(leaf, start_index, includeUncommitted, completion);
    signal.wait_for_level();
}

void check_leaf(
    TreeType& tree, const fr& leaf, index_t leaf_index, bool expected_success, bool includeUncommitted = true)
{
    Signal signal;
    tree.get_leaf(leaf_index, includeUncommitted, [&](const TypedResponse<GetLeafResponse>& response) {
        EXPECT_EQ(response.success, expected_success);
        if (expected_success) {
            EXPECT_EQ(response.inner.leaf, leaf);
        }
        signal.signal_level();
    });
    signal.wait_for_level();
}

void check_sibling_path(fr expected_root, fr node, index_t index, fr_sibling_path sibling_path)
{
    fr left, right, hash = node;
    for (size_t i = 0; i < sibling_path.size(); ++i) {
        if (index % 2 == 0) {
            left = hash;
            right = sibling_path[i];
        } else {
            left = sibling_path[i];
            right = hash;
        }

        hash = Poseidon2HashPolicy::hash_pair(left, right);
        index >>= 1;
    }

    EXPECT_EQ(hash, expected_root);
}

TEST_F(PersistedAppendOnlyTreeTest, can_create)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    EXPECT_NO_THROW(Store store(name, depth, db));
    Store store(name, depth, db);

    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    check_size(tree, 0);
    check_root(tree, memdb.root());
}

TEST_F(PersistedAppendOnlyTreeTest, can_only_recreate_with_same_name_and_depth)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);

    EXPECT_ANY_THROW(Store store_wrong_name("Wrong name", depth, db));
    EXPECT_ANY_THROW(Store store_wrong_depth(name, depth + 1, db));
}

TEST_F(PersistedAppendOnlyTreeTest, can_add_value_and_get_sibling_path)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);

    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    check_size(tree, 0);
    check_root(tree, memdb.root());

    memdb.update_element(0, VALUES[0]);
    add_value(tree, VALUES[0]);

    check_size(tree, 1);
    check_root(tree, memdb.root());
    check_sibling_path(tree, 0, memdb.get_sibling_path(0));
}

TEST_F(PersistedAppendOnlyTreeTest, reports_an_error_if_tree_is_overfilled)
{
    constexpr size_t depth = 4;
    std::string name = random_string();
    std::string directory = random_temp_directory();
    std::filesystem::create_directories(directory);
    auto environment = std::make_unique<LMDBEnvironment>(directory, 1024, 1, 2);
    LMDBStore db(*environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);

    ThreadPool pool(1);
    TreeType tree(store, pool);

    std::vector<fr> values;
    for (uint32_t i = 0; i < 16; i++) {
        values.push_back(VALUES[i]);
    }
    add_values(tree, values);

    Signal signal;
    auto add_completion = [&](const TypedResponse<AddDataResponse>& response) {
        EXPECT_EQ(response.success, false);
        EXPECT_EQ(response.message, "Tree is full");
        signal.signal_level();
    };
    tree.add_value(VALUES[16], add_completion);
    signal.wait_for_level();
    std::filesystem::remove_all(directory);
}

TEST_F(PersistedAppendOnlyTreeTest, errors_are_caught_and_handled)
{
    // We use a deep tree with a small amount of storage (20 * 1024) bytes
    constexpr size_t depth = 16;
    std::string name = random_string();
    std::string directory = random_temp_directory();
    std::filesystem::create_directories(directory);
    auto environment = std::make_unique<LMDBEnvironment>(directory, 300, 1, 2);
    LMDBStore db(*environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);

    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    // check the committed data only, so we read from the db
    check_size(tree, 0, false);
    check_root(tree, memdb.root(), false);

    fr empty_root = memdb.root();

    // Add lots of values to the tree
    uint32_t num_values_to_add = 16 * 1024;
    std::vector<fr> values(num_values_to_add, VALUES[0]);
    for (uint32_t i = 0; i < num_values_to_add; i++) {
        memdb.update_element(i, VALUES[0]);
    }
    add_values(tree, values);

    // check the uncommitted data is accurate
    check_size(tree, num_values_to_add, true);
    check_root(tree, memdb.root(), true);

    // trying to commit that should fail
    Signal signal;
    auto completion = [&](const Response& response) -> void {
        EXPECT_EQ(response.success, false);
        signal.signal_level();
    };

    tree.commit(completion);
    signal.wait_for_level();

    // At this stage, the tree is still in an uncommited state despite the error
    // Reading both committed and uncommitted data shold be ok

    // check the uncommitted data is accurate
    check_size(tree, num_values_to_add, true);
    check_root(tree, memdb.root(), true);

    // Reading committed data should still work
    check_size(tree, 0, false);
    check_root(tree, empty_root, false);

    // Now rollback the tree
    rollback_tree(tree);

    // committed and uncommitted data should be as an empty tree
    check_size(tree, 0, true);
    check_root(tree, empty_root, true);

    // Reading committed data should still work
    check_size(tree, 0, false);
    check_root(tree, empty_root, false);

    // // Now add a single value and commit it
    add_value(tree, VALUES[0]);
    commit_tree(tree);

    MemoryTree<Poseidon2HashPolicy> memdb2(depth);
    memdb2.update_element(0, VALUES[0]);

    // committed and uncommitted data should be equal to the tree with 1 item
    check_size(tree, 1, true);
    check_root(tree, memdb2.root(), true);

    // Reading committed data should still work
    check_size(tree, 1, false);
    check_root(tree, memdb2.root(), false);
}

TEST_F(PersistedAppendOnlyTreeTest, can_commit_and_restore)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    MemoryTree<Poseidon2HashPolicy> memdb(depth);
    {
        LMDBStore db(*_environment, name, false, false, integer_key_cmp);
        Store store(name, depth, db);

        ThreadPool pool(1);
        TreeType tree(store, pool);

        check_size(tree, 0);
        check_root(tree, memdb.root());
        check_sibling_path(tree, 0, memdb.get_sibling_path(0));

        bb::fr initial_root = memdb.root();
        fr_sibling_path initial_sibling_path = memdb.get_sibling_path(0);
        memdb.update_element(0, VALUES[0]);
        add_value(tree, VALUES[0]);

        // check uncommitted state
        check_size(tree, 1);
        check_root(tree, memdb.root());
        check_sibling_path(tree, 0, memdb.get_sibling_path(0));

        // check committed state
        check_size(tree, 0, false);
        check_root(tree, initial_root, false);
        check_sibling_path(tree, 0, initial_sibling_path, false);

        // commit the changes
        commit_tree(tree);
        // now committed and uncommitted should be the same

        // check uncommitted state
        check_size(tree, 1);
        check_root(tree, memdb.root());
        check_sibling_path(tree, 0, memdb.get_sibling_path(0));

        // check committed state
        check_size(tree, 1, false);
        check_root(tree, memdb.root(), false);
        check_sibling_path(tree, 0, memdb.get_sibling_path(0), false);
    }

    // Re-create the store and tree, it should be the same as how we left it
    {
        LMDBStore db(*_environment, name, false, false, integer_key_cmp);
        Store store(name, depth, db);

        ThreadPool pool(1);
        TreeType tree(store, pool);

        // check uncommitted state
        check_size(tree, 1);
        check_root(tree, memdb.root());
        check_sibling_path(tree, 0, memdb.get_sibling_path(0));

        // check committed state
        check_size(tree, 1, false);
        check_root(tree, memdb.root(), false);
        check_sibling_path(tree, 0, memdb.get_sibling_path(0), false);
    }
}

TEST_F(PersistedAppendOnlyTreeTest, test_size)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);

    check_size(tree, 0);

    // Add a new non-zero leaf at index 0.
    add_value(tree, 30);
    check_size(tree, 1);

    // Add second.
    add_value(tree, 10);
    check_size(tree, 2);

    // Add third.
    add_value(tree, 20);
    check_size(tree, 3);

    // Add forth but with same value.
    add_value(tree, 40);
    check_size(tree, 4);
}

TEST_F(PersistedAppendOnlyTreeTest, test_find_leaf_index)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);

    add_value(tree, 30);
    add_value(tree, 10);
    add_value(tree, 20);
    add_value(tree, 40);

    // check the committed state and that the uncommitted state is empty
    check_find_leaf_index(tree, 10, 1, true, true);
    check_find_leaf_index(tree, 10, 0, false, false);

    check_find_leaf_index(tree, 15, 0, false, true);
    check_find_leaf_index(tree, 15, 0, false, false);

    check_find_leaf_index(tree, 40, 3, true, true);
    check_find_leaf_index(tree, 30, 0, true, true);
    check_find_leaf_index(tree, 20, 2, true, true);

    check_find_leaf_index(tree, 40, 0, false, false);
    check_find_leaf_index(tree, 30, 0, false, false);
    check_find_leaf_index(tree, 20, 0, false, false);

    commit_tree(tree);

    std::vector<fr> values{ 15, 18, 26, 2, 48 };
    add_values(tree, values);

    // check the now committed state
    check_find_leaf_index(tree, 40, 3, true, false);
    check_find_leaf_index(tree, 30, 0, true, false);
    check_find_leaf_index(tree, 20, 2, true, false);

    // check the new uncommitted state
    check_find_leaf_index(tree, 18, 5, true, true);
    check_find_leaf_index(tree, 18, 0, false, false);

    commit_tree(tree);

    values = { 16, 4, 18, 22, 101 };
    add_values(tree, values);

    // we now have duplicate leaf 18, one committed the other not
    check_find_leaf_index(tree, 18, 5, true, true);
    check_find_leaf_index(tree, 18, 5, true, false);

    // verify the find index from api
    check_find_leaf_index_from(tree, 18, 0, 5, true, true);
    check_find_leaf_index_from(tree, 18, 6, 11, true, true);
    check_find_leaf_index_from(tree, 18, 6, 0, false, false);

    commit_tree(tree);

    // add another leaf 18
    add_value(tree, 18);

    // should return the first index
    check_find_leaf_index_from(tree, 18, 0, 5, true, false);
    check_find_leaf_index_from(tree, 18, 0, 5, true, true);

    add_value(tree, 88);
    // and another uncommitted 18
    add_value(tree, 18);

    add_value(tree, 32);

    // should return the first uncommitted
    check_find_leaf_index_from(tree, 18, 12, 14, true, true);
    check_find_leaf_index_from(tree, 18, 15, 16, true, true);

    // look past the last instance of this leaf
    check_find_leaf_index_from(tree, 18, 17, 0, false, true);

    // look beyond the end of uncommitted
    check_find_leaf_index_from(tree, 18, 18, 0, false, true);

    // look beyond the end of committed and don't include uncomitted
    check_find_leaf_index_from(tree, 18, 14, 0, false, false);
}

TEST_F(PersistedAppendOnlyTreeTest, can_add_multiple_values)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    for (size_t i = 0; i < NUM_VALUES; ++i) {
        fr mock_root = memdb.update_element(i, VALUES[i]);
        add_value(tree, VALUES[i]);
        check_root(tree, mock_root);

        check_sibling_path(tree, 0, memdb.get_sibling_path(0));
        check_sibling_path(tree, i, memdb.get_sibling_path(i));
    }
}

TEST_F(PersistedAppendOnlyTreeTest, can_add_multiple_values_in_a_batch)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    for (size_t i = 0; i < NUM_VALUES; ++i) {
        memdb.update_element(i, VALUES[i]);
    }
    add_values(tree, VALUES);
    check_size(tree, NUM_VALUES);
    check_root(tree, memdb.root());
    check_sibling_path(tree, 0, memdb.get_sibling_path(0));
    check_sibling_path(tree, NUM_VALUES - 1, memdb.get_sibling_path(NUM_VALUES - 1));
}

TEST_F(PersistedAppendOnlyTreeTest, can_be_filled)
{
    constexpr size_t depth = 3;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    check_size(tree, 0);
    check_root(tree, memdb.root());

    for (size_t i = 0; i < 8; i++) {
        memdb.update_element(i, VALUES[i]);
        add_value(tree, VALUES[i]);
    }

    check_root(tree, memdb.root());
    check_sibling_path(tree, 0, memdb.get_sibling_path(0));
    check_sibling_path(tree, 7, memdb.get_sibling_path(7));
}

TEST_F(PersistedAppendOnlyTreeTest, can_add_single_whilst_reading)
{
    constexpr size_t depth = 10;
    MemoryTree<Poseidon2HashPolicy> memdb(depth);
    fr_sibling_path initial_path = memdb.get_sibling_path(0);
    memdb.update_element(0, VALUES[0]);
    fr_sibling_path final_sibling_path = memdb.get_sibling_path(0);

    uint32_t num_reads = 16 * 1024;
    std::vector<fr_sibling_path> paths(num_reads);

    {
        std::string name = random_string();
        LMDBStore db(*_environment, name, false, false, integer_key_cmp);
        Store store(name, depth, db);
        ThreadPool pool(8);
        TreeType tree(store, pool);

        check_size(tree, 0);

        Signal signal(2);

        auto add_completion = [&](const TypedResponse<AddDataResponse>&) {
            signal.signal_level(1);
            auto commit_completion = [&](const Response&) { signal.signal_level(0); };
            tree.commit(commit_completion);
        };
        tree.add_value(VALUES[0], add_completion);

        for (size_t i = 0; i < num_reads; i++) {
            auto completion = [&, i](const TypedResponse<GetSiblingPathResponse>& response) {
                paths[i] = response.inner.path;
            };
            tree.get_sibling_path(0, completion, false);
        }
        signal.wait_for_level(0);
    }

    for (auto& path : paths) {
        EXPECT_TRUE(path == initial_path || path == final_sibling_path);
    }
}

TEST_F(PersistedAppendOnlyTreeTest, can_get_inserted_leaves)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);

    add_values(tree, { 30, 10, 20, 40 });

    check_leaf(tree, 30, 0, true);
    check_leaf(tree, 10, 1, true);
    check_leaf(tree, 20, 2, true);
    check_leaf(tree, 40, 3, true);

    check_leaf(tree, 0, 4, false);
}

TEST_F(PersistedAppendOnlyTreeTest, returns_sibling_path)
{
    constexpr size_t depth = 4;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    ThreadPool pool(1);
    TreeType tree(store, pool);
    MemoryTree<Poseidon2HashPolicy> memdb(depth);

    add_values(tree, { 30, 10, 20 });
    memdb.update_element(0, 30);
    memdb.update_element(1, 10);
    memdb.update_element(2, 20);

    tree.get_subtree_sibling_path(
        0,
        [&](auto& resp) {
            fr_sibling_path expected_sibling_path = memdb.get_sibling_path(3);
            EXPECT_EQ(resp.inner.path, expected_sibling_path);
        },
        true);

    tree.get_subtree_sibling_path(
        2,
        [&](auto& resp) {
            fr_sibling_path expected_sibling_path = { memdb.get_node(2, 1), memdb.get_node(1, 1) };
            EXPECT_EQ(resp.inner.path, expected_sibling_path);
        },
        true);
}
