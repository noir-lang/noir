#include "pow.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/gtest.h>

namespace bb::test_pow {

using FF = bb::fr;

TEST(PowPolynomial, FullPowConsistency)
{
    constexpr size_t d = 5;
    std::vector<FF> betas(d);
    for (auto& beta : betas) {
        beta = FF::random_element();
    }

    PowPolynomial<FF> pow_polynomial(betas);
    std::array<FF, d> variables{};
    for (auto& u_i : variables) {
        u_i = FF::random_element();
        pow_polynomial.partially_evaluate(u_i);
    }

    size_t beta_idx = 0;
    FF expected_eval = 1;
    for (auto& u_i : variables) {
        expected_eval *= FF(1) - u_i + u_i * pow_polynomial.betas[beta_idx];
        beta_idx++;
    }

    EXPECT_EQ(pow_polynomial.partial_evaluation_result, expected_eval);
}

TEST(PowPolynomial, PowPolynomialsOnPowers)
{
    auto betas = std::vector<FF>{ 2, 4, 16 };
    auto pow = PowPolynomial(betas);
    pow.compute_values();
    auto expected_values = std::vector<FF>{ 1, 2, 4, 8, 16, 32, 64, 128 };
    EXPECT_EQ(expected_values, pow.pow_betas);
}
} // namespace bb::test_pow
