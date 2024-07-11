#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

class ClientIVCTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Flavor = ClientIVC::Flavor;
    using FF = typename Flavor::FF;
    using VerificationKey = Flavor::VerificationKey;
    using Builder = ClientIVC::ClientCircuit;
    using ProverInstance = ClientIVC::ProverInstance;
    using VerifierInstance = ClientIVC::VerifierInstance;
    using FoldProof = ClientIVC::FoldProof;
    using DeciderProver = ClientIVC::DeciderProver;
    using DeciderVerifier = ClientIVC::DeciderVerifier;
    using ProverInstances = ProverInstances_<Flavor>;
    using FoldingProver = ProtoGalaxyProver_<ProverInstances>;
    using VerifierInstances = VerifierInstances_<Flavor>;
    using FoldingVerifier = ProtoGalaxyVerifier_<VerifierInstances>;

    /**
     * @brief Prove and verify the IVC scheme
     * @details Constructs four proofs: merge, eccvm, translator, decider; Verifies these four plus the final folding
     * proof constructed on the last accumulation round
     *
     */
    static bool prove_and_verify(ClientIVC& ivc)
    {
        auto proof = ivc.prove();

        auto verifier_inst = std::make_shared<VerifierInstance>(ivc.instance_vk);
        return ivc.verify(proof, { ivc.verifier_accumulator, verifier_inst });
    }

    /**
     * @brief Construct mock circuit with arithmetic gates and goblin ops
     * @details Currently default sized to 2^16 to match kernel. (Note: dummy op gates added to avoid non-zero
     * polynomials will bump size to next power of 2)
     *
     */
    static Builder create_mock_circuit(ClientIVC& ivc, size_t log2_num_gates = 15)
    {
        Builder circuit{ ivc.goblin.op_queue };
        MockCircuits::construct_arithmetic_circuit(circuit, log2_num_gates);

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/911): We require goblin ops to be added to the
        // function circuit because we cannot support zero commtiments. While the builder handles this at
        // finalisation stage via the add_gates_to_ensure_all_polys_are_non_zero function for other MegaHonk
        // circuits (where we don't explicitly need to add goblin ops), in ClientIVC merge proving happens prior to
        // folding where the absense of goblin ecc ops will result in zero commitments.
        MockCircuits::construct_goblin_ecc_op_circuit(circuit);
        return circuit;
    }
};

/**
 * @brief A simple-as-possible test demonstrating IVC for two mock circuits
 *
 */
TEST_F(ClientIVCTests, Basic)
{
    ClientIVC ivc;

    // Initialize the IVC with an arbitrary circuit
    Builder circuit_0 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_0);

    // Create another circuit and accumulate
    Builder circuit_1 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_1);

    EXPECT_TRUE(prove_and_verify(ivc));
};

/**
 * @brief Check that the IVC fails to verify if an intermediate fold proof is invalid
 *
 */
TEST_F(ClientIVCTests, BasicFailure)
{
    ClientIVC ivc;

    // Initialize the IVC with an arbitrary circuit
    Builder circuit_0 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_0);

    // Create another circuit and accumulate
    Builder circuit_1 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_1);

    // Tamper with the fold proof just created in the last accumulation step
    for (auto& val : ivc.fold_output.proof) {
        if (val > 0) { // tamper by finding the first non-zero value and incrementing it by 1
            val += 1;
            break;
        }
    }

    // Accumulate another circuit; this involves recursive folding verification of the bad proof
    Builder circuit_2 = create_mock_circuit(ivc);
    ivc.accumulate(circuit_2);

    // The bad fold proof should result in an invalid witness in the final circuit and the IVC should fail to verify
    EXPECT_FALSE(prove_and_verify(ivc));
};

/**
 * @brief Prove and verify accumulation of an arbitrary set of circuits
 *
 */
TEST_F(ClientIVCTests, BasicLarge)
{
    ClientIVC ivc;

    // Construct a set of arbitrary circuits
    size_t NUM_CIRCUITS = 5;
    std::vector<Builder> circuits;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        circuits.emplace_back(create_mock_circuit(ivc));
    }

    // Accumulate each circuit
    for (auto& circuit : circuits) {
        ivc.accumulate(circuit);
    }

    EXPECT_TRUE(prove_and_verify(ivc));
};

/**
 * @brief Using a structured trace allows for the accumulation of circuits of varying size
 *
 */
TEST_F(ClientIVCTests, BasicStructured)
{
    ClientIVC ivc;
    ivc.trace_structure = TraceStructure::SMALL_TEST;

    // Construct some circuits of varying size
    Builder circuit_0 = create_mock_circuit(ivc, /*log2_num_gates=*/5);
    Builder circuit_1 = create_mock_circuit(ivc, /*log2_num_gates=*/8);
    Builder circuit_2 = create_mock_circuit(ivc, /*log2_num_gates=*/11);

    // The circuits can be accumulated as normal due to the structured trace
    ivc.accumulate(circuit_0);
    ivc.accumulate(circuit_1);
    ivc.accumulate(circuit_2);

    EXPECT_TRUE(prove_and_verify(ivc));
};

/**
 * @brief Prove and verify accumulation of an arbitrary set of circuits using precomputed verification keys
 *
 */
TEST_F(ClientIVCTests, PrecomputedVerificationKeys)
{
    ClientIVC ivc;

    // Construct a set of arbitrary circuits
    size_t NUM_CIRCUITS = 3;
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

    EXPECT_TRUE(prove_and_verify(ivc));
};

/**
 * @brief Perform accumulation with a structured trace and precomputed verification keys
 *
 */
TEST_F(ClientIVCTests, StructuredPrecomputedVKs)
{
    ClientIVC ivc;
    ivc.trace_structure = TraceStructure::SMALL_TEST;

    // Construct a set of arbitrary circuits
    size_t NUM_CIRCUITS = 3;
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

    EXPECT_TRUE(prove_and_verify(ivc));
};
