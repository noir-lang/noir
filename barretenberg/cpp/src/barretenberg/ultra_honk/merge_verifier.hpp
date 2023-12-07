#pragma once

#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace proof_system::honk {

/**
 * @brief Verifier class for the Goblin ECC op queue transcript merge protocol
 *
 * @tparam Flavor
 */
template <typename Flavor> class MergeVerifier_ {
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using Commitment = typename Flavor::Commitment;
    using PCS = typename Flavor::PCS;
    using Curve = typename Flavor::Curve;
    using OpeningClaim = typename pcs::OpeningClaim<Curve>;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;

  public:
    std::shared_ptr<Transcript> transcript;
    std::shared_ptr<ECCOpQueue> op_queue;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;

    explicit MergeVerifier_();
    bool verify_proof(const plonk::proof& proof);
};

extern template class MergeVerifier_<honk::flavor::Ultra>;
extern template class MergeVerifier_<honk::flavor::GoblinUltra>;

} // namespace proof_system::honk