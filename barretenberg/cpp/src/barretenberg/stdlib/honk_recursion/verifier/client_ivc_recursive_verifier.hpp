#pragma once
#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/decider_recursive_verifier.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/goblin_recursive_verifier.hpp"

namespace bb::stdlib::recursion::honk {
class ClientIVCRecursiveVerifier {
    using Builder = UltraCircuitBuilder;                   // The circuit will be an Ultra circuit
    using RecursiveFlavor = MegaRecursiveFlavor_<Builder>; // The verifier algorithms are Mega
    using RecursiveVerifierInstances = RecursiveVerifierInstances_<RecursiveFlavor, 2>;
    using DeciderVerifier = DeciderRecursiveVerifier_<RecursiveFlavor>;
    using FoldingVerifier = ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;
    using GoblinVerifier = GoblinRecursiveVerifier;

  public:
    using Proof = ClientIVC::Proof;
    using FoldVerifierInput = FoldingVerifier::VerifierInput;
    using GoblinVerifierInput = GoblinVerifier::VerifierInput;
    struct VerifierInput {
        FoldVerifierInput fold_input;
        GoblinVerifierInput goblin_input;
    };

    ClientIVCRecursiveVerifier(std::shared_ptr<Builder> builder, VerifierInput& verifier_input)
        : builder(builder)
        , verifier_input(verifier_input){};

    void verify(const ClientIVC::Proof&);

  private:
    std::shared_ptr<Builder> builder;
    VerifierInput verifier_input;
};
} // namespace bb::stdlib::recursion::honk