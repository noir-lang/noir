#include <gtest/gtest.h>

#include <barretenberg/polynomials/polynomial_arithmetic.hpp>
#include <barretenberg/waffle/composer/turbo_composer.hpp>
#include <barretenberg/waffle/proof_system/preprocess.hpp>
#include <barretenberg/waffle/proof_system/prover/prover.hpp>
#include <barretenberg/waffle/proof_system/verifier/verifier.hpp>
#include <barretenberg/waffle/proof_system/widgets/arithmetic_widget.hpp>
#include <barretenberg/waffle/stdlib/byte_array/byte_array.hpp>
#include <barretenberg/waffle/stdlib/crypto/hash/pedersen.hpp>
#include <barretenberg/waffle/stdlib/crypto/hash/sha256.hpp>
#include <barretenberg/waffle/stdlib/field/field.hpp>
#include <barretenberg/waffle/stdlib/merkle_tree/hash.hpp>
#include <barretenberg/waffle/stdlib/merkle_tree/merkle_tree.hpp>

#include <algorithm>
#include <memory>
#include <numeric>
#include <random>

namespace test_stdlib_merkle_tree {
using namespace barretenberg;
using namespace plonk;
using namespace plonk::stdlib::merkle_tree;

typedef waffle::TurboComposer Composer;
typedef stdlib::field_t<Composer> field_t;
typedef stdlib::bool_t<Composer> bool_t;
typedef stdlib::byte_array<Composer> byte_array;
typedef stdlib::witness_t<Composer> witness_t;

static std::vector<std::string> VALUES = []() {
    std::vector<std::string> values(1024);
    for (size_t i = 0; i < 1024; ++i) {
        std::string v(64, 0);
        *(size_t*)v.data() = i;
        values[i] = v;
    }
    return values;
}();

TEST(stdlib_merkle_tree, compress_native_vs_circuit)
{
    fr x = uint256_t(0x5ec473eb273a8011, 0x50160109385471ca, 0x2f3095267e02607d, 0x02586f4a39e69b86);
    Composer composer = Composer();
    witness_t y = witness_t(&composer, x);
    auto z = plonk::stdlib::pedersen::compress(y, y);
    auto zz = crypto::pedersen::compress_native(x, x);
    EXPECT_EQ(z.get_value(), zz);
}

TEST(stdlib_merkle_tree, hash_value_native_vs_circuit)
{
    std::string x = VALUES[1];
    Composer composer = Composer();
    byte_array y(&composer, x);
    field_t z = plonk::stdlib::merkle_tree::hash_value(y);
    fr zz = plonk::stdlib::merkle_tree::hash_value_native(x);
    EXPECT_EQ(z.get_value(), zz);
}

TEST(stdlib_merkle_tree, test_memory_store)
{
    fr e00 = plonk::stdlib::merkle_tree::hash_value_native(VALUES[1]);
    fr e01 = plonk::stdlib::merkle_tree::hash_value_native(VALUES[2]);
    fr e02 = plonk::stdlib::merkle_tree::hash_value_native(VALUES[3]);
    fr e03 = plonk::stdlib::merkle_tree::hash_value_native(VALUES[4]);
    fr e10 = stdlib::merkle_tree::compress_native({ e00, e01 });
    fr e11 = stdlib::merkle_tree::compress_native({ e02, e03 });
    fr root = stdlib::merkle_tree::compress_native({ e10, e11 });

    stdlib::merkle_tree::MemoryStore db(2);

    for (size_t i = 0; i < 4; ++i) {
        db.update_element(i, VALUES[i + 1]);
    }

    for (size_t i = 0; i < 4; ++i) {
        EXPECT_EQ(db.get_element(i), VALUES[i + 1]);
    }

    stdlib::merkle_tree::fr_hash_path expected = {
        std::make_pair(e00, e01),
        std::make_pair(e10, e11),
    };

    EXPECT_EQ(db.get_hash_path(0), expected);
    EXPECT_EQ(db.get_hash_path(1), expected);

    expected = {
        std::make_pair(e02, e03),
        std::make_pair(e10, e11),
    };

    EXPECT_EQ(db.get_hash_path(2), expected);
    EXPECT_EQ(db.get_hash_path(3), expected);
    EXPECT_EQ(db.root(), root);
}

TEST(stdlib_merkle_tree, test_leveldb_vs_memory_consistency)
{
    constexpr size_t depth = 10;
    stdlib::merkle_tree::MemoryStore memdb(depth);

    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", depth);

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
    stdlib::merkle_tree::MemoryStore memdb(10);

    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 10);

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
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 64);

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
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

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
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 128);

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
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 128);

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
        stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 128);
        db.update_element(0, VALUES[1]);
        db.update_element(1, VALUES[2]);
        db.update_element(2, VALUES[3]);
        root = db.root();
        db.commit();
    }
    {
        stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 128);

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
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 128);
    std::vector<std::pair<stdlib::merkle_tree::LevelDbStore::index_t, std::string>> entries;

    for (size_t i = 0; i < 1024; i++) {
        stdlib::merkle_tree::LevelDbStore::index_t index;
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
    stdlib::merkle_tree::MemoryStore memdb(10);

    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 10);

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
        stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

        auto before = db.get_hash_path(1);
        db.update_element(0, VALUES[1]);
        auto after = db.get_hash_path(1);

        EXPECT_NE(before[0], after[0]);
        EXPECT_NE(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }

    {
        leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
        stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

        auto before = db.get_hash_path(7);
        db.update_element(0x0, VALUES[1]);
        auto after = db.get_hash_path(7);

        EXPECT_EQ(before[0], after[0]);
        EXPECT_EQ(before[1], after[1]);
        EXPECT_NE(before[2], after[2]);
    }
}

TEST(stdlib_merkle_tree, test_check_membership)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

    Composer composer = Composer();

    byte_array zero = field_t(witness_t(&composer, fr::zero()));
    byte_array value = zero;
    value.write(zero);
    field_t root = witness_t(&composer, db.root());

    bool_t is_member =
        check_membership(composer, root, create_witness_hash_path(composer, db.get_hash_path(0)), value, zero);
    EXPECT_EQ(is_member.get_value(), true);

    auto prover = composer.create_prover();
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_merkle_tree, test_assert_check_membership)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

    Composer composer = Composer();

    byte_array zero = field_t(witness_t(&composer, fr::zero()));
    byte_array value = zero;
    value.write(zero);
    field_t root = witness_t(&composer, db.root());

    assert_check_membership(composer, root, create_witness_hash_path(composer, db.get_hash_path(0)), value, zero);

    auto prover = composer.create_prover();
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_merkle_tree, test_assert_check_membership_fail)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

    Composer composer = Composer();

    byte_array zero = field_t(witness_t(&composer, fr::zero()));
    byte_array value = field_t(witness_t(&composer, fr::one()));
    value.write(zero);
    field_t root = witness_t(&composer, db.root());

    assert_check_membership(composer, root, create_witness_hash_path(composer, db.get_hash_path(0)), value, zero);

    auto prover = composer.create_prover();
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, false);
}

TEST(stdlib_merkle_tree, test_update_members)
{
    leveldb::DestroyDB("/tmp/leveldb_test", leveldb::Options());
    stdlib::merkle_tree::LevelDbStore db("/tmp/leveldb_test", 3);

    Composer composer = Composer();

    byte_array zero = field_t(witness_t(&composer, fr::zero()));

    byte_array old_value = zero;
    old_value.write(zero);
    hash_path<Composer> old_path = create_witness_hash_path(composer, db.get_hash_path(0));
    field_t old_root = witness_t(&composer, db.root());

    byte_array new_value = field_t(witness_t(&composer, fr::one()));
    new_value.write(zero);
    auto new_path_fr = get_new_hash_path(db.get_hash_path(0), 0, new_value.get_value());
    hash_path<Composer> new_path = create_witness_hash_path(composer, new_path_fr);
    field_t new_root = witness_t(&composer, get_hash_path_root(new_path_fr));

    update_membership(composer, new_root, new_path, new_value, old_root, old_path, old_value, zero);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

} // namespace test_stdlib_merkle_tree