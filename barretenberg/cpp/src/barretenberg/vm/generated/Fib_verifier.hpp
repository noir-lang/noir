

#pragma once
#include "barretenberg/flavor/generated/Fib_flavor.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace proof_system::honk {
class FibVerifier {
    using Flavor = honk::flavor::FibFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    using VerificationKey = Flavor::VerificationKey;
    using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;
    using Transcript = Flavor::Transcript;

  public:
    explicit FibVerifier(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    FibVerifier(FibVerifier&& other) noexcept;
    FibVerifier(const FibVerifier& other) = delete;

    FibVerifier& operator=(const FibVerifier& other) = delete;
    FibVerifier& operator=(FibVerifier&& other) noexcept;

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

} // namespace proof_system::honk
