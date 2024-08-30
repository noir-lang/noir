#include "gate_separator.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/gtest.h>

using namespace bb;

TEST(GateSeparatorPolynomial, FullPowConsistency)
{
    constexpr size_t d = 5;
    std::vector<fr> betas(d);
    for (auto& beta : betas) {
        beta = fr::random_element();
    }

    GateSeparatorPolynomial<fr> poly(betas);
    std::array<fr, d> variables{};
    for (auto& u_i : variables) {
        u_i = fr::random_element();
        poly.partially_evaluate(u_i);
    }

    size_t beta_idx = 0;
    fr expected_eval = 1;
    for (auto& u_i : variables) {
        expected_eval *= fr(1) - u_i + u_i * poly.betas[beta_idx];
        beta_idx++;
    }

    EXPECT_EQ(poly.partial_evaluation_result, expected_eval);
}

TEST(GateSeparatorPolynomial, GateSeparatorPolynomialsOnPowers)
{
    std::vector<fr> betas{ 2, 4, 16 };
    GateSeparatorPolynomial<fr> poly(betas, betas.size());
    std::vector<fr> expected_values{ 1, 2, 4, 8, 16, 32, 64, 128 };
    EXPECT_EQ(expected_values, poly.beta_products);
}
