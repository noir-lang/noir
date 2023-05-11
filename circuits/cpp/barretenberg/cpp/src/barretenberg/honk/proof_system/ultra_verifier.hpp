#pragma once
// #include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"

namespace proof_system::honk {
template <typename Flavor> class UltraVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCSVerificationKey = typename Flavor::PCSParams::VK;

  public:
    explicit UltraVerifier_(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    UltraVerifier_(UltraVerifier_&& other);
    UltraVerifier_(const UltraVerifier_& other) = delete;
    UltraVerifier_& operator=(const UltraVerifier_& other) = delete;
    UltraVerifier_& operator=(UltraVerifier_&& other);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> kate_g1_elements;
    std::map<std::string, FF> kate_fr_elements;
    std::shared_ptr<PCSVerificationKey> kate_verification_key;
    VerifierTranscript<FF> transcript;
};

extern template class UltraVerifier_<honk::flavor::Ultra>;

using UltraVerifier = UltraVerifier_<honk::flavor::Ultra>;

} // namespace proof_system::honk
