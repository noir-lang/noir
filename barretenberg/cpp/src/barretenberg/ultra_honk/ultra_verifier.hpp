#pragma once
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace proof_system::honk {
template <typename Flavor> class UltraVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;

  public:
    explicit UltraVerifier_(const std::shared_ptr<Transcript>& transcript,
                            const std::shared_ptr<VerificationKey>& verifier_key = nullptr);
    explicit UltraVerifier_(const std::shared_ptr<VerificationKey>& verifier_key);
    UltraVerifier_(UltraVerifier_&& other);

    UltraVerifier_& operator=(const UltraVerifier_& other) = delete;
    UltraVerifier_& operator=(UltraVerifier_&& other);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

extern template class UltraVerifier_<honk::flavor::Ultra>;
extern template class UltraVerifier_<honk::flavor::GoblinUltra>;

using UltraVerifier = UltraVerifier_<honk::flavor::Ultra>;

} // namespace proof_system::honk
