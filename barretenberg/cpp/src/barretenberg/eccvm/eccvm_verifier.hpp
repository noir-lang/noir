#pragma once
#include "barretenberg/eccvm/eccvm_flavor.hpp"

namespace bb {
class ECCVMVerifier {
    using Flavor = ECCVMFlavor;
    using FF = typename Flavor::FF;
    using Curve = typename Flavor::Curve;
    using Commitment = typename Flavor::Commitment;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Transcript = typename Flavor::Transcript;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using PCS = typename Flavor::PCS;

  public:
    explicit ECCVMVerifier(const std::shared_ptr<VerificationKey>& verifier_key)
        : key(verifier_key){};

    explicit ECCVMVerifier(const std::shared_ptr<ECCVMVerifier::ProvingKey>& proving_key)
        : ECCVMVerifier(std::make_shared<ECCVMFlavor::VerificationKey>(proving_key)){};

    bool verify_proof(const HonkProof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::map<std::string, FF> pcs_fr_elements;
    std::shared_ptr<Transcript> transcript;
};
} // namespace bb
