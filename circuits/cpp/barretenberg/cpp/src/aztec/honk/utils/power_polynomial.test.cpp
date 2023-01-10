#include "power_polynomial.hpp"
#include <numeric/random/engine.hpp>
#include <gtest/gtest.h>

TEST(power_polynomial, test_full_polynomial_correctness)
{
    const size_t order = 17;
    const size_t n = size_t(1) << order;
    barretenberg::fr zeta = barretenberg::fr::random_element();
    barretenberg::polynomial power_polynomial = honk::power_polynomial::generate_vector(zeta, n);

    barretenberg::fr current_power = barretenberg::fr::one();
    for (size_t i = 0; i < n; i++) {
        EXPECT_EQ(power_polynomial[i], current_power);
        if (power_polynomial[i] != current_power) {
            break;
        }
        current_power *= zeta;
    }
}

TEST(power_polynomial, test_evaluation_correctness)
{
    const size_t order = 30;
    const size_t n = size_t(1) << order;
    barretenberg::fr zeta = barretenberg::fr::random_element();
    // Not using the debug engine, because we want to test randomly
    size_t random_index = static_cast<size_t>(numeric::random::get_engine().get_random_uint32()) % n;
    std::vector<barretenberg::fr> variables;
    for (size_t i = 0; i < order; i++) {
        variables.emplace_back((random_index >> i) & 1);
    }
    EXPECT_EQ(zeta.pow(static_cast<size_t>(random_index)),
              honk::power_polynomial::evaluate<barretenberg::fr>(zeta, variables));
}