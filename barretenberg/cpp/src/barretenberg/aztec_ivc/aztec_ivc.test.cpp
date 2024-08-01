#include "barretenberg/aztec_ivc/aztec_ivc.hpp"
#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

class AztecIVCTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Flavor = AztecIVC::Flavor;
    using FF = typename Flavor::FF;
    using VerificationKey = Flavor::VerificationKey;
    using Builder = AztecIVC::ClientCircuit;
    using ProverInstance = AztecIVC::ProverInstance;
    using VerifierInstance = AztecIVC::VerifierInstance;
    using FoldProof = AztecIVC::FoldProof;
    using DeciderProver = AztecIVC::DeciderProver;
    using DeciderVerifier = AztecIVC::DeciderVerifier;
    using ProverInstances = ProverInstances_<Flavor>;
    using FoldingProver = ProtoGalaxyProver_<ProverInstances>;
    using VerifierInstances = VerifierInstances_<Flavor>;
    using FoldingVerifier = ProtoGalaxyVerifier_<VerifierInstances>;

    /**
     * @brief Construct mock circuit with arithmetic gates and goblin ops
     * @details Currently default sized to 2^16 to match kernel. (Note: dummy op gates added to avoid non-zero
     * polynomials will bump size to next power of 2)
     *
     */
    static Builder create_mock_circuit(AztecIVC& ivc, size_t log2_num_gates = 16)
    {
        Builder circuit{ ivc.goblin.op_queue };
        MockCircuits::construct_arithmetic_circuit(circuit, log2_num_gates);

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/911): We require goblin ops to be added to the
        // function circuit because we cannot support zero commtiments. While the builder handles this at
        // finalisation stage via the add_gates_to_ensure_all_polys_are_non_zero function for other MegaHonk
        // circuits (where we don't explicitly need to add goblin ops), in AztecIVC merge proving happens prior to
        // folding where the absense of goblin ecc ops will result in zero commitments.
        MockCircuits::construct_goblin_ecc_op_circuit(circuit);
        return circuit;
    }

    /**
     * @brief Tamper with a proof by finding the first non-zero value and incrementing it by 1
     *
     */
    static void tamper_with_proof(FoldProof& proof)
    {
        for (auto& val : proof) {
            if (val > 0) {
                val += 1;
                break;
            }
        }
    }
};

/**
 * @brief A simple-as-possible test demonstrating IVC for two mock circuits
 * @details When accumulating only two circuits, only a single round of folding is performed thus no recursive
 * verfication occurs.
 *
 */
TEST_F(AztecIVCTests, Basic)
{
    AztecIVC ivc;

    // Initialize the IVC with an arbitrary circuit
    Builder circuit_0 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_0);

    // Create another circuit and accumulate
    Builder circuit_1 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_1);

    EXPECT_TRUE(ivc.prove_and_verify());
};

/**
 * @brief Check that the IVC fails to verify if an intermediate fold proof is invalid
 * @details When accumulating 4 circuits, there are 3 fold proofs to verify (the first two are recursively verfied and
 * the 3rd is verified as part of the IVC proof). Check that if any of one of these proofs is invalid, the IVC will fail
 * to verify.
 *
 */
TEST_F(AztecIVCTests, BadProofFailure)
{
    // Confirm that the IVC verifies if nothing is tampered with
    {
        AztecIVC ivc;
        ivc.trace_structure = TraceStructure::SMALL_TEST;

        // Construct a set of arbitrary circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = create_mock_circuit(ivc, /*log2_num_gates=*/5);
            ivc.accumulate(circuit);
        }
        EXPECT_TRUE(ivc.prove_and_verify());
    }

    // The IVC fails to verify if the FIRST fold proof is tampered with
    {
        AztecIVC ivc;
        ivc.trace_structure = TraceStructure::SMALL_TEST;

        // Construct a set of arbitrary circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = create_mock_circuit(ivc, /*log2_num_gates=*/5);
            ivc.accumulate(circuit);

            if (idx == 2) {
                EXPECT_EQ(ivc.verification_queue.size(), 2);        // two proofs after 3 calls to accumulation
                tamper_with_proof(ivc.verification_queue[0].proof); // tamper with first proof
            }
        }

        EXPECT_FALSE(ivc.prove_and_verify());
    }

    // The IVC fails to verify if the SECOND fold proof is tampered with
    {
        AztecIVC ivc;
        ivc.trace_structure = TraceStructure::SMALL_TEST;

        // Construct a set of arbitrary circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = create_mock_circuit(ivc, /*log2_num_gates=*/5);
            ivc.accumulate(circuit);

            if (idx == 2) {
                EXPECT_EQ(ivc.verification_queue.size(), 2);        // two proofs after 3 calls to accumulation
                tamper_with_proof(ivc.verification_queue[1].proof); // tamper with second proof
            }
        }

        EXPECT_FALSE(ivc.prove_and_verify());
    }

    // The IVC fails to verify if the 3rd/FINAL fold proof is tampered with
    {
        AztecIVC ivc;
        ivc.trace_structure = TraceStructure::SMALL_TEST;

        // Construct a set of arbitrary circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = create_mock_circuit(ivc, /*log2_num_gates=*/5);
            ivc.accumulate(circuit);
        }

        // Only a single proof should be present in the queue when verification of the IVC is performed
        EXPECT_EQ(ivc.verification_queue.size(), 1);
        tamper_with_proof(ivc.verification_queue[0].proof); // tamper with the final fold proof

        EXPECT_FALSE(ivc.prove_and_verify());
    }

    EXPECT_TRUE(true);
};

/**
 * @brief Prove and verify accumulation of an arbitrary set of circuits
 *
 */
TEST_F(AztecIVCTests, BasicLarge)
{
    AztecIVC ivc;

    // Construct a set of arbitrary circuits
    size_t NUM_CIRCUITS = 6;
    std::vector<Builder> circuits;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        circuits.emplace_back(create_mock_circuit(ivc));
    }

    // Accumulate each circuit
    for (auto& circuit : circuits) {
        ivc.accumulate(circuit);
    }

    info(ivc.goblin.op_queue->get_current_size());

    EXPECT_TRUE(ivc.prove_and_verify());
};

/**
 * @brief Using a structured trace allows for the accumulation of circuits of varying size
 *
 */
TEST_F(AztecIVCTests, BasicStructured)
{
    AztecIVC ivc;
    ivc.trace_structure = TraceStructure::SMALL_TEST;

    // Construct some circuits of varying size
    Builder circuit_0 = create_mock_circuit(ivc, /*log2_num_gates=*/5);
    Builder circuit_1 = create_mock_circuit(ivc, /*log2_num_gates=*/6);
    Builder circuit_2 = create_mock_circuit(ivc, /*log2_num_gates=*/7);
    Builder circuit_3 = create_mock_circuit(ivc, /*log2_num_gates=*/8);

    // The circuits can be accumulated as normal due to the structured trace
    ivc.accumulate(circuit_0);
    ivc.accumulate(circuit_1);
    ivc.accumulate(circuit_2);
    ivc.accumulate(circuit_3);

    EXPECT_TRUE(ivc.prove_and_verify());
};

/**
 * @brief Prove and verify accumulation of an arbitrary set of circuits using precomputed verification keys
 *
 */
TEST_F(AztecIVCTests, PrecomputedVerificationKeys)
{
    AztecIVC ivc;

    // Construct a set of arbitrary circuits
    size_t NUM_CIRCUITS = 4;
    std::vector<Builder> circuits;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        circuits.emplace_back(create_mock_circuit(ivc));
    }

    // Precompute the verification keys that will be needed for the IVC
    auto precomputed_vkeys = ivc.precompute_folding_verification_keys(circuits);

    // Accumulate each circuit using the precomputed VKs
    for (auto [circuit, precomputed_vk] : zip_view(circuits, precomputed_vkeys)) {
        ivc.accumulate(circuit, precomputed_vk);
    }

    EXPECT_TRUE(ivc.prove_and_verify());
};

/**
 * @brief Perform accumulation with a structured trace and precomputed verification keys
 *
 */
TEST_F(AztecIVCTests, StructuredPrecomputedVKs)
{
    AztecIVC ivc;
    ivc.trace_structure = TraceStructure::SMALL_TEST;

    // Construct a set of arbitrary circuits
    size_t NUM_CIRCUITS = 4;
    std::vector<Builder> circuits;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        circuits.emplace_back(create_mock_circuit(ivc, /*log2_num_gates=*/5));
    }

    // Precompute the (structured) verification keys that will be needed for the IVC
    auto precomputed_vkeys = ivc.precompute_folding_verification_keys(circuits);

    // Accumulate each circuit
    for (auto [circuit, precomputed_vk] : zip_view(circuits, precomputed_vkeys)) {
        ivc.accumulate(circuit, precomputed_vk);
    }

    EXPECT_TRUE(ivc.prove_and_verify());
};
