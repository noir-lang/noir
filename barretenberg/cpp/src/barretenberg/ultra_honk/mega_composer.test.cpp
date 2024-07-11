#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/merge_prover.hpp"
#include "barretenberg/ultra_honk/merge_verifier.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();

class MegaHonkComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Point = Curve::AffineElement;
    using CommitmentKey = bb::CommitmentKey<Curve>;
    using MergeProver = MergeProver_<MegaFlavor>;
    using MergeVerifier = MergeVerifier_<MegaFlavor>;

    /**
     * @brief Construct and a verify a Honk proof
     *
     */
    bool construct_and_verify_honk_proof(auto& builder)
    {
        auto instance = std::make_shared<ProverInstance_<MegaFlavor>>(builder);
        MegaProver prover(instance);
        auto verification_key = std::make_shared<MegaFlavor::VerificationKey>(instance->proving_key);
        MegaVerifier verifier(verification_key);
        auto proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        return verified;
    }

    /**
     * @brief Construct and verify a Goblin ECC op queue merge proof
     *
     */
    bool construct_and_verify_merge_proof(auto& op_queue)
    {
        MergeProver merge_prover{ op_queue };
        MergeVerifier merge_verifier;
        auto merge_proof = merge_prover.construct_proof();
        bool verified = merge_verifier.verify_proof(merge_proof);

        return verified;
    }
};
} // namespace

/**
 * @brief Test proof construction/verification for a circuit with ECC op gates, public inputs, and basic arithmetic
 * gates
 *
 */
TEST_F(MegaHonkComposerTests, Basic)
{
    MegaCircuitBuilder builder;

    GoblinMockCircuits::construct_simple_circuit(builder);

    // Construct and verify Honk proof
    bool honk_verified = construct_and_verify_honk_proof(builder);
    EXPECT_TRUE(honk_verified);
}

/**
 * @brief Test proof construction/verification for a structured execution trace
 *
 */
TEST_F(MegaHonkComposerTests, BasicStructured)
{
    MegaCircuitBuilder builder;

    GoblinMockCircuits::construct_simple_circuit(builder);

    // Construct and verify Honk proof using a structured trace
    TraceStructure trace_structure = TraceStructure::SMALL_TEST;
    auto instance = std::make_shared<ProverInstance_<MegaFlavor>>(builder, trace_structure);
    MegaProver prover(instance);
    auto verification_key = std::make_shared<MegaFlavor::VerificationKey>(instance->proving_key);
    MegaVerifier verifier(verification_key);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

/**
 * @brief Test proof construction/verification for a circuit with ECC op gates, public inputs, and basic arithmetic
 * gates
 * @note We simulate op queue interactions with a previous circuit so the actual circuit under test utilizes an op queue
 * with non-empty 'previous' data. This avoid complications with zero-commitments etc.
 *
 */
TEST_F(MegaHonkComposerTests, SingleCircuit)
{
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(op_queue);
    auto builder = MegaCircuitBuilder{ op_queue };

    GoblinMockCircuits::construct_simple_circuit(builder);

    // Construct and verify Honk proof
    bool honk_verified = construct_and_verify_honk_proof(builder);
    EXPECT_TRUE(honk_verified);

    // Construct and verify Goblin ECC op queue Merge proof
    auto merge_verified = construct_and_verify_merge_proof(op_queue);
    EXPECT_TRUE(merge_verified);
}

/**
 * @brief Test Merge proof construction/verification for multiple circuits with ECC op gates, public inputs, and
 * basic arithmetic gates
 *
 */
TEST_F(MegaHonkComposerTests, MultipleCircuitsMergeOnly)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(op_queue);

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        auto builder = MegaCircuitBuilder{ op_queue };

        GoblinMockCircuits::construct_simple_circuit(builder);

        // Construct and verify Goblin ECC op queue Merge proof
        auto merge_verified = construct_and_verify_merge_proof(op_queue);
        EXPECT_TRUE(merge_verified);
    }
}

/**
 * @brief Test Honk proof construction/verification for multiple circuits with ECC op gates, public inputs, and
 * basic arithmetic gates
 *
 */
TEST_F(MegaHonkComposerTests, MultipleCircuitsHonkOnly)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(op_queue);

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        auto builder = MegaCircuitBuilder{ op_queue };

        GoblinMockCircuits::construct_simple_circuit(builder);

        // Construct and verify Honk proof
        bool honk_verified = construct_and_verify_honk_proof(builder);
        EXPECT_TRUE(honk_verified);
    }
}

/**
 * @brief Test Honk and Merge proof construction/verification for multiple circuits with ECC op gates, public inputs,
 * and basic arithmetic gates
 *
 */
TEST_F(MegaHonkComposerTests, MultipleCircuitsHonkAndMerge)
{
    // Instantiate EccOpQueue. This will be shared across all circuits in the series
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(op_queue);

    // Construct multiple test circuits that share an ECC op queue. Generate and verify a proof for each.
    size_t NUM_CIRCUITS = 3;
    for (size_t i = 0; i < NUM_CIRCUITS; ++i) {
        auto builder = MegaCircuitBuilder{ op_queue };

        GoblinMockCircuits::construct_simple_circuit(builder);

        // Construct and verify Honk proof
        bool honk_verified = construct_and_verify_honk_proof(builder);
        EXPECT_TRUE(honk_verified);

        // Construct and verify Goblin ECC op queue Merge proof
        auto merge_verified = construct_and_verify_merge_proof(op_queue);
        EXPECT_TRUE(merge_verified);
    }

    // Compute the commitments to the aggregate op queue directly and check that they match those that were computed
    // iteratively during transcript aggregation by the provers and stored in the op queue.
    size_t aggregate_op_queue_size = op_queue->get_current_size();
    auto ultra_ops = op_queue->get_aggregate_transcript();
    auto commitment_key = std::make_shared<CommitmentKey>(aggregate_op_queue_size);
    size_t idx = 0;
    for (const auto& result : op_queue->get_ultra_ops_commitments()) {
        auto expected = commitment_key->commit(ultra_ops[idx++]);
        EXPECT_EQ(result, expected);
    }
}
