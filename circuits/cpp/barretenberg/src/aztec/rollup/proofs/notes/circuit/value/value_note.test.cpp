#include "value_note.hpp"
#include "../../../../fixtures/user_context.hpp"
#include "../../native/value/value_note.hpp"
#include "../../constants.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
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
    uint32_t nonce_value = 1;

    native::value::value_note note = { note_value,
                                       asset_id_value,
                                       nonce_value,
                                       user.owner.public_key,
                                       user.note_secret,
                                       0,
                                       fr::random_element() };
    auto expected = note.commit();
    auto circuit_note = circuit::value::value_note(witness_data(composer, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    waffle::TurboProver prover = composer.create_prover();

    EXPECT_FALSE(composer.failed);
    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(value_note, commits_with_0_value)
{
    auto user = rollup::fixtures::create_user_context();
    Composer composer = Composer();

    fr note_value(0);
    uint32_t asset_id_value = 0xaabbccddULL;
    uint32_t nonce_value(0);

    native::value::value_note note = { note_value,
                                       asset_id_value,
                                       nonce_value,
                                       user.owner.public_key,
                                       user.note_secret,
                                       0,
                                       fr::random_element() };
    auto expected = note.commit();
    auto circuit_note = circuit::value::value_note(witness_data(composer, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    waffle::TurboProver prover = composer.create_prover();

    EXPECT_FALSE(composer.failed);
    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
