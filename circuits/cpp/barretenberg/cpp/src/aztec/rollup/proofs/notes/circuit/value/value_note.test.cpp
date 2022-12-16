#include "value_note.hpp"
#include "../../../../fixtures/user_context.hpp"
#include "../../native/value/value_note.hpp"
#include "../../constants.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types;
using namespace rollup::proofs::notes;
using namespace rollup::proofs::notes::circuit::value;

TEST(value_note, commits)
{
    auto user = rollup::fixtures::create_user_context();
    Composer composer = Composer();

    fr note_value = fr::random_element();
    note_value.data[3] = note_value.data[3] & 0x0FFFFFFFFFFFFFFFULL;
    note_value = note_value.to_montgomery_form();

    uint32_t asset_id_value = 666;
    bool account_required = true;

    native::value::value_note note = {
        note_value, asset_id_value, account_required, user.owner.public_key, user.note_secret, 0, fr::random_element()
    };
    auto expected = note.commit();
    auto circuit_note = circuit::value::value_note(witness_data(composer, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    Prover prover = composer.create_prover();

    EXPECT_FALSE(composer.failed());
    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(value_note, commits_with_0_value)
{
    auto user = rollup::fixtures::create_user_context();
    Composer composer = Composer();

    uint32_t asset_id_value = 0x2abbccddULL; // needs to be less than 30 bits

    native::value::value_note note = {
        .value = 0,
        .asset_id = asset_id_value,
        .account_required = false,
        .owner = user.owner.public_key,
        .secret = user.note_secret,
        .creator_pubkey = 0,
        .input_nullifier = fr::random_element(),
    };
    auto expected = note.commit();
    auto circuit_note = circuit::value::value_note(witness_data(composer, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    Prover prover = composer.create_prover();

    EXPECT_FALSE(composer.failed());
    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(value_note, commit_with_oversized_asset_id_fails)
{
    auto user = rollup::fixtures::create_user_context();
    Composer composer = Composer();

    native::value::value_note note = {
        .value = 0,
        .asset_id = (1 << 30),
        .account_required = false,
        .owner = user.owner.public_key,
        .secret = user.note_secret,
        .creator_pubkey = 0,
        .input_nullifier = fr::random_element(),
    };
    auto expected = note.commit();
    auto circuit_note = circuit::value::value_note(witness_data(composer, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    Prover prover = composer.create_prover();

    EXPECT_TRUE(composer.failed());
    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, false);
}