#pragma once
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {
template <typename Flavor> class DeciderVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;
    using VerifierInstance = VerifierInstance_<Flavor>;
    using DeciderProof = std::vector<FF>;

  public:
    explicit DeciderVerifier_();
    /**
     * @brief Constructor from prover instance and a transcript assumed to be initialized with a full honk proof
     * @details Used in the case where an external transcript already exists and has been initialized with a proof, e.g.
     * when the decider is being used in the context of the larger honk protocol.
     *
     */
    explicit DeciderVerifier_(const std::shared_ptr<VerifierInstance>& accumulator,
                              const std::shared_ptr<Transcript>& transcript);
    /**
     * @brief Constructor from prover instance
     *
     */
    explicit DeciderVerifier_(const std::shared_ptr<VerifierInstance>& accumulator);

    bool verify_proof(const DeciderProof&); // used when a decider proof is known explicitly
    bool verify();                          // used when transcript that has been initialized with a proof
    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierInstance> accumulator;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

using UltraDeciderVerifier = DeciderVerifier_<UltraFlavor>;
using MegaDeciderVerifier = DeciderVerifier_<MegaFlavor>;

} // namespace bb
