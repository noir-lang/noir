#pragma once

#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/op_queue/ecc_op_queue.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

/**
 * @brief Prover class for the Goblin ECC op queue transcript merge protocol
 *
 */
template <typename Flavor> class MergeProver_ {
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using Commitment = typename Flavor::Commitment;
    using PCS = typename Flavor::PCS;
    using Curve = typename Flavor::Curve;
    using OpeningClaim = ProverOpeningClaim<Curve>;
    using OpeningPair = bb::OpeningPair<Curve>;
    using Transcript = NativeTranscript;

  public:
    std::shared_ptr<Transcript> transcript;

    explicit MergeProver_(const std::shared_ptr<ECCOpQueue>&);

    BB_PROFILE HonkProof construct_proof();

  private:
    std::shared_ptr<ECCOpQueue> op_queue;
    std::shared_ptr<CommitmentKey> pcs_commitment_key;
    static constexpr size_t NUM_WIRES = GoblinUltraFlavor::NUM_WIRES;
};

} // namespace bb
