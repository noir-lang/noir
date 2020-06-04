#include "leveldb_store.hpp"
#include "leveldb_tree.hpp"
#include "membership.hpp"
#include "memory_store.hpp"
#include "memory_tree.hpp"
#include <gtest/gtest.h>
#include <leveldb/db.h>
#include <stdlib/types/turbo.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

const std::string DB_PATH = "/tmp/leveldb_test";

TEST(stdlib_merkle_tree, test_check_membership)
{
    LevelDbStore::destroy(DB_PATH);
    LevelDbStore store(DB_PATH);
    LevelDbTree db(store, 3);

    Composer composer = Composer();

    byte_array_ct zero(field_ct(witness_ct(&composer, fr::zero())));
    byte_array_ct value = zero;
    value.write(zero);
    field_ct root = witness_ct(&composer, db.root());

    bool_ct is_member =
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
    LevelDbStore::destroy(DB_PATH);
    LevelDbStore store(DB_PATH);
    LevelDbTree db(store, 3);

    Composer composer = Composer();

    byte_array_ct zero(field_ct(witness_ct(&composer, fr::zero())));
    byte_array_ct value = zero;
    value.write(zero);
    field_ct root = witness_ct(&composer, db.root());

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
    LevelDbStore::destroy(DB_PATH);
    LevelDbStore store(DB_PATH);
    LevelDbTree db(store, 3);

    Composer composer = Composer();

    byte_array_ct zero(field_ct(witness_ct(&composer, fr::zero())));
    byte_array_ct value(field_ct(witness_ct(&composer, fr::one())));
    value.write(zero);
    field_ct root = witness_ct(&composer, db.root());

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
    LevelDbStore::destroy(DB_PATH);
    LevelDbStore store(DB_PATH);
    LevelDbTree db(store, 3);

    Composer composer = Composer();

    byte_array_ct zero(field_ct(witness_ct(&composer, fr::zero())));

    byte_array_ct old_value = zero;
    old_value.write(zero);
    hash_path<Composer> old_path = create_witness_hash_path(composer, db.get_hash_path(0));
    field_ct old_root = witness_ct(&composer, db.root());

    byte_array_ct new_value(field_ct(witness_ct(&composer, fr::one())));
    new_value.write(zero);
    auto new_path_fr = get_new_hash_path(db.get_hash_path(0), 0, new_value.get_value());
    hash_path<Composer> new_path = create_witness_hash_path(composer, new_path_fr);
    field_ct new_root = witness_ct(&composer, get_hash_path_root(new_path_fr));

    update_membership(composer, new_root, new_path, new_value, old_root, old_path, old_value, zero);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_merkle_tree, test_tree)
{
    size_t depth = 3;
    size_t num = 1UL << depth;
    MemoryStore store;
    MerkleTree db(store, depth);
    MemoryTree mem_tree(depth);

    Composer composer = Composer();

    auto zero_field = field_ct(witness_ct(&composer, fr::zero()));
    auto zero = byte_array_ct(&composer).write(zero_field).write(zero_field);
    auto values = std::vector<byte_array_ct>(num, zero);
    auto root = field_ct(&composer, mem_tree.root());

    assert_check_tree(composer, root, values);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}