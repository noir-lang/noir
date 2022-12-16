#include "../../types/types.hpp"
#include <common/test.hpp>
#include <numeric/random/engine.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(stdlib_group, test_fixed_base_scalar_mul)
{
    auto scalar = uint256_t(123, 0, 0, 0);
    auto priv_key = grumpkin::fr(scalar);
    auto pub_key = crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_1).generator * priv_key;

    Composer composer;
    auto priv_key_witness = field_ct(witness_ct(&composer, fr(scalar)));

    auto result = group_ct::fixed_base_scalar_mul<128>(priv_key_witness, 0);

    EXPECT_EQ(result.x.get_value(), pub_key.x);
    EXPECT_EQ(result.y.get_value(), pub_key.y);

    auto native_result = crypto::pedersen::fixed_base_scalar_mul<128>(barretenberg::fr(scalar), 0);
    EXPECT_EQ(native_result.x, pub_key.x);
    EXPECT_EQ(native_result.y, pub_key.y);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_group, test_fixed_base_scalar_mul_zero_fails)
{
    auto scalar = uint256_t(0, 0, 0, 0);

    Composer composer;
    auto priv_key_witness = field_ct(witness_ct(&composer, fr(scalar)));
    group_ct::fixed_base_scalar_mul<128>(priv_key_witness, 0);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, false);
    EXPECT_EQ(composer.err(), "input scalar to fixed_base_scalar_mul_internal cannot be 0");
}

TEST(stdlib_group, test_fixed_base_scalar_mul_with_two_limbs)
{
    const uint256_t scalar = engine.get_random_uint256();

    auto priv_key_low = (scalar.slice(0, 128));
    auto priv_key_high = (scalar.slice(128, 256));
    auto priv_key = grumpkin::fr(scalar);
    auto pub_key = grumpkin::g1::one * priv_key;
    pub_key = pub_key.normalize();
    Composer composer;
    auto priv_key_low_witness = field_ct(witness_ct(&composer, fr(priv_key_low)));
    auto priv_key_high_witness = field_ct(witness_ct(&composer, fr(priv_key_high)));

    auto result = group_ct::fixed_base_scalar_mul(priv_key_low_witness, priv_key_high_witness);

    EXPECT_EQ(result.x.get_value(), pub_key.x);
    EXPECT_EQ(result.y.get_value(), pub_key.y);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
