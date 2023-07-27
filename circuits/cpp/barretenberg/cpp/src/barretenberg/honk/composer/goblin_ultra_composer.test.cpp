#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

using namespace proof_system::honk;

namespace test_ultra_honk_composer {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

class GoblinUltraHonkComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }
};

/**
 * @brief Test proof construction/verification for a circuit with ECC op gates, public inputs, and basic arithmetic
 * gates
 *
 */
TEST_F(GoblinUltraHonkComposerTests, SimpleCircuit)
{
    auto builder = UltraCircuitBuilder();

    // Define an arbitrary number of operations/gates
    size_t num_ecc_ops = 3;
    size_t num_conventional_gates = 10;

    // Add some ecc op gates
    for (size_t i = 0; i < num_ecc_ops; ++i) {
        auto point = g1::affine_one * fr::random_element();
        auto scalar = fr::random_element();
        builder.queue_ecc_mul_accum(point, scalar);
    }

    // Add some conventional gates that utlize public inputs
    for (size_t i = 0; i < num_conventional_gates; ++i) {
        fr a = fr::random_element();
        fr b = fr::random_element();
        fr c = fr::random_element();
        fr d = a + b + c;
        uint32_t a_idx = builder.add_public_variable(a);
        uint32_t b_idx = builder.add_variable(b);
        uint32_t c_idx = builder.add_variable(c);
        uint32_t d_idx = builder.add_variable(d);

        builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
    }

    auto composer = GoblinUltraComposer();
    auto prover = composer.create_prover(builder);
    auto verifier = composer.create_verifier(builder);
    auto proof = prover.construct_proof();
    bool verified = verifier.verify_proof(proof);
    EXPECT_EQ(verified, true);
}

} // namespace test_ultra_honk_composer
