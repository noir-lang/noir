#pragma once
#include "barretenberg/honk/flavor/ecc_vm.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"

namespace proof_system::honk {
template <typename Flavor> class ECCVMVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;

  public:
    explicit ECCVMVerifier_(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    ECCVMVerifier_(std::shared_ptr<VerificationKey> key,
                   std::map<std::string, Commitment> commitments,
                   std::map<std::string, FF> pcs_fr_elements,
                   std::shared_ptr<VerifierCommitmentKey> pcs_verification_key,
                   VerifierTranscript<FF> transcript)
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
    VerifierTranscript<FF> transcript;
};

extern template class ECCVMVerifier_<honk::flavor::ECCVM>;
extern template class ECCVMVerifier_<honk::flavor::ECCVMGrumpkin>;

using ECCVMVerifier = ECCVMVerifier_<honk::flavor::ECCVM>;
using ECCVMVerifierGrumpkin = ECCVMVerifier_<honk::flavor::ECCVMGrumpkin>;

} // namespace proof_system::honk
