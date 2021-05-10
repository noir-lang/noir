#include "encrypt.hpp"
#include "../../constants.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::notes;
using namespace rollup::proofs::notes::circuit::value;

TEST(encrypt_note, encrypts)
{
    Composer composer = Composer();

    grumpkin::g1::element note_owner_pub_key = grumpkin::g1::element::random_element();

    fr view_key_value = fr::random_element();
    view_key_value.data[3] = view_key_value.data[3] & 0x03FFFFFFFFFFFFFFULL;
    view_key_value = view_key_value.to_montgomery_form();

    fr note_value = fr::random_element();
    note_value.data[3] = note_value.data[3] & 0x0FFFFFFFFFFFFFFFULL;
    note_value = note_value.to_montgomery_form();

    fr asset_id_value = 0xaabbccddULL;
    fr nonce_value = 1;

    grumpkin::g1::element left = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(
        note_value, GeneratorIndex::JOIN_SPLIT_NOTE_VALUE);
    grumpkin::g1::element right =
        crypto::pedersen::fixed_base_scalar_mul<250>(view_key_value, GeneratorIndex::JOIN_SPLIT_NOTE_SECRET);
    grumpkin::g1::element top =
        crypto::pedersen::fixed_base_scalar_mul<32>(asset_id_value, GeneratorIndex::JOIN_SPLIT_NOTE_ASSET_ID);
    grumpkin::g1::element top3 =
        crypto::pedersen::fixed_base_scalar_mul<32>(nonce_value, GeneratorIndex::JOIN_SPLIT_NOTE_NONCE);

    grumpkin::g1::element expected;
    expected = left + right;
    expected += top;
    expected += top3;
    expected = expected.normalize();

    grumpkin::g1::affine_element hashed_pub_key = crypto::pedersen::compress_to_point_native(
        note_owner_pub_key.x, note_owner_pub_key.y, GeneratorIndex::JOIN_SPLIT_NOTE_OWNER);

    expected += hashed_pub_key;
    expected = expected.normalize();

    field_ct view_key = witness_ct(&composer, view_key_value);
    field_ct value = witness_ct(&composer, note_value);
    field_ct note_owner_x = witness_ct(&composer, note_owner_pub_key.x);
    field_ct note_owner_y = witness_ct(&composer, note_owner_pub_key.y);
    field_ct asset_id = witness_ct(&composer, asset_id_value);
    field_ct nonce = witness_ct(&composer, nonce_value);
    witness_data plaintext{ { note_owner_x, note_owner_y }, value, view_key, asset_id, nonce };

    point_ct result = encrypt(plaintext);
    composer.assert_equal_constant(result.x.witness_index, expected.x);
    composer.assert_equal_constant(result.y.witness_index, expected.y);

    waffle::TurboProver prover = composer.create_prover();

    EXPECT_FALSE(composer.failed);
    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(encrypt_note, encrypts_with_0_value)
{
    Composer composer = Composer();

    grumpkin::g1::element note_owner_pub_key = grumpkin::g1::element::random_element();

    fr view_key_value = fr::random_element();
    view_key_value.data[3] = view_key_value.data[3] & 0x03FFFFFFFFFFFFFFULL;
    view_key_value = view_key_value.to_montgomery_form();

    fr note_value(0);
    fr asset_id_value = 0xaabbccddULL;
    fr nonce_value(0);

    grumpkin::g1::element expected = crypto::pedersen::fixed_base_scalar_mul<250>(view_key_value, 1);
    expected += crypto::pedersen::fixed_base_scalar_mul<32>(asset_id_value, 2);
    grumpkin::g1::affine_element hashed_pub_key =
        crypto::pedersen::compress_to_point_native(note_owner_pub_key.x, note_owner_pub_key.y, 3);

    expected += hashed_pub_key;
    expected = expected.normalize();

    field_ct view_key = witness_ct(&composer, view_key_value);
    field_ct value = witness_ct(&composer, note_value);
    field_ct note_owner_x = witness_ct(&composer, note_owner_pub_key.x);
    field_ct note_owner_y = witness_ct(&composer, note_owner_pub_key.y);
    field_ct asset_id = witness_ct(&composer, asset_id_value);
    field_ct nonce = witness_ct(&composer, nonce_value);

    witness_data plaintext{ { note_owner_x, note_owner_y }, value, view_key, asset_id, nonce };

    point_ct result = encrypt(plaintext);
    composer.assert_equal_constant(result.x.witness_index, expected.x);
    composer.assert_equal_constant(result.y.witness_index, expected.y);

    waffle::TurboProver prover = composer.create_prover();

    EXPECT_FALSE(composer.failed);
    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
