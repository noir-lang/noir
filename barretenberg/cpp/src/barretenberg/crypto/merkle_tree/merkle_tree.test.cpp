#include "merkle_tree.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "memory_store.hpp"
#include "memory_tree.hpp"

using namespace bb;
using namespace bb::stdlib;
using namespace bb::crypto::merkle_tree;

using Builder = UltraCircuitBuilder;

using field_ct = field_t<Builder>;
using witness_ct = witness_t<Builder>;
namespace {
auto& engine = numeric::get_debug_randomness();
auto& random_engine = numeric::get_randomness();
} // namespace

static std::vector<fr> VALUES = []() {
    std::vector<fr> values(1024);
    for (size_t i = 0; i < 1024; ++i) {
        values[i] = i;
    }
    return values;
}();

TEST(crypto_merkle_tree, test_kv_memory_vs_memory_consistency)
{
    constexpr size_t depth = 10;
    MemoryTree<PedersenHashPolicy> memdb(depth);

    MemoryStore store;
    MerkleTree<MemoryStore, PedersenHashPolicy> db(store, depth);

    std::vector<size_t> indicies(1 << depth);
    std::iota(indicies.begin(), indicies.end(), 0);
    std::random_device rd;
    std::mt19937 g(rd());
    std::shuffle(indicies.begin(), indicies.end(), g);

    for (size_t i = 0; i < indicies.size(); ++i) {
        size_t idx = indicies[i];
        memdb.update_element(idx, VALUES[idx]);
        db.update_element(idx, VALUES[idx]);
    }

    for (size_t i = 0; i < indicies.size(); ++i) {
        size_t idx = indicies[i];
        EXPECT_EQ(db.get_hash_path(idx), memdb.get_hash_path(idx));
    }

    EXPECT_EQ(db.root(), memdb.root());
}

TEST(crypto_merkle_tree, test_size)
{
    MemoryStore store;
    auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 256);

    EXPECT_EQ(db.size(), 0ULL);

    // Add first.
    db.update_element(0, VALUES[1]);
    EXPECT_EQ(db.size(), 1ULL);

    // Add second.
    db.update_element(1, VALUES[2]);
    EXPECT_EQ(db.size(), 2ULL);

    // Set second to same value.
    db.update_element(1, VALUES[2]);
    EXPECT_EQ(db.size(), 2ULL);

    // Set second to new value.
    db.update_element(1, VALUES[3]);
    EXPECT_EQ(db.size(), 2ULL);

    // Set third to new value.
    db.update_element(2, VALUES[4]);
    EXPECT_EQ(db.size(), 3ULL);
}

TEST(crypto_merkle_tree, test_get_hash_path)
{
    MemoryTree<PedersenHashPolicy> memdb(10);

    MemoryStore store;
    auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 10);

    EXPECT_EQ(memdb.get_hash_path(512), db.get_hash_path(512));

    memdb.update_element(512, VALUES[512]);
    db.update_element(512, VALUES[512]);

    EXPECT_EQ(db.get_hash_path(512), memdb.get_hash_path(512));

    for (size_t i = 0; i < 1024; ++i) {
        memdb.update_element(i, VALUES[i]);
        db.update_element(i, VALUES[i]);
    }

    EXPECT_EQ(db.get_hash_path(512), memdb.get_hash_path(512));
}

TEST(crypto_merkle_tree, test_get_sibling_path)
{
    MemoryTree<PedersenHashPolicy> memdb(10);

    MemoryStore store;
    auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 10);

    EXPECT_EQ(memdb.get_sibling_path(512), db.get_sibling_path(512));

    memdb.update_element(512, VALUES[512]);
    db.update_element(512, VALUES[512]);

    EXPECT_EQ(db.get_sibling_path(512), memdb.get_sibling_path(512));

    for (size_t i = 0; i < 1024; ++i) {
        memdb.update_element(i, VALUES[i]);
        db.update_element(i, VALUES[i]);
    }

    EXPECT_EQ(db.get_sibling_path(512), memdb.get_sibling_path(512));
}

TEST(crypto_merkle_tree, test_get_hash_path_layers)
{
    {
        MemoryStore store;
        auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 3);

        auto before = db.get_hash_path(1);
        db.update_element(0, VALUES[1]);
        auto after = db.get_hash_path(1);

        EXPECT_NE(before[0], after[0]);
        EXPECT_NE(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }

    {
        MemoryStore store;
        auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 3);

        auto before = db.get_hash_path(7);
        db.update_element(0x0, VALUES[1]);
        auto after = db.get_hash_path(7);

        EXPECT_EQ(before[0], after[0]);
        EXPECT_EQ(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }
}

TEST(crypto_merkle_tree, test_get_sibling_path_layers)
{
    {
        MemoryStore store;
        auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 3);

        auto before = db.get_sibling_path(1);
        db.update_element(0, VALUES[1]);
        auto after = db.get_sibling_path(1);

        EXPECT_NE(before[0], after[0]);
        EXPECT_EQ(before[1], after[1]);
        EXPECT_EQ(before[2], after[2]);
    }

    {
        MemoryStore store;
        auto db = MerkleTree<MemoryStore, PedersenHashPolicy>(store, 3);

        auto before = db.get_sibling_path(7);
        db.update_element(0x0, VALUES[1]);
        auto after = db.get_sibling_path(7);

        EXPECT_EQ(before[0], after[0]);
        EXPECT_EQ(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }
}
