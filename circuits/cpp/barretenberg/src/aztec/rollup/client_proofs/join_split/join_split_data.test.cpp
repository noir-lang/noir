#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "join_split_data.cpp"
#include "join_split.hpp"
#include "sign_notes.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::join_split;

class client_proofs_join_split_data : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(null_crs_factory));
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db");
        init_verification_key(std::move(crs_factory));
    }

    virtual void SetUp()
    {
        merkle_tree::LevelDbStore::destroy("/tmp/client_proofs_join_split_data_db");
        store = std::make_unique<merkle_tree::LevelDbStore>("/tmp/client_proofs_join_split_data_db");
        tree = std::make_unique<merkle_tree::LevelDbTree>(*store, 32);
        user = rollup::tx::create_user_context();
    }

    std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
    {
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }

    rollup::tx::user_context user;
    std::unique_ptr<merkle_tree::LevelDbStore> store;
    std::unique_ptr<merkle_tree::LevelDbTree> tree;
};

TEST_F(client_proofs_join_split_data, test_proof_to_data)
{
    tx_note input_note1 = { user.public_key, 60, user.note_secret };
    tx_note input_note2 = { user.public_key, 40, user.note_secret };
    auto enc_note1 = encrypt_note(input_note1);
    tree->update_element(0, create_leaf_data(enc_note1));
    auto enc_note2 = encrypt_note(input_note2);
    tree->update_element(1, create_leaf_data(enc_note2));

    tx_note output_note1 = { user.public_key, 80, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 100;
    tx.public_output = 20;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };
    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              { user.private_key, user.public_key });

    auto prover = new_join_split_prover(tx);
    auto proof = prover.construct_proof();
    auto data = join_split_data(proof.proof_data);

    EXPECT_EQ(data.public_input, tx.public_input);
    EXPECT_EQ(data.public_output, tx.public_output);
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.nullifier1, static_cast<uint128_t>(0x8e918141efe8189b) << 64 | 0x9304085fa8822c2b);
    EXPECT_EQ(data.nullifier2, static_cast<uint128_t>(0x4cc6a449b48527bc) << 64 | 0x3d02fd1e11a213bb);
}
