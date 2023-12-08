#include "testing.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include <gtest/gtest.h>

namespace barretenberg::test_testing_utils {

TEST(HonkTestingUtils, ProverPolynomials)
{
    using Flavor = proof_system::honk::flavor::Ultra;
    auto [storage, prover_polynomials] =
        proof_system::honk::get_sequential_prover_polynomials<Flavor>(/*log_circuit_size=*/2, /*starting_value=*/0);
    auto& first_polynomial = prover_polynomials.get_all()[0];
    EXPECT_EQ(storage.get_all()[0][0], first_polynomial[0]);
    EXPECT_EQ(storage.get_all()[0][1], first_polynomial[1]);
};

} // namespace barretenberg::test_testing_utils
