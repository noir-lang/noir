#pragma once

#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::honk {

/**
 * @brief Prover class for the Goblin ECC op queue transcript merge protocol
 *
 * @tparam Flavor
 */
template <typename Flavor> class MergeProver_ {
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using Commitment = typename Flavor::Commitment;
    using PCS = typename Flavor::PCS;
    using Curve = typename Flavor::Curve;
    using OpeningClaim = typename pcs::ProverOpeningClaim<Curve>;
    using OpeningPair = typename pcs::OpeningPair<Curve>;
    using Transcript = BaseTranscript;

  public:
    std::shared_ptr<Transcript> transcript;
    std::shared_ptr<ECCOpQueue> op_queue;
    std::shared_ptr<CommitmentKey> pcs_commitment_key;

    explicit MergeProver_(const std::shared_ptr<CommitmentKey>&,
                          const std::shared_ptr<ECCOpQueue>&,
                          const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    BBERG_PROFILE plonk::proof& construct_proof();

  private:
    plonk::proof proof;
};

} // namespace bb::honk
