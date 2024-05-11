#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/vm/generated/spike_circuit_builder.hpp"
#include "barretenberg/vm/generated/spike_flavor.hpp"

// Proofs
#include "barretenberg/vm/generated/spike_composer.hpp"
#include "barretenberg/vm/generated/spike_prover.hpp"
#include "barretenberg/vm/generated/spike_verifier.hpp"

#include <gtest/gtest.h>

using namespace bb;
namespace {
auto& engine = numeric::get_debug_randomness();
}

class SpikePublicColumnsTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};

// Test file for testing public inputs evaluations are the same in the verifier and in sumcheck
//
// The first test runs the verification with the same public inputs in the verifier and in the prover, prover inputs are
// set in the below function The second failure test runs the verification with the different public inputs
bool verify_spike_with_public_with_public_inputs(std::vector<SpikeFlavor::FF> verifier_public__inputs)
{
    using Builder = SpikeCircuitBuilder;
    using Row = Builder::Row;
    Builder circuit_builder;

    srs::init_crs_factory("../srs_db/ignition");

    const size_t circuit_size = 16;
    std::vector<Row> rows;

    // Add to the public input column that is increasing
    for (size_t i = 0; i < circuit_size; i++) {
        // Make sure the external and trace public inputs are the same
        Row row{ .Spike_kernel_inputs__is_public = i + 1 };
        rows.push_back(row);
    }

    circuit_builder.set_trace(std::move(rows));

    // Create a prover and verifier
    auto composer = SpikeComposer();
    auto prover = composer.create_prover(circuit_builder);
    HonkProof proof = prover.construct_proof();

    auto verifier = composer.create_verifier(circuit_builder);

    return verifier.verify_proof(proof, verifier_public__inputs);
}

TEST(SpikePublicColumnsTests, VerificationSuccess)
{
    std::vector<SpikeFlavor::FF> public_inputs = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 };
    bool verified = verify_spike_with_public_with_public_inputs(public_inputs);
    ASSERT_TRUE(verified);
}

TEST(SpikePublicColumnsTests, VerificationFailure)
{
    std::vector<SpikeFlavor::FF> public_inputs = {
        10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160
    };
    bool verified = verify_spike_with_public_with_public_inputs(public_inputs);
    ASSERT_FALSE(verified);
}