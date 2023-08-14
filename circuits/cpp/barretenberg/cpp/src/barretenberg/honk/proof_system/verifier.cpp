#include "./verifier.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"

using namespace barretenberg;
using namespace proof_system::honk::sumcheck;

namespace proof_system::honk {
template <typename Flavor>
StandardVerifier_<Flavor>::StandardVerifier_(std::shared_ptr<typename Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

template <typename Flavor>
StandardVerifier_<Flavor>::StandardVerifier_(StandardVerifier_&& other)
    : key(other.key)
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

template <typename Flavor> StandardVerifier_<Flavor>& StandardVerifier_<Flavor>::operator=(StandardVerifier_&& other)
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    pcs_fr_elements.clear();
    return *this;
}

/**
* @brief This function verifies a Honk proof for given program settings.
*
* @details A Standard Honk proof contains the following:
    Multilinear evaluations:
        w_i(X),        i = 1,2,3
        sigma_i(X),    i = 1,2,3
        q_i(X),        i = 1,2,3,4,5
        z_perm(X),
        L_0(X),
        id(X)

    Univariate evaluations:
        a_0 = Fold_{-r}^(0)(-r),
        a_l = Fold^(l)(-r^{2^l}), i = 1,...,d-1

    Univariate polynomials (evaluations over MAX_RELATION_LENGTH-many points):
        S_l, l = 0,...,d-1

    Commitments:
        [w_i]_1,        i = 1,2,3
        [z_perm]_1,
        [Fold^(l)]_1,   l = 1,...,d-1
        [Q]_1,
        [W]_1
*/
template <typename Flavor> bool StandardVerifier_<Flavor>::verify_proof(const plonk::proof& proof)
{
    using FF = typename Flavor::FF;
    using GroupElement = typename Flavor::GroupElement;
    using Commitment = typename Flavor::Commitment;
    using Curve = typename Flavor::Curve;
    using Gemini = pcs::gemini::GeminiVerifier_<Curve>;
    using Shplonk = pcs::shplonk::ShplonkVerifier_<Curve>;
    using PCS = typename Flavor::PCS;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;

    transcript = VerifierTranscript<FF>{ proof.proof_data };

    auto commitments = VerifierCommitments(key);
    auto commitment_labels = CommitmentLabels();

    // TODO(Adrian): Change the initialization of the transcript to take the VK hash?
    const auto circuit_size = transcript.template receive_from_prover<uint32_t>("circuit_size");
    const auto public_input_size = transcript.template receive_from_prover<uint32_t>("public_input_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }
    if (public_input_size != key->num_public_inputs) {
        return false;
    }

    std::vector<FF> public_inputs;
    for (size_t i = 0; i < public_input_size; ++i) {
        auto public_input_i = transcript.template receive_from_prover<FF>("public_input_" + std::to_string(i));
        public_inputs.emplace_back(public_input_i);
    }

    // Get commitments to the wires
    commitments.w_l = transcript.template receive_from_prover<Commitment>(commitment_labels.w_l);
    commitments.w_r = transcript.template receive_from_prover<Commitment>(commitment_labels.w_r);
    commitments.w_o = transcript.template receive_from_prover<Commitment>(commitment_labels.w_o);

    // Get permutation challenges
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    const FF public_input_delta = compute_public_input_delta<Flavor>(public_inputs, beta, gamma, circuit_size);

    sumcheck::RelationParameters<FF> relation_parameters{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    // Get commitment to Z_PERM
    commitments.z_perm = transcript.template receive_from_prover<Commitment>(commitment_labels.z_perm);

    // Execute Sumcheck Verifier
    auto sumcheck = SumcheckVerifier<Flavor>(circuit_size, transcript);
    std::optional sumcheck_output = sumcheck.verify(relation_parameters);

    // If Sumcheck does not return an output, sumcheck verification has failed
    if (!sumcheck_output.has_value()) {
        return false;
    }

    auto [multivariate_challenge, purported_evaluations] = *sumcheck_output;

    // Execute Gemini/Shplonk verification:

    // Construct inputs for Gemini verifier:
    // - Multivariate opening point u = (u_0, ..., u_{d-1})
    // - batched unshifted and to-be-shifted polynomial commitments
    auto batched_commitment_unshifted = GroupElement::zero();
    auto batched_commitment_to_be_shifted = GroupElement::zero();

    // Compute powers of batching challenge rho
    FF rho = transcript.get_challenge("rho");
    std::vector<FF> rhos = pcs::gemini::powers_of_rho(rho, Flavor::NUM_ALL_ENTITIES);

    // Compute batched multivariate evaluation
    FF batched_evaluation = FF::zero();
    size_t evaluation_idx = 0;
    for (auto& value : purported_evaluations.get_unshifted_then_shifted()) {
        batched_evaluation += value * rhos[evaluation_idx];
        ++evaluation_idx;
    }

    // Construct batched commitment for NON-shifted polynomials
    size_t commitment_idx = 0;
    for (auto& commitment : commitments.get_unshifted()) {
        batched_commitment_unshifted += commitment * rhos[commitment_idx];
        ++commitment_idx;
    }

    // Construct batched commitment for to-be-shifted polynomials
    for (auto& commitment : commitments.get_to_be_shifted()) {
        batched_commitment_to_be_shifted += commitment * rhos[commitment_idx];
        ++commitment_idx;
    }

    // Produce a Gemini claim consisting of:
    // - d+1 commitments [Fold_{r}^(0)], [Fold_{-r}^(0)], and [Fold^(l)], l = 1:d-1
    // - d+1 evaluations a_0_pos, and a_l, l = 0:d-1
    auto gemini_claim = Gemini::reduce_verification(multivariate_challenge,
                                              batched_evaluation,
                                              batched_commitment_unshifted,
                                              batched_commitment_to_be_shifted,
                                              transcript);

    // Produce a Shplonk claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    auto shplonk_claim = Shplonk::reduce_verification(pcs_verification_key, gemini_claim, transcript);

    // Verify the Shplonk claim with KZG or IPA
    return PCS::verify(pcs_verification_key, shplonk_claim, transcript);
}

template class StandardVerifier_<honk::flavor::Standard>;
template class StandardVerifier_<honk::flavor::StandardGrumpkin>;

} // namespace proof_system::honk
