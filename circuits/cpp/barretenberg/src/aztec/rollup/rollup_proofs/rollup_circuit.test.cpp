#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "create_rollup.hpp"
#include "rollup_proof_data.hpp"
#include "verify_rollup.hpp"
#include <rollup/client_proofs/join_split/join_split.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>
#include <common/test.hpp>

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace rollup::client_proofs::join_split;

class rollup_proofs_rollup_circuit : public ::testing::Test {
  protected:
    rollup_proofs_rollup_circuit()
        : data_tree(store, 32, 0)
        , null_tree(store, 128, 1)
        , user(rollup::tx::create_user_context())
    {}

    static void SetUpTestCase()
    {
        old = std::cerr.rdbuf();
        std::cerr.rdbuf(swallow.rdbuf());
        inner_circuit_data = compute_join_split_circuit_data();
        padding_proof = create_noop_join_split_proof(inner_circuit_data);
    }

    static void TearDownTestCase()
    {
        std::cerr.rdbuf(old);
    }

    uint32_t append_note(uint32_t value)
    {
        tx_note note = { user.public_key, value, user.note_secret };
        auto enc_note = encrypt_note(note);
        uint32_t index = static_cast<uint32_t>(data_tree.size());
        auto leaf_data = create_leaf_data(enc_note);
        data_tree.update_element(index, leaf_data);
        return index;
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
    rollup::tx::user_context user;
    static join_split_circuit_data inner_circuit_data;
    static std::vector<uint8_t> padding_proof;
    static std::streambuf *old;
    static std::stringstream swallow;

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

TEST_F(rollup_proofs_rollup_circuit, test_1_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_bad_rollup_root_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    rollup.rollup_root = fr::random_element();

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_incorrect_data_start_index_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    rollup.data_start_index = 0;

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_bad_join_split_proof_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 0 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_reuse_spent_note_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    join_split_data join_split_data(join_split_proof);
    null_tree.update_element(join_split_data.nullifier1, { 64, 1 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_incorrect_new_data_root_fails)
{
    size_t rollup_size = 1;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    rollup.new_data_root = fr::random_element();

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

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
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_2_proofs_in_2_rollup)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_insertion_of_subtree_at_non_empty_location_fails)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_same_input_note_in_two_proofs_fails)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 2, 1 }, { 80, 50 }, { 70, 60 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_proofs_rollup_circuit, test_nullifier_hash_path_consistency)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, rollup_size, padding_proof);

    std::swap(rollup.new_null_roots[2], rollup.new_null_roots[3]);
    std::swap(rollup.new_null_paths[2], rollup.new_null_paths[3]);
    std::swap(rollup.old_null_paths[2], rollup.old_null_paths[3]);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, false);
    auto verified = verify_rollup_logic(rollup, rollup_circuit_data);

    EXPECT_FALSE(verified);
}

HEAVY_TEST_F(rollup_proofs_rollup_circuit, test_2_proofs_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_note(100);
    append_note(50);
    append_note(80);
    append_note(60);
    auto join_split_proof1 = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto join_split_proof2 = create_join_split_proof({ 2, 3 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, rollup_size, padding_proof);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, true);
    auto result = verify_rollup(rollup, rollup_circuit_data);

    EXPECT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.num_txs, txs.size());
    EXPECT_EQ(rollup_data.inner_proof_data.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = join_split_data(txs[i]);
        auto inner_data = rollup_data.inner_proof_data[i];
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    }
}
