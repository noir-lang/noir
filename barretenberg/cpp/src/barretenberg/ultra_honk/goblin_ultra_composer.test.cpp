#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"

using namespace bb::honk;

namespace test_ultra_honk_composer {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

class GoblinUltraHonkComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

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
     * @brief Construct and a verify a Honk proof
     *
     */
    bool construct_and_verify_honk_proof(auto& composer, auto& builder)
    {
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto verifier = composer.create_verifier(instance);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        return verified;
    }

    /**
     * @brief Construct and verify a Goblin ECC op queue merge proof
     *
     */
    bool construct_and_verify_merge_proof(auto& composer, auto& op_queue)
    {
        auto merge_prover = composer.create_merge_prover(op_queue);
        auto merge_verifier = composer.create_merge_verifier();
        auto merge_proof = merge_prover.construct_proof();
        bool verified = merge_verifier.verify_proof(merge_proof);

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
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    auto builder = bb::GoblinUltraCircuitBuilder{ op_queue };

    generate_test_circuit(builder);

    auto composer = GoblinUltraComposer();

    // Construct and verify Honk proof
    auto honk_verified = construct_and_verify_honk_proof(composer, builder);
    EXPECT_TRUE(honk_verified);

    // Construct and verify Goblin ECC op queue Merge proof
    auto merge_verified = construct_and_verify_merge_proof(composer, op_queue);
    EXPECT_TRUE(merge_verified);
}

/**
 * @brief Test Merge proof construction/verification for multiple circuits with ECC op gates, public inputs, and
 * basic arithmetic gates
 *
 */
TEST_F(GoblinUltraHonkComposerTests, MultipleCircuitsMergeOnly)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        auto builder = bb::GoblinUltraCircuitBuilder{ op_queue };

        generate_test_circuit(builder);

        auto composer = GoblinUltraComposer();

        // Construct and verify Goblin ECC op queue Merge proof
        auto merge_verified = construct_and_verify_merge_proof(composer, op_queue);
        EXPECT_TRUE(merge_verified);
    }
}

/**
 * @brief Test Honk proof construction/verification for multiple circuits with ECC op gates, public inputs, and
 * basic arithmetic gates
 *
 */
TEST_F(GoblinUltraHonkComposerTests, MultipleCircuitsHonkOnly)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        auto builder = bb::GoblinUltraCircuitBuilder{ op_queue };

        generate_test_circuit(builder);

        auto composer = GoblinUltraComposer();

        // Construct and verify Honk proof
        auto honk_verified = construct_and_verify_honk_proof(composer, builder);
        EXPECT_TRUE(honk_verified);
    }
}

/**
 * @brief Test Honk and Merge proof construction/verification for multiple circuits with ECC op gates, public inputs,
 * and basic arithmetic gates
 *
 */
TEST_F(GoblinUltraHonkComposerTests, MultipleCircuitsHonkAndMerge)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        auto builder = bb::GoblinUltraCircuitBuilder{ op_queue };

        generate_test_circuit(builder);

        auto composer = GoblinUltraComposer();

        // Construct and verify Honk proof
        auto honk_verified = construct_and_verify_honk_proof(composer, builder);
        EXPECT_TRUE(honk_verified);

        // Construct and verify Goblin ECC op queue Merge proof
        auto merge_verified = construct_and_verify_merge_proof(composer, op_queue);
        EXPECT_TRUE(merge_verified);
    }

    // Compute the commitments to the aggregate op queue directly and check that they match those that were computed
    // iteratively during transcript aggregation by the provers and stored in the op queue.
    size_t aggregate_op_queue_size = op_queue->current_ultra_ops_size;
    auto crs_factory = std::make_shared<bb::srs::factories::FileCrsFactory<Curve>>("../srs_db/ignition");
    auto commitment_key = std::make_shared<CommitmentKey>(aggregate_op_queue_size, crs_factory);
    size_t idx = 0;
    for (auto& result : op_queue->ultra_ops_commitments) {
        auto expected = commitment_key->commit(op_queue->ultra_ops[idx++]);
        EXPECT_EQ(result, expected);
    }
}

} // namespace test_ultra_honk_composer
