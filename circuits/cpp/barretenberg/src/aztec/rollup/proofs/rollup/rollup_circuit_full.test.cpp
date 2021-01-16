#include "compute_circuit_data.hpp"
#include "create_rollup.hpp"
#include "rollup_proof_data.hpp"
#include "verify.hpp"
#include "../../fixtures/user_context.hpp"
#include "../inner_proof_data.hpp"
#include "../join_split/join_split.hpp"
#include "../join_split/join_split_circuit.hpp"
#include "../account/account.hpp"
#include "../notes/native/sign_notes.hpp"
#include "../notes/native/encrypt_note.hpp"
#include "../notes/native/account_note.hpp"
#include "../join_split/compute_circuit_data.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include <common/test.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>
#include <stdlib/merkle_tree/membership.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace notes::native;
using namespace plonk::stdlib::merkle_tree;

class rollup_tests_full : public ::testing::Test {
  protected:
    rollup_tests_full()
        : data_tree(store, DATA_TREE_DEPTH, 0)
        , null_tree(store, NULL_TREE_DEPTH, 1)
        , root_tree(store, ROOT_TREE_DEPTH, 2)
    {
        update_root_tree_with_data_root(0);
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        std::string CRS_PATH = "../srs_db/ignition";
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        old = std::cerr.rdbuf();
        // std::cerr.rdbuf(swallow.rdbuf());
        account_cd = account::compute_circuit_data(srs);
        join_split_cd = join_split::compute_circuit_data(srs);
        padding_proof = join_split_cd.padding_proof;
    }

    static void TearDownTestCase() { std::cerr.rdbuf(old); }

    uint32_t append_note(uint32_t value, uint32_t asset_id, uint32_t nonce)
    {
        value_note note = { value, asset_id, nonce, user.owner.public_key, user.note_secret };
        auto enc_note = encrypt_note(note);
        uint32_t index = static_cast<uint32_t>(data_tree.size());
        auto leaf_data = create_leaf_data(enc_note);
        data_tree.update_element(index, leaf_data);
        return index;
    }

    void append_notes(std::vector<uint32_t> const& values)
    {
        for (auto v : values) {
            append_note(v, asset_id, 0);
        }
    }

    std::vector<uint8_t> create_account_leaf_data(fr const& account_alias_id,
                                                  grumpkin::g1::affine_element const& owner_key,
                                                  grumpkin::g1::affine_element const& signing_key)
    {
        auto enc_note = encrypt_account_note({ account_alias_id, owner_key, signing_key });
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }

    void append_account_notes()
    {
        auto account_alias_id = fixtures::generate_account_alias_id(user.alias_hash, 1);
        data_tree.update_element(
            data_tree.size(),
            create_account_leaf_data(account_alias_id, user.owner.public_key, user.signing_keys[0].public_key));
        data_tree.update_element(
            data_tree.size(),
            create_account_leaf_data(account_alias_id, user.owner.public_key, user.signing_keys[1].public_key));
    }

    void update_root_tree_with_data_root(size_t index)
    {
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(index, data_root);
    }

    std::vector<uint8_t> create_join_split_proof(std::array<uint32_t, 2> in_note_idx,
                                                 std::array<uint32_t, 2> in_note_value,
                                                 std::array<uint32_t, 2> out_note_value,
                                                 uint256_t public_input = 0,
                                                 uint256_t public_output = 0,
                                                 uint32_t account_note_idx = 0,
                                                 uint32_t nonce = 0)
    {
        value_note input_note1 = { in_note_value[0], asset_id, nonce, user.owner.public_key, user.note_secret };
        value_note input_note2 = { in_note_value[1], asset_id, nonce, user.owner.public_key, user.note_secret };
        value_note output_note1 = { out_note_value[0], asset_id, nonce, user.owner.public_key, user.note_secret };
        value_note output_note2 = { out_note_value[1], asset_id, nonce, user.owner.public_key, user.note_secret };

        join_split::join_split_tx tx;
        tx.public_input = public_input + tx_fee;
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
        tx.account_private_key = user.owner.private_key;
        tx.asset_id = asset_id;
        tx.alias_hash = user.alias_hash;
        tx.nonce = nonce;

        uint8_t owner_address[] = { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                    0x00, 0xb4, 0x42, 0xd3, 0x7d, 0xd2, 0x93, 0xa4, 0x3a, 0xde, 0x80,
                                    0x43, 0xe5, 0xa5, 0xb9, 0x57, 0x0f, 0x75, 0xc5, 0x96, 0x04 };
        tx.input_owner = from_buffer<fr>(owner_address);
        tx.output_owner = fr::random_element(rand_engine);

        auto signer = nonce ? user.signing_keys[0] : user.owner;
        tx.signature = sign_notes(tx, signer, rand_engine);

        Composer composer =
            Composer(join_split_cd.proving_key, join_split_cd.verification_key, join_split_cd.num_gates);
        composer.rand_engine = rand_engine;
        join_split_circuit(composer, tx);
        auto prover = composer.create_unrolled_prover();
        auto join_split_proof = prover.construct_proof();

        return join_split_proof.proof_data;
    }

    std::vector<uint8_t> create_account_proof(uint32_t nonce = 0, uint32_t account_note_idx = 0)
    {
        account::account_tx tx;
        tx.merkle_root = data_tree.root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.num_new_keys = 2;
        tx.new_signing_pub_key_1 = user.signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = user.signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.nonce = nonce;
        tx.migrate = true;
        tx.gibberish = fr::random_element();
        tx.account_index = account_note_idx;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_path = data_tree.get_hash_path(account_note_idx);
        tx.sign(nonce ? user.signing_keys[0] : user.owner);

        Composer composer = Composer(account_cd.proving_key, account_cd.verification_key, account_cd.num_gates);
        composer.rand_engine = rand_engine;
        account_circuit(composer, tx);
        auto prover = composer.create_unrolled_prover();
        auto account_proof = prover.construct_proof();

        return account_proof.proof_data;
    }

    MemoryStore store;
    MerkleTree<MemoryStore> data_tree;
    MerkleTree<MemoryStore> null_tree;
    MerkleTree<MemoryStore> root_tree;
    fixtures::user_context user;
    numeric::random::Engine* rand_engine;
    static std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
    static join_split::circuit_data join_split_cd;
    static account::circuit_data account_cd;
    static std::vector<uint8_t> padding_proof;
    static std::streambuf* old;
    static std::stringstream swallow;
    const uint32_t asset_id = 1;
    const uint256_t tx_fee = 7;

  private:
    std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
    {
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }
};

std::shared_ptr<waffle::DynamicFileReferenceStringFactory> rollup_tests_full::srs;
join_split::circuit_data rollup_tests_full::join_split_cd;
account::circuit_data rollup_tests_full::account_cd;
std::vector<uint8_t> rollup_tests_full::padding_proof;
std::streambuf* rollup_tests_full::old;
std::stringstream rollup_tests_full::swallow;

// Full proofs.
HEAVY_TEST_F(rollup_tests_full, test_1_proof_in_1_rollup_full_proof)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);

    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, join_split_cd, account_cd, srs, "", true, false, false);
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.num_txs, 0UL);
    EXPECT_EQ(rollup_data.inner_proofs.size(), 1UL);

    auto tx_data = inner_proof_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
    EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
}

HEAVY_TEST_F(rollup_tests_full, test_1_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, join_split_cd, account_cd, srs, "", true, false, false);
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.num_txs, 0UL);
    EXPECT_EQ(rollup_data.inner_proofs.size(), 2UL);

    auto tx_data = inner_proof_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
    EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
}

HEAVY_TEST_F(rollup_tests_full, test_2_proofs_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto join_split_proof2 = create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, root_tree, rollup_size);

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, join_split_cd, account_cd, srs, "", true, false, false);
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 8UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee * 2 : 0UL);
    }
    EXPECT_EQ(rollup_data.num_txs, 0UL);
    EXPECT_EQ(rollup_data.inner_proofs.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = inner_proof_data(txs[i]);
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
        EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
    }
}

HEAVY_TEST_F(rollup_tests_full, test_1_js_proof_1_account_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto account_proof = create_account_proof();
    auto txs = std::vector{ join_split_proof, account_proof };

    auto rollup = create_rollup(txs, data_tree, null_tree, root_tree, rollup_size);

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, join_split_cd, account_cd, srs, "", true, false, false);
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 8UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.num_txs, 0UL);
    EXPECT_EQ(rollup_data.inner_proofs.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = inner_proof_data(txs[i]);
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
        EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
    }
}

HEAVY_TEST_F(rollup_tests_full, test_1_proof_in_3_of_4_rollup_full_proof)
{
    size_t rollup_size = 3;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, join_split_cd, account_cd, srs, "", true, false, false);
    auto result = verify_rollup(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0UL);
    EXPECT_EQ(rollup_data.rollup_size, 4UL);
    EXPECT_EQ(rollup_data.data_start_index, 8UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.num_txs, 0UL);
    EXPECT_EQ(rollup_data.inner_proofs.size(), 1UL);

    auto tx_data = inner_proof_data(join_split_proof);

    {
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

    for (size_t i = 1; i < rollup_data.inner_proofs.size(); ++i) {
        auto inner_data = rollup_data.inner_proofs[i];
        auto zero_arr = std::array<uint8_t, 64>();
        EXPECT_EQ(inner_data.public_input, uint256_t(0));
        EXPECT_EQ(inner_data.public_output, uint256_t(0));
        EXPECT_EQ(inner_data.new_note1, zero_arr);
        EXPECT_EQ(inner_data.new_note2, zero_arr);
        EXPECT_EQ(inner_data.nullifier1, uint256_t(0));
        EXPECT_EQ(inner_data.nullifier2, uint256_t(0));
        EXPECT_EQ(inner_data.input_owner, fr(0));
        EXPECT_EQ(inner_data.output_owner, fr(0));
    }
}

} // namespace rollup
} // namespace proofs
} // namespace rollup