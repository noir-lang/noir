#include <gtest/gtest.h>

#include <barretenberg/waffle/composer/turbo_composer.hpp>
#include <barretenberg/waffle/proof_system/preprocess.hpp>
#include <barretenberg/waffle/proof_system/prover/prover.hpp>
#include <barretenberg/waffle/proof_system/verifier/verifier.hpp>
#include <barretenberg/waffle/proof_system/widgets/arithmetic_widget.hpp>

#include <barretenberg/waffle/stdlib/bitarray/bitarray.hpp>
#include <barretenberg/waffle/stdlib/common.hpp>
#include <barretenberg/waffle/stdlib/crypto/schnorr/schnorr.hpp>
#include <barretenberg/waffle/stdlib/uint32/uint32.hpp>

#include <barretenberg/curves/grumpkin/grumpkin.hpp>
#include <barretenberg/misc_crypto/schnorr/schnorr.hpp>

#include <iostream>
#include <memory>

using namespace barretenberg;
using namespace plonk;

typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::uint32<waffle::TurboComposer> uint32;
typedef stdlib::bitarray<waffle::TurboComposer> bitarray;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

namespace test_stdlib_schnorr {
TEST(stdlib_schnorr, test_scalar_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    grumpkin::fr scalar_mont = grumpkin::fr::random_element();
    grumpkin::fr scalar = scalar_mont.from_montgomery_form();

    bitarray scalar_bits(&composer, 256);
    for (size_t i = 0; i < 256; ++i) {
        scalar_bits[255 - i] = bool_t(&composer, scalar.get_bit(i));
    }

    grumpkin::g1::element expected = grumpkin::g1::one * scalar_mont;
    expected = expected.normalize();
    plonk::stdlib::point input{ witness_t(&composer, grumpkin::g1::affine_one.x),
                                witness_t(&composer, grumpkin::g1::affine_one.y) };

    plonk::stdlib::point output = plonk::stdlib::schnorr::variable_base_mul(input, scalar_bits);

    EXPECT_EQ((output.x.get_value() == expected.x), true);
    EXPECT_EQ((output.y.get_value() == expected.y), true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_schnorr, verify_signature)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    // erm...what do we do now?
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            message_string, account);

    bool first_result =
        crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    stdlib::point pub_key{ witness_t(&composer, account.public_key.x), witness_t(&composer, account.public_key.y) };
    stdlib::schnorr::signature_bits sig = stdlib::schnorr::convert_signature(&composer, signature);
    stdlib::bitarray<waffle::TurboComposer> message = stdlib::schnorr::convert_message(&composer, message_string);
    bool signature_result = stdlib::schnorr::verify_signature(message, pub_key, sig);

    EXPECT_EQ(signature_result, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
} // namespace test_stdlib_schnorr
