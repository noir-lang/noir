#include "../../../../fixtures/user_context.hpp"
#include "../../constants.hpp"
#include "../../native/value/value_note.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "value_note.hpp"
#include <gtest/gtest.h>

namespace join_split_example {
using namespace barretenberg;
using namespace proof_system::plonk::stdlib;
using namespace join_split_example::proofs::notes;
using namespace join_split_example::proofs::notes::circuit::value;
TEST(value_note, commits)
{
    auto user = join_split_example::fixtures::create_user_context();
    auto builder = Builder();

    fr note_value = fr::random_element();
    note_value.data[3] = note_value.data[3] & 0x0FFFFFFFFFFFFFFFULL;
    note_value = note_value.to_montgomery_form();

    uint32_t asset_id_value = 666;
    bool account_required = true;

    native::value::value_note note = {
        note_value, asset_id_value, account_required, user.owner.public_key, user.note_secret, 0, fr::random_element()
    };
    auto expected = note.commit();
    auto circuit_note = circuit::value::value_note(witness_data(builder, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    auto composer = Composer();
    auto prover = composer.create_prover(builder);

    EXPECT_FALSE(builder.failed());
    printf("composer gates = %zu\n", builder.get_num_gates());
    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(value_note, commits_with_0_value)
{
    auto builder = Builder();

    auto user = join_split_example::fixtures::create_user_context();

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
    auto circuit_note = circuit::value::value_note(witness_data(builder, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    Composer composer = Composer();

    auto prover = composer.create_prover(builder);

    EXPECT_FALSE(builder.failed());
    printf("composer gates = %zu\n", builder.get_num_gates());
    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(value_note, commit_with_oversized_asset_id_fails)
{
    auto builder = Builder();

    auto user = join_split_example::fixtures::create_user_context();

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
    auto circuit_note = circuit::value::value_note(witness_data(builder, note));

    auto result = circuit_note.commitment;
    result.assert_equal(expected);

    Composer composer = Composer();
    auto prover = composer.create_prover(builder);

    EXPECT_TRUE(builder.failed());
    printf("composer gates = %zu\n", builder.get_num_gates());
    auto verifier = composer.create_verifier(builder);

    plonk::proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, false);
}
} // namespace join_split_example