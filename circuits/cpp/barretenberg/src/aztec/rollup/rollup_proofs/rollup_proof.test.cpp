#include "../tx/user_context.hpp"
#include "../client_proofs/join_split/sign_notes.hpp"
#include "compute_rollup_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/types/turbo.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace plonk::stdlib::types::turbo;

waffle::plonk_proof create_join_split_proof()
{
    std::cout << "Creating join split proof." << std::endl;

    merkle_tree::LevelDbStore::destroy("/tmp/rollup_proofs");
    auto store = merkle_tree::LevelDbStore("/tmp/rollup_proofs");
    auto tree = merkle_tree::LevelDbTree(store, 32);
    auto user = rollup::tx::create_user_context();

    tx_note gibberish = { user.public_key, 0, fr::random_element() };
    tx_note output_note1 = { user.public_key, 100, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 100;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree.root();
    tx.input_path = { tree.get_hash_path(0), tree.get_hash_path(1) };
    tx.input_note = { gibberish, gibberish };
    tx.output_note = { output_note1, output_note2 };

    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              { user.private_key, user.public_key });

    Composer composer = Composer("../srs_db/ignition");
    join_split_circuit(composer, tx);

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();
    std::cout << "Done." << std::endl;

    auto verifier = composer.create_verifier();
    auto verified = verifier.verify_proof(proof);

    EXPECT_TRUE(verified);

    return proof;
}

TEST(rollup_proofs, test_rollup_1_proofs)
{
    auto join_split_proof = create_join_split_proof();
    auto rollup_circuit_data = compute_rollup_circuit_data(1);

    Composer composer =
        Composer(rollup_circuit_data.proving_key, rollup_circuit_data.verification_key, rollup_circuit_data.num_gates);
    rollup_circuit(composer, 1, { join_split_proof }, rollup_circuit_data.inner_verification_key);

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();

    auto verifier = composer.create_verifier();
    auto verified = verifier.verify_proof(proof);

    EXPECT_TRUE(verified);
}