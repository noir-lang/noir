#include "indexed_tree.hpp"
#include "../fixtures.hpp"
#include "../hash.hpp"
#include "../node_store/array_store.hpp"
#include "../nullifier_tree/nullifier_memory_tree.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/common/thread_pool.hpp"
#include "barretenberg/crypto/merkle_tree/hash_path.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_leaf.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_store.hpp"
#include "barretenberg/crypto/merkle_tree/node_store/cached_tree_store.hpp"
#include "barretenberg/crypto/merkle_tree/response.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <cstdint>
#include <filesystem>
#include <future>
#include <gtest/gtest.h>
#include <memory>
#include <stdexcept>
#include <vector>

using namespace bb;
using namespace bb::crypto::merkle_tree;

using HashPolicy = Poseidon2HashPolicy;

using Store = CachedTreeStore<LMDBStore, NullifierLeafValue>;
using TreeType = IndexedTree<Store, HashPolicy>;
using IndexedNullifierLeafType = IndexedLeaf<NullifierLeafValue>;
using IndexedPublicDataLeafType = IndexedLeaf<PublicDataLeafValue>;

using CompletionCallback = TreeType::AddCompletionCallback;

class PersistedIndexedTreeTest : public testing::Test {
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

std::string PersistedIndexedTreeTest::_directory;

template <typename LeafValueType>
void check_size(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                index_t expected_size,
                bool includeUncommitted = true)
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

template <typename LeafValueType>
fr get_root(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
            bool includeUncommitted = true)
{
    fr r;
    Signal signal;
    auto completion = [&](const TypedResponse<TreeMetaResponse>& response) -> void {
        r = response.inner.root;
        signal.signal_level();
    };
    tree.get_meta_data(includeUncommitted, completion);
    signal.wait_for_level();
    return r;
}

template <typename LeafValueType>
void check_root(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                fr expected_root,
                bool includeUncommitted = true)
{
    fr root = get_root(tree, includeUncommitted);
    EXPECT_EQ(root, expected_root);
}

template <typename LeafValueType>
fr_sibling_path get_sibling_path(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                                 index_t index,
                                 bool includeUncommitted = true)
{
    fr_sibling_path h;
    Signal signal;
    auto completion = [&](const TypedResponse<GetSiblingPathResponse>& response) -> void {
        h = response.inner.path;
        signal.signal_level();
    };
    tree.get_sibling_path(index, completion, includeUncommitted);
    signal.wait_for_level();
    return h;
}

template <typename LeafValueType>
IndexedLeaf<LeafValueType> get_leaf(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                                    index_t index,
                                    bool includeUncommitted = true)
{
    std::optional<IndexedLeaf<LeafValueType>> l;
    Signal signal;
    auto completion = [&](const TypedResponse<GetIndexedLeafResponse<LeafValueType>>& leaf) -> void {
        l = leaf.inner.indexed_leaf;
        signal.signal_level();
    };
    tree.get_leaf(index, includeUncommitted, completion);
    signal.wait_for_level();
    return l.value();
}

template <typename LeafValueType>
std::pair<bool, index_t> get_low_leaf(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                                      const LeafValueType& leaf,
                                      bool includeUncommitted = true)
{
    std::pair<bool, index_t> low_leaf_info;
    Signal signal;
    auto completion = [&](const auto& leaf) -> void {
        low_leaf_info = leaf.inner;
        signal.signal_level();
    };
    tree.find_low_leaf(leaf.get_key(), includeUncommitted, completion);
    signal.wait_for_level();
    return low_leaf_info;
}

template <typename LeafValueType>
void check_find_leaf_index(TreeType& tree,
                           const LeafValueType& leaf,
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

    tree.find_leaf_index(leaf, includeUncommitted, completion);
    signal.wait_for_level();
}

template <typename LeafValueType>
void check_sibling_path(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                        index_t index,
                        fr_sibling_path expected_sibling_path,
                        bool includeUncommitted = true)
{
    fr_sibling_path path = get_sibling_path(tree, index, includeUncommitted);
    EXPECT_EQ(path, expected_sibling_path);
}

template <typename LeafValueType>
void commit_tree(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree)
{
    Signal signal;
    auto completion = [&](const Response& response) -> void {
        EXPECT_EQ(response.success, true);
        signal.signal_level();
    };
    tree.commit(completion);
    signal.wait_for_level();
}

template <typename LeafValueType>
void add_value(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
               const LeafValueType& value)
{
    Signal signal;
    auto completion = [&](const TypedResponse<AddIndexedDataResponse<LeafValueType>>&) -> void {
        signal.signal_level();
    };

    tree.add_or_update_value(value, completion);
    signal.wait_for_level();
}

template <typename LeafValueType>
void add_values(IndexedTree<CachedTreeStore<LMDBStore, LeafValueType>, Poseidon2HashPolicy>& tree,
                const std::vector<NullifierLeafValue>& values)
{
    Signal signal;
    auto completion = [&](const TypedResponse<AddIndexedDataResponse<LeafValueType>>&) -> void {
        signal.signal_level();
    };

    tree.add_or_update_values(values, completion);
    signal.wait_for_level();
}

TEST_F(PersistedIndexedTreeTest, can_create)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    EXPECT_NO_THROW(Store store(name, depth, db));
    Store store(name, depth, db);
    ThreadPool workers(1);
    TreeType tree = TreeType(store, workers, 2);
    check_size(tree, 2);

    NullifierMemoryTree<HashPolicy> memdb(10);
    check_root(tree, memdb.root());
}

TEST_F(PersistedIndexedTreeTest, can_only_recreate_with_same_name_and_depth)
{
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);

    EXPECT_ANY_THROW(Store store_wrong_name("Wrong name", depth, db));
    EXPECT_ANY_THROW(Store store_wrong_depth(name, depth + 1, db));
}

TEST_F(PersistedIndexedTreeTest, test_size)
{
    index_t current_size = 2;
    ThreadPool workers(1);
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, current_size);

    check_size(tree, current_size);

    // We assume that the first leaf is already filled with (0, 0, 0).
    for (uint32_t i = 0; i < 4; i++) {
        add_value(tree, NullifierLeafValue(VALUES[i]));
        check_size(tree, ++current_size);
    }
}

TEST_F(PersistedIndexedTreeTest, indexed_tree_must_have_at_least_2_initial_size)
{
    index_t current_size = 1;
    ThreadPool workers(1);
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    EXPECT_THROW(TreeType(store, workers, current_size), std::runtime_error);
}

TEST_F(PersistedIndexedTreeTest, reports_an_error_if_tree_is_overfilled)
{
    index_t current_size = 2;
    ThreadPool workers(1);
    constexpr size_t depth = 4;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, current_size);

    std::vector<NullifierLeafValue> values;
    for (uint32_t i = 0; i < 14; i++) {
        values.emplace_back(VALUES[i]);
    }
    add_values(tree, values);

    Signal signal;
    auto add_completion = [&](const TypedResponse<AddIndexedDataResponse<NullifierLeafValue>>& response) {
        EXPECT_EQ(response.success, false);
        EXPECT_EQ(response.message, "Tree is full");
        signal.signal_level();
    };
    tree.add_or_update_value(NullifierLeafValue(VALUES[16]), add_completion);
    signal.wait_for_level();
}

TEST_F(PersistedIndexedTreeTest, test_get_sibling_path)
{
    index_t current_size = 2;
    NullifierMemoryTree<HashPolicy> memdb(10, current_size);

    ThreadPool workers(1);
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, current_size);

    check_size(tree, current_size);
    check_root(tree, memdb.root());
    check_sibling_path(tree, 0, memdb.get_sibling_path(0));

    memdb.update_element(VALUES[512]);
    add_value(tree, NullifierLeafValue(VALUES[512]));

    // std::cout << memdb.get_sibling_path(0) << std::endl;
    // std::cout << memdb.get_hash_path(0) << std::endl;

    // std::cout << get_sibling_path(tree, 0, true) << std::endl;
    // std::cout << get_sibling_path(tree, 1, true) << std::endl;

    check_size(tree, ++current_size);
    check_sibling_path(tree, 0, memdb.get_sibling_path(0));
    check_sibling_path(tree, 1, memdb.get_sibling_path(1));

    uint32_t num_to_append = 512;

    for (uint32_t i = 0; i < num_to_append; ++i) {
        memdb.update_element(VALUES[i]);
        add_value(tree, NullifierLeafValue(VALUES[i]));
    }
    check_size(tree, num_to_append + current_size);
    check_sibling_path(tree, 0, memdb.get_sibling_path(0));
    check_sibling_path(tree, 512, memdb.get_sibling_path(512));
}

TEST_F(PersistedIndexedTreeTest, test_find_leaf_index)
{
    index_t initial_size = 2;
    ThreadPool workers(1);
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, initial_size);

    add_value(tree, NullifierLeafValue(30));
    add_value(tree, NullifierLeafValue(10));
    add_value(tree, NullifierLeafValue(20));
    add_value(tree, NullifierLeafValue(40));

    // check the committed state and that the uncommitted state is empty
    check_find_leaf_index(tree, NullifierLeafValue(10), 1 + initial_size, true, true);
    check_find_leaf_index(tree, NullifierLeafValue(10), 0, false, false);

    check_find_leaf_index(tree, NullifierLeafValue(15), 0, false, true);
    check_find_leaf_index(tree, NullifierLeafValue(15), 0, false, false);

    check_find_leaf_index(tree, NullifierLeafValue(40), 3 + initial_size, true, true);
    check_find_leaf_index(tree, NullifierLeafValue(30), 0 + initial_size, true, true);
    check_find_leaf_index(tree, NullifierLeafValue(20), 2 + initial_size, true, true);

    check_find_leaf_index(tree, NullifierLeafValue(40), 0, false, false);
    check_find_leaf_index(tree, NullifierLeafValue(30), 0, false, false);
    check_find_leaf_index(tree, NullifierLeafValue(20), 0, false, false);

    commit_tree(tree);

    std::vector<NullifierLeafValue> values{ NullifierLeafValue(15),
                                            NullifierLeafValue(18),
                                            NullifierLeafValue(26),
                                            NullifierLeafValue(2),
                                            NullifierLeafValue(48) };
    add_values(tree, values);

    // check the now committed state
    check_find_leaf_index(tree, NullifierLeafValue(40), 3 + initial_size, true, false);
    check_find_leaf_index(tree, NullifierLeafValue(30), 0 + initial_size, true, false);
    check_find_leaf_index(tree, NullifierLeafValue(20), 2 + initial_size, true, false);

    // check the new uncommitted state
    check_find_leaf_index(tree, NullifierLeafValue(18), 5 + initial_size, true, true);
    check_find_leaf_index(tree, NullifierLeafValue(18), 0, false, false);

    commit_tree(tree);

    values = { NullifierLeafValue(16), NullifierLeafValue(4), NullifierLeafValue(22), NullifierLeafValue(101) };
    add_values(tree, values);

    // we now have duplicate leaf 18, one committed the other not
    check_find_leaf_index(tree, NullifierLeafValue(18), 5 + initial_size, true, true);
    check_find_leaf_index(tree, NullifierLeafValue(18), 5 + initial_size, true, false);
}

TEST_F(PersistedIndexedTreeTest, can_commit_and_restore)
{
    NullifierMemoryTree<HashPolicy> memdb(10);
    index_t current_size = 2;
    ThreadPool workers(1);
    constexpr size_t depth = 10;
    std::string name = random_string();

    {
        LMDBStore db(*_environment, name, false, false, integer_key_cmp);
        Store store(name, depth, db);
        auto tree = TreeType(store, workers, current_size);

        check_size(tree, current_size);
        check_root(tree, memdb.root());
        check_sibling_path(tree, 0, memdb.get_sibling_path(0));

        add_value(tree, NullifierLeafValue(VALUES[512]));

        // Committed data should not have changed
        check_size(tree, current_size, false);
        check_root(tree, memdb.root(), false);
        check_sibling_path(tree, 0, memdb.get_sibling_path(0), false);
        check_sibling_path(tree, 1, memdb.get_sibling_path(1), false);

        memdb.update_element(VALUES[512]);

        // Uncommitted data should have changed
        check_size(tree, current_size + 1, true);
        check_root(tree, memdb.root(), true);
        check_sibling_path(tree, 0, memdb.get_sibling_path(0), true);
        check_sibling_path(tree, 1, memdb.get_sibling_path(1), true);

        // Now commit
        commit_tree(tree);

        // Now committed data should have changed
        check_size(tree, ++current_size, false);
        check_root(tree, memdb.root(), false);
        check_sibling_path(tree, 0, memdb.get_sibling_path(0), false);
        check_sibling_path(tree, 1, memdb.get_sibling_path(1), false);
    }

    // Now restore and it should continue from where we left off
    {
        LMDBStore db(*_environment, name, false, false, integer_key_cmp);
        Store store(name, depth, db);
        auto tree = TreeType(store, workers, current_size);

        // check uncommitted state
        check_size(tree, current_size);
        check_root(tree, memdb.root());
        check_sibling_path(tree, 0, memdb.get_sibling_path(0));

        // check committed state
        check_size(tree, current_size, false);
        check_root(tree, memdb.root(), false);
        check_sibling_path(tree, 0, memdb.get_sibling_path(0), false);
    }
}

TEST_F(PersistedIndexedTreeTest, test_batch_insert)
{
    auto& random_engine = numeric::get_randomness();
    const uint32_t batch_size = 16;
    const uint32_t num_batches = 16;
    uint32_t depth = 10;
    ThreadPool workers(1);
    ThreadPool multi_workers(8);
    NullifierMemoryTree<HashPolicy> memdb(depth, batch_size);

    std::string name1 = random_string();
    LMDBStore db1(*_environment, name1, false, false, integer_key_cmp);
    Store store1(name1, depth, db1);
    auto tree1 = TreeType(store1, workers, batch_size);

    std::string name2 = random_string();
    LMDBStore db2(*_environment, name2, false, false, integer_key_cmp);
    Store store2(name2, depth, db2);
    auto tree2 = TreeType(store2, workers, batch_size);

    check_root(tree1, memdb.root());
    check_root(tree2, memdb.root());

    check_sibling_path(tree1, 0, memdb.get_sibling_path(0));
    check_sibling_path(tree2, 0, memdb.get_sibling_path(0));

    check_sibling_path(tree1, 512, memdb.get_sibling_path(512));
    check_sibling_path(tree2, 512, memdb.get_sibling_path(512));

    for (uint32_t i = 0; i < num_batches; i++) {
        std::vector<NullifierLeafValue> batch;
        std::vector<fr_sibling_path> memory_tree_sibling_paths;
        for (uint32_t j = 0; j < batch_size; j++) {
            batch.emplace_back(random_engine.get_random_uint256());
            fr_sibling_path path = memdb.update_element(batch[j].value);
            memory_tree_sibling_paths.push_back(path);
        }
        std::shared_ptr<std::vector<LowLeafWitnessData<NullifierLeafValue>>> tree1_low_leaf_witness_data;
        std::shared_ptr<std::vector<LowLeafWitnessData<NullifierLeafValue>>> tree2_low_leaf_witness_data;
        {
            Signal signal;
            CompletionCallback completion =
                [&](const TypedResponse<AddIndexedDataResponse<NullifierLeafValue>>& response) {
                    tree1_low_leaf_witness_data = response.inner.low_leaf_witness_data;
                    signal.signal_level();
                };
            tree1.add_or_update_values(batch, completion);
            signal.wait_for_level();
        }
        {
            Signal signal;
            CompletionCallback completion =
                [&](const TypedResponse<AddIndexedDataResponse<NullifierLeafValue>>& response) {
                    tree2_low_leaf_witness_data = response.inner.low_leaf_witness_data;
                    signal.signal_level();
                };
            tree2.add_or_update_values(batch, completion);
            signal.wait_for_level();
        }
        check_root(tree1, memdb.root());
        check_root(tree2, memdb.root());

        check_sibling_path(tree1, 0, memdb.get_sibling_path(0));
        check_sibling_path(tree2, 0, memdb.get_sibling_path(0));

        check_sibling_path(tree1, 512, memdb.get_sibling_path(512));
        check_sibling_path(tree2, 512, memdb.get_sibling_path(512));

        for (uint32_t j = 0; j < batch_size; j++) {
            EXPECT_EQ(tree1_low_leaf_witness_data->at(j).leaf, tree2_low_leaf_witness_data->at(j).leaf);
            EXPECT_EQ(tree1_low_leaf_witness_data->at(j).index, tree2_low_leaf_witness_data->at(j).index);
            EXPECT_EQ(tree1_low_leaf_witness_data->at(j).path, tree2_low_leaf_witness_data->at(j).path);
        }
    }
}

TEST_F(PersistedIndexedTreeTest, reports_an_error_if_batch_contains_duplicate)
{
    index_t current_size = 2;
    ThreadPool workers(1);
    constexpr size_t depth = 10;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, current_size);

    std::vector<NullifierLeafValue> values;
    for (uint32_t i = 0; i < 16; i++) {
        values.emplace_back(VALUES[i]);
    }
    values[8] = values[0];

    Signal signal;
    auto add_completion = [&](const TypedResponse<AddIndexedDataResponse<NullifierLeafValue>>& response) {
        EXPECT_EQ(response.success, false);
        EXPECT_EQ(response.message, "Duplicate key not allowed in same batch");
        signal.signal_level();
    };
    tree.add_or_update_values(values, add_completion);
    signal.wait_for_level();
}

template <typename LeafValueType> fr hash_leaf(const IndexedLeaf<LeafValueType>& leaf)
{
    return HashPolicy::hash(leaf.get_hash_inputs());
}

bool verify_sibling_path(TreeType& tree, const IndexedNullifierLeafType& leaf_value, const uint32_t idx)
{
    fr root = get_root(tree, true);
    fr_sibling_path path = get_sibling_path(tree, idx, true);
    auto current = hash_leaf(leaf_value);
    uint32_t depth_ = static_cast<uint32_t>(path.size());
    uint32_t index = idx;
    for (uint32_t i = 0; i < depth_; ++i) {
        fr left = (index & 1) ? path[i] : current;
        fr right = (index & 1) ? current : path[i];
        current = HashPolicy::hash_pair(left, right);
        index >>= 1;
    }
    return current == root;
}

IndexedNullifierLeafType create_indexed_nullifier_leaf(const fr& value, index_t nextIndex, const fr& nextValue)
{
    return IndexedNullifierLeafType{ NullifierLeafValue(value), nextIndex, nextValue };
}

IndexedPublicDataLeafType create_indexed_public_data_leaf(const fr& slot,
                                                          const fr& value,
                                                          index_t nextIndex,
                                                          const fr& nextValue)
{
    return IndexedPublicDataLeafType{ PublicDataLeafValue(slot, value), nextIndex, nextValue };
}

TEST_F(PersistedIndexedTreeTest, test_indexed_memory)
{
    index_t current_size = 2;
    ThreadPool workers(8);
    // Create a depth-3 indexed merkle tree
    constexpr size_t depth = 3;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, current_size);

    /**
     * Intial state:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       1       1       0       0        0       0       0       0
     *  nextIdx   1       0       0       0        0       0       0       0
     *  nextVal   0       0       0       0        0       0       0       0
     */
    IndexedNullifierLeafType zero_leaf(NullifierLeafValue(0), 1, 1);
    IndexedNullifierLeafType one_leaf(NullifierLeafValue(1), 0, 0);
    check_size(tree, current_size);
    EXPECT_EQ(get_leaf(tree, 0), zero_leaf);
    EXPECT_EQ(get_leaf(tree, 1), one_leaf);

    /**
     * Add new value 30:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       1       30      0        0       0       0       0
     *  nextIdx   1       2       0       0        0       0       0       0
     *  nextVal   1       30      0       0        0       0       0       0
     */
    add_value(tree, NullifierLeafValue(30));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_nullifier_leaf(0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_nullifier_leaf(1, 2, 30));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_nullifier_leaf(30, 0, 0));

    /**
     * Add new value 10:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       1       30      10       0       0       0       0
     *  nextIdx   1       3       0       2        0       0       0       0
     *  nextVal   1       10      0       30       0       0       0       0
     */
    add_value(tree, NullifierLeafValue(10));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_nullifier_leaf(0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_nullifier_leaf(1, 3, 10));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_nullifier_leaf(30, 0, 0));
    EXPECT_EQ(get_leaf(tree, 3), create_indexed_nullifier_leaf(10, 2, 30));

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       1       30      10       20      0       0       0
     *  nextIdx   1       3       0       4        2       0       0       0
     *  nextVal   1       10      0       20       30      0       0       0
     */
    add_value(tree, NullifierLeafValue(20));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_nullifier_leaf(0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_nullifier_leaf(1, 3, 10));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_nullifier_leaf(30, 0, 0));
    EXPECT_EQ(get_leaf(tree, 3), create_indexed_nullifier_leaf(10, 4, 20));
    EXPECT_EQ(get_leaf(tree, 4), create_indexed_nullifier_leaf(20, 2, 30));

    // Adding the same value must not affect anything
    // tree.update_element(20);
    // EXPECT_EQ(tree.get_leaves().size(), 4);
    // EXPECT_EQ(tree.get_leaves()[0], hash_leaf({ 0, 2, 10 }));
    // EXPECT_EQ(tree.get_leaves()[1], hash_leaf({ 30, 0, 0 }));
    // EXPECT_EQ(tree.get_leaves()[2], hash_leaf({ 10, 3, 20 }));
    // EXPECT_EQ(tree.get_leaves()[3], hash_leaf({ 20, 1, 30 }));

    /**
     * Add new value 50:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       1       30      10       20      50      0       0
     *  nextIdx   1       3       5       4        2       0       0       0
     *  nextVal   1       10      50      20       30      0       0       0
     */
    add_value(tree, NullifierLeafValue(50));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_nullifier_leaf(0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_nullifier_leaf(1, 3, 10));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_nullifier_leaf(30, 5, 50));
    EXPECT_EQ(get_leaf(tree, 3), create_indexed_nullifier_leaf(10, 4, 20));
    EXPECT_EQ(get_leaf(tree, 4), create_indexed_nullifier_leaf(20, 2, 30));
    EXPECT_EQ(get_leaf(tree, 5), create_indexed_nullifier_leaf(50, 0, 0));

    // Manually compute the node values
    auto e000 = hash_leaf(get_leaf(tree, 0));
    auto e001 = hash_leaf(get_leaf(tree, 1));
    auto e010 = hash_leaf(get_leaf(tree, 2));
    auto e011 = hash_leaf(get_leaf(tree, 3));
    auto e100 = hash_leaf(get_leaf(tree, 4));
    auto e101 = hash_leaf(get_leaf(tree, 5));
    auto e110 = fr::zero();
    auto e111 = fr::zero();

    auto e00 = HashPolicy::hash_pair(e000, e001);
    auto e01 = HashPolicy::hash_pair(e010, e011);
    auto e10 = HashPolicy::hash_pair(e100, e101);
    auto e11 = HashPolicy::hash_pair(e110, e111);

    auto e0 = HashPolicy::hash_pair(e00, e01);
    auto e1 = HashPolicy::hash_pair(e10, e11);
    auto root = HashPolicy::hash_pair(e0, e1);

    // Check the hash path at index 2 and 3
    // Note: This merkle proof would also serve as a non-membership proof of values in (10, 20) and (20, 30)
    fr_sibling_path expected = {
        e001,
        e01,
        e1,
    };
    check_sibling_path(tree, 0, expected);
    expected = {
        e000,
        e01,
        e1,
    };
    check_sibling_path(tree, 1, expected);
    expected = {
        e011,
        e00,
        e1,
    };
    check_sibling_path(tree, 2, expected);
    expected = {
        e010,
        e00,
        e1,
    };
    check_sibling_path(tree, 3, expected);
    check_root(tree, root);

    // Check the hash path at index 6 and 7
    expected = {
        e111,
        e10,
        e0,
    };
    check_sibling_path(tree, 6, expected);
    expected = {
        e110,
        e10,
        e0,
    };
    check_sibling_path(tree, 7, expected);
}

TEST_F(PersistedIndexedTreeTest, test_indexed_tree)
{
    index_t current_size = 2;
    ThreadPool workers(1);
    // Create a depth-8 indexed merkle tree
    constexpr uint32_t depth = 8;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, current_size);

    IndexedNullifierLeafType zero_leaf = create_indexed_nullifier_leaf(0, 1, 1);
    check_size(tree, current_size);
    EXPECT_EQ(hash_leaf(get_leaf(tree, 0)), hash_leaf(zero_leaf));

    // Add 20 random values to the tree
    for (uint32_t i = 0; i < 20; i++) {
        auto value = fr::random_element();
        add_value(tree, NullifierLeafValue(value));
        ++current_size;
    }

    auto abs_diff = [](uint256_t a, uint256_t b) {
        if (a > b) {
            return (a - b);
        } else {
            return (b - a);
        }
    };

    check_size(tree, current_size);

    // Check if a new random value is not a member of this tree.
    fr new_member = fr::random_element();
    std::vector<uint256_t> differences;
    for (uint32_t i = 0; i < uint32_t(21); i++) {
        uint256_t diff_hi = abs_diff(uint256_t(new_member), uint256_t(get_leaf(tree, i).value.get_key()));
        uint256_t diff_lo = abs_diff(uint256_t(new_member), uint256_t(get_leaf(tree, i).value.get_key()));
        differences.push_back(diff_hi + diff_lo);
    }
    auto it = std::min_element(differences.begin(), differences.end());
    auto index = static_cast<uint32_t>(it - differences.begin());

    // Merkle proof at `index` proves non-membership of `new_member`
    EXPECT_TRUE(verify_sibling_path(tree, get_leaf(tree, index), index));
}

TEST_F(PersistedIndexedTreeTest, can_add_single_whilst_reading)
{
    constexpr size_t depth = 10;
    NullifierMemoryTree<HashPolicy> memdb(10);
    fr_sibling_path initial_path = memdb.get_sibling_path(0);
    memdb.update_element(VALUES[0]);
    fr_sibling_path final_sibling_path = memdb.get_sibling_path(0);

    uint32_t num_reads = 16 * 1024;
    std::vector<fr_sibling_path> paths(num_reads);

    {
        std::string name = random_string();
        LMDBStore db(*_environment, name, false, false, integer_key_cmp);
        Store store(name, depth, db);
        ThreadPool pool(8);
        TreeType tree(store, pool, 2);

        check_size(tree, 2);

        Signal signal(2);

        auto add_completion = [&](const TypedResponse<AddIndexedDataResponse<NullifierLeafValue>>&) {
            signal.signal_level(1);
            auto commit_completion = [&](const Response&) { signal.signal_level(); };
            tree.commit(commit_completion);
        };
        tree.add_or_update_value(VALUES[0], add_completion);

        for (size_t i = 0; i < num_reads; i++) {
            auto completion = [&, i](const TypedResponse<GetSiblingPathResponse>& response) {
                paths[i] = response.inner.path;
            };
            tree.get_sibling_path(0, completion, false);
        }
        signal.wait_for_level();
    }

    // for (auto& path : paths) {
    // EXPECT_TRUE(path == initial_path || path == final_sibling_path);
    // }
}

TEST_F(PersistedIndexedTreeTest, test_indexed_memory_with_public_data_writes)
{
    index_t current_size = 2;
    ThreadPool workers(8);
    // Create a depth-3 indexed merkle tree
    constexpr size_t depth = 3;
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    CachedTreeStore<LMDBStore, PublicDataLeafValue> store(name, depth, db);
    auto tree =
        IndexedTree<CachedTreeStore<LMDBStore, PublicDataLeafValue>, Poseidon2HashPolicy>(store, workers, current_size);

    /**
     * Intial state:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  slot      0       1       0       0        0       0       0       0
     *  val       0       0       0       0        0       0       0       0
     *  nextIdx   1       0       0       0        0       0       0       0
     *  nextVal   1       0       0       0        0       0       0       0
     */
    IndexedPublicDataLeafType zero_leaf = create_indexed_public_data_leaf(0, 0, 1, 1);
    IndexedPublicDataLeafType one_leaf = create_indexed_public_data_leaf(1, 0, 0, 0);
    check_size(tree, current_size);
    EXPECT_EQ(get_leaf(tree, 0), zero_leaf);
    EXPECT_EQ(get_leaf(tree, 1), one_leaf);

    /**
     * Add new slot:value 30:5:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  slot      0       1       30      0        0       0       0       0
     *  val       0       0       5       0        0       0       0       0
     *  nextIdx   1       2       0       0        0       0       0       0
     *  nextVal   1       30      0       0        0       0       0       0
     */
    add_value(tree, PublicDataLeafValue(30, 5));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_public_data_leaf(0, 0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_public_data_leaf(1, 0, 2, 30));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_public_data_leaf(30, 5, 0, 0));

    /**
     * Add new slot:value 10:20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  slot      0       1       30      10        0       0       0       0
     *  val       0       0       5       20        0       0       0       0
     *  nextIdx   1       3       0       2         0       0       0       0
     *  nextVal   1       10      0       30        0       0       0       0
     */
    add_value(tree, PublicDataLeafValue(10, 20));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_public_data_leaf(0, 0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_public_data_leaf(1, 0, 3, 10));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_public_data_leaf(30, 5, 0, 0));
    EXPECT_EQ(get_leaf(tree, 3), create_indexed_public_data_leaf(10, 20, 2, 30));

    /**
     * Update value at slot 30 to 6:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  slot      0       1       30      10       0       0       0       0
     *  val       0       0       6       20       0       0       0       0
     *  nextIdx   1       3       0       2        0       0       0       0
     *  nextVal   1       10      0       30       0       0       0       0
     */
    add_value(tree, PublicDataLeafValue(30, 6));
    // The size still increases as we pad with an empty leaf
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_public_data_leaf(0, 0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_public_data_leaf(1, 0, 3, 10));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_public_data_leaf(30, 6, 0, 0));
    EXPECT_EQ(get_leaf(tree, 3), create_indexed_public_data_leaf(10, 20, 2, 30));
    EXPECT_EQ(get_leaf(tree, 4), create_indexed_public_data_leaf(0, 0, 0, 0));

    /**
     * Add new value slot:value 50:8:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  slot      0       1       30      10       0       50      0       0
     *  val       0       0       6       20       0       8       0       0
     *  nextIdx   1       3       5       2        0       0       0       0
     *  nextVal   1       10      50      30       0       0       0       0
     */
    add_value(tree, PublicDataLeafValue(50, 8));
    check_size(tree, ++current_size);
    EXPECT_EQ(get_leaf(tree, 0), create_indexed_public_data_leaf(0, 0, 1, 1));
    EXPECT_EQ(get_leaf(tree, 1), create_indexed_public_data_leaf(1, 0, 3, 10));
    EXPECT_EQ(get_leaf(tree, 2), create_indexed_public_data_leaf(30, 6, 5, 50));
    EXPECT_EQ(get_leaf(tree, 3), create_indexed_public_data_leaf(10, 20, 2, 30));
    EXPECT_EQ(get_leaf(tree, 4), create_indexed_public_data_leaf(0, 0, 0, 0));
    EXPECT_EQ(get_leaf(tree, 5), create_indexed_public_data_leaf(50, 8, 0, 0));

    // Manually compute the node values
    auto e000 = hash_leaf(get_leaf(tree, 0));
    auto e001 = hash_leaf(get_leaf(tree, 1));
    auto e010 = hash_leaf(get_leaf(tree, 2));
    auto e011 = hash_leaf(get_leaf(tree, 3));
    auto e100 = fr::zero(); // tree doesn't hash 0 leaves!
    auto e101 = hash_leaf(get_leaf(tree, 5));
    auto e110 = fr::zero();
    auto e111 = fr::zero();

    auto e00 = HashPolicy::hash_pair(e000, e001);
    auto e01 = HashPolicy::hash_pair(e010, e011);
    auto e10 = HashPolicy::hash_pair(e100, e101);
    auto e11 = HashPolicy::hash_pair(e110, e111);

    auto e0 = HashPolicy::hash_pair(e00, e01);
    auto e1 = HashPolicy::hash_pair(e10, e11);
    auto root = HashPolicy::hash_pair(e0, e1);

    fr_sibling_path expected = {
        e001,
        e01,
        e1,
    };
    check_sibling_path(tree, 0, expected);
    expected = {
        e000,
        e01,
        e1,
    };
    check_sibling_path(tree, 1, expected);
    expected = {
        e011,
        e00,
        e1,
    };
    check_sibling_path(tree, 2, expected);
    expected = {
        e010,
        e00,
        e1,
    };
    check_sibling_path(tree, 3, expected);
    check_root(tree, root);

    // Check the hash path at index 6 and 7
    expected = {
        e111,
        e10,
        e0,
    };
    check_sibling_path(tree, 6, expected);
    expected = {
        e110,
        e10,
        e0,
    };
    check_sibling_path(tree, 7, expected);
}

TEST_F(PersistedIndexedTreeTest, returns_low_leaves)
{
    // Create a depth-8 indexed merkle tree
    constexpr uint32_t depth = 8;

    ThreadPool workers(1);
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, 2);

    auto predecessor = get_low_leaf(tree, NullifierLeafValue(42));

    EXPECT_EQ(predecessor.first, false);
    EXPECT_EQ(predecessor.second, 1);

    add_value(tree, NullifierLeafValue(42));

    predecessor = get_low_leaf(tree, NullifierLeafValue(42));
    // returns the current leaf since it exists already. Inserting 42 again would modify the existing leaf
    EXPECT_EQ(predecessor.first, true);
    EXPECT_EQ(predecessor.second, 2);
}

TEST_F(PersistedIndexedTreeTest, duplicates)
{
    // Create a depth-8 indexed merkle tree
    constexpr uint32_t depth = 8;

    ThreadPool workers(1);
    std::string name = random_string();
    LMDBStore db(*_environment, name, false, false, integer_key_cmp);
    Store store(name, depth, db);
    auto tree = TreeType(store, workers, 2);

    add_value(tree, NullifierLeafValue(42));
    check_size(tree, 3);

    commit_tree(tree);

    add_value(tree, NullifierLeafValue(42));
    commit_tree(tree);
    check_size(tree, 3);
}
