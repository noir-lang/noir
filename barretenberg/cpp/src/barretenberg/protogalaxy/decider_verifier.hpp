#pragma once
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::honk {
template <typename Flavor> class DeciderVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;

  public:
    explicit DeciderVerifier_();
    explicit DeciderVerifier_(const std::shared_ptr<Transcript>& transcript,
                              const std::shared_ptr<VerificationKey>& verifier_key = nullptr);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

using DeciderVerifier = DeciderVerifier_<honk::flavor::Ultra>;

} // namespace bb::honk
