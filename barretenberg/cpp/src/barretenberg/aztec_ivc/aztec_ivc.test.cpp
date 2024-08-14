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
     * @details Defaulted to add 2^16 gates (which will bump to next power of two with the addition of dummy gates).
     * The size of the baseline circuit needs to be ~2x the number of gates appended to the kernel circuits via
     * recursive verifications (currently ~60k) to ensure that the circuits being folded are equal in size. (This is
     * only necessary if the structured trace is not in use).
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
     * @brief A test utility for generating alternating mock app and kernel circuits and precomputing verification keys
     *
     */
    class MockCircuitProducer {
        using ClientCircuit = AztecIVC::ClientCircuit;

        bool is_kernel = false;

      public:
        ClientCircuit create_next_circuit(AztecIVC& ivc, size_t log2_num_gates = 16)
        {
            ClientCircuit circuit{ ivc.goblin.op_queue };
            circuit = create_mock_circuit(ivc, log2_num_gates); // construct mock base logic
            if (is_kernel) {
                ivc.complete_kernel_circuit_logic(circuit); // complete with recursive verifiers etc
            }
            is_kernel = !is_kernel; // toggle is_kernel on/off alternatingly

            return circuit;
        }

        auto precompute_verification_keys(const size_t num_circuits,
                                          TraceStructure trace_structure,
                                          size_t log2_num_gates = 16)
        {
            AztecIVC ivc; // temporary IVC instance needed to produce the complete kernel circuits
            ivc.trace_structure = trace_structure;

            std::vector<std::shared_ptr<VerificationKey>> vkeys;

            for (size_t idx = 0; idx < num_circuits; ++idx) {
                ClientCircuit circuit = create_next_circuit(ivc, log2_num_gates); // create the next circuit
                ivc.accumulate(circuit);                                          // accumulate the circuit
                vkeys.emplace_back(ivc.instance_vk);                              // save the VK for the circuit
            }
            is_kernel = false;

            return vkeys;
        }
    };

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

    MockCircuitProducer circuit_producer;

    // Initialize the IVC with an arbitrary circuit
    Builder circuit_0 = circuit_producer.create_next_circuit(ivc);
    ivc.accumulate(circuit_0);

    // Create another circuit and accumulate
    Builder circuit_1 = circuit_producer.create_next_circuit(ivc);
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

        MockCircuitProducer circuit_producer;

        // Construct and accumulate a set of mocked private function execution circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = circuit_producer.create_next_circuit(ivc, /*log2_num_gates=*/5);
            ivc.accumulate(circuit);
        }
        EXPECT_TRUE(ivc.prove_and_verify());
    }

    // The IVC fails to verify if the FIRST fold proof is tampered with
    {
        AztecIVC ivc;
        ivc.trace_structure = TraceStructure::SMALL_TEST;

        MockCircuitProducer circuit_producer;

        // Construct and accumulate a set of mocked private function execution circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = circuit_producer.create_next_circuit(ivc, /*log2_num_gates=*/5);
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

        MockCircuitProducer circuit_producer;

        // Construct and accumulate a set of mocked private function execution circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = circuit_producer.create_next_circuit(ivc, /*log2_num_gates=*/5);
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

        MockCircuitProducer circuit_producer;

        // Construct and accumulate a set of mocked private function execution circuits
        size_t NUM_CIRCUITS = 4;
        for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
            auto circuit = circuit_producer.create_next_circuit(ivc, /*log2_num_gates=*/5);
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

    MockCircuitProducer circuit_producer;

    // Construct and accumulate a set of mocked private function execution circuits
    size_t NUM_CIRCUITS = 6;
    std::vector<Builder> circuits;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        auto circuit = circuit_producer.create_next_circuit(ivc);
        ivc.accumulate(circuit);
    }

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

    MockCircuitProducer circuit_producer;

    size_t NUM_CIRCUITS = 4;

    // Construct and accumulate some circuits of varying size
    size_t log2_num_gates = 5;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        auto circuit = circuit_producer.create_next_circuit(ivc, log2_num_gates);
        ivc.accumulate(circuit);
        log2_num_gates += 2;
    }

    EXPECT_TRUE(ivc.prove_and_verify());
};

/**
 * @brief Prove and verify accumulation of an arbitrary set of circuits using precomputed verification keys
 *
 */
TEST_F(AztecIVCTests, PrecomputedVerificationKeys)
{
    AztecIVC ivc;

    size_t NUM_CIRCUITS = 4;

    MockCircuitProducer circuit_producer;

    auto precomputed_vks = circuit_producer.precompute_verification_keys(NUM_CIRCUITS, TraceStructure::NONE);

    // Construct and accumulate set of circuits using the precomputed vkeys
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        auto circuit = circuit_producer.create_next_circuit(ivc);
        ivc.accumulate(circuit, precomputed_vks[idx]);
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

    size_t NUM_CIRCUITS = 4;
    size_t log2_num_gates = 5; // number of gates in baseline mocked circuit

    MockCircuitProducer circuit_producer;

    auto precomputed_vks =
        circuit_producer.precompute_verification_keys(NUM_CIRCUITS, ivc.trace_structure, log2_num_gates);

    // Construct and accumulate set of circuits using the precomputed vkeys
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        auto circuit = circuit_producer.create_next_circuit(ivc, log2_num_gates);
        ivc.accumulate(circuit, precomputed_vks[idx]);
    }

    EXPECT_TRUE(ivc.prove_and_verify());
};
