#include "eccvm_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/common/ref_array.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

/**
 * Create ECCVMProver_ from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <IsECCVMFlavor Flavor>
ECCVMProver_<Flavor>::ECCVMProver_(const std::shared_ptr<typename Flavor::ProvingKey>& input_key,
                                   const std::shared_ptr<PCSCommitmentKey>& commitment_key,
                                   const std::shared_ptr<Transcript>& transcript)
    : transcript(transcript)
    , key(input_key)
    , commitment_key(commitment_key)
{
    // this will be initialized properly later
    key->z_perm = Polynomial(key->circuit_size);
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), key->get_all())) {
        ASSERT(flavor_get_label(prover_polynomials, prover_poly) == flavor_get_label(*key, key_poly));
        prover_poly = key_poly.share();
    }
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_shifted(), key->get_to_be_shifted())) {
        ASSERT(flavor_get_label(prover_polynomials, prover_poly) == (flavor_get_label(*key, key_poly) + "_shift"));
        prover_poly = key_poly.shifted();
    }
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);

    transcript->send_to_verifier("circuit_size", circuit_size);
}

/**
 * @brief Compute commitments to the first three wires
 *
 */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_wire_commitments_round()
{
    auto wire_polys = key->get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < wire_polys.size(); ++idx) {
        transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }
}

/**
 * @brief Compute sorted witness-table accumulator
 *
 */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_log_derivative_commitments_round()
{
    // Compute and add beta to relation parameters
    auto [beta, gamma] = transcript->template get_challenges<FF>("beta", "gamma");

    // TODO(#583)(@zac-williamson): fix Transcript to be able to generate more than 2 challenges per round! oof.
    auto beta_sqr = beta * beta;
    relation_parameters.gamma = gamma;
    relation_parameters.beta = beta;
    relation_parameters.beta_sqr = beta_sqr;
    relation_parameters.beta_cube = beta_sqr * beta;
    relation_parameters.eccvm_set_permutation_delta =
        gamma * (gamma + beta_sqr) * (gamma + beta_sqr + beta_sqr) * (gamma + beta_sqr + beta_sqr + beta_sqr);
    relation_parameters.eccvm_set_permutation_delta = relation_parameters.eccvm_set_permutation_delta.invert();
    // Compute inverse polynomial for our logarithmic-derivative lookup method
    compute_logderivative_inverse<Flavor, typename Flavor::LookupRelation>(
        prover_polynomials, relation_parameters, key->circuit_size);
    transcript->send_to_verifier(commitment_labels.lookup_inverses, commitment_key->commit(key->lookup_inverses));
    prover_polynomials.lookup_inverses = key->lookup_inverses.share();
}

/**
 * @brief Compute permutation and lookup grand product polynomials and commitments
 *
 */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_grand_product_computation_round()
{
    // Compute permutation grand product and their commitments
    compute_permutation_grand_products<Flavor>(key, prover_polynomials, relation_parameters);

    transcript->send_to_verifier(commitment_labels.z_perm, commitment_key->commit(key->z_perm));
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_relation_check_rounds()
{
    using Sumcheck = SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);
    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters, alpha, gate_challenges);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_zeromorph_rounds()
{
    ZeroMorph::prove(prover_polynomials.get_unshifted(),
                     prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript);
}

/**
 * @brief Batch open the transcript polynomials as univariates for Translator consistency check
 * TODO(#768): Find a better way to do this. See issue for details.
 *
 * @tparam Flavor
 */
template <IsECCVMFlavor Flavor> void ECCVMProver_<Flavor>::execute_transcript_consistency_univariate_opening_round()
{
    // Since IPA cannot currently handle polynomials for which the latter half of the coefficients are 0, we hackily
    // batch the constant polynomial 1 in with the 5 transcript polynomials. See issue #768 for more details.
    Polynomial hack(key->circuit_size);
    for (size_t idx = 0; idx < key->circuit_size; idx++) {
        hack[idx] = 1;
    }
    transcript->send_to_verifier("Translation:hack_commitment", commitment_key->commit(hack));

    // Get the challenge at which we evaluate the polynomials as univariates
    evaluation_challenge_x = transcript->template get_challenge<FF>("Translation:evaluation_challenge_x");

    translation_evaluations.op = key->transcript_op.evaluate(evaluation_challenge_x);
    translation_evaluations.Px = key->transcript_Px.evaluate(evaluation_challenge_x);
    translation_evaluations.Py = key->transcript_Py.evaluate(evaluation_challenge_x);
    translation_evaluations.z1 = key->transcript_z1.evaluate(evaluation_challenge_x);
    translation_evaluations.z2 = key->transcript_z2.evaluate(evaluation_challenge_x);

    // Add the univariate evaluations to the transcript
    transcript->send_to_verifier("Translation:op", translation_evaluations.op);
    transcript->send_to_verifier("Translation:Px", translation_evaluations.Px);
    transcript->send_to_verifier("Translation:Py", translation_evaluations.Py);
    transcript->send_to_verifier("Translation:z1", translation_evaluations.z1);
    transcript->send_to_verifier("Translation:z2", translation_evaluations.z2);
    transcript->send_to_verifier("Translation:hack_evaluation", hack.evaluate(evaluation_challenge_x));

    // Get another challenge for batching the univariate claims
    FF ipa_batching_challenge = transcript->template get_challenge<FF>("Translation:ipa_batching_challenge");

    // Collect the polynomials and evaluations to be batched
    RefArray univariate_polynomials{ key->transcript_op, key->transcript_Px, key->transcript_Py,
                                     key->transcript_z1, key->transcript_z2, hack };
    std::array<FF, univariate_polynomials.size()> univariate_evaluations;
    for (auto [eval, polynomial] : zip_view(univariate_evaluations, univariate_polynomials)) {
        eval = polynomial.evaluate(evaluation_challenge_x);
    }

    // Construct the batched polynomial and batched evaluation
    Polynomial batched_univariate{ key->circuit_size };
    FF batched_evaluation{ 0 };
    auto batching_scalar = FF(1);
    for (auto [polynomial, eval] : zip_view(univariate_polynomials, univariate_evaluations)) {
        batched_univariate.add_scaled(polynomial, batching_scalar);
        batched_evaluation += eval * batching_scalar;
        batching_scalar *= ipa_batching_challenge;
    }

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/922): We are doing another round of IPA here with
    // exactly the same labels and no domain separation so if/when labels are going to matter we are clashing.
    PCS::compute_opening_proof(
        commitment_key, { evaluation_challenge_x, batched_evaluation }, batched_univariate, transcript);

    // Get another challenge for batching the univariate claims
    translation_batching_challenge_v = transcript->template get_challenge<FF>("Translation:batching_challenge");
}

template <IsECCVMFlavor Flavor> HonkProof& ECCVMProver_<Flavor>::export_proof()
{
    proof = transcript->export_proof();
    return proof;
}

template <IsECCVMFlavor Flavor> HonkProof& ECCVMProver_<Flavor>::construct_proof()
{
    BB_OP_COUNT_TIME_NAME("ECCVMProver::construct_proof");

    execute_preamble_round();

    execute_wire_commitments_round();

    execute_log_derivative_commitments_round();

    execute_grand_product_computation_round();

    execute_relation_check_rounds();

    execute_zeromorph_rounds();

    execute_transcript_consistency_univariate_opening_round();

    return export_proof();
}

template class ECCVMProver_<ECCVMFlavor>;

} // namespace bb
