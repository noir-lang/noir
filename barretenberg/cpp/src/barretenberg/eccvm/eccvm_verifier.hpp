#pragma once
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::honk {
template <typename Flavor> class ECCVMVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;

  public:
    explicit ECCVMVerifier_(const std::shared_ptr<VerificationKey>& verifier_key = nullptr);
    ECCVMVerifier_(const std::shared_ptr<VerificationKey>& key,
                   std::map<std::string, Commitment> commitments,
                   std::map<std::string, FF> pcs_fr_elements,
                   const std::shared_ptr<VerifierCommitmentKey>& pcs_verification_key,
                   const std::shared_ptr<Transcript>& transcript)
        : key(std::move(key))
        , commitments(std::move(commitments))
        , pcs_fr_elements(std::move(pcs_fr_elements))
        , pcs_verification_key(std::move(pcs_verification_key))
        , transcript(std::move(transcript))
    {}
    ECCVMVerifier_(ECCVMVerifier_&& other) noexcept;
    ECCVMVerifier_(const ECCVMVerifier_& other) = delete;
    ECCVMVerifier_& operator=(const ECCVMVerifier_& other) = delete;
    ECCVMVerifier_& operator=(ECCVMVerifier_&& other) noexcept;
    ~ECCVMVerifier_() = default;

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::map<std::string, FF> pcs_fr_elements;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

using ECCVMVerifierGrumpkin = ECCVMVerifier_<honk::flavor::ECCVM>;

} // namespace bb::honk
