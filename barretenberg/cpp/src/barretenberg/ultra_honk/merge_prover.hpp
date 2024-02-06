#pragma once

#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

/**
 * @brief Prover class for the Goblin ECC op queue transcript merge protocol
 *
 */
class MergeProver {
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Polynomial = polynomial;
    using CommitmentKey = bb::CommitmentKey<Curve>;
    using Commitment = Curve::AffineElement;
    using PCS = bb::KZG<Curve>;
    using OpeningClaim = typename bb::ProverOpeningClaim<Curve>;
    using Transcript = BaseTranscript;

  public:
    std::shared_ptr<Transcript> transcript;

    explicit MergeProver(const std::shared_ptr<ECCOpQueue>&);

    BB_PROFILE HonkProof construct_proof();

  private:
    std::shared_ptr<ECCOpQueue> op_queue;
    std::shared_ptr<CommitmentKey> pcs_commitment_key;
    static constexpr size_t NUM_WIRES = GoblinUltraFlavor::NUM_WIRES;
};

} // namespace bb
