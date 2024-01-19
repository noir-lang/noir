#include "./ultra_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace bb;
using namespace bb::honk::sumcheck;

namespace bb::honk {
template <typename Flavor>
UltraVerifier_<Flavor>::UltraVerifier_(const std::shared_ptr<Transcript>& transcript,
                                       const std::shared_ptr<VerificationKey>& verifier_key)
    : key(verifier_key)
    , transcript(transcript)
{}

/**
 * @brief Construct an UltraVerifier directly from a verification key
 *
 * @tparam Flavor
 * @param verifier_key
 */
template <typename Flavor>
UltraVerifier_<Flavor>::UltraVerifier_(const std::shared_ptr<VerificationKey>& verifier_key)
    : key(verifier_key)
    , pcs_verification_key(std::make_unique<VerifierCommitmentKey>(0, bb::srs::get_crs_factory()))
    , transcript(std::make_shared<Transcript>())
{}

template <typename Flavor>
UltraVerifier_<Flavor>::UltraVerifier_(UltraVerifier_&& other)
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

template <typename Flavor> UltraVerifier_<Flavor>& UltraVerifier_<Flavor>::operator=(UltraVerifier_&& other)
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

/**
 * @brief This function verifies an Ultra Honk proof for a given Flavor.
 *
 */
template <typename Flavor> bool UltraVerifier_<Flavor>::verify_proof(const plonk::proof& proof)
{
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using Curve = typename Flavor::Curve;
    using ZeroMorph = pcs::zeromorph::ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;

    bb::RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof.proof_data);

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    // TODO(Adrian): Change the initialization of the transcript to take the VK hash?
    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");
    const auto public_input_size = transcript->template receive_from_prover<uint32_t>("public_input_size");
    const auto pub_inputs_offset = transcript->template receive_from_prover<uint32_t>("pub_inputs_offset");

    if (circuit_size != key->circuit_size) {
        return false;
    }
    if (public_input_size != key->num_public_inputs) {
        return false;
    }

    std::vector<FF> public_inputs;
    for (size_t i = 0; i < public_input_size; ++i) {
        auto public_input_i = transcript->template receive_from_prover<FF>("public_input_" + std::to_string(i));
        public_inputs.emplace_back(public_input_i);
    }

    // Get commitments to first three wire polynomials
    commitments.w_l = transcript->template receive_from_prover<Commitment>(commitment_labels.w_l);
    commitments.w_r = transcript->template receive_from_prover<Commitment>(commitment_labels.w_r);
    commitments.w_o = transcript->template receive_from_prover<Commitment>(commitment_labels.w_o);

    // If Goblin, get commitments to ECC op wire polynomials and DataBus columns
    if constexpr (IsGoblinFlavor<Flavor>) {
        commitments.ecc_op_wire_1 =
            transcript->template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_1);
        commitments.ecc_op_wire_2 =
            transcript->template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_2);
        commitments.ecc_op_wire_3 =
            transcript->template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_3);
        commitments.ecc_op_wire_4 =
            transcript->template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_4);
        commitments.calldata = transcript->template receive_from_prover<Commitment>(commitment_labels.calldata);
        commitments.calldata_read_counts =
            transcript->template receive_from_prover<Commitment>(commitment_labels.calldata_read_counts);
    }

    // Get challenge for sorted list batching and wire four memory records
    FF eta = transcript->get_challenge("eta");
    relation_parameters.eta = eta;

    // Get commitments to sorted list accumulator and fourth wire
    commitments.sorted_accum = transcript->template receive_from_prover<Commitment>(commitment_labels.sorted_accum);
    commitments.w_4 = transcript->template receive_from_prover<Commitment>(commitment_labels.w_4);

    // Get permutation challenges
    auto [beta, gamma] = challenges_to_field_elements<FF>(transcript->get_challenges("beta", "gamma"));

    // If Goblin (i.e. using DataBus) receive commitments to log-deriv inverses polynomial
    if constexpr (IsGoblinFlavor<Flavor>) {
        commitments.lookup_inverses =
            transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_inverses);
    }

    const FF public_input_delta =
        compute_public_input_delta<Flavor>(public_inputs, beta, gamma, circuit_size, pub_inputs_offset);
    const FF lookup_grand_product_delta = compute_lookup_grand_product_delta<FF>(beta, gamma, circuit_size);

    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = public_input_delta;
    relation_parameters.lookup_grand_product_delta = lookup_grand_product_delta;

    // Get commitment to permutation and lookup grand products
    commitments.z_perm = transcript->template receive_from_prover<Commitment>(commitment_labels.z_perm);
    commitments.z_lookup = transcript->template receive_from_prover<Commitment>(commitment_labels.z_lookup);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);
    RelationSeparator alphas;
    for (size_t idx = 0; idx < alphas.size(); idx++) {
        alphas[idx] = transcript->get_challenge("Sumcheck:alpha_" + std::to_string(idx));
    }

    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->get_challenge("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alphas, gate_challenges);

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

template class UltraVerifier_<honk::flavor::Ultra>;
template class UltraVerifier_<honk::flavor::GoblinUltra>;

} // namespace bb::honk
