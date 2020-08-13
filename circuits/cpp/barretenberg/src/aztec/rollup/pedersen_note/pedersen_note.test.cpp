#include "pedersen_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::pedersen_note;

TEST(rollup_pedersen_note, test_new_pedersen_note)
{
    Composer composer = Composer();

    grumpkin::g1::element note_owner_pub_key = grumpkin::g1::element::random_element();

    fr view_key_value = fr::random_element();
    view_key_value.data[3] = view_key_value.data[3] & 0x03FFFFFFFFFFFFFFULL;
    view_key_value = view_key_value.to_montgomery_form();

    fr note_value = fr{ 9999, 0, 0, 0 }.to_montgomery_form();

    grumpkin::g1::element left = crypto::pedersen::fixed_base_scalar_mul<32>(note_value, 0);
    grumpkin::g1::element right = crypto::pedersen::fixed_base_scalar_mul<250>(view_key_value, 1);
    grumpkin::g1::element expected;
    expected = left + right;
    expected = expected.normalize();

    // TODO MAKE THIS HASH INDEX NOT ZERO
    grumpkin::g1::affine_element hashed_pub_key =
        crypto::pedersen::compress_to_point_native(note_owner_pub_key.x, note_owner_pub_key.y, 0);

    expected += hashed_pub_key;
    expected = expected.normalize();

    field_ct view_key = witness_ct(&composer, view_key_value);
    field_ct note_value_field = witness_ct(&composer, note_value);
    field_ct note_owner_x = witness_ct(&composer, note_owner_pub_key.x);
    field_ct note_owner_y = witness_ct(&composer, note_owner_pub_key.y);

    uint32_ct value(note_value_field);

    private_note plaintext{ { note_owner_x, note_owner_y }, value, view_key };

    public_note result = encrypt_note(plaintext);
    composer.assert_equal_constant(result.ciphertext.x.witness_index, expected.x);
    composer.assert_equal_constant(result.ciphertext.y.witness_index, expected.y);

    waffle::TurboProver prover = composer.create_prover();

    EXPECT_FALSE(composer.failed);
    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(rollup_pedersen_note, test_new_pedersen_note_zero)
{
    Composer composer = Composer();

    grumpkin::g1::element note_owner_pub_key = grumpkin::g1::element::random_element();

    fr view_key_value = fr::random_element();
    view_key_value.data[3] = view_key_value.data[3] & 0x03FFFFFFFFFFFFFFULL;
    view_key_value = view_key_value.to_montgomery_form();

    fr note_value = fr{ 0, 0, 0, 0 }.to_montgomery_form();

    grumpkin::g1::element expected = crypto::pedersen::fixed_base_scalar_mul<250>(view_key_value, 1);

    grumpkin::g1::affine_element hashed_pub_key =
        crypto::pedersen::compress_to_point_native(note_owner_pub_key.x, note_owner_pub_key.y);

    expected += hashed_pub_key;
    expected = expected.normalize();

    field_ct view_key = witness_ct(&composer, view_key_value);
    field_ct note_value_field = witness_ct(&composer, note_value);
    field_ct note_owner_x = witness_ct(&composer, note_owner_pub_key.x);
    field_ct note_owner_y = witness_ct(&composer, note_owner_pub_key.y);

    uint32_ct value(note_value_field);

    private_note plaintext{ { note_owner_x, note_owner_y }, value, view_key };

    public_note result = encrypt_note(plaintext);
    composer.assert_equal_constant(result.ciphertext.x.witness_index, expected.x);
    composer.assert_equal_constant(result.ciphertext.y.witness_index, expected.y);

    waffle::TurboProver prover = composer.create_prover();

    EXPECT_FALSE(composer.failed);
    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}