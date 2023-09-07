#include "prover.hpp"
#include "barretenberg/honk/proof_system/grand_product_library.hpp"
#include "barretenberg/honk/proof_system/prover_library.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"

namespace proof_system::honk {

/**
 * Create Prover from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <StandardFlavor Flavor>
StandardProver_<Flavor>::StandardProver_(const std::shared_ptr<ProvingKey> input_key,
                                         const std::shared_ptr<CommitmentKey> commitment_key)
    : key(input_key)
    , queue(commitment_key, transcript)
    , pcs_commitment_key(commitment_key)
{
    prover_polynomials.q_c = key->q_c;
    prover_polynomials.q_l = key->q_l;
    prover_polynomials.q_r = key->q_r;
    prover_polynomials.q_o = key->q_o;
    prover_polynomials.q_m = key->q_m;
    prover_polynomials.sigma_1 = key->sigma_1;
    prover_polynomials.sigma_2 = key->sigma_2;
    prover_polynomials.sigma_3 = key->sigma_3;
    prover_polynomials.id_1 = key->id_1;
    prover_polynomials.id_2 = key->id_2;
    prover_polynomials.id_3 = key->id_3;
    prover_polynomials.lagrange_first = key->lagrange_first;
    prover_polynomials.lagrange_last = key->lagrange_last;
    prover_polynomials.w_l = key->w_l;
    prover_polynomials.w_r = key->w_r;
    prover_polynomials.w_o = key->w_o;

    // Add public inputs to transcript from the second wire polynomial
    std::span<FF> public_wires_source = prover_polynomials.w_r;

    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_inputs.emplace_back(public_wires_source[i]);
    }
}

/**
 * - Add commitment to wires 1,2,3 to work queue
 * - Add PI to transcript (I guess PI will stay in w_2 for now?)
 *
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::compute_wire_commitments()
{
    size_t wire_idx = 0; // TODO(#391) zip
    auto wire_polys = key->get_wires();
    for (auto& label : commitment_labels.get_wires()) {
        queue.add_commitment(wire_polys[wire_idx], label);
        ++wire_idx;
    }
}

/**
 * - Add circuit size, public input size, and public inputs to transcript
 *
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(key->num_public_inputs);

    transcript.send_to_verifier("circuit_size", circuit_size);
    transcript.send_to_verifier("public_input_size", num_public_inputs);

    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        auto public_input_i = public_inputs[i];
        transcript.send_to_verifier("public_input_" + std::to_string(i), public_input_i);
    }
}

/**
 * - compute wire commitments
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_wire_commitments_round()
{
    compute_wire_commitments();
}

/**
 * For Standard Honk, this is a non-op (just like for Standard/Turbo Plonk).
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_tables_round()
{
    // No operations are needed here for Standard Honk
}

/**
 * - Do Fiat-Shamir to get "beta" challenge (Note: gamma = beta^2)
 * - Compute grand product polynomial (permutation only) and commitment
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_grand_product_computation_round()
{
    // Compute and store parameters required by relations in Sumcheck
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    auto public_input_delta = compute_public_input_delta<Flavor>(public_inputs, beta, gamma, key->circuit_size);

    relation_parameters = proof_system::RelationParameters<FF>{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    grand_product_library::compute_grand_products<Flavor>(key, prover_polynomials, relation_parameters);

    queue.add_commitment(key->z_perm, commitment_labels.z_perm);
}

/**
 * - Do Fiat-Shamir to get "alpha" challenge
 * - Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all
 *   evaluations at u being calculated.
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);

    sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters);
}

/**
 * - Get rho challenge
 * - Compute d+1 Fold polynomials and their evaluations.
 *
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_univariatization_round()
{
    const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

    // Generate batching challenge ρ and powers 1,ρ,…,ρᵐ⁻¹
    FF rho = transcript.get_challenge("rho");
    std::vector<FF> rhos = pcs::gemini::powers_of_rho(rho, NUM_POLYNOMIALS);

    // Batch the unshifted polynomials and the to-be-shifted polynomials using ρ
    Polynomial batched_poly_unshifted(key->circuit_size); // batched unshifted polynomials
    size_t poly_idx = 0;                                  // TODO(#391) zip
    for (auto& unshifted_poly : prover_polynomials.get_unshifted()) {
        batched_poly_unshifted.add_scaled(unshifted_poly, rhos[poly_idx]);
        ++poly_idx;
    }

    Polynomial batched_poly_to_be_shifted(key->circuit_size); // batched to-be-shifted polynomials
    for (auto& to_be_shifted_poly : prover_polynomials.get_to_be_shifted()) {
        batched_poly_to_be_shifted.add_scaled(to_be_shifted_poly, rhos[poly_idx]);
        ++poly_idx;
    };

    // Compute d-1 polynomials Fold^(i), i = 1, ..., d-1.
    fold_polynomials = Gemini::compute_fold_polynomials(
        sumcheck_output.challenge_point, std::move(batched_poly_unshifted), std::move(batched_poly_to_be_shifted));

    // Compute and add to trasnscript the commitments [Fold^(i)], i = 1, ..., d-1
    for (size_t l = 0; l < key->log_circuit_size - 1; ++l) {
        queue.add_commitment(fold_polynomials[l + 2], "Gemini:FOLD_" + std::to_string(l + 1));
    }
}

/**
 * - Do Fiat-Shamir to get "r" challenge
 * - Compute remaining two partially evaluated Fold polynomials Fold_{r}^(0) and Fold_{-r}^(0).
 * - Compute and aggregate opening pairs (challenge, evaluation) for each of d Fold polynomials.
 * - Add d-many Fold evaluations a_i, i = 0, ..., d-1 to the transcript, excluding eval of Fold_{r}^(0)
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_pcs_evaluation_round()
{
    const FF r_challenge = transcript.get_challenge("Gemini:r");
    gemini_output = Gemini::compute_fold_polynomial_evaluations(
        sumcheck_output.challenge_point, std::move(fold_polynomials), r_challenge);

    for (size_t l = 0; l < key->log_circuit_size; ++l) {
        std::string label = "Gemini:a_" + std::to_string(l);
        const auto& evaluation = gemini_output.opening_pairs[l + 1].evaluation;
        transcript.send_to_verifier(label, evaluation);
    }
}

/**
 * - Do Fiat-Shamir to get "nu" challenge.
 * - Compute commitment [Q]_1
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_shplonk_batched_quotient_round()
{
    nu_challenge = transcript.get_challenge("Shplonk:nu");

    batched_quotient_Q =
        Shplonk::compute_batched_quotient(gemini_output.opening_pairs, gemini_output.witnesses, nu_challenge);

    // commit to Q(X) and add [Q] to the transcript
    queue.add_commitment(batched_quotient_Q, "Shplonk:Q");
}

/**
 * - Do Fiat-Shamir to get "z" challenge.
 * - Compute polynomial Q(X) - Q_z(X)
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_shplonk_partial_evaluation_round()
{
    const FF z_challenge = transcript.get_challenge("Shplonk:z");
    shplonk_output = Shplonk::compute_partially_evaluated_batched_quotient(
        gemini_output.opening_pairs, gemini_output.witnesses, std::move(batched_quotient_Q), nu_challenge, z_challenge);
}

/**
 * - Compute final PCS opening proof:
 * - For KZG, this is the quotient commitment [W]_1
 * - For IPA, the vectors L and R
 * */
template <StandardFlavor Flavor> void StandardProver_<Flavor>::execute_final_pcs_round()
{
    PCS::compute_opening_proof(pcs_commitment_key, shplonk_output.opening_pair, shplonk_output.witness, transcript);
}

template <StandardFlavor Flavor> plonk::proof& StandardProver_<Flavor>::export_proof()
{
    proof.proof_data = transcript.proof_data;
    return proof;
}

template <StandardFlavor Flavor> plonk::proof& StandardProver_<Flavor>::construct_proof()
{
    // Add circuit size and public input size to transcript.
    execute_preamble_round();

    // Compute wire commitments; Add PI to transcript
    execute_wire_commitments_round();
    queue.process_queue();

    // Currently a no-op; may execute some "random widgets", commit to W_4, do RAM/ROM stuff
    // if this prover structure is kept when we bring tables to Honk.
    // Suggestion: Maybe we shouldn't mix and match proof creation for different systems and
    // instead instatiate construct_proof differently for each?
    execute_tables_round();

    // Fiat-Shamir: beta & gamma
    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();
    queue.process_queue();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho
    // Compute Fold polynomials and their commitments.
    execute_univariatization_round();
    queue.process_queue();

    // Fiat-Shamir: r
    // Compute Fold evaluations
    execute_pcs_evaluation_round();

    // Fiat-Shamir: nu
    // Compute Shplonk batched quotient commitment Q
    execute_shplonk_batched_quotient_round();
    queue.process_queue();

    // Fiat-Shamir: z
    // Compute partial evaluation Q_z
    execute_shplonk_partial_evaluation_round();

    // Fiat-Shamir: z
    // Compute final PCS opening proof (this is KZG quotient commitment or IPA opening proof)
    execute_final_pcs_round();
    // TODO(#479): queue.process_queue after the work_queue has been (re)added to KZG/IPA

    return export_proof();
}

template class StandardProver_<flavor::Standard>;
template class StandardProver_<flavor::StandardGrumpkin>;

} // namespace proof_system::honk
