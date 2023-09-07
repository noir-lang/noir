#include "pow.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/gtest.h>

namespace barretenberg::test_pow {

using FF = barretenberg::fr;

TEST(SumcheckPow, FullPowConsistency)
{
    constexpr size_t d = 5;

    FF zeta = FF::random_element();

    PowUnivariate<FF> pow_univariate(zeta);
    std::array<FF, d> variables{};
    for (auto& u_i : variables) {
        u_i = FF::random_element();
        pow_univariate.partially_evaluate(u_i);
    }
    
    FF zeta_power = zeta;
    FF expected_eval = 1;
    for (auto& u_i : variables) {
        expected_eval *= FF(1) - u_i + u_i * zeta_power;
        zeta_power *= zeta_power;
    }
    
    EXPECT_EQ(pow_univariate.partial_evaluation_constant, expected_eval);
}
} // namespace barretenberg::test_pow
