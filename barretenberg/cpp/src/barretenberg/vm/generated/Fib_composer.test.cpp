#include "barretenberg/vm/generated/Fib_composer.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/flavor/generated/Fib_flavor.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/Fib_trace.hpp"
#include "barretenberg/proof_system/plookup_tables/types.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"
#include "barretenberg/vm/generated/Fib_prover.hpp"
#include "barretenberg/vm/generated/Fib_verifier.hpp"
#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <string>
#include <vector>

using namespace proof_system::honk;

namespace example_relation_honk_composer {

class FibTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { barretenberg::srs::init_crs_factory("../srs_db/ignition"); };
};

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST_F(FibTests, powdre2e)
{
    barretenberg::srs::init_crs_factory("../srs_db/ignition");

    auto circuit_builder = proof_system::FibCircuitBuilder();

    auto rows = proof_system::FibTraceBuilder::build_trace();
    circuit_builder.set_trace(std::move(rows));

    auto composer = FibComposer();

    bool circuit_gud = circuit_builder.check_circuit();
    info("circuit gud");
    ASSERT_TRUE(circuit_gud);

    auto prover = composer.create_prover(circuit_builder);
    auto proof = prover.construct_proof();
    info(proof);

    auto verifier = composer.create_verifier(circuit_builder);
    bool verified = verifier.verify_proof(proof);
    ASSERT_TRUE(verified);

    info("We verified a proof!");
}

} // namespace example_relation_honk_composer