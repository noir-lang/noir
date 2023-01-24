#include "field.hpp"
#include "pow.hpp"
#include <gtest/gtest.h>

#include <numeric/random/engine.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <stdlib/types/turbo.hpp>

namespace test_stdlib_field_pow {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using namespace plonk::stdlib::types::turbo;

TEST(stdlib_field_pow, test_pow)
{
    Composer composer;

    barretenberg::fr base_val(engine.get_random_uint256());
    uint32_t exponent_val = engine.get_random_uint32();

    field_ct base = witness_ct(&composer, base_val);
    uint32_ct exponent = witness_ct(&composer, exponent_val);

    field_ct result = plonk::stdlib::pow<Composer>(base, exponent);

    barretenberg::fr expected = base_val.pow(exponent_val);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_field_pow, test_pow_zero)
{
    Composer composer;

    barretenberg::fr base_val(engine.get_random_uint256());
    uint32_t exponent_val = 0;

    field_ct base = witness_ct(&composer, base_val);
    uint32_ct exponent = witness_ct(&composer, exponent_val);

    field_ct result = plonk::stdlib::pow<Composer>(base, exponent);

    EXPECT_EQ(result.get_value(), barretenberg::fr(1));

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_field_pow, test_pow_one)
{
    Composer composer;

    barretenberg::fr base_val(engine.get_random_uint256());
    uint32_t exponent_val = 1;

    field_ct base = witness_ct(&composer, base_val);
    uint32_ct exponent = witness_ct(&composer, exponent_val);

    field_ct result = plonk::stdlib::pow<Composer>(base, exponent);

    EXPECT_EQ(result.get_value(), base_val);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_field_pow, test_pow_both_constant)
{
    Composer composer;

    const size_t num_gates_start = composer.n;

    barretenberg::fr base_val(engine.get_random_uint256());
    uint32_t exponent_val = engine.get_random_uint32();

    field_ct base(&composer, base_val);
    uint32_ct exponent(&composer, exponent_val);

    field_ct result = plonk::stdlib::pow<Composer>(base, exponent);

    barretenberg::fr expected = base_val.pow(exponent_val);

    EXPECT_EQ(result.get_value(), expected);

    const size_t num_gates_end = composer.n;

    EXPECT_EQ(num_gates_start, num_gates_end);
}

TEST(stdlib_field_pow, test_pow_base_constant)
{
    Composer composer;

    barretenberg::fr base_val(engine.get_random_uint256());
    uint32_t exponent_val = engine.get_random_uint32();

    field_ct base(&composer, base_val);
    uint32_ct exponent = witness_ct(&composer, exponent_val);

    field_ct result = plonk::stdlib::pow<Composer>(base, exponent);

    barretenberg::fr expected = base_val.pow(exponent_val);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_field_pow, test_pow_exponent_constant)
{
    Composer composer;

    barretenberg::fr base_val(engine.get_random_uint256());
    uint32_t exponent_val = engine.get_random_uint32();

    field_ct base = witness_ct(&composer, base_val);
    uint32_ct exponent(&composer, exponent_val);

    field_ct result = plonk::stdlib::pow<Composer>(base, exponent);

    barretenberg::fr expected = base_val.pow(exponent_val);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stdlib_field_pow