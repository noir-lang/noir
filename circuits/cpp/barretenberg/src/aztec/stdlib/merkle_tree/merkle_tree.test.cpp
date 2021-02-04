#include "leveldb_store.hpp"
#include "merkle_tree.hpp"
#include "memory_store.hpp"
#include "memory_tree.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <numeric/random/engine.hpp>
#include <stdlib/types/turbo.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;

namespace {
auto& engine = numeric::random::get_debug_engine();
auto& random_engine = numeric::random::get_engine();
} // namespace

static std::vector<LevelDbTree::value_t> VALUES = []() {
    std::vector<LevelDbTree::value_t> values(1024);
    for (size_t i = 0; i < 1024; ++i) {
        LevelDbTree::value_t v(64, 0);
        *(size_t*)v.data() = i;
        values[i] = v;
    }
    return values;
}();

TEST(stdlib_merkle_tree, test_kv_memory_vs_memory_consistency)
{
    constexpr size_t depth = 10;
    MemoryTree memdb(depth);

    MemoryStore store;
    MerkleTree db(store, depth);

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
        EXPECT_EQ(db.get_element(idx), memdb.get_element(idx));
        EXPECT_EQ(db.get_hash_path(idx), memdb.get_hash_path(idx));
    }

    EXPECT_EQ(db.root(), memdb.root());
}

TEST(stdlib_merkle_tree, test_size)
{
    MemoryStore store;
    auto db = MerkleTree(store, 256);

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

TEST(stdlib_merkle_tree, test_get_hash_path)
{
    MemoryTree memdb(10);

    MemoryStore store;
    auto db = MerkleTree(store, 10);

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

TEST(stdlib_merkle_tree, test_leveldb_get_hash_path_layers)
{
    {
        MemoryStore store;
        auto db = MerkleTree(store, 3);

        auto before = db.get_hash_path(1);
        db.update_element(0, VALUES[1]);
        auto after = db.get_hash_path(1);

        EXPECT_NE(before[0], after[0]);
        EXPECT_NE(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }

    {
        MemoryStore store;
        auto db = MerkleTree(store, 3);

        auto before = db.get_hash_path(7);
        db.update_element(0x0, VALUES[1]);
        auto after = db.get_hash_path(7);

        EXPECT_EQ(before[0], after[0]);
        EXPECT_EQ(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }
}

#ifndef __wasm__
std::string DB_PATH = format("/tmp/leveldb_test_", random_engine.get_random_uint128());

TEST(stdlib_merkle_tree, test_leveldb_vs_memory_consistency)
{
    constexpr size_t depth = 10;
    MemoryTree memdb(depth);

    LevelDbStore::destroy(DB_PATH);
    LevelDbStore store(DB_PATH);
    LevelDbTree db(store, depth);

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
        EXPECT_EQ(db.get_element(idx), memdb.get_element(idx));
        EXPECT_EQ(db.get_hash_path(idx), memdb.get_hash_path(idx));
    }

    EXPECT_EQ(db.root(), memdb.root());

    LevelDbStore::destroy(DB_PATH);
}

TEST(stdlib_merkle_tree, test_leveldb_persistence)
{
    LevelDbStore::destroy(DB_PATH);

    fr root;
    {
        LevelDbStore store(DB_PATH);
        LevelDbTree db(store, 256);
        db.update_element(0, VALUES[1]);
        db.update_element(1, VALUES[2]);
        db.update_element(2, VALUES[3]);
        root = db.root();
        store.commit();
    }
    {
        LevelDbStore store(DB_PATH);
        LevelDbTree db(store, 256);

        EXPECT_EQ(db.root(), root);
        EXPECT_EQ(db.size(), 3ULL);
        EXPECT_EQ(db.get_element(0), VALUES[1]);
        EXPECT_EQ(db.get_element(1), VALUES[2]);
        EXPECT_EQ(db.get_element(2), VALUES[3]);
    }

    LevelDbStore::destroy(DB_PATH);
}
#endif