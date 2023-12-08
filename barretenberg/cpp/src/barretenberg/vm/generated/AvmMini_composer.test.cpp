#include "barretenberg/vm/generated/AvmMini_composer.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/flavor/generated/AvmMini_flavor.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/proof_system/circuit_builder/AvmMini_helper.hpp"
#include "barretenberg/proof_system/circuit_builder/AvmMini_trace.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"
#include "barretenberg/vm/generated/AvmMini_prover.hpp"
#include "barretenberg/vm/generated/AvmMini_verifier.hpp"

#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <string>
#include <vector>

using namespace proof_system::honk;

namespace example_relation_honk_composer {

class AvmMiniTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { barretenberg::srs::init_crs_factory("../srs_db/ignition"); };
};

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST_F(AvmMiniTests, basic)
{
    auto trace_builder = proof_system::AvmMiniTraceBuilder();
    auto circuit_builder = proof_system::AvmMiniCircuitBuilder();

    trace_builder.callDataCopy(0, 3, 2, std::vector<FF>{ 45, 23, 12 });

    //           Memory layout:    [0,0,45,23,12,0,0,0,....]
    trace_builder.add(2, 3, 4); // [0,0,45,23,68,0,0,0,....]
    trace_builder.add(4, 5, 5); // [0,0,45,23,68,68,0,0,....]
    trace_builder.add(5, 5, 5); // [0,0,45,23,68,136,0,0,....]
    trace_builder.add(5, 6, 7); // [0,0,45,23,68,136,0,136,0....]
    trace_builder.sub(7, 6, 8); // [0,0,45,23,68,136,0,136,136,0....]
    trace_builder.mul(8, 8, 8); // [0,0,45,23,68,136,0,136,136^2,0....]
    trace_builder.div(3, 5, 1); // [0,23*136^(-1),45,23,68,136,0,136,136^2,0....]
    trace_builder.div(1, 1, 9); // [0,23*136^(-1),45,23,68,136,0,136,136^2,1,0....]
    trace_builder.div(9, 0, 4); // [0,23*136^(-1),45,23,1/0,136,0,136,136^2,1,0....] Error: division by 0

    trace_builder.returnOP(1, 8);

    auto trace = trace_builder.finalize();
    circuit_builder.set_trace(std::move(trace));

    EXPECT_TRUE(circuit_builder.check_circuit());

    auto composer = AvmMiniComposer();
    auto prover = composer.create_prover(circuit_builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_verifier(circuit_builder);
    bool verified = verifier.verify_proof(proof);

    if (!verified) {
        proof_system::log_avmMini_trace(circuit_builder.rows, 0, 10);
    }

    EXPECT_TRUE(verified);
}

} // namespace example_relation_honk_composer