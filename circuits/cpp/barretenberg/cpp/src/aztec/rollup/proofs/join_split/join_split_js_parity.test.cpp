#include "../../constants.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "index.hpp"
#include "../notes/native/index.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <stdlib/merkle_tree/index.hpp>
#include <crypto/sha256/sha256.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace barretenberg;
// using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs::notes::native;
using key_pair = rollup::fixtures::grumpkin_key_pair;

/**
 * This test mirrors the test in join_split_prover.test.ts
 */
class join_split_js_parity_tests : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_shared<waffle::ReferenceStringFactory>();
        init_proving_key(null_crs_factory, false);
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db/ignition");
        init_verification_key(std::move(crs_factory));
        info("vk hash: ", get_verification_key()->sha256_hash());
    }

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32);
    }

    void append_notes(std::vector<value::value_note> const& notes)
    {
        for (auto note : notes) {
            tree->update_element(tree->size(), note.commit());
        }
    }

    waffle::plonk_proof sign_and_create_proof(join_split_tx& tx, key_pair const& signing_key)
    {
        tx.signature = sign_join_split_tx(tx, signing_key);

        auto prover = new_join_split_prover(tx, false);
        return prover.construct_proof();
    }

    bool sign_and_verify(join_split_tx& tx, key_pair const& signing_key)
    {
        return verify_proof(sign_and_create_proof(tx, signing_key));
    }

    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
};

TEST_F(join_split_js_parity_tests, test_full_proof)
{
    auto private_key = from_buffer<grumpkin::fr>(std::vector<uint8_t>{
        0x0b, 0x9b, 0x3a, 0xde, 0xe6, 0xb3, 0xd8, 0x1b, 0x28, 0xa0, 0x88, 0x6b, 0x2a, 0x84, 0x15, 0xc7,
        0xda, 0x31, 0x29, 0x1a, 0x5e, 0x96, 0xbb, 0x7a, 0x56, 0x63, 0x9e, 0x17, 0x7d, 0x30, 0x1b, 0xeb });
    auto public_key = grumpkin::g1::one * private_key;
    auto note_secret = from_buffer<barretenberg::fr>(std::vector<uint8_t>{
        0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11,
        0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11 });
    auto input_nullifier1 = from_buffer<barretenberg::fr>(std::vector<uint8_t>{
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01 });
    auto input_nullifier2 = from_buffer<barretenberg::fr>(std::vector<uint8_t>{
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02 });

    value::value_note input_note1 = {
        .value = 100,
        .asset_id = 0,
        .account_required = 0,
        .owner = public_key,
        .secret = note_secret,
        .creator_pubkey = 0,
        .input_nullifier = input_nullifier1,
    };
    value::value_note input_note2 = {
        .value = 50,
        .asset_id = 0,
        .account_required = 0,
        .owner = public_key,
        .secret = note_secret,
        .creator_pubkey = 0,
        .input_nullifier = input_nullifier2,
    };

    append_notes({ input_note1, input_note2 });

    auto input_note1_nullifier = compute_nullifier(input_note1.commit(), private_key, true);
    auto input_note2_nullifier = compute_nullifier(input_note2.commit(), private_key, true);
    value::value_note output_note1 = { 80, 0, 0, public_key, note_secret, 0, input_note1_nullifier };
    value::value_note output_note2 = { 50, 0, 0, public_key, note_secret, 0, input_note2_nullifier };

    join_split_tx tx;
    tx.proof_id = ProofIds::SEND;
    tx.public_value = 0;
    tx.public_owner = 0;
    tx.asset_id = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.old_data_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };
    tx.partial_claim_note.bridge_call_data = 0;
    tx.partial_claim_note.deposit_value = 0;
    tx.partial_claim_note.note_secret = 0;
    tx.partial_claim_note.input_nullifier = 0;
    tx.account_private_key = private_key;
    tx.alias_hash = rollup::fixtures::generate_alias_hash("penguin");
    tx.account_required = false;
    tx.account_note_index = 0;
    tx.account_note_path = tree->get_hash_path(0);
    tx.signing_pub_key = public_key;
    tx.backward_link = 0;
    tx.allow_chain = 0;
    tx.signature.e = { 0 };
    tx.signature.s = { 0 };

    // To assert that the C++ and TypeScript code produces the same input data.
    info("tx buffer hash: ", sha256::sha256(to_buffer(tx)));

    auto proof = sign_and_create_proof(tx, { private_key, public_key });
    auto proof_data = inner_proof_data(proof.proof_data);

    auto output_note1_commitment = tx.output_note[0].commit();
    auto output_note2_commitment = tx.output_note[1].commit();

    EXPECT_EQ(proof_data.proof_id, ProofIds::SEND);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, uint256_t(input_note1_nullifier));
    EXPECT_EQ(proof_data.nullifier2, uint256_t(input_note2_nullifier));
    EXPECT_EQ(proof_data.public_value, tx.public_value);
    EXPECT_EQ(proof_data.public_owner, tx.public_owner);
    EXPECT_EQ(proof_data.asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(20));
    EXPECT_EQ(proof_data.tx_fee_asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));

    // auto& stats = get_proving_key()->underlying_store.get_stats();
    // for (auto& e : stats) {
    //     info(e.first, ": reads: ", e.second.second, " writes: ", e.second.first);
    // }
}

} // namespace join_split
} // namespace proofs
} // namespace rollup