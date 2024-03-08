#include "barretenberg/examples/join_split/notes/native/value/value_note.hpp"
#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/examples/join_split/constants.hpp"
#include "barretenberg/examples/join_split/types.hpp"
#include "barretenberg/examples/join_split/user_context.hpp"
#include "value_note.hpp"
#include <gtest/gtest.h>

namespace bb::join_split_example {
using namespace bb;
using namespace bb::stdlib;
using namespace bb::join_split_example::proofs::notes;
using namespace bb::join_split_example::proofs::notes::circuit::value;

class ValueNote : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(ValueNote, Commits)
{
    auto user = join_split_example::fixtures::create_user_context();
    Builder builder;

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

    EXPECT_FALSE(builder.failed());
    EXPECT_EQ(CircuitChecker::check(builder), true);
}

TEST_F(ValueNote, CommitsWith0Value)
{
    auto user = join_split_example::fixtures::create_user_context();
    Builder builder;

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

    EXPECT_EQ(CircuitChecker::check(builder), true);
}

TEST_F(ValueNote, CommitWithOversizedAssetIdFails)
{
    auto user = join_split_example::fixtures::create_user_context();
    Builder builder;

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

    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(CircuitChecker::check(builder), false);
}
} // namespace bb::join_split_example