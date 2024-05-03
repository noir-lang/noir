#include "decider_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

template <typename Flavor>
DeciderVerifier_<Flavor>::DeciderVerifier_(const std::shared_ptr<VerifierInstance>& accumulator,
                                           const std::shared_ptr<Transcript>& transcript)
    : accumulator(accumulator)
    , pcs_verification_key(accumulator->verification_key->pcs_verification_key)
    , transcript(transcript)
{}

template <typename Flavor>
DeciderVerifier_<Flavor>::DeciderVerifier_()
    : pcs_verification_key(std::make_unique<VerifierCommitmentKey>())
    , transcript(std::make_shared<Transcript>())
{}

/**
 * @brief This function verifies an Ultra Honk proof for a given Flavor, produced for a relaxed instance (ϕ, \vec{β*},
 * e*).
 *
 */
template <typename Flavor> bool DeciderVerifier_<Flavor>::verify_proof(const HonkProof& proof)
{
    using PCS = typename Flavor::PCS;
    using ZeroMorph = ZeroMorphVerifier_<PCS>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;

    transcript = std::make_shared<Transcript>(proof);

    VerifierCommitments commitments{ accumulator->verification_key, accumulator->witness_commitments };

    auto sumcheck = SumcheckVerifier<Flavor>(
        static_cast<size_t>(accumulator->verification_key->log_circuit_size), transcript, accumulator->target_sum);

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(accumulator->relation_parameters, accumulator->alphas, accumulator->gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
                                            commitments.get_to_be_shifted(),
                                            claimed_evaluations.get_unshifted(),
                                            claimed_evaluations.get_shifted(),
                                            multivariate_challenge,
                                            transcript);

    auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);

    return sumcheck_verified.value() && verified;
}

template class DeciderVerifier_<UltraFlavor>;
template class DeciderVerifier_<GoblinUltraFlavor>;

} // namespace bb
