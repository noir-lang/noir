#include "client_ivc_recursive_verifier.hpp"

namespace bb::stdlib::recursion::honk {

void ClientIVCRecursiveVerifier::verify(const ClientIVC::Proof& proof, VerifierInput& data)
{
    // WORKTODO: Perform Goblin recursive verification here

    // Perform recursive folding verification
    FoldingVerifier folding_verifier{ builder, data };
    auto recursive_verifier_accumulator = folding_verifier.verify_folding_proof(proof.folding_proof);
    auto native_verifier_acc = std::make_shared<VerifierInput::Instance>(recursive_verifier_accumulator->get_value());

    // Perform recursive decider verification
    DeciderVerifier decider{ builder, native_verifier_acc };
    decider.verify_proof(proof.decider_proof);
}

} // namespace bb::stdlib::recursion::honk