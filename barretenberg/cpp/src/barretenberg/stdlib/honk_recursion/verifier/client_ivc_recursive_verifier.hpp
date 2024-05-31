#pragma once
#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/decider_recursive_verifier.hpp"

namespace bb::stdlib::recursion::honk {
class ClientIVCRecursiveVerifier {
    using Builder = UltraCircuitBuilder;                   // The circuit will be an Ultra circuit
    using RecursiveFlavor = MegaRecursiveFlavor_<Builder>; // The verifier algorithms are Mega
    using RecursiveVerifierInstances = RecursiveVerifierInstances_<RecursiveFlavor, 2>;

    Builder* builder;

  public:
    using DeciderVerifier = DeciderRecursiveVerifier_<RecursiveFlavor>;
    using FoldingVerifier = ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;
    using VerifierInput = FoldingVerifier::VerifierInput;

    ClientIVCRecursiveVerifier(Builder* builder)
        : builder(builder){};

    void verify(const ClientIVC::Proof&, VerifierInput&);
};
} // namespace bb::stdlib::recursion::honk