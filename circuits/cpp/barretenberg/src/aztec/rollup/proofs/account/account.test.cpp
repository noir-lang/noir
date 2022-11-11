#include "account.hpp"

#include "../../constants.hpp"
#include "../../fixtures/user_context.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "../notes/constants.hpp"
#include "../notes/native/index.hpp"

#include <common/streams.hpp>
#include <common/test.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup;
using namespace rollup::proofs;
using namespace rollup::proofs::account;
using namespace rollup::proofs::notes::native::account;

class account_tests : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto crs_factory = std::shared_ptr<waffle::ReferenceStringFactory>(
            new waffle::FileReferenceStringFactory("../srs_db/ignition"));
        init_proving_key(crs_factory, false);
        init_verification_key(crs_factory);
    }

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32);
        alice = rollup::fixtures::create_user_context();
        bob = rollup::fixtures::create_user_context();
    }

    /**
     * Add two account notes for the user.
     */
    void preload_account_notes()
    {
        tree->update_element(
            tree->size(),
            create_account_leaf_data(alice.alias_hash, alice.owner.public_key, alice.signing_keys[0].public_key));
        tree->update_element(
            tree->size(),
            create_account_leaf_data(alice.alias_hash, alice.owner.public_key, alice.signing_keys[1].public_key));
    }

    fr create_account_leaf_data(fr const& account_alias_hash,
                                grumpkin::g1::affine_element const& owner_key,
                                grumpkin::g1::affine_element const& signing_key)
    {
        return account_note{ account_alias_hash, owner_key, signing_key }.commit();
    }

    uint256_t compute_account_alias_hash_nullifier(fr const& account_alias_hash)
    {
        const std::vector<fr> hash_elements{ account_alias_hash };
        auto result =
            crypto::pedersen::compress_native(hash_elements, notes::GeneratorIndex::ACCOUNT_ALIAS_HASH_NULLIFIER);
        return uint256_t(result);
    }

    uint256_t compute_account_public_key_nullifier(grumpkin::g1::affine_element const& account_public_key)
    {
        return crypto::pedersen::compress_native({ account_public_key.x, account_public_key.y },
                                                 notes::GeneratorIndex::ACCOUNT_PUBLIC_KEY_NULLIFIER);
    }

    account_tx create_new_account_tx(const rollup::fixtures::user_context& user)
    {
        account_tx tx;
        tx.merkle_root = tree->root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.new_signing_pub_key_1 = user.signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = user.signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.create = true;
        tx.migrate = false;
        tx.account_note_index = 0;
        tx.signing_pub_key = user.owner.public_key;
        tx.account_note_path = tree->get_hash_path(0);
        tx.sign(user.owner);
        return tx;
    }

    account_tx create_migrate_account_tx(const rollup::fixtures::user_context& user,
                                         const rollup::fixtures::grumpkin_key_pair& new_account_key,
                                         const rollup::fixtures::grumpkin_key_pair new_signing_keys[2])
    {
        account_tx tx;
        tx.merkle_root = tree->root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = new_account_key.public_key;
        tx.new_signing_pub_key_1 = new_signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = new_signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.create = false;
        tx.migrate = true;
        tx.account_note_index = 0;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_note_path = tree->get_hash_path(0);
        tx.sign(user.signing_keys[0]);
        return tx;
    }

    account_tx create_add_signing_keys_account_tx(const rollup::fixtures::user_context& user,
                                                  const rollup::fixtures::grumpkin_key_pair new_signing_keys[2])
    {
        account_tx tx;
        tx.merkle_root = tree->root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.new_signing_pub_key_1 = new_signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = new_signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.create = false;
        tx.migrate = false;
        tx.account_note_index = 0;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_note_path = tree->get_hash_path(0);
        tx.sign(user.signing_keys[0]);
        return tx;
    }

    bool verify(account_tx& tx)
    {
        auto prover = new_account_prover(tx, false);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    struct verify_logic_result {
        bool valid;
        std::string err;
    };

    verify_logic_result verify_logic(account_tx& tx)
    {
        Composer composer(get_proving_key(), nullptr);
        account_circuit(composer, tx);
        if (composer.failed) {
            info("Circuit logic failed: " + composer.err);
        }
        return { !composer.failed, composer.err };
    }

    rollup::fixtures::user_context alice;
    rollup::fixtures::user_context bob;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
};

TEST_F(account_tests, test_create_account)
{
    auto tx = create_new_account_tx(alice);
    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_migrate_account)
{
    preload_account_notes();
    auto tx = create_migrate_account_tx(alice, bob.owner, bob.signing_keys);

    EXPECT_TRUE(verify_logic(tx).valid);
}

// Initial migration

TEST_F(account_tests, test_account_with_create_and_migrate_fails)
{
    auto tx = create_migrate_account_tx(alice, bob.owner, bob.signing_keys);
    tx.create = true;

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "cannot both create and migrate an account");
}

// Signature

TEST_F(account_tests, test_wrong_account_key_pair_fails)
{
    auto tx = create_new_account_tx(bob);
    auto keys = rollup::fixtures::create_key_pair(nullptr);
    tx.sign(keys); // sign the tx with the wrong signing private key

    EXPECT_FALSE(tx.account_public_key == keys.public_key);
    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

TEST_F(account_tests, test_migrate_account_with_account_key_fails)
{
    preload_account_notes();
    auto tx = create_migrate_account_tx(alice, bob.owner, bob.signing_keys);

    // Set the signing key to equal the owner public key and sign with the public key.
    // The signature will be correct, but the circuit will look for an account note
    // with the "wrong" owner public key
    tx.signing_pub_key = alice.owner.public_key;
    tx.sign(alice.owner);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

// Account membership

TEST_F(account_tests, test_alternative_signing_key_1)
{
    preload_account_notes();
    auto tx = create_add_signing_keys_account_tx(alice, bob.signing_keys);

    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_alternative_signing_key_2)
{
    preload_account_notes();
    auto tx = create_add_signing_keys_account_tx(alice, bob.signing_keys);
    tx.account_note_index = 1;
    tx.account_note_path = tree->get_hash_path(1);
    tx.sign(alice.signing_keys[1]);

    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_wrong_alias_hash_fails)
{
    preload_account_notes();
    auto tx = create_add_signing_keys_account_tx(alice, bob.signing_keys);
    // The circuit will calculate an 'old' account note with the wrong alias, so the membership check should fail.
    tx.alias_hash = rollup::fixtures::generate_alias_hash("penguin"); // it's actually "pebble"
    tx.sign(alice.signing_keys[0]);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

TEST_F(account_tests, test_account_key_equals_spending_key_1_fails)
{
    auto tx = create_new_account_tx(alice);
    tx.new_signing_pub_key_1 = alice.owner.public_key;
    tx.sign(alice.owner);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account note 1: public key matches spending key");
}

TEST_F(account_tests, test_account_key_equals_spending_key_2_fails)
{
    auto tx = create_new_account_tx(alice);
    tx.new_signing_pub_key_2 = alice.owner.public_key;
    tx.sign(alice.owner);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account note 2: public key matches spending key");
}

// Account public key

TEST_F(account_tests, test_migrate_to_new_account_public_key)
{
    preload_account_notes();
    auto new_keys = rollup::fixtures::create_key_pair(nullptr);
    auto tx = create_migrate_account_tx(alice, new_keys, alice.signing_keys);

    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_change_account_public_key_without_migrating_fails)
{
    preload_account_notes();
    auto tx = create_migrate_account_tx(alice, bob.owner, bob.signing_keys);
    tx.migrate = false;
    // regen signature as `nullifier_1 = 0` if migrate == false
    tx.sign(alice.signing_keys[0]);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "cannot change account keys unless migrating");
}

TEST_F(account_tests, test_create_account_when_account_exists_creates_nullifier_collision)
{
    preload_account_notes();
    auto tx = create_new_account_tx(alice);
    auto prover = new_account_prover(tx, false);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    EXPECT_TRUE(verify_logic(tx).valid);
    EXPECT_TRUE(verify_proof(proof));

    auto note1_commitment = account_note{ tx.alias_hash, tx.account_public_key, tx.new_signing_pub_key_1 }.commit();
    auto note2_commitment = account_note{ tx.alias_hash, tx.account_public_key, tx.new_signing_pub_key_2 }.commit();

    EXPECT_EQ(data.proof_id, ProofIds::ACCOUNT);
    EXPECT_EQ(data.note_commitment1, note1_commitment);
    EXPECT_EQ(data.note_commitment2, note2_commitment);
    EXPECT_EQ(data.nullifier2, compute_account_public_key_nullifier(alice.owner.public_key)); // public key of new acct
    EXPECT_EQ(data.public_value, 0);
    EXPECT_EQ(data.public_owner, fr(0));
    EXPECT_EQ(data.asset_id, uint256_t(0));
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.tx_fee, uint256_t(0));
    EXPECT_EQ(data.tx_fee_asset_id, uint256_t(0));
    EXPECT_EQ(data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(data.defi_root, fr(0));
    EXPECT_EQ(data.backward_link, fr(0));
    EXPECT_EQ(data.allow_chain, uint256_t(0));

    // Validate output nullifier = nullifier of alice's alias_hash
    EXPECT_EQ(data.nullifier1, compute_account_alias_hash_nullifier(alice.alias_hash));
}

TEST_F(account_tests, test_create_account_full_proof_and_detect_circuit_change)
{
    auto tx = create_new_account_tx(alice);
    auto prover = new_account_prover(tx, false);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    auto note1_commitment = account_note{ tx.alias_hash, tx.account_public_key, tx.new_signing_pub_key_1 }.commit();
    auto note2_commitment = account_note{ tx.alias_hash, tx.account_public_key, tx.new_signing_pub_key_2 }.commit();

    EXPECT_EQ(data.proof_id, ProofIds::ACCOUNT);
    EXPECT_EQ(data.note_commitment1, note1_commitment);
    EXPECT_EQ(data.note_commitment2, note2_commitment);
    EXPECT_EQ(data.nullifier1, compute_account_alias_hash_nullifier(tx.alias_hash));
    EXPECT_EQ(data.nullifier2, compute_account_public_key_nullifier(alice.owner.public_key)); // public key of new acct
    EXPECT_EQ(data.public_value, 0);
    EXPECT_EQ(data.public_owner, fr(0));
    EXPECT_EQ(data.asset_id, uint256_t(0));
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.tx_fee, uint256_t(0));
    EXPECT_EQ(data.tx_fee_asset_id, uint256_t(0));
    EXPECT_EQ(data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(data.defi_root, fr(0));
    EXPECT_EQ(data.backward_link, fr(0));
    EXPECT_EQ(data.allow_chain, uint256_t(0));

    EXPECT_TRUE(verify_proof(proof));
    // The below part detects change in the account circuit
    size_t number_of_gates_acc = get_number_of_gates();
    auto vk_hash_acc = get_verification_key()->sha256_hash();
    // If the below assertions fail, consider changing the variable is_circuit_change_expected to 1 in
    // rollup/constants.hpp and see if atleast the next power of two limit is not exceeded. Please change the constant
    // values accordingly and set is_circuit_change_expected to 0 in rollup/constants.hpp before merging.
    if (!(circuit_gate_count::is_circuit_change_expected)) {
        EXPECT_TRUE(number_of_gates_acc == circuit_gate_count::ACCOUNT)
            << "The gate count for the account circuit is changed.";
        EXPECT_TRUE(from_buffer<uint256_t>(vk_hash_acc) == circuit_vk_hash::ACCOUNT)
            << "The verification key hash for the account circuit is changed: " << from_buffer<uint256_t>(vk_hash_acc);
        // For the next power of two limit, we need to consider that we reserve four gates for adding
        // randomness/zero-knowledge
        EXPECT_TRUE(number_of_gates_acc <=
                    circuit_gate_next_power_of_two::ACCOUNT - waffle::ComposerBase::NUM_RESERVED_GATES)
            << "You have exceeded the next power of two limit for the account circuit.";
    } else {
        EXPECT_TRUE(number_of_gates_acc <=
                    circuit_gate_next_power_of_two::ACCOUNT - waffle::ComposerBase::NUM_RESERVED_GATES)
            << "You have exceeded the next power of two limit for the account circuit.";
    }
}

TEST_F(account_tests, test_migrate_account_full_proof)
{
    preload_account_notes();
    const auto& new_account_key = bob.owner;
    const auto& new_signing_keys = bob.signing_keys;
    auto tx = create_migrate_account_tx(alice, new_account_key, new_signing_keys);
    auto prover = new_account_prover(tx, false);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    auto note1_commitment = account_note{ .alias_hash = tx.alias_hash,
                                          .owner_key = new_account_key.public_key,
                                          .signing_key = new_signing_keys[0].public_key }
                                .commit();

    auto note2_commitment = account_note{ .alias_hash = tx.alias_hash,
                                          .owner_key = new_account_key.public_key,
                                          .signing_key = new_signing_keys[1].public_key }
                                .commit();

    EXPECT_EQ(data.proof_id, ProofIds::ACCOUNT);
    EXPECT_EQ(data.note_commitment1, note1_commitment);
    EXPECT_EQ(data.note_commitment2, note2_commitment);
    EXPECT_EQ(data.nullifier1, 0);
    // nullifier2 = public key of new account
    EXPECT_EQ(data.nullifier2, compute_account_public_key_nullifier(new_account_key.public_key));
    EXPECT_EQ(data.public_value, 0);
    EXPECT_EQ(data.public_owner, fr(0));
    EXPECT_EQ(data.asset_id, uint256_t(0));
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.tx_fee, uint256_t(0));
    EXPECT_EQ(data.tx_fee_asset_id, uint256_t(0));
    EXPECT_EQ(data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(data.defi_root, fr(0));
    EXPECT_EQ(data.backward_link, fr(0));
    EXPECT_EQ(data.allow_chain, uint256_t(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(account_tests, test_add_signing_keys_to_account_full_proof)
{
    preload_account_notes();
    const auto& new_signing_keys = bob.signing_keys;
    auto tx = create_add_signing_keys_account_tx(alice, new_signing_keys);

    auto prover = new_account_prover(tx, false);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    auto note1_commitment = account_note{ tx.alias_hash, tx.account_public_key, tx.new_signing_pub_key_1 }.commit();
    auto note2_commitment = account_note{ tx.alias_hash, tx.account_public_key, tx.new_signing_pub_key_2 }.commit();

    EXPECT_EQ(data.proof_id, ProofIds::ACCOUNT);
    EXPECT_EQ(data.note_commitment1, note1_commitment);
    EXPECT_EQ(data.note_commitment2, note2_commitment);
    EXPECT_EQ(data.nullifier1, uint256_t(0));
    EXPECT_EQ(data.nullifier2, uint256_t(0));
    EXPECT_EQ(data.public_value, uint256_t(0));
    EXPECT_EQ(data.public_owner, fr(0));
    EXPECT_EQ(data.asset_id, uint256_t(0));
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.tx_fee, uint256_t(0));
    EXPECT_EQ(data.tx_fee_asset_id, uint256_t(0));
    EXPECT_EQ(data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(data.defi_root, fr(0));
    EXPECT_EQ(data.backward_link, fr(0));
    EXPECT_EQ(data.allow_chain, uint256_t(0));

    EXPECT_TRUE(verify_proof(proof));
}
