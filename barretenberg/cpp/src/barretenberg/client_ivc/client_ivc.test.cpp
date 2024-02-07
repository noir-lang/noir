#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/protogalaxy_recursive_verifier.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

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
    using Builder = ClientIVC::ClientCircuit;
    using Composer = GoblinUltraComposer;
    using Accumulator = ClientIVC::Accumulator;
    using FoldProof = ClientIVC::FoldProof;

    using GURecursiveFlavor = GoblinUltraRecursiveFlavor_<Builder>;
    using RecursiveVerifierInstances = ::bb::VerifierInstances_<GURecursiveFlavor, 2>;
    using FoldingRecursiveVerifier =
        bb::stdlib::recursion::honk::ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;

    /**
     * @brief Construct mock circuit with arithmetic gates and goblin ops
     * @details Currently default sized to 2^16 to match kernel. (Note: op gates will bump size to next power of 2)
     *
     */
    static Builder create_mock_circuit(ClientIVC& ivc, size_t num_gates = 1 << 15)
    {
        Builder circuit{ ivc.goblin.op_queue };
        GoblinMockCircuits::construct_arithmetic_circuit(circuit, num_gates);
        GoblinMockCircuits::construct_goblin_ecc_op_circuit(circuit);
        return circuit;
    }

    /**
     * @brief Construct mock kernel consisting of two recursive folding verifiers
     *
     * @param builder
     * @param fctn_fold_proof
     * @param kernel_fold_proof
     */
    static void construct_mock_folding_kernel(Builder& builder,
                                              FoldProof& fctn_fold_proof,
                                              FoldProof& kernel_fold_proof)
    {
        FoldingRecursiveVerifier verifier_1{ &builder };
        verifier_1.verify_folding_proof(fctn_fold_proof);

        FoldingRecursiveVerifier verifier_2{ &builder };
        verifier_2.verify_folding_proof(kernel_fold_proof);
    }

    /**
     * @brief Perform native fold verification and run decider prover/verifier
     *
     */
    static void EXPECT_FOLDING_AND_DECIDING_VERIFIED(const Accumulator& accumulator, const FoldProof& fold_proof)
    {
        // Verify fold proof
        Composer composer;
        auto folding_verifier = composer.create_folding_verifier();
        bool folding_verified = folding_verifier.verify_folding_proof(fold_proof);
        EXPECT_TRUE(folding_verified);

        // Run decider
        auto decider_prover = composer.create_decider_prover(accumulator);
        auto decider_verifier = composer.create_decider_verifier(accumulator);
        auto decider_proof = decider_prover.construct_proof();
        bool decision = decider_verifier.verify_proof(decider_proof);
        EXPECT_TRUE(decision);
    }
};

/**
 * @brief A full Goblin test using PG that mimicks the basic aztec client architecture
 *
 */
TEST_F(ClientIVCTests, Full)
{
    ClientIVC ivc;

    // Initialize IVC with function circuit
    Builder function_circuit = create_mock_circuit(ivc);
    ivc.initialize(function_circuit);

    // Accumulate kernel circuit (first kernel mocked as simple circuit since no folding proofs yet)
    Builder kernel_circuit = create_mock_circuit(ivc);
    FoldProof kernel_fold_proof = ivc.accumulate(kernel_circuit);
    EXPECT_FOLDING_AND_DECIDING_VERIFIED(ivc.fold_output.accumulator, kernel_fold_proof);

    size_t NUM_CIRCUITS = 1;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        // Accumulate function circuit
        Builder function_circuit = create_mock_circuit(ivc);
        FoldProof function_fold_proof = ivc.accumulate(function_circuit);
        EXPECT_FOLDING_AND_DECIDING_VERIFIED(ivc.fold_output.accumulator, function_fold_proof);

        // Accumulate kernel circuit
        Builder kernel_circuit{ ivc.goblin.op_queue };
        construct_mock_folding_kernel(kernel_circuit, function_fold_proof, kernel_fold_proof);
        FoldProof kernel_fold_proof = ivc.accumulate(kernel_circuit);
        EXPECT_FOLDING_AND_DECIDING_VERIFIED(ivc.fold_output.accumulator, kernel_fold_proof);
    }

    // Constuct four proofs: merge, eccvm, translator, decider
    auto proof = ivc.prove();

    // Verify all four proofs
    EXPECT_TRUE(ivc.verify(proof));
}