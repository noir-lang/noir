#include "../../native/value/compute_nullifier.hpp"
#include "../../../../fixtures/user_context.hpp"
#include "../../native/value/value_note.hpp"
#include "./compute_nullifier.hpp"
#include "./value_note.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include <gtest/gtest.h>

namespace bb::join_split_example {
using namespace bb::join_split_example::proofs::notes;

TEST(compute_nullifier_circuit, native_consistency)
{
    auto user = join_split_example::fixtures::create_user_context();
    auto priv_key = uint256_t(user.owner.private_key);

    auto native_input_note =
        native::value::value_note{ 100, 0, 0, user.owner.public_key, user.note_secret, 0, fr::random_element() };
    auto native_commitment = native_input_note.commit();
    auto native_nullifier = native::compute_nullifier(native_commitment, priv_key, true);
    Builder builder;
    auto circuit_witness_data = circuit::value::witness_data(builder, native_input_note);
    auto circuit_input_note = circuit::value::value_note(circuit_witness_data);
    auto circuit_nullifier = circuit::compute_nullifier(
        circuit_input_note.commitment, field_ct(witness_ct(&builder, priv_key)), bool_ct(witness_ct(&builder, true)));

    EXPECT_EQ(circuit_nullifier.get_value(), native_nullifier);
}
} // namespace bb::join_split_example