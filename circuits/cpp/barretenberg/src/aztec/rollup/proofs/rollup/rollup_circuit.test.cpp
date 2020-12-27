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
#include "../notes/constants.hpp"
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

class rollup_tests : public ::testing::Test {
  protected:
    rollup_tests()
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
        std::string CRS_PATH = "../srs_db";
        old = std::cerr.rdbuf();
        // std::cerr.rdbuf(swallow.rdbuf());
        account_cd = account::compute_circuit_data(CRS_PATH);
        join_split_cd = join_split::compute_circuit_data(CRS_PATH);
        padding_proof = join_split_cd.padding_proof;
        rollup_1_keyless = rollup::get_circuit_data(1, join_split_cd, account_cd, CRS_PATH, "", false, false, false);
        rollup_2_keyless = rollup::get_circuit_data(2, join_split_cd, account_cd, CRS_PATH, "", false, false, false);
    }

    static void TearDownTestCase() { std::cerr.rdbuf(old); }

    uint32_t append_note(uint32_t value)
    {
        value_note note = { user.owner.public_key, value, user.note_secret, 0, 0 };
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

    void nullify_account_alias_id(fr const& account_alias_id)
    {
        const std::vector<fr> hash_elements{
            fr(1),
            account_alias_id,
        };
        auto nullifier = crypto::pedersen::compress_native(hash_elements, notes::ACCOUNT_ALIAS_ID_HASH_INDEX);

        null_tree.update_element(uint256_t(nullifier), { 1 });
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
                                                 uint32_t account_note_idx = 0,
                                                 uint32_t nonce = 0)
    {
        value_note input_note1 = { user.owner.public_key, in_note_value[0], user.note_secret, 0, nonce };
        value_note input_note2 = { user.owner.public_key, in_note_value[1], user.note_secret, 0, nonce };
        value_note output_note1 = { user.owner.public_key, out_note_value[0], user.note_secret, 0, nonce };
        value_note output_note2 = { user.owner.public_key, out_note_value[1], user.note_secret, 0, nonce };

        join_split::join_split_tx tx;
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
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = user.alias_hash;
        tx.nonce = nonce;
        tx.asset_id = 0;

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
        join_split::join_split_circuit(composer, tx);
        if (composer.failed) {
            std::cout << "Join-split logic failed: " << composer.err << std::endl;
        }
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
        account::account_circuit(composer, tx);
        if (composer.failed) {
            std::cout << "Account logic failed: " << composer.err << std::endl;
        }
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
    static join_split::circuit_data join_split_cd;
    static account::circuit_data account_cd;
    static std::vector<uint8_t> padding_proof;
    static std::streambuf* old;
    static std::stringstream swallow;
    static rollup::circuit_data rollup_1_keyless;
    static rollup::circuit_data rollup_2_keyless;

  private:
    std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
    {
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }
};

join_split::circuit_data rollup_tests::join_split_cd;
account::circuit_data rollup_tests::account_cd;
std::vector<uint8_t> rollup_tests::padding_proof;
std::streambuf* rollup_tests::old;
std::stringstream rollup_tests::swallow;
rollup::circuit_data rollup_tests::rollup_1_keyless;
rollup::circuit_data rollup_tests::rollup_2_keyless;

TEST_F(rollup_tests, test_padding_proof)
{
    Composer composer = Composer(join_split_cd.proving_key, join_split_cd.verification_key, join_split_cd.num_gates);
    join_split::join_split_circuit(composer, join_split::noop_tx());
    auto verifier = composer.create_unrolled_verifier();
    EXPECT_TRUE(verifier.verify_proof({ padding_proof }));
}

TEST_F(rollup_tests, test_1_deposit_proof_in_1_rollup)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(join_split_cd, data_tree.root());

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_tests, test_0_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    auto rollup = create_padding_rollup(rollup_size, join_split_cd.padding_proof);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_tests, test_1_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_tests, test_1_proof_with_old_root_in_1_rollup)
{
    size_t rollup_size = 1;

    // Insert rollup 0 at index 1.
    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);

    // Create proof which references root at index 1.
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto data_root_index = 1U;

    // Insert rollup 1.
    append_notes({ 30, 40 });
    update_root_tree_with_data_root(2);

    // Create rollup 2 with old join-split.
    auto rollup =
        create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size, { data_root_index });

    inner_proof_data data(join_split_proof);
    EXPECT_TRUE(data.merkle_root != rollup.old_data_root);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_tests, test_1_proof_with_invalid_old_null_root_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    rollup.old_null_root = fr::random_element();

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_tests, test_incorrect_data_start_index_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    rollup.data_start_index = 0;

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_tests, test_bad_join_split_proof_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 0 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_tests, test_reuse_spent_note_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    inner_proof_data inner_proof_data(join_split_proof);
    null_tree.update_element(uint256_t(inner_proof_data.nullifier1), { 64, 1 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(verified);
}

// Account
TEST_F(rollup_tests, test_1_account_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    auto create_account = create_account_proof();
    auto rollup_0 = create_rollup({ create_account }, data_tree, null_tree, root_tree, rollup_size);
    EXPECT_TRUE(verify_rollup_logic(rollup_0, rollup_1_keyless));
}

TEST_F(rollup_tests, test_reuse_nullified_account_alias_id_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    auto account_alias_id = fixtures::generate_account_alias_id(user.alias_hash, 0);
    nullify_account_alias_id(account_alias_id);
    update_root_tree_with_data_root(1);

    auto account_proof = create_account_proof();
    auto rollup = create_rollup({ account_proof }, data_tree, null_tree, root_tree, rollup_size);

    EXPECT_FALSE(verify_rollup_logic(rollup, rollup_1_keyless));
}

// Rollups of size 2.
TEST_F(rollup_tests, test_1_proof_in_2_rollup)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 100, 50 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_tests, test_2_proofs_in_2_rollup)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(verified);
}

TEST_F(rollup_tests, test_create_rollup_picks_correct_data_start_index)
{
    size_t rollup_size = 2;

    append_account_notes();
    // Add a couple of additional notes taking total to 6.
    append_notes({ 100, 50, 0, 0 });
    update_root_tree_with_data_root(1);
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup({ join_split_proof }, data_tree, null_tree, root_tree, rollup_size);

    EXPECT_EQ(rollup.data_start_index, 8UL);
}

TEST_F(rollup_tests, test_same_input_note_in_two_proofs_fails)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 6, 5 }, { 80, 50 }, { 70, 60 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, root_tree, rollup_size);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(verified);
}

TEST_F(rollup_tests, test_nullifier_hash_path_consistency)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    update_root_tree_with_data_root(1);
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(txs, data_tree, null_tree, root_tree, rollup_size);

    std::swap(rollup.new_null_roots[2], rollup.new_null_roots[3]);
    std::swap(rollup.new_null_paths[2], rollup.new_null_paths[3]);
    std::swap(rollup.old_null_paths[2], rollup.old_null_paths[3]);

    auto verified = verify_rollup_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(verified);
}

} // namespace rollup
} // namespace proofs
} // namespace rollup