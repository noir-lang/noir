#include <gtest/gtest.h>
#include "../../../../fixtures/user_context.hpp"
#include "./compute_nullifier.hpp"
#include "./value_note.hpp"
#include "../../native/value/compute_nullifier.hpp"
#include "../../native/value/value_note.hpp"
#include <stdlib/types/turbo.hpp>

using namespace rollup::proofs::notes;
using namespace plonk::stdlib::types::turbo;

TEST(compute_nullifier_circuit, native_consistency)
{
    auto user = rollup::fixtures::create_user_context();
    auto priv_key = uint256_t(user.owner.private_key);

    auto native_input_note =
        native::value::value_note{ 100, 0, 0, user.owner.public_key, user.note_secret, 0, fr::random_element() };
    auto native_commitment = native_input_note.commit();
    auto native_nullifier = native::compute_nullifier(native_commitment, priv_key, true);

    Composer composer;
    auto circuit_witness_data = circuit::value::witness_data(composer, native_input_note);
    auto circuit_input_note = circuit::value::value_note(circuit_witness_data);
    auto circuit_nullifier = circuit::compute_nullifier(
        circuit_input_note.commitment, field_ct(witness_ct(&composer, priv_key)), bool_ct(witness_ct(&composer, true)));

    EXPECT_EQ(circuit_nullifier.get_value(), native_nullifier);
}