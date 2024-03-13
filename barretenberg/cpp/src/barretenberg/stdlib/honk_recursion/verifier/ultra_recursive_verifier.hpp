#pragma once
#include "barretenberg/flavor/goblin_ultra_recursive.hpp"
#include "barretenberg/flavor/ultra_recursive.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::stdlib::recursion::honk {
template <typename Flavor> class UltraRecursiveVerifier_ {
  public:
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using VerificationKey = typename Flavor::VerificationKey;
    using NativeVerificationKey = typename Flavor::NativeVerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Builder = typename Flavor::CircuitBuilder;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using PairingPoints = std::array<GroupElement, 2>;
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<Builder>>;

    explicit UltraRecursiveVerifier_(Builder* builder,
                                     const std::shared_ptr<NativeVerificationKey>& native_verifier_key);

    // TODO(luke): Eventually this will return something like aggregation_state but I'm simplifying for now until we
    // determine the exact interface. Simply returns the two pairing points.
    PairingPoints verify_proof(const HonkProof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    Builder* builder;
    std::shared_ptr<Transcript> transcript;
};

// Instance declarations for Ultra and Goblin-Ultra verifier circuits with both conventional Ultra and Goblin-Ultra
// arithmetization.
using UltraRecursiveVerifier = UltraRecursiveVerifier_<UltraCircuitBuilder>;
} // namespace bb::stdlib::recursion::honk
