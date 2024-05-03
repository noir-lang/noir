#include "barretenberg/stdlib/honk_recursion/verifier/decider_recursive_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::stdlib::recursion::honk {

/**
 * @brief This function verifies an Ultra Honk proof for a given Flavor, produced for a relaxed instance (ϕ, \vec{β*},
 * e*).
 *
 */
template <typename Flavor>
std::array<typename Flavor::GroupElement, 2> DeciderRecursiveVerifier_<Flavor>::verify_proof(const HonkProof& proof)
{
    using Sumcheck = ::bb::SumcheckVerifier<Flavor>;
    using PCS = typename Flavor::PCS;
    using ZeroMorph = ::bb::ZeroMorphVerifier_<PCS>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using Transcript = typename Flavor::Transcript;

    StdlibProof<Builder> stdlib_proof = bb::convert_proof_to_witness(builder, proof);
    transcript = std::make_shared<Transcript>(stdlib_proof);

    VerifierCommitments commitments{ accumulator->verification_key, accumulator->witness_commitments };

    auto sumcheck = Sumcheck(
        static_cast<size_t>(accumulator->verification_key->log_circuit_size), transcript, accumulator->target_sum);

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(accumulator->relation_parameters, accumulator->alphas, accumulator->gate_challenges);

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
                                            commitments.get_to_be_shifted(),
                                            claimed_evaluations.get_unshifted(),
                                            claimed_evaluations.get_shifted(),
                                            multivariate_challenge,
                                            transcript);

    return pairing_points;
}

template class DeciderRecursiveVerifier_<bb::UltraRecursiveFlavor_<UltraCircuitBuilder>>;
template class DeciderRecursiveVerifier_<bb::GoblinUltraRecursiveFlavor_<GoblinUltraCircuitBuilder>>;
template class DeciderRecursiveVerifier_<bb::UltraRecursiveFlavor_<GoblinUltraCircuitBuilder>>;
template class DeciderRecursiveVerifier_<bb::GoblinUltraRecursiveFlavor_<UltraCircuitBuilder>>;
template class DeciderRecursiveVerifier_<bb::UltraRecursiveFlavor_<CircuitSimulatorBN254>>;
template class DeciderRecursiveVerifier_<bb::GoblinUltraRecursiveFlavor_<CircuitSimulatorBN254>>;
} // namespace bb::stdlib::recursion::honk
