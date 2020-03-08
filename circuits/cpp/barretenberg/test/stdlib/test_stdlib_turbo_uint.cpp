#include <gtest/gtest.h>

#include <barretenberg/waffle/composer/turbo_composer.hpp>
#include <barretenberg/waffle/proof_system/preprocess.hpp>
#include <barretenberg/waffle/proof_system/prover/prover.hpp>
#include <barretenberg/waffle/proof_system/verifier/verifier.hpp>

#include <barretenberg/waffle/stdlib/bool/bool.hpp>
#include <barretenberg/waffle/stdlib/common.hpp>
#include <barretenberg/waffle/stdlib/uint/uint.hpp>

#include "../test_helpers.hpp"
#include <iostream>
#include <memory>

using namespace barretenberg;
using namespace plonk;

namespace test_stlib_turbo_uint {
typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::uint<waffle::TurboComposer, uint32_t> uint32;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;

TEST(stdlib_turbo_uint32, test_add)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto add_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = engine.get_random_uint32();
        uint32_t expected = a_val + b_val;
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 c = a + b;
        c = c.normalize();

        uint32_t result = uint32_t(c.get_value());

        EXPECT_EQ(result, expected);
    };

    add_integers(false, false);
    add_integers(false, true);
    add_integers(true, false);
    add_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_sub)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto sub_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = engine.get_random_uint32();
        uint32_t const_shift_val = engine.get_random_uint32();
        uint32_t expected = a_val - (b_val + const_shift_val);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 b_shift = uint32(&composer, const_shift_val);
        uint32 c = b + b_shift;
        uint32 d = a - c;
        d = d.normalize();

        uint32_t result = uint32_t(d.get_value());

        EXPECT_EQ(result, expected);
    };

    sub_integers(false, false);
    sub_integers(false, true);
    sub_integers(true, false);
    sub_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto mul_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = engine.get_random_uint32();
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t expected = (a_val + const_a) * (b_val + const_b);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c * d;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    mul_integers(false, false);
    mul_integers(false, true);
    mul_integers(true, false);
    mul_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_divide)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto divide_integers = [&composer](bool lhs_constant = false,
                                             bool rhs_constant = false,
                                             bool dividend_is_divisor = false,
                                             bool dividend_zero = false,
                                             bool divisor_zero = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = dividend_is_divisor ? a_val : engine.get_random_uint32();
        uint32_t const_a = dividend_zero ? 0 - a_val : engine.get_random_uint32();
        uint32_t const_b =
            divisor_zero ? 0 - b_val : (dividend_is_divisor ? const_a : engine.get_random_uint32());
        uint32_t expected = (a_val + const_a) / (b_val + const_b);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c / d;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    divide_integers(false, false, false, false, false);
    divide_integers(false, true, false, false, false);
    divide_integers(true, false, false, false, false);
    divide_integers(true, true, false, false, false);

    divide_integers(false, false, true, false, false);
    divide_integers(false, true, true, false, false);
    divide_integers(true, false, true, false, false);
    divide_integers(true, true, true, false, false);

    divide_integers(false, false, false, true, false);
    divide_integers(false, true, false, true, false);
    divide_integers(true, false, false, true, false);
    divide_integers(true, true, false, true, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_modulo)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto mod_integers = [&composer](bool lhs_constant = false,
                                          bool rhs_constant = false,
                                          bool dividend_is_divisor = false,
                                          bool dividend_zero = false,
                                          bool divisor_zero = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = dividend_is_divisor ? a_val : engine.get_random_uint32();
        uint32_t const_a = dividend_zero ? 0 - a_val : engine.get_random_uint32();
        uint32_t const_b =
            divisor_zero ? 0 - b_val : (dividend_is_divisor ? const_a : engine.get_random_uint32());
        uint32_t expected = (a_val + const_a) % (b_val + const_b);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c % d;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    mod_integers(false, false, false, false, false);
    mod_integers(false, true, false, false, false);
    mod_integers(true, false, false, false, false);
    mod_integers(true, true, false, false, false);

    mod_integers(false, false, true, false, false);
    mod_integers(false, true, true, false, false);
    mod_integers(true, false, true, false, false);
    mod_integers(true, true, true, false, false);

    mod_integers(false, false, false, true, false);
    mod_integers(false, true, false, true, false);
    mod_integers(true, false, false, true, false);
    mod_integers(true, true, false, true, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_divide_by_zero_fails)
{

    const auto divide_integers = [](bool lhs_constant = false,
                                    bool rhs_constant = false,
                                    bool dividend_is_divisor = false,
                                    bool dividend_zero = false,
                                    bool divisor_zero = false) {
        waffle::TurboComposer composer = waffle::TurboComposer();

        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = dividend_is_divisor ? a_val : engine.get_random_uint32();
        uint32_t const_a = dividend_zero ? 0 - a_val : engine.get_random_uint32();
        uint32_t const_b =
            divisor_zero ? 0 - b_val : (dividend_is_divisor ? const_a : engine.get_random_uint32());
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c / d;
        e = e.normalize();

        waffle::TurboProver prover = composer.create_prover();

        waffle::TurboVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, false);
    };

    divide_integers(false, false, false, false, true);
    divide_integers(true, true, false, false, true);
}

TEST(stdlib_turbo_uint32, test_and)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto and_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = engine.get_random_uint32();
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t expected = (a_val + const_a) & (b_val + const_b);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c & d;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    and_integers(false, false);
    and_integers(false, true);
    and_integers(true, false);
    and_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_xor)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto xor_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = engine.get_random_uint32();
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t expected = (a_val + const_a) ^ (b_val + const_b);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c ^ d;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    xor_integers(false, false);
    xor_integers(false, true);
    xor_integers(true, false);
    xor_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_or)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto or_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val = engine.get_random_uint32();
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t expected = (a_val + const_a) | (b_val + const_b);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 b = rhs_constant ? uint32(&composer, b_val) : witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        uint32 e = c | d;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    or_integers(false, false);
    or_integers(false, false);
    or_integers(false, false);
    or_integers(false, false);
    or_integers(false, false);
    or_integers(false, true);
    or_integers(true, false);
    or_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_not)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto not_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
        uint32_t a_val = engine.get_random_uint32();
        uint32_t const_a = engine.get_random_uint32();
        uint32_t expected = ~(a_val + const_a);
        uint32 a = lhs_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        uint32 e = ~c;
        e = e.normalize();

        uint32_t result = uint32_t(e.get_value());

        EXPECT_EQ(result, expected);
    };

    not_integers(false, false);
    not_integers(false, true);
    not_integers(true, false);
    not_integers(true, true);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_gt)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto compare_integers = [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val;
        if (force_equal) {
            b_val = a_val + const_a - const_b;
        } else if (force_lt) {
            b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
        } else if (force_gt) {
            b_val = (a_val + const_a - const_b) == UINT32_MAX ? const_a - const_b + (a_val--)
                                                              : a_val - const_b + const_a + 1;
        } else {
            b_val = engine.get_random_uint32();
        }
        bool expected = (b_val + const_b) > (a_val + const_a);
        uint32 a = witness_t(&composer, a_val);
        uint32 b = witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        bool_t e = d > c;
        bool result = bool(e.get_value());

        EXPECT_EQ(result, expected);
    };

    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_lt)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto compare_integers = [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val;
        if (force_equal) {
            b_val = a_val + const_a - const_b;
        } else if (force_lt) {
            b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
        } else if (force_gt) {
            b_val = (a_val + const_a - const_b) == UINT32_MAX ? const_a - const_b + (a_val--)
                                                              : a_val - const_b + const_a + 1;
        } else {
            b_val = engine.get_random_uint32();
        }
        bool expected = (b_val + const_b) < (a_val + const_a);
        uint32 a = witness_t(&composer, a_val);
        uint32 b = witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        bool_t e = d < c;
        bool result = bool(e.get_value());

        EXPECT_EQ(result, expected);
    };

    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_gte)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto compare_integers = [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val;
        if (force_equal) {
            b_val = a_val + const_a - const_b;
        } else if (force_lt) {
            b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
        } else if (force_gt) {
            b_val = (a_val + const_a - const_b) == UINT32_MAX ? const_a - const_b + (a_val--)
                                                              : a_val - const_b + const_a + 1;
        } else {
            b_val = engine.get_random_uint32();
        }
        bool expected = (b_val + const_b) >= (a_val + const_a);
        uint32 a = witness_t(&composer, a_val);
        uint32 b = witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        bool_t e = d >= c;
        bool result = bool(e.get_value());
        EXPECT_EQ(result, expected);
    };

    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_lte)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto compare_integers = [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val;
        if (force_equal) {
            b_val = a_val + const_a - const_b;
        } else if (force_lt) {
            b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
        } else if (force_gt) {
            b_val = (a_val + const_a - const_b) == UINT32_MAX ? const_a - const_b + (a_val--)
                                                              : a_val - const_b + const_a + 1;
        } else {
            b_val = engine.get_random_uint32();
        }
        bool expected = (b_val + const_b) <= (a_val + const_a);
        uint32 a = witness_t(&composer, a_val);
        uint32 b = witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        bool_t e = d <= c;
        bool result = bool(e.get_value());

        EXPECT_EQ(result, expected);
    };

    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_equality_operator)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto compare_integers = [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val;
        if (force_equal) {
            b_val = a_val + const_a - const_b;
        } else if (force_lt) {
            b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
        } else if (force_gt) {
            b_val = (a_val + const_a - const_b) == UINT32_MAX ? const_a - const_b + (a_val--)
                                                              : a_val - const_b + const_a + 1;
        } else {
            b_val = engine.get_random_uint32();
        }
        bool expected = (b_val + const_b) == (a_val + const_a);
        uint32 a = witness_t(&composer, a_val);
        uint32 b = witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        bool_t e = d == c;
        bool result = bool(e.get_value());

        EXPECT_EQ(result, expected);
    };

    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_not_equality_operator)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto compare_integers = [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t const_b = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t b_val;
        if (force_equal) {
            b_val = a_val + const_a - const_b;
        } else if (force_lt) {
            b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
        } else if (force_gt) {
            b_val = (a_val + const_a - const_b) == UINT32_MAX ? const_a - const_b + (a_val--)
                                                              : a_val - const_b + const_a + 1;
        } else {
            b_val = engine.get_random_uint32();
        }
        bool expected = (b_val + const_b) != (a_val + const_a);
        uint32 a = witness_t(&composer, a_val);
        uint32 b = witness_t(&composer, b_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 b_shift = uint32(&composer, const_b);
        uint32 c = a + a_shift;
        uint32 d = b + b_shift;
        bool_t e = d != c;
        bool result = bool(e.get_value());

        EXPECT_EQ(result, expected);
    };

    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);
    compare_integers(false, false, true);
    compare_integers(false, true, false);
    compare_integers(true, false, false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_logical_not)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto not_integer = [&composer](bool force_zero) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t a_val = force_zero ? 0 - const_a : engine.get_random_uint32();
        bool expected = !(const_a + a_val);
        uint32 a = witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        bool_t e = !c;
        bool result = bool(e.get_value());

        EXPECT_EQ(result, expected);
    };

    not_integer(true);
    not_integer(true);
    not_integer(false);
    not_integer(false);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_right_shift)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto shift_integer = [&composer](const bool is_constant, const uint32_t shift) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t expected = (a_val + const_a) >> shift;
        uint32 a = is_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        uint32 d = c >> shift;
        uint32_t result = uint32_t(d.get_value());

        EXPECT_EQ(result, expected);
    };

    for (uint32_t i = 0; i < 32; ++i) {
        shift_integer(false, i);
        shift_integer(true, i);
    }
    printf("calling preprocess\n");
    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_left_shift)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto shift_integer = [&composer](const bool is_constant, const uint32_t shift) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t expected = (a_val + const_a) << shift;
        uint32 a = is_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        uint32 d = c << shift;
        uint32_t result = uint32_t(d.get_value());

        EXPECT_EQ(result, expected);
    };

    for (uint32_t i = 0; i < 32; ++i) {
        shift_integer(true, i);
        shift_integer(false, i);
    }

    printf("calling preprocess\n");
    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_ror)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto ror_integer = [&composer](const bool is_constant, const uint32_t rotation) {
        const auto ror = [](const uint32_t in, const uint32_t rval) {
            return rval ? (in >> rval) | (in << (32 - rval)) : in;
        };

        uint32_t const_a = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t expected = ror(const_a + a_val, rotation);
        uint32 a = is_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        uint32 d = c.ror(rotation);
        uint32_t result = uint32_t(d.get_value());

        EXPECT_EQ(result, expected);
    };

    for (uint32_t i = 0; i < 32; ++i) {
        ror_integer(true, i);
        ror_integer(false, i);
    }

    printf("calling preprocess\n");
    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_rol)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto rol_integer = [&composer](const bool is_constant, const uint32_t rotation) {
        const auto rol = [](const uint32_t in, const uint32_t rval) {
            return rval ? (in << rval) | (in >> (32 - rval)) : in;
        };

        uint32_t const_a = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t expected = rol(const_a + a_val, rotation);
        uint32 a = is_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        uint32 d = c.rol(rotation);
        uint32_t result = uint32_t(d.get_value());

        EXPECT_EQ(result, expected);
    };

    for (uint32_t i = 0; i < 32; ++i) {
        rol_integer(true, i);
        rol_integer(false, i);
    }

    printf("calling preprocess\n");
    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_turbo_uint32, test_at)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    const auto bit_test = [&composer](const bool is_constant) {
        uint32_t const_a = engine.get_random_uint32();
        uint32_t a_val = engine.get_random_uint32();
        uint32_t c_val = const_a + a_val;
        uint32 a = is_constant ? uint32(&composer, a_val) : witness_t(&composer, a_val);
        uint32 a_shift = uint32(&composer, const_a);
        uint32 c = a + a_shift;
        for (size_t i = 0; i < 32; ++i) {
            bool_t result = c.at(i);
            bool expected = (((c_val >> i) & 1UL) == 1UL) ? true : false;
            EXPECT_EQ(result.get_value(), expected);
        }
    };

    bit_test(false);
    bit_test(true);

    printf("calling preprocess\n");
    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stlib_turbo_uint