#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "pow.hpp"
#include <gtest/gtest.h>

namespace proof_system::honk::sumcheck::pow_test {

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

    FF expected_eval = proof_system::honk::power_polynomial::evaluate<FF>(zeta, variables);
    EXPECT_EQ(pow_univariate.partial_evaluation_constant, expected_eval);
}
} // namespace proof_system::honk::sumcheck::pow_test
