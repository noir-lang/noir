#include "compute_rollup_circuit_data.hpp"
#include "create_rollup.hpp"
#include "rollup_proof_data.hpp"
#include "verify_rollup.hpp"
#include "../../fixtures/user_context.hpp"
#include "../inner_proof_data.hpp"
#include "../join_split/join_split.hpp"
#include "../join_split/join_split_circuit.hpp"
#include "../notes/sign_notes.hpp"
#include "../join_split/compute_join_split_circuit_data.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
#include "../inner_proof_data.hpp"
#include <common/test.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>
#include <stdlib/merkle_tree/membership.hpp>

using namespace barretenberg;
using rollup::proofs::inner_proof_data;
using namespace rollup::proofs::rollup;
using namespace rollup::proofs::join_split;
using namespace rollup::proofs::account;
using namespace plonk::stdlib::merkle_tree;

class rollup_full_tests : public ::testing::Test {
  protected:
    rollup_full_tests()
        : data_tree(store, 32, 0)
        , null_tree(store, 128, 1)
        , root_tree(store, 28, 2)
    {
        update_root_tree_with_data_root(0);
        rand_engine = &numeric::random::get_debug_engine(true);
        user = rollup::fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        std::string CRS_PATH = "../srs_db/ignition";
        old = std::cerr.rdbuf();
        // std::cerr.rdbuf(swallow.rdbuf());
        account_cd = compute_account_circuit_data(CRS_PATH);
        join_split_cd = compute_join_split_circuit_data(CRS_PATH);
        padding_proof = join_split_cd.padding_proof;
    }

    static void TearDownTestCase() { std::cerr.rdbuf(old); }

    uint32_t append_note(uint32_t value)
    {
        tx_note note = { user.owner.public_key, value, user.note_secret, 0 };
        auto enc_note = encrypt_note(note);
        uint32_t index = static_cast<uint32_t>(data_tree.size());
        auto leaf_data = create_leaf_data(enc_note);
        data_tree.update_element(index, leaf_data);
        return index;
    }

    void append_notes(std::vector<uint32_t> const& values)
    {
        for (auto v : values) {
            append_note(v);
        }
    }

    std::vector<uint8_t> create_account_leaf_data(grumpkin::g1::affine_element const& owner_key,
                                                  grumpkin::g1::affine_element const& signing_key)
    {
        std::vector<uint8_t> buf;
        write(buf, owner_key.x);
        write(buf, signing_key.x);
        return buf;
    }

    void append_account_notes()
    {
        data_tree.update_element(data_tree.size(),
                                 create_account_leaf_data(user.owner.public_key, user.signing_keys[0].public_key));
        data_tree.update_element(data_tree.size(),
                                 create_account_leaf_data(user.owner.public_key, user.signing_keys[1].public_key));
    }

    void nullify_account(grumpkin::g1::affine_element const& owner_key, grumpkin::g1::affine_element const& signing_key)
    {
        auto data = create_account_leaf_data(owner_key, signing_key);
        auto nullifier = merkle_tree::hash_value_native(data);
        null_tree.update_element(uint128_t(nullifier), { 1 });
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
                                                 uint32_t public_output = 0,
                                                 uint32_t account_note_idx = 0)
    {
        tx_note input_note1 = { user.owner.public_key, in_note_value[0], user.note_secret, 0 };
        tx_note input_note2 = { user.owner.public_key, in_note_value[1], user.note_secret, 0 };
        tx_note output_note1 = { user.owner.public_key, out_note_value[0], user.note_secret, 0 };
        tx_note output_note2 = { user.owner.public_key, out_note_value[1], user.note_secret, 0 };

        join_split_tx tx;
        tx.public_input = public_input;
        tx.public_output = public_output;
        tx.num_input_notes = 2;
        tx.input_index = { in_note_idx[0], in_note_idx[1] };
        tx.old_data_root = data_tree.root();
        tx.input_path = { data_tree.get_hash_path(in_note_idx[0]), data_tree.get_hash_path(in_note_idx[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.account_index = account_note_idx;
        tx.account_path = data_tree.get_hash_path(account_note_idx);
        tx.signing_pub_key = user.signing_keys[0].public_key;

        uint8_t owner_address[] = { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                    0x00, 0xb4, 0x42, 0xd3, 0x7d, 0xd2, 0x93, 0xa4, 0x3a, 0xde, 0x80,
                                    0x43, 0xe5, 0xa5, 0xb9, 0x57, 0x0f, 0x75, 0xc5, 0x96, 0x04 };
        tx.input_owner = from_buffer<fr>(owner_address);
        tx.output_owner = fr::random_element(rand_engine);

        tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                  tx.output_owner,
                                  { user.signing_keys[0].private_key, user.signing_keys[0].public_key },
                                  rand_engine);

        Composer composer =
            Composer(join_split_cd.proving_key, join_split_cd.verification_key, join_split_cd.num_gates);
        composer.rand_engine = rand_engine;
        join_split_circuit(composer, tx);
        auto prover = composer.create_unrolled_prover();
        auto join_split_proof = prover.construct_proof();

        return join_split_proof.proof_data;
    }

    MemoryStore store;
    MerkleTree<MemoryStore> data_tree;
    MerkleTree<MemoryStore> null_tree;
    MerkleTree<MemoryStore> root_tree;
    rollup::fixtures::user_context user;
    numeric::random::Engine* rand_engine;
    static join_split_circuit_data join_split_cd;
    static account_circuit_data account_cd;
    static std::vector<uint8_t> padding_proof;
    static std::streambuf* old;
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

join_split_circuit_data rollup_full_tests::join_split_cd;
account_circuit_data rollup_full_tests::account_cd;
std::vector<uint8_t> rollup_full_tests::padding_proof;
std::streambuf* rollup_full_tests::old;
std::stringstream rollup_full_tests::swallow;

// Full proofs.
HEAVY_TEST_F(rollup_full_tests, test_1_proof_in_1_rollup_full_proof)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);

    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto rollup_circuit_data =
        compute_rollup_circuit_data(rollup_size, join_split_cd, account_cd, true, "../srs_db/ignition");
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(rollup_data.num_txs, 1U);
    EXPECT_EQ(rollup_data.inner_proofs.size(), 1U);

    auto tx_data = inner_proof_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
    EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
}

HEAVY_TEST_F(rollup_full_tests, test_1_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup(1, { join_split_proof }, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto rollup_circuit_data =
        compute_rollup_circuit_data(rollup_size, join_split_cd, account_cd, true, "../srs_db/ignition");
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(rollup_data.num_txs, 1U);
    EXPECT_EQ(rollup_data.inner_proofs.size(), 1U);

    auto tx_data = inner_proof_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
    EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
}

HEAVY_TEST_F(rollup_full_tests, test_2_proofs_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto join_split_proof2 = create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(1, txs, data_tree, null_tree, root_tree, rollup_size, padding_proof);

    auto rollup_circuit_data =
        compute_rollup_circuit_data(rollup_size, join_split_cd, account_cd, true, "../srs_db/ignition");
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 8UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(rollup_data.num_txs, txs.size());
    EXPECT_EQ(rollup_data.inner_proofs.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = inner_proof_data(txs[i]);
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
        EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
    }
}