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
    using ProverAccumulator = ClientIVC::ProverAccumulator;
    using VerifierAccumulator = ClientIVC::VerifierAccumulator;
    using VerifierInstance = ClientIVC::VerifierInstance;
    using FoldProof = ClientIVC::FoldProof;
    using VerifierFoldData = GoblinMockCircuits::VerifierFoldData;
    using GURecursiveFlavor = GoblinUltraRecursiveFlavor_<Builder>;
    using RecursiveVerifierInstance = ::bb::stdlib::recursion::honk::RecursiveVerifierInstance_<GURecursiveFlavor>;
    using RecursiveVerifierAccumulator = std::shared_ptr<RecursiveVerifierInstance>;
    using RecursiveVerifierInstances = ::bb::stdlib::recursion::honk::RecursiveVerifierInstances_<GURecursiveFlavor, 2>;
    using FoldingRecursiveVerifier =
        bb::stdlib::recursion::honk::ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;

    /**
     * @brief Construct mock circuit with arithmetic gates and goblin ops
     * @details Currently default sized to 2^16 to match kernel. (Note: op gates will bump size to next power of
     2)
     *
     */
    static Builder create_mock_circuit(ClientIVC& ivc, size_t log2_num_gates = 15)
    {
        Builder circuit{ ivc.goblin.op_queue };
        GoblinMockCircuits::construct_arithmetic_circuit(circuit, log2_num_gates);
        GoblinMockCircuits::construct_goblin_ecc_op_circuit(circuit);
        return circuit;
    }

    /**
     * @brief Construct mock kernel consisting of two recursive folding verifiers to verify the folding of the previous
     * function circuit and kernel circuit.
     *
     * @param builder
     * @param func_accum contains the folding proof for the function circuit and the corresponsing function
     * verifier instance
     * @param kernel_accum contains the folding proof for the kernel circuit and the corresponding kernel verifier
     * instance
     * @returns the updated verifier accumulator
     */
    static VerifierAccumulator construct_mock_folding_kernel(Builder& builder,
                                                             VerifierFoldData& func_accum,
                                                             VerifierFoldData& kernel_accum,
                                                             VerifierAccumulator& prev_kernel_accum)
    {

        FoldingRecursiveVerifier verifier_1{ &builder, prev_kernel_accum, { func_accum.inst_vk } };
        auto fctn_verifier_accum = verifier_1.verify_folding_proof(func_accum.fold_proof);
        auto native_acc = std::make_shared<ClientIVC::VerifierInstance>(fctn_verifier_accum->get_value());
        FoldingRecursiveVerifier verifier_2{ &builder, native_acc, { kernel_accum.inst_vk } };
        auto kernel_verifier_accum = verifier_2.verify_folding_proof(kernel_accum.fold_proof);
        return std::make_shared<ClientIVC::VerifierInstance>(kernel_verifier_accum->get_value());
    }

    /**
     * @brief Perform native fold verification and run decider prover/verifier
     *
     */
    static VerifierAccumulator update_accumulator_and_decide_native(
        const ProverAccumulator& prover_accumulator,
        const FoldProof& fold_proof,
        const VerifierAccumulator& prev_verifier_accumulator,
        const std::shared_ptr<Flavor::VerificationKey>& verifier_inst_vk)
    {
        // Verify fold proof
        Composer composer;
        auto new_verifier_inst = std::make_shared<VerifierInstance>(verifier_inst_vk);
        auto folding_verifier = composer.create_folding_verifier({ prev_verifier_accumulator, new_verifier_inst });
        auto verifier_accumulator = folding_verifier.verify_folding_proof(fold_proof);

        // Run decider
        auto decider_prover = composer.create_decider_prover(prover_accumulator);
        auto decider_verifier = composer.create_decider_verifier(verifier_accumulator);
        auto decider_proof = decider_prover.construct_proof();
        bool decision = decider_verifier.verify_proof(decider_proof);
        EXPECT_TRUE(decision);

        return verifier_accumulator;
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

    auto function_vk = ivc.prover_fold_output.accumulator->verification_key;
    auto foo_verifier_instance = std::make_shared<VerifierInstance>(function_vk);
    // Accumulate kernel circuit (first kernel mocked as simple circuit since no folding proofs yet)
    Builder kernel_circuit = create_mock_circuit(ivc);
    FoldProof kernel_fold_proof = ivc.accumulate(kernel_circuit);
    // This will have a different verification key because we added the recursive merge verification to the circuit
    auto function_vk_with_merge = ivc.prover_instance->verification_key;
    auto kernel_vk = function_vk_with_merge;
    auto intermediary_acc = update_accumulator_and_decide_native(
        ivc.prover_fold_output.accumulator, kernel_fold_proof, foo_verifier_instance, kernel_vk);

    VerifierFoldData kernel_fold_output = { kernel_fold_proof, function_vk_with_merge };
    size_t NUM_CIRCUITS = 1;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        // Accumulate function circuit
        Builder function_circuit = create_mock_circuit(ivc);
        FoldProof function_fold_proof = ivc.accumulate(function_circuit);

        intermediary_acc = update_accumulator_and_decide_native(
            ivc.prover_fold_output.accumulator, function_fold_proof, intermediary_acc, function_vk_with_merge);

        VerifierFoldData function_fold_output = { function_fold_proof, function_vk_with_merge };
        // Accumulate kernel circuit
        Builder kernel_circuit{ ivc.goblin.op_queue };
        foo_verifier_instance = construct_mock_folding_kernel(
            kernel_circuit, kernel_fold_output, function_fold_output, foo_verifier_instance);
        FoldProof kernel_fold_proof = ivc.accumulate(kernel_circuit);
        kernel_vk = ivc.prover_instance->verification_key;

        intermediary_acc = update_accumulator_and_decide_native(
            ivc.prover_fold_output.accumulator, kernel_fold_proof, intermediary_acc, kernel_vk);

        VerifierFoldData kernel_fold_output = { kernel_fold_proof, kernel_vk };
    }

    // Constuct four proofs: merge, eccvm, translator, decider
    auto proof = ivc.prove();
    auto inst = std::make_shared<VerifierInstance>(kernel_vk);
    // Verify all four proofs
    EXPECT_TRUE(ivc.verify(proof, { foo_verifier_instance, inst }));
};