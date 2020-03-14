#include "leveldb_store.hpp"
#include "memory_store.hpp"
#include <gtest/gtest.h>
#include <stdlib/types/turbo.hpp>
#include <sys/random.h>

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;

static std::vector<std::string> VALUES = []() {
    std::vector<std::string> values(1024);
    for (size_t i = 0; i < 1024; ++i) {
        std::string v(64, 0);
        *(size_t*)v.data() = i;
        values[i] = v;
    }
    return values;
}();

TEST(stdlib_merkle_tree, test_leveldb_vs_memory_consistency)
{
    constexpr size_t depth = 10;
    MemoryStore memdb(depth);

    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", depth);

    std::vector<size_t> indicies(1 << depth);
    std::iota(indicies.begin(), indicies.end(), 0);
    std::random_device rd;
    std::mt19937 g(rd());
    std::shuffle(indicies.begin(), indicies.end(), g);

    for (size_t i = 0; i < indicies.size(); ++i) {
        size_t idx = indicies[i];
        memdb.update_element(idx, VALUES[idx]);
        db.update_element(idx, VALUES[idx]);
        EXPECT_EQ(db.get_element(idx), memdb.get_element(idx));
        EXPECT_EQ(db.get_hash_path(idx), memdb.get_hash_path(idx));
    }

    EXPECT_EQ(db.root(), memdb.root());
}

TEST(stdlib_merkle_tree, test_leveldb_update_members)
{
    MemoryStore memdb(10);

    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 10);

    for (size_t i = 0; i < 1024; ++i) {
        EXPECT_EQ(db.get_element(i), VALUES[0]);
    }
    for (size_t i = 0; i < 1024; ++i) {
        memdb.update_element(i, VALUES[i]);
        db.update_element(i, VALUES[i]);
    }
    for (size_t i = 0; i < 1024; ++i) {
        EXPECT_EQ(db.get_element(i), memdb.get_element(i));
    }

    EXPECT_TRUE((db.root() == memdb.root()));
}

TEST(stdlib_merkle_tree, test_leveldb_deep)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 64);

    for (size_t i = 0; i < 1024; ++i) {
        EXPECT_EQ(db.get_element(i), VALUES[0]);
    }
    for (size_t i = 0; i < 1024; ++i) {
        db.update_element(i, VALUES[i]);
    }
    for (size_t i = 0; i < 1024; ++i) {
        EXPECT_EQ(db.get_element(i), VALUES[i]);
    }
}

TEST(stdlib_merkle_tree, test_leveldb_forks)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 3);

    db.update_element(0, VALUES[0]);
    db.update_element(4, VALUES[4]);
    db.update_element(3, VALUES[3]);
    db.update_element(6, VALUES[6]);
    db.update_element(2, VALUES[2]);
    db.update_element(7, VALUES[7]);
    db.update_element(1, VALUES[1]);
    db.update_element(5, VALUES[5]);

    for (size_t i = 0; i < 8; ++i) {
        EXPECT_EQ(db.get_element(i), VALUES[i]);
    }
}

TEST(stdlib_merkle_tree, test_leveldb_deep_forks)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 128);

    db.update_element(15956002367106947048ULL, VALUES[1]);
    db.update_element(13261513317649820665ULL, VALUES[2]);
    db.update_element(11344316348679559144ULL, VALUES[3]);
    db.update_element(1485930635714443825ULL, VALUES[4]);
    db.update_element(18347723794972374003ULL, VALUES[5]);

    EXPECT_EQ(db.get_element(15956002367106947048ULL), VALUES[1]);
    EXPECT_EQ(db.get_element(13261513317649820665ULL), VALUES[2]);
    EXPECT_EQ(db.get_element(11344316348679559144ULL), VALUES[3]);
    EXPECT_EQ(db.get_element(1485930635714443825ULL), VALUES[4]);
    EXPECT_EQ(db.get_element(18347723794972374003ULL), VALUES[5]);
    EXPECT_EQ(db.get_element(18347723794972374002ULL), VALUES[0]);
}

TEST(stdlib_merkle_tree, test_leveldb_size)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 128);

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

TEST(stdlib_merkle_tree, test_leveldb_persistence)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());

    fr root;
    {
        LevelDbStore db("/tmp/leveldb_test", 128);
        db.update_element(0, VALUES[1]);
        db.update_element(1, VALUES[2]);
        db.update_element(2, VALUES[3]);
        root = db.root();
        db.commit();
    }
    {
        LevelDbStore db("/tmp/leveldb_test", 128);

        EXPECT_EQ(db.root(), root);
        EXPECT_EQ(db.size(), 3ULL);
        EXPECT_EQ(db.get_element(0), VALUES[1]);
        EXPECT_EQ(db.get_element(1), VALUES[2]);
        EXPECT_EQ(db.get_element(2), VALUES[3]);
    }
}

TEST(stdlib_merkle_tree, test_leveldb_update_1024_random)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 128);
    std::vector<std::pair<LevelDbStore::index_t, std::string>> entries;

    for (size_t i = 0; i < 1024; i++) {
        LevelDbStore::index_t index;
        int got_entropy = getentropy((void*)&index, sizeof(index));
        ASSERT(got_entropy == 0);
        db.update_element(index, VALUES[i]);
        entries.push_back(std::make_pair(index, VALUES[i]));
    }

    for (auto e : entries) {
        EXPECT_EQ(db.get_element(e.first), e.second);
    }
}

TEST(stdlib_merkle_tree, test_leveldb_get_hash_path)
{
    MemoryStore memdb(10);

    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    LevelDbStore db("/tmp/leveldb_test", 10);

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
        leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
        LevelDbStore db("/tmp/leveldb_test", 3);

        auto before = db.get_hash_path(1);
        db.update_element(0, VALUES[1]);
        auto after = db.get_hash_path(1);

        EXPECT_NE(before[0], after[0]);
        EXPECT_NE(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }

    {
        leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
        LevelDbStore db("/tmp/leveldb_test", 3);

        auto before = db.get_hash_path(7);
        db.update_element(0x0, VALUES[1]);
        auto after = db.get_hash_path(7);

        EXPECT_EQ(before[0], after[0]);
        EXPECT_EQ(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }
}