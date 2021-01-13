#include <gtest/gtest.h>
#include "../../../fixtures/user_context.hpp"
#include "../native/compute_nullifier.hpp"
#include "../native/encrypt_note.hpp"
#include "../circuit/compute_nullifier.hpp"
#include "../circuit/encrypt_note.hpp"
#include <stdlib/types/turbo.hpp>

using namespace rollup::proofs::notes;
using namespace plonk::stdlib::types::turbo;

TEST(compute_nullifier_circuit, native_consistency)
{
    auto user = rollup::fixtures::create_user_context();
    auto priv_key = uint256_t(user.owner.private_key);

    auto native_input_note = native::value_note{ 100, 0, 0, user.owner.public_key, user.note_secret };
    auto native_enc_note = native::encrypt_note(native_input_note);
    auto native_nullifier = native::compute_nullifier(native_enc_note, 1, priv_key, true);

    Composer composer;
    auto circuit_input_note = circuit::create_value_note_witness(composer, native_input_note);
    auto circuit_enc_note = circuit::encrypt_note(circuit_input_note);
    auto circuit_nullifier = circuit::compute_nullifier(circuit_enc_note,
                                                        field_ct(witness_ct(&composer, 1)),
                                                        field_ct(witness_ct(&composer, priv_key)),
                                                        bool_ct(witness_ct(&composer, true)));

    EXPECT_EQ(circuit_nullifier.get_value(), native_nullifier);
}