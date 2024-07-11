#include "barretenberg/client_ivc/client_ivc.hpp"

namespace bb {

/**
 * @brief Accumulate a circuit into the IVC scheme
 * @details If this is the first circuit being accumulated, initialize the prover and verifier accumulators. Otherwise,
 * fold the instance for the provided circuit into the accumulator. If a previous fold proof exists, a recursive folding
 * verification is appended to the provided circuit prior to its accumulation. Similarly, if a merge proof exists, a
 * recursive merge verifier is appended.
 *
 * @param circuit Circuit to be accumulated/folded
 * @param precomputed_vk Optional precomputed VK (otherwise will be computed herein)
 */
void ClientIVC::accumulate(ClientCircuit& circuit, const std::shared_ptr<VerificationKey>& precomputed_vk)
{
    // If a previous fold proof exists, add a recursive folding verification to the circuit
    if (!fold_output.proof.empty()) {
        BB_OP_COUNT_TIME_NAME("construct_circuits");
        FoldingRecursiveVerifier verifier{ &circuit, { verifier_accumulator, { instance_vk } } };
        auto verifier_accum = verifier.verify_folding_proof(fold_output.proof);
        verifier_accumulator = std::make_shared<VerifierInstance>(verifier_accum->get_value());
    }

    // Construct a merge proof (and add a recursive merge verifier to the circuit if a previous merge proof exists)
    goblin.merge(circuit);

    // Construct the prover instance for circuit
    prover_instance = std::make_shared<ProverInstance>(circuit, trace_structure);

    // Track the maximum size of each block for all circuits porcessed (for debugging purposes only)
    max_block_sizes.update(circuit);

    // Set the instance verification key from precomputed if available, else compute it
    if (precomputed_vk) {
        instance_vk = precomputed_vk;
    } else {
        instance_vk = std::make_shared<VerificationKey>(prover_instance->proving_key);
    }

    // If the IVC is uninitialized, simply initialize the prover and verifier accumulator instances
    if (!initialized) {
        fold_output.accumulator = prover_instance;
        verifier_accumulator = std::make_shared<VerifierInstance>(instance_vk);
        initialized = true;
    } else { // Otherwise, fold the new instance into the accumulator
        FoldingProver folding_prover({ fold_output.accumulator, prover_instance });
        fold_output = folding_prover.fold_instances();
    }
}

/**
 * @brief Construct a proof for the IVC, which, if verified, fully establishes its correctness
 *
 * @return Proof
 */
ClientIVC::Proof ClientIVC::prove()
{
    max_block_sizes.print(); // print minimum structured sizes for each block
    return { fold_output.proof, decider_prove(), goblin.prove() };
};

bool ClientIVC::verify(const Proof& proof,
                       const std::shared_ptr<VerifierInstance>& accumulator,
                       const std::shared_ptr<VerifierInstance>& final_verifier_instance,
                       const std::shared_ptr<ClientIVC::ECCVMVerificationKey>& eccvm_vk,
                       const std::shared_ptr<ClientIVC::TranslatorVerificationKey>& translator_vk)
{
    // Goblin verification (merge, eccvm, translator)
    GoblinVerifier goblin_verifier{ eccvm_vk, translator_vk };
    bool goblin_verified = goblin_verifier.verify(proof.goblin_proof);

    // Decider verification
    ClientIVC::FoldingVerifier folding_verifier({ accumulator, final_verifier_instance });
    auto verifier_accumulator = folding_verifier.verify_folding_proof(proof.folding_proof);

    ClientIVC::DeciderVerifier decider_verifier(verifier_accumulator);
    bool decision = decider_verifier.verify_proof(proof.decider_proof);
    return goblin_verified && decision;
}

/**
 * @brief Verify a full proof of the IVC
 *
 * @param proof
 * @return bool
 */
bool ClientIVC::verify(Proof& proof, const std::vector<std::shared_ptr<VerifierInstance>>& verifier_instances)
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
HonkProof ClientIVC::decider_prove() const
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
 * @return std::vector<std::shared_ptr<ClientIVC::VerificationKey>>
 */
std::vector<std::shared_ptr<ClientIVC::VerificationKey>> ClientIVC::precompute_folding_verification_keys(
    std::vector<ClientCircuit> circuits)
{
    std::vector<std::shared_ptr<VerificationKey>> vkeys;

    for (auto& circuit : circuits) {
        accumulate(circuit);
        vkeys.emplace_back(instance_vk);
    }

    // Reset the scheme so it can be reused for actual accumulation, maintaining the trace structure setting as is
    TraceStructure structure = trace_structure;
    *this = ClientIVC();
    this->trace_structure = structure;

    return vkeys;
}

/**
 * @brief Construct and verify a proof for the IVC
 * @note Use of this method only makes sense when the prover and verifier are the same entity, e.g. in
 * development/testing.
 *
 */
bool ClientIVC::prove_and_verify()
{
    auto proof = prove();

    auto verifier_inst = std::make_shared<VerifierInstance>(this->instance_vk);
    return verify(proof, { this->verifier_accumulator, verifier_inst });
}

} // namespace bb