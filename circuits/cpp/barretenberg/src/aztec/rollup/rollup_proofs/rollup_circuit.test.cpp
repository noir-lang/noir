#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "create_rollup.hpp"
#include "rollup_proof_data.hpp"
#include "verify_rollup.hpp"
#include <common/test.hpp>
#include <rollup/client_proofs/join_split/join_split.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>
#include <stdlib/merkle_tree/membership.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::merkle_tree;

std::string CRS_PATH = "../srs_db";

class rollup_proofs_rollup_circuit : public ::testing::Test {
  protected:
    rollup_proofs_rollup_circuit()
        : data_tree(store, 32, 0)
        , null_tree(store, 128, 1)
        , root_tree(store, 28, 2)
        , user(rollup::tx::create_user_context())
    {
        update_root_tree_with_data_root(0);
    }

    static void SetUpTestCase()
    {
        old = std::cerr.rdbuf();
        // std::cerr.rdbuf(swallow.rdbuf());
        inner_circuit_data = compute_join_split_circuit_data(CRS_PATH);
        padding_proof = inner_circuit_data.padding_proof;
        rollup_1_keyless = compute_rollup_circuit_data(1, inner_circuit_data, false, CRS_PATH);
        rollup_2_keyless = compute_rollup_circuit_data(2, inner_circuit_data, false, CRS_PATH);
    }

    static void TearDownTestCase() { std::cerr.rdbuf(old); }

    uint32_t append_note(uint32_t value)
    {
        tx_note note = { user.public_key, value, user.note_secret };
        auto enc_note = encrypt_note(note);
        uint32_t index = static_cast<uint32_t>(data_tree.size());
        auto leaf_data = create_leaf_data(enc_note);
        data_tree.update_element(index, leaf_data);
        return index;
    }

    void update_root_tree_with_data_root(size_t index)
    {
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(index, data_root);
    }

    std::vector<uint8_t> create_join_split_proof(std::array<uint32_t, 2> in_note_idx,
                                                 std::array<uint32_t, 2> in_note_value,
                                                 std::array<uint32_t, 2> out_note_value,
                                                 uint32_t public_input = 0,
                                                 uint32_t public_output = 0)
    {
        tx_note input_note1 = { user.public_key, in_note_value[0], user.note_secret };
        tx_note input_note2 = { user.public_key, in_note_value[1], user.note_secret };

        tx_note output_note1 = { user.public_key, out_note_value[0], user.note_secret };
        tx_note output_note2 = { user.public_key, out_note_value[1], user.note_secret };

        join_split_tx tx;
        tx.owner_pub_key = user.public_key;
        tx.public_input = public_input;
        tx.public_output = public_output;
        tx.num_input_notes = 2;
        tx.input_index = { in_note_idx[0], in_note_idx[1] };
        tx.merkle_root = data_tree.root();
        tx.input_path = { data_tree.get_hash_path(in_note_idx[0]), data_tree.get_hash_path(in_note_idx[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                  { user.private_key, user.public_key });
        uint8_t owner_address[] = { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                    0x00, 0xb4, 0x42, 0xd3, 0x7d, 0xd2, 0x93, 0xa4, 0x3a, 0xde, 0x80,
                                    0x43, 0xe5, 0xa5, 0xb9, 0x57, 0x0f, 0x75, 0xc5, 0x96, 0x04 };
        tx.public_owner = from_buffer<fr>(owner_address);

        Composer composer =
            Composer(inner_circuit_data.proving_key, inner_circuit_data.verification_key, inner_circuit_data.num_gates);
        join_split_circuit(composer, tx);
        auto prover = composer.create_unrolled_prover();
        auto join_split_proof = prover.construct_proof();

        return join_split_proof.proof_data;
    }

    void add_proof_notes_to_data_tree(std::vector<uint8_t> const& proof)
    {
        auto encNote1 = std::vector(proof.data() + (2 * 32), proof.data() + (2 * 32 + 64));
        auto encNote2 = std::vector(proof.data() + (4 * 32), proof.data() + (2 * 32 + 64));
        data_tree.update_element(data_tree.size(), encNote1);
        data_tree.update_element(data_tree.size(), encNote2);
    }

    MemoryStore store;
    MerkleTree<MemoryStore> data_tree;
    MerkleTree<MemoryStore> null_tree;
    MerkleTree<MemoryStore> root_tree;
    rollup::tx::user_context user;
    static join_split_circuit_data inner_circuit_data;
    static std::vector<uint8_t> padding_proof;
    static std::streambuf* old;
    static std::stringstream swallow;
    static rollup_circuit_data rollup_1_keyless;
    static rollup_circuit_data rollup_2_keyless;

  private:
    std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
    {
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }
};

join_split_circuit_data rollup_proofs_rollup_circuit::inner_circuit_data;
std::vector<uint8_t> rollup_proofs_rollup_circuit::padding_proof;
std::streambuf* rollup_proofs_rollup_circuit::old;
std::stringstream rollup_proofs_rollup_circuit::swallow;
rollup_circuit_data rollup_proofs_rollup_circuit::rollup_1_keyless;
rollup_circuit_data rollup_proofs_rollup_circuit::rollup_2_keyless;

TEST_F(rollup_proofs_rollup_circuit, test_1_deposit_proof_in_1_rollup)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(inner_circuit_data, data_tree.root());

    auto rollup = create_rollup(0, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_1_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_1_proof_in_1_rollup_twice)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);

    // Two notes were added to data tree in create_rollup. Add the new data root to root tree.
    update_root_tree_with_data_root(2);
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 70, 80 }, { 90, 60 });
    auto rollup2 = create_rollup(2, { join_split_proof2 }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    verified = verify_rollup_logic(rollup2, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_1_proof_in_1_rollup_with_wrong_rollup_id_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);

    update_root_tree_with_data_root(2);
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 70, 80 }, { 90, 60 });
    auto rollup2 = create_rollup(1, { join_split_proof2 }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    verified = verify_rollup_logic(rollup2, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_1_proof_with_old_root_in_1_rollup)
{
    size_t rollup_size = 1;

    // Insert rollup 0 at index 1.
    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);

    // Create proof which references root at index 1.
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto data_root_index = 1U;

    // Insert rollup 1.
    append_note(30);
    append_note(40);
    update_root_tree_with_data_root(2);

    // Create rollup 2 with old join-split.
    auto rollup = create_rollup(
        2, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof, { data_root_index });

    join_split_data data(join_split_proof);
    EXPECT_TRUE(data.merkle_root != rollup.old_data_root);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_1_proof_with_invalid_old_root_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    rollup.old_null_root = fr::random_element();

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_bad_rollup_root_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    rollup.rollup_root = fr::random_element();

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_incorrect_data_start_index_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    rollup.data_start_index = 0;

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_bad_join_split_proof_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 0 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_reuse_spent_note_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    join_split_data join_split_data(join_split_proof);
    null_tree.update_element(join_split_data.nullifier1, { 64, 1 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_incorrect_new_data_root_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    rollup.new_data_root = fr::random_element();

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_incorrect_new_data_roots_root_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    rollup.new_data_roots_root = fr::random_element();

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

// Rollups of size 2.
TEST_F(rollup_proofs_rollup_circuit, test_1_proof_in_2_rollup)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_2_proofs_in_2_rollup)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(1, txs, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_insertion_of_subtree_at_non_empty_location_fails)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_same_input_note_in_two_proofs_fails)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 2, 1 }, { 80, 50 }, { 70, 60 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(1, txs, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_nullifier_hash_path_consistency)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(1, txs, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    std::swap(rollup.new_null_roots[2], rollup.new_null_roots[3]);
    std::swap(rollup.new_null_paths[2], rollup.new_null_paths[3]);
    std::swap(rollup.old_null_paths[2], rollup.old_null_paths[3]);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(verified);
}

// Full proofs.
HEAVY_TEST_F(rollup_proofs_rollup_circuit, test_1_proof_in_1_rollup_full_proof)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    update_root_tree_with_data_root(1);

    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, true, "../srs_db/ignition");
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.data_start_index, 2UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(rollup_data.num_txs, 1U);
    EXPECT_EQ(rollup_data.inner_proofs.size(), 1U);

    auto tx_data = join_split_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.public_owner, tx_data.public_owner);
}

HEAVY_TEST_F(rollup_proofs_rollup_circuit, test_2_proofs_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(1, txs, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, true, "../srs_db/ignition");
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(rollup_data.num_txs, txs.size());
    EXPECT_EQ(rollup_data.inner_proofs.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = join_split_data(txs[i]);
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.public_owner, tx_data.public_owner);
    }
}
