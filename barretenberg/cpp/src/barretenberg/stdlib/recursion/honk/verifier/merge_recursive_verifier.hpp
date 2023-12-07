#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/recursion/honk/transcript/transcript.hpp"

namespace proof_system::plonk::stdlib::recursion::goblin {
template <typename CircuitBuilder> class MergeRecursiveVerifier_ {
  public:
    using Curve = bn254<CircuitBuilder>;
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::Element;
    using GroupElement = typename Curve::Element;
    using KZG = ::proof_system::honk::pcs::kzg::KZG<Curve>;
    using OpeningClaim = ::proof_system::honk::pcs::OpeningClaim<Curve>;
    using PairingPoints = std::array<GroupElement, 2>;
    using Transcript = honk::Transcript<CircuitBuilder>;

    CircuitBuilder* builder;
    std::shared_ptr<Transcript> transcript;

    static constexpr size_t NUM_WIRES = arithmetization::UltraHonk<FF>::NUM_WIRES;

    explicit MergeRecursiveVerifier_(CircuitBuilder* builder);

    PairingPoints verify_proof(const plonk::proof& proof);
};

extern template class MergeRecursiveVerifier_<GoblinUltraCircuitBuilder>;

} // namespace proof_system::plonk::stdlib::recursion::goblin
