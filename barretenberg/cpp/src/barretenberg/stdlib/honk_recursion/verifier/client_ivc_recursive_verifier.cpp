#include "client_ivc_recursive_verifier.hpp"

namespace bb::stdlib::recursion::honk {

void ClientIVCRecursiveVerifier::verify(const ClientIVC::Proof& proof)
{
    // Perform recursive folding verification
    FoldingVerifier folding_verifier{ builder.get(), verifier_input.fold_input };
    auto recursive_verifier_accumulator = folding_verifier.verify_folding_proof(proof.folding_proof);
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