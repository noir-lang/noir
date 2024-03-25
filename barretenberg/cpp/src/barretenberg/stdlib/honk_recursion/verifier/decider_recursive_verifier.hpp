#pragma once
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/recursive_verifier_instance.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_recursive_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_recursive_flavor.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::stdlib::recursion::honk {
template <typename Flavor> class DeciderRecursiveVerifier_ {
    using NativeFlavor = typename Flavor::NativeFlavor;
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Builder = typename Flavor::CircuitBuilder;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using PairingPoints = std::array<GroupElement, 2>;
    using Instance = RecursiveVerifierInstance_<Flavor>;
    using NativeInstance = bb::VerifierInstance_<NativeFlavor>;
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<Builder>>;

  public:
    explicit DeciderRecursiveVerifier_(Builder* builder, std::shared_ptr<NativeInstance> accumulator)
        : builder(builder)
        , accumulator(std::make_shared<Instance>(builder, accumulator)){};

    PairingPoints verify_proof(const HonkProof& proof);

    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    Builder* builder;
    std::shared_ptr<Instance> accumulator;
    std::shared_ptr<Transcript> transcript;
};

} // namespace bb::stdlib::recursion::honk