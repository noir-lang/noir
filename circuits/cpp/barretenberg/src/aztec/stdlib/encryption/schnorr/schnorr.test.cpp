#include "schnorr.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <gtest/gtest.h>
#include <stdlib/types/turbo.hpp>

namespace test_stdlib_schnorr {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;


TEST(stdlib_schnorr, convert_field_into_wnaf)
{
    Composer composer = Composer();

    grumpkin::fq scalar_mont = grumpkin::fq::random_element();
    grumpkin::fq scalar = scalar_mont.from_montgomery_form();
    scalar.data[2] = 0ULL;
    scalar.data[3] = 0ULL;
    scalar = scalar.to_montgomery_form();

    field_ct input(&composer, scalar);
    plonk::stdlib::schnorr::convert_field_into_wnaf(&composer, input);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_schnorr, test_scalar_mul_alternate)
{
    Composer composer = Composer();

    grumpkin::fr scalar_mont = grumpkin::fr::random_element();
    grumpkin::fr scalar = scalar_mont.from_montgomery_form();

    uint256_t scalar_low{ scalar.data[0], scalar.data[1], 0ULL, 0ULL };
    uint256_t scalar_high{ scalar.data[2], scalar.data[3], 0ULL, 0ULL };

    field_ct input_lo = witness_ct(&composer, scalar_low);
    field_ct input_hi = witness_ct(&composer, scalar_high);

    grumpkin::g1::element expected = grumpkin::g1::one * scalar_mont;
    expected = expected.normalize();
    point_ct point_input{ witness_ct(&composer, grumpkin::g1::affine_one.x),
                    witness_ct(&composer, grumpkin::g1::affine_one.y) };

    point_ct output = plonk::stdlib::schnorr::variable_base_mul(point_input, input_lo, input_hi);

    EXPECT_EQ(output.x.get_value(), expected.x);
    EXPECT_EQ(output.y.get_value(), expected.y);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}


TEST(stdlib_schnorr, test_scalar_mul)
{
    Composer composer = Composer();

    grumpkin::fr scalar_mont = grumpkin::fr::random_element();
    grumpkin::fr scalar = scalar_mont.from_montgomery_form();

    bit_array_ct scalar_bits(&composer, 256);
    for (size_t i = 0; i < 256; ++i) {
        scalar_bits[255 - i] = bool_ct(&composer, scalar.get_bit(i));
    }

    grumpkin::g1::element expected = grumpkin::g1::one * scalar_mont;
    expected = expected.normalize();
    point_ct input{ witness_ct(&composer, grumpkin::g1::affine_one.x),
                    witness_ct(&composer, grumpkin::g1::affine_one.y) };

    point_ct output = plonk::stdlib::schnorr::variable_base_mul(input, scalar_bits);

    EXPECT_EQ((output.x.get_value() == expected.x), true);
    EXPECT_EQ((output.y.get_value() == expected.y), true);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_schnorr, verify_signature)
{
    Composer composer = Composer();

    // erm...what do we do now?
    std::string message_string = "small msg";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_string,
                                                                                                      account);

    bool first_result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    point_ct pub_key{ witness_ct(&composer, account.public_key.x), witness_ct(&composer, account.public_key.y) };
    stdlib::schnorr::signature_bits sig = stdlib::schnorr::convert_signature(&composer, signature);
    byte_array_ct message(&composer, message_string);
    bool signature_result = stdlib::schnorr::verify_signature(message, pub_key, sig);

    EXPECT_EQ(signature_result, true);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
} // namespace test_stdlib_schnorr
