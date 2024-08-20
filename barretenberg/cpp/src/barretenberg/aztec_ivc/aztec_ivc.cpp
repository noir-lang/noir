#include "barretenberg/aztec_ivc/aztec_ivc.hpp"

namespace bb {

/**
 * @brief Append logic to complete a kernel circuit
 * @details A kernel circuit may contain some combination of PG recursive verification, merge recursive verification,
 * and databus commitment consistency checks. This method appends this logic to a provided kernel circuit.
 *
 * @param circuit
 */
void AztecIVC::complete_kernel_circuit_logic(ClientCircuit& circuit)
{
    circuit.databus_propagation_data.is_kernel = true;

    // The folding verification queue should be either empty or contain two fold proofs
    ASSERT(verification_queue.empty() || verification_queue.size() == 2);

    for (auto& [proof, vkey] : verification_queue) {

        // Construct stdlib accumulator, vkey and proof
        auto stdlib_verifier_accum = std::make_shared<RecursiveVerifierInstance>(&circuit, verifier_accumulator);
        auto stdlib_vkey = std::make_shared<RecursiveVerificationKey>(&circuit, vkey);
        auto stdlib_proof = bb::convert_proof_to_witness(&circuit, proof);

        // Perform folding recursive verification
        FoldingRecursiveVerifier verifier{ &circuit, stdlib_verifier_accum, { stdlib_vkey } };
        auto verifier_accum = verifier.verify_folding_proof(stdlib_proof);
        verifier_accumulator = std::make_shared<VerifierInstance>(verifier_accum->get_value());

        // Perform databus commitment consistency checks and propagate return data commitments via public inputs
        bus_depot.execute(verifier.instances);
    }
    verification_queue.clear();

    // Recusively verify all merge proofs in queue
    for (auto& proof : merge_verification_queue) {
        goblin.verify_merge(circuit, proof);
    }
    merge_verification_queue.clear();
}

/**
 * @brief Execute prover work for instance accumulation
 * @details Construct an instance for the provided circuit. If this is the first instance in the IVC, simply initialize
 * the folding accumulator. Otherwise, execute the PG prover to fold the instance into the accumulator and produce a
 * folding proof. Also execute the merge protocol to produce a merge proof.
 *
 * @param circuit
 * @param precomputed_vk
 */
void AztecIVC::accumulate(ClientCircuit& circuit, const std::shared_ptr<VerificationKey>& precomputed_vk)
{
    // Construct merge proof for the present circuit and add to merge verification queue
    MergeProof merge_proof = goblin.prove_merge(circuit);
    merge_verification_queue.emplace_back(merge_proof);

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1069): Do proper aggregation with merge recursive
    // verifier.
    circuit.add_recursive_proof(stdlib::recursion::init_default_agg_obj_indices<ClientCircuit>(circuit));

    // Construct the prover instance for circuit
    auto prover_instance = std::make_shared<ProverInstance>(circuit, trace_structure);

    // Set the instance verification key from precomputed if available, else compute it
    instance_vk = precomputed_vk ? precomputed_vk : std::make_shared<VerificationKey>(prover_instance->proving_key);

    // If this is the first circuit simply initialize the prover and verifier accumulator instances
    if (!initialized) {
        fold_output.accumulator = prover_instance;
        verifier_accumulator = std::make_shared<VerifierInstance>(instance_vk);
        initialized = true;
    } else { // Otherwise, fold the new instance into the accumulator
        FoldingProver folding_prover({ fold_output.accumulator, prover_instance });
        fold_output = folding_prover.prove();

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
    max_block_size_tracker.print();               // print minimum structured sizes for each block
    ASSERT(verification_queue.size() == 1);       // ensure only a single fold proof remains in the queue
    ASSERT(merge_verification_queue.size() == 1); // ensure only a single merge proof remains in the queue
    FoldProof& fold_proof = verification_queue[0].proof;
    MergeProof& merge_proof = merge_verification_queue[0];
    return { fold_proof, decider_prove(), goblin.prove(merge_proof) };
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