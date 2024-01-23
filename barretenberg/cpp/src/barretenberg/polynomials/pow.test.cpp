#include "pow.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/gtest.h>

using namespace bb;

TEST(PowPolynomial, FullPowConsistency)
{
    constexpr size_t d = 5;
    std::vector<fr> betas(d);
    for (auto& beta : betas) {
        beta = fr::random_element();
    }

    PowPolynomial<fr> pow_polynomial(betas);
    std::array<fr, d> variables{};
    for (auto& u_i : variables) {
        u_i = fr::random_element();
        pow_polynomial.partially_evaluate(u_i);
    }

    size_t beta_idx = 0;
    fr expected_eval = 1;
    for (auto& u_i : variables) {
        expected_eval *= fr(1) - u_i + u_i * pow_polynomial.betas[beta_idx];
        beta_idx++;
    }

    EXPECT_EQ(pow_polynomial.partial_evaluation_result, expected_eval);
}

TEST(PowPolynomial, PowPolynomialsOnPowers)
{
    auto betas = std::vector<fr>{ 2, 4, 16 };
    auto pow = PowPolynomial(betas);
    pow.compute_values();
    auto expected_values = std::vector<fr>{ 1, 2, 4, 8, 16, 32, 64, 128 };
    EXPECT_EQ(expected_values, pow.pow_betas);
}
