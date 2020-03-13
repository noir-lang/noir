#include <gtest/gtest.h>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "schnorr.hpp"

namespace test_stdlib_schnorr {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

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
    point input{ witness_ct(&composer, grumpkin::g1::affine_one.x), witness_ct(&composer, grumpkin::g1::affine_one.y) };

    point output = plonk::stdlib::schnorr::variable_base_mul(input, scalar_bits);

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
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_string,
                                                                                                      account);

    bool first_result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    point pub_key{ witness_ct(&composer, account.public_key.x), witness_ct(&composer, account.public_key.y) };
    stdlib::schnorr::signature_bits sig = stdlib::schnorr::convert_signature(&composer, signature);
    bit_array_ct message = stdlib::schnorr::convert_message(&composer, message_string);
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
