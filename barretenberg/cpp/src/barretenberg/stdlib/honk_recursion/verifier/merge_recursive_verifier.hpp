#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

namespace bb::stdlib::recursion::goblin {
template <typename CircuitBuilder> class MergeRecursiveVerifier_ {
  public:
    using Curve = bn254<CircuitBuilder>;
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::Element;
    using GroupElement = typename Curve::Element;
    using KZG = ::bb::KZG<Curve>;
    using OpeningClaim = ::bb::OpeningClaim<Curve>;
    using PairingPoints = std::array<GroupElement, 2>;
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<CircuitBuilder>>;

    CircuitBuilder* builder;
    std::shared_ptr<Transcript> transcript;

    static constexpr size_t NUM_WIRES = MegaArith<FF>::NUM_WIRES;

    explicit MergeRecursiveVerifier_(CircuitBuilder* builder);

    PairingPoints verify_proof(const HonkProof& proof);
};

} // namespace bb::stdlib::recursion::goblin
