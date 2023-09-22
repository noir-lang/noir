#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

using namespace proof_system::honk;

namespace test_ultra_honk_composer {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

class GoblinUltraHonkComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Point = Curve::AffineElement;
    using CommitmentKey = pcs::CommitmentKey<Curve>;

    /**
     * @brief Generate a simple test circuit with some ECC op gates and conventional arithmetic gates
     *
     * @param builder
     */
    void generate_test_circuit(auto& builder)
    {
        // Add some ecc op gates
        for (size_t i = 0; i < 3; ++i) {
            auto point = Point::one() * FF::random_element();
            auto scalar = FF::random_element();
            builder.queue_ecc_mul_accum(point, scalar);
        }
        builder.queue_ecc_eq();

        // Add some conventional gates that utilize public inputs
        for (size_t i = 0; i < 10; ++i) {
            FF a = FF::random_element();
            FF b = FF::random_element();
            FF c = FF::random_element();
            FF d = a + b + c;
            uint32_t a_idx = builder.add_public_variable(a);
            uint32_t b_idx = builder.add_variable(b);
            uint32_t c_idx = builder.add_variable(c);
            uint32_t d_idx = builder.add_variable(d);

            builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, FF(1), FF(1), FF(1), FF(-1), FF(0) });
        }
    }

    /**
     * @brief Construct a goblin ultra circuit then generate a verify its proof
     *
     * @param op_queue
     * @return auto
     */
    bool construct_test_circuit_then_generate_and_verify_proof(auto& op_queue)
    {
        auto builder = proof_system::GoblinUltraCircuitBuilder(op_queue);

        generate_test_circuit(builder);

        auto composer = GoblinUltraComposer();
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto verifier = composer.create_verifier(instance);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        return verified;
    }
};

/**
 * @brief Test proof construction/verification for a circuit with ECC op gates, public inputs, and basic arithmetic
 * gates
 * @note We simulate op queue interactions with a previous circuit so the actual circuit under test utilizes an op queue
 * with non-empty 'previous' data. This avoid complications with zero-commitments etc.
 *
 */
TEST_F(GoblinUltraHonkComposerTests, SingleCircuit)
{
    auto op_queue = std::make_shared<proof_system::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    // Construct a test circuit then generate and verify its proof
    auto verified = construct_test_circuit_then_generate_and_verify_proof(op_queue);

    EXPECT_EQ(verified, true);
}

/**
 * @brief Test proof construction/verification for a circuit with ECC op gates, public inputs, and basic arithmetic
 * gates
 *
 */
TEST_F(GoblinUltraHonkComposerTests, MultipleCircuits)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<proof_system::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        construct_test_circuit_then_generate_and_verify_proof(op_queue);
    }

    // Compute the commitments to the aggregate op queue directly and check that they match those that were computed
    // iteratively during transcript aggregation by the provers and stored in the op queue.
    size_t aggregate_op_queue_size = op_queue->current_ultra_ops_size;
    auto crs_factory = std::make_shared<barretenberg::srs::factories::FileCrsFactory<Curve>>("../srs_db/ignition");
    auto commitment_key = std::make_shared<CommitmentKey>(aggregate_op_queue_size, crs_factory);
    size_t idx = 0;
    for (auto& result : op_queue->ultra_ops_commitments) {
        auto expected = commitment_key->commit(op_queue->ultra_ops[idx++]);
        EXPECT_EQ(result, expected);
    }
}

} // namespace test_ultra_honk_composer
