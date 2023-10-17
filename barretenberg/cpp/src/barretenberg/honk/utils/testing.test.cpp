#include "testing.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include <gtest/gtest.h>

namespace barretenberg::test_testing_utils {

TEST(HonkTestingUtils, ProverPolynomials)
{
    using Flavor = proof_system::honk::flavor::Ultra;
    auto [storage, prover_polynomials] =
        proof_system::honk::get_sequential_prover_polynomials<Flavor>(/*log_circuit_size=*/2, /*starting_value=*/0);
    EXPECT_EQ(storage[0][0], prover_polynomials._data[0][0]);
    EXPECT_EQ(storage[0][1], prover_polynomials._data[0][1]);
};

} // namespace barretenberg::test_testing_utils
