#include "barretenberg/aztec_ivc/aztec_ivc.hpp"

namespace bb {

/**
 * @brief Accumulate a circuit into the IVC scheme
 * @details If this is the first circuit being accumulated, initialize the prover and verifier accumulators. Otherwise,
 * fold the instance for the provided circuit into the accumulator. When two fold proofs have been enqueued, two
 * recursive folding verifications are appended to the next circuit that is accumulated, which must be a kernel.
 * Similarly, if a merge proof exists, a recursive merge verifier is appended.
 *
 * @param circuit Circuit to be accumulated/folded
 * @param precomputed_vk Optional precomputed VK (otherwise will be computed herein)
 */
void AztecIVC::accumulate(ClientCircuit& circuit, const std::shared_ptr<VerificationKey>& precomputed_vk)
{
    circuit_count++; // increment the count of circuits processed into the IVC

    // When there are two fold proofs present, append two recursive verifiers to the kernel
    if (verification_queue.size() == 2) {
        BB_OP_COUNT_TIME_NAME("construct_circuits");
        ASSERT(circuit_count % 2 == 0); // ensure this is a kernel

        for (auto& [proof, vkey] : verification_queue) {
            FoldingRecursiveVerifier verifier{ &circuit, { verifier_accumulator, { vkey } } };
            auto verifier_accum = verifier.verify_folding_proof(proof);
            verifier_accumulator = std::make_shared<VerifierInstance>(verifier_accum->get_value());
            info("Num gates = ", circuit.get_num_gates());
        }
        verification_queue.clear();
    }

    // Construct a merge proof (and add a recursive merge verifier to the circuit if a previous merge proof exists)
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1063): update recursive merge verification to only
    // occur in kernels, similar to folding recursive verification.
    goblin.merge(circuit);

    // Construct the prover instance for circuit
    auto prover_instance = std::make_shared<ProverInstance>(circuit, trace_structure);

    // Set the instance verification key from precomputed if available, else compute it
    if (precomputed_vk) {
        instance_vk = precomputed_vk;
    } else {
        instance_vk = std::make_shared<VerificationKey>(prover_instance->proving_key);
    }

    // If this is the first circuit simply initialize the prover and verifier accumulator instances
    if (circuit_count == 1) {
        fold_output.accumulator = prover_instance;
        verifier_accumulator = std::make_shared<VerifierInstance>(instance_vk);
    } else { // Otherwise, fold the new instance into the accumulator
        FoldingProver folding_prover({ fold_output.accumulator, prover_instance });
        fold_output = folding_prover.fold_instances();

        // Add fold proof and corresponding verification key to the verification queue
        verification_queue.emplace_back(fold_output.proof, instance_vk);
    }

    // Track the maximum size of each block for all circuits porcessed (for debugging purposes only)
    max_block_size_tracker.update(circuit);
}

/**
 * @brief Construct a proof for the IVC, which, if verified, fully establishes its correctness
 *
 * @return Proof
 */
AztecIVC::Proof AztecIVC::prove()
{
    max_block_size_tracker.print();         // print minimum structured sizes for each block
    ASSERT(verification_queue.size() == 1); // ensure only a single fold proof remains in the queue
    auto& fold_proof = verification_queue[0].proof;
    return { fold_proof, decider_prove(), goblin.prove() };
};

bool AztecIVC::verify(const Proof& proof,
                      const std::shared_ptr<VerifierInstance>& accumulator,
                      const std::shared_ptr<VerifierInstance>& final_verifier_instance,
                      const std::shared_ptr<AztecIVC::ECCVMVerificationKey>& eccvm_vk,
                      const std::shared_ptr<AztecIVC::TranslatorVerificationKey>& translator_vk)
{
    // Goblin verification (merge, eccvm, translator)
    GoblinVerifier goblin_verifier{ eccvm_vk, translator_vk };
    bool goblin_verified = goblin_verifier.verify(proof.goblin_proof);

    // Decider verification
    AztecIVC::FoldingVerifier folding_verifier({ accumulator, final_verifier_instance });
    auto verifier_accumulator = folding_verifier.verify_folding_proof(proof.folding_proof);

    AztecIVC::DeciderVerifier decider_verifier(verifier_accumulator);
    bool decision = decider_verifier.verify_proof(proof.decider_proof);
    return goblin_verified && decision;
}

/**
 * @brief Verify a full proof of the IVC
 *
 * @param proof
 * @return bool
 */
bool AztecIVC::verify(Proof& proof, const std::vector<std::shared_ptr<VerifierInstance>>& verifier_instances)
{
    auto eccvm_vk = std::make_shared<ECCVMVerificationKey>(goblin.get_eccvm_proving_key());
    auto translator_vk = std::make_shared<TranslatorVerificationKey>(goblin.get_translator_proving_key());
    return verify(proof, verifier_instances[0], verifier_instances[1], eccvm_vk, translator_vk);
}

/**
 * @brief Internal method for constructing a decider proof
 *
 * @return HonkProof
 */
HonkProof AztecIVC::decider_prove() const
{
    MegaDeciderProver decider_prover(fold_output.accumulator);
    return decider_prover.construct_proof();
}

/**
 * @brief Given a set of circuits, compute the verification keys that will be required by the IVC scheme
 * @details The verification keys computed here are in general not the same as the verification keys for the
 * raw input circuits because recursive verifier circuits (merge and/or folding) may be appended to the incoming
 * circuits as part accumulation.
 * @note This method exists for convenience and is not not meant to be used in practice for IVC. Given a set of
 * circuits, it could be run once and for all to compute then save the required VKs. It also provides a convenient
 * (albeit innefficient) way of separating out the cost of computing VKs from a benchmark.
 *
 * @param circuits A copy of the circuits to be accumulated (passing by reference would alter the original circuits)
 * @return std::vector<std::shared_ptr<AztecIVC::VerificationKey>>
 */
std::vector<std::shared_ptr<AztecIVC::VerificationKey>> AztecIVC::precompute_folding_verification_keys(
    std::vector<ClientCircuit> circuits)
{
    std::vector<std::shared_ptr<VerificationKey>> vkeys;

    for (auto& circuit : circuits) {
        accumulate(circuit);
        vkeys.emplace_back(instance_vk);
    }

    // Reset the scheme so it can be reused for actual accumulation, maintaining the trace structure setting as is
    TraceStructure structure = trace_structure;
    *this = AztecIVC();
    this->trace_structure = structure;

    return vkeys;
}

/**
 * @brief Construct and verify a proof for the IVC
 * @note Use of this method only makes sense when the prover and verifier are the same entity, e.g. in
 * development/testing.
 *
 */
bool AztecIVC::prove_and_verify()
{
    auto proof = prove();

    ASSERT(verification_queue.size() == 1); // ensure only a single fold proof remains in the queue
    auto verifier_inst = std::make_shared<VerifierInstance>(this->verification_queue[0].instance_vk);
    return verify(proof, { this->verifier_accumulator, verifier_inst });
}

} // namespace bb