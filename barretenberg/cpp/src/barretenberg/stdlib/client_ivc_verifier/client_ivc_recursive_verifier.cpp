#include "client_ivc_recursive_verifier.hpp"

namespace bb::stdlib::recursion::honk {

/**
 * @brief Performs recursive verification of the Client IVC proof.
 *
 * @todo (https://github.com/AztecProtocol/barretenberg/issues/934):  Add logic for accumulating the pairing points
 * produced by the verifiers (and potentially IPA accumulators for ECCVM verifier)
 */
void ClientIVCRecursiveVerifier::verify(const ClientIVC::Proof& proof)
{
    // Construct stdlib accumulator, vkey and proof
    auto stdlib_verifier_accum =
        std::make_shared<RecursiveVerifierInstance>(builder.get(), verifier_input.fold_input.accumulator);
    auto stdlib_instance_vk =
        std::make_shared<RecursiveVerificationKey>(builder.get(), verifier_input.fold_input.instance_vks[0]);
    auto stdlib_proof = bb::convert_proof_to_witness(builder.get(), proof.folding_proof);

    // Perform recursive folding verification
    FoldingVerifier folding_verifier{ builder.get(), stdlib_verifier_accum, { stdlib_instance_vk } };
    auto recursive_verifier_accumulator = folding_verifier.verify_folding_proof(stdlib_proof);
    auto native_verifier_acc =
        std::make_shared<FoldVerifierInput::Instance>(recursive_verifier_accumulator->get_value());

    // Perform recursive decider verification
    DeciderVerifier decider{ builder.get(), native_verifier_acc };
    decider.verify_proof(proof.decider_proof);

    // Perform Goblin recursive verification
    GoblinVerifier goblin_verifier{ builder.get(), verifier_input.goblin_input };
    goblin_verifier.verify(proof.goblin_proof);
}

} // namespace bb::stdlib::recursion::honk