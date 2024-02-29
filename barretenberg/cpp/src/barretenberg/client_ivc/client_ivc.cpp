#include "barretenberg/client_ivc/client_ivc.hpp"

namespace bb {

ClientIVC::ClientIVC()
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/723):
    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(goblin.op_queue);
}

/**
 * @brief Initialize the IVC with a first circuit
 * @details Initializes the accumulator and performs the initial goblin merge
 *
 * @param circuit
 */
void ClientIVC::initialize(ClientCircuit& circuit)
{
    goblin.merge(circuit); // Construct new merge proof
    Composer composer;
    prover_fold_output.accumulator = composer.create_prover_instance(circuit);
}

/**
 * @brief Accumulate a circuit into the IVC scheme
 * @details Performs goblin merge, generates circuit instance, folds into accumulator and constructs a folding proof
 *
 * @param circuit Circuit to be accumulated/folded
 * @return FoldProof
 */
ClientIVC::FoldProof ClientIVC::accumulate(ClientCircuit& circuit)
{
    goblin.merge(circuit); // Add recursive merge verifier and construct new merge proof
    Composer composer;
    prover_instance = composer.create_prover_instance(circuit);
    auto folding_prover = composer.create_folding_prover({ prover_fold_output.accumulator, prover_instance });
    prover_fold_output = folding_prover.fold_instances();
    return prover_fold_output.folding_data;
}

/**
 * @brief Construct a proof for the IVC, which, if verified, fully establishes its correctness
 *
 * @return Proof
 */
ClientIVC::Proof ClientIVC::prove()
{
    return { prover_fold_output.folding_data, decider_prove(), goblin.prove() };
}

/**
 * @brief Verify a full proof of the IVC
 *
 * @param proof
 * @return bool
 */
bool ClientIVC::verify(Proof& proof, const std::vector<VerifierAccumulator>& verifier_instances)
{
    // Goblin verification (merge, eccvm, translator)
    bool goblin_verified = goblin.verify(proof.goblin_proof);

    // Decider verification
    Composer composer;
    auto folding_verifier = composer.create_folding_verifier({ verifier_instances[0], verifier_instances[1] });
    auto verifier_accumulator = folding_verifier.verify_folding_proof(proof.fold_proof);

    auto decider_verifier = composer.create_decider_verifier(verifier_accumulator);
    bool decision = decider_verifier.verify_proof(proof.decider_proof);
    return goblin_verified && decision;
}

/**
 * @brief Internal method for constructing a decider proof
 *
 * @return HonkProof
 */
HonkProof ClientIVC::decider_prove() const
{
    Composer composer;
    auto decider_prover = composer.create_decider_prover(prover_fold_output.accumulator);
    return decider_prover.construct_proof();
}

/**
 * @brief Precompute the array of verification keys by simulating folding. There will be 4 different verification keys:
 * initial function verification key (without recursive merge verifier), subsequent function verification key (with
 * recursive merge verifier), initial kernel verification key (with recursive merge verifier appended, no previous
 * kernel to fold), "full" kernel verification key( two recursive folding verifiers and merge verifier).
 *
 */
void ClientIVC::precompute_folding_verification_keys()
{
    using VerifierInstance = VerifierInstance_<GoblinUltraFlavor>;

    ClientCircuit initial_function_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_mock_function_circuit(initial_function_circuit);

    // Initialise both the first prover and verifier accumulator from the inital function circuit
    initialize(initial_function_circuit);
    vks.first_func_vk = prover_fold_output.accumulator->verification_key;
    auto initial_verifier_acc = std::make_shared<VerifierInstance>(vks.first_func_vk);

    // Accumulate the next function circuit
    ClientCircuit function_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_mock_function_circuit(function_circuit);
    auto function_fold_proof = accumulate(function_circuit);

    // Create its verification key (we have called accumulate so it includes the recursive merge verifier)
    vks.func_vk = prover_instance->verification_key;

    // Create the initial kernel iteration and precompute its verification key
    ClientCircuit kernel_circuit{ goblin.op_queue };
    auto kernel_acc = GoblinMockCircuits::construct_mock_folding_kernel(
        kernel_circuit, { function_fold_proof, vks.func_vk }, {}, initial_verifier_acc);
    auto kernel_fold_proof = accumulate(kernel_circuit);
    vks.first_kernel_vk = prover_instance->verification_key;

    // Create another mock function circuit to run the full kernel
    function_circuit = ClientCircuit{ goblin.op_queue };
    GoblinMockCircuits::construct_mock_function_circuit(function_circuit);
    function_fold_proof = accumulate(function_circuit);

    // Create the full kernel circuit and compute verification key
    kernel_circuit = GoblinUltraCircuitBuilder{ goblin.op_queue };
    kernel_acc = GoblinMockCircuits::construct_mock_folding_kernel(
        kernel_circuit, { function_fold_proof, vks.func_vk }, { kernel_fold_proof, vks.first_kernel_vk }, kernel_acc);
    kernel_fold_proof = accumulate(kernel_circuit);

    vks.kernel_vk = prover_instance->verification_key;

    // Clean the ivc state
    goblin.op_queue = std::make_shared<Goblin::OpQueue>();
    goblin.merge_proof_exists = false;
    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(goblin.op_queue);
}

} // namespace bb