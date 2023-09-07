#include "ultra_prover.hpp"
#include "barretenberg/honk/pcs/claim.hpp"
#include "barretenberg/honk/proof_system/grand_product_library.hpp"
#include "barretenberg/honk/proof_system/prover_library.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript_wrappers.hpp"

namespace proof_system::honk {

/**
 * Create UltraProver_ from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <UltraFlavor Flavor>
UltraProver_<Flavor>::UltraProver_(std::shared_ptr<typename Flavor::ProvingKey> input_key,
                                   std::shared_ptr<CommitmentKey> commitment_key)
    : key(input_key)
    , queue(commitment_key, transcript)
    , pcs_commitment_key(commitment_key)
{
    prover_polynomials.q_c = key->q_c;
    prover_polynomials.q_l = key->q_l;
    prover_polynomials.q_r = key->q_r;
    prover_polynomials.q_o = key->q_o;
    prover_polynomials.q_4 = key->q_4;
    prover_polynomials.q_m = key->q_m;
    prover_polynomials.q_arith = key->q_arith;
    prover_polynomials.q_sort = key->q_sort;
    prover_polynomials.q_elliptic = key->q_elliptic;
    prover_polynomials.q_aux = key->q_aux;
    prover_polynomials.q_lookup = key->q_lookup;
    prover_polynomials.sigma_1 = key->sigma_1;
    prover_polynomials.sigma_2 = key->sigma_2;
    prover_polynomials.sigma_3 = key->sigma_3;
    prover_polynomials.sigma_4 = key->sigma_4;
    prover_polynomials.id_1 = key->id_1;
    prover_polynomials.id_2 = key->id_2;
    prover_polynomials.id_3 = key->id_3;
    prover_polynomials.id_4 = key->id_4;
    prover_polynomials.table_1 = key->table_1;
    prover_polynomials.table_2 = key->table_2;
    prover_polynomials.table_3 = key->table_3;
    prover_polynomials.table_4 = key->table_4;
    prover_polynomials.table_1_shift = key->table_1.shifted();
    prover_polynomials.table_2_shift = key->table_2.shifted();
    prover_polynomials.table_3_shift = key->table_3.shifted();
    prover_polynomials.table_4_shift = key->table_4.shifted();
    prover_polynomials.lagrange_first = key->lagrange_first;
    prover_polynomials.lagrange_last = key->lagrange_last;
    prover_polynomials.w_l = key->w_l;
    prover_polynomials.w_r = key->w_r;
    prover_polynomials.w_o = key->w_o;
    prover_polynomials.w_l_shift = key->w_l.shifted();
    prover_polynomials.w_r_shift = key->w_r.shifted();
    prover_polynomials.w_o_shift = key->w_o.shifted();

    if constexpr (IsGoblinFlavor<Flavor>) {
        prover_polynomials.ecc_op_wire_1 = key->ecc_op_wire_1;
        prover_polynomials.ecc_op_wire_2 = key->ecc_op_wire_2;
        prover_polynomials.ecc_op_wire_3 = key->ecc_op_wire_3;
        prover_polynomials.ecc_op_wire_4 = key->ecc_op_wire_4;
        prover_polynomials.lagrange_ecc_op = key->lagrange_ecc_op;
    }

    // Add public inputs to transcript from the second wire polynomial; This requires determination of the offset at
    // which the PI have been written into the wires relative to the 0th index.
    std::span<FF> public_wires_source = prover_polynomials.w_r;
    pub_inputs_offset = Flavor::has_zero_row ? 1 : 0;
    if constexpr (IsGoblinFlavor<Flavor>) {
        pub_inputs_offset += key->num_ecc_op_gates;
    }

    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        size_t idx = i + pub_inputs_offset;
        public_inputs.emplace_back(public_wires_source[idx]);
    }
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(key->num_public_inputs);

    transcript.send_to_verifier("circuit_size", circuit_size);
    transcript.send_to_verifier("public_input_size", num_public_inputs);
    transcript.send_to_verifier("pub_inputs_offset", static_cast<uint32_t>(pub_inputs_offset));

    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        auto public_input_i = public_inputs[i];
        transcript.send_to_verifier("public_input_" + std::to_string(i), public_input_i);
    }
}

/**
 * @brief Compute commitments to the first three wires
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_wire_commitments_round()
{
    // Commit to the first three wire polynomials; the fourth is committed to after the addition of memory records.
    auto wire_polys = key->get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        queue.add_commitment(wire_polys[idx], labels[idx]);
    }

    // If Goblin, commit to the ECC op wire polynomials
    if constexpr (IsGoblinFlavor<Flavor>) {
        auto op_wire_polys = key->get_ecc_op_wires();
        auto labels = commitment_labels.get_ecc_op_wires();
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            queue.add_commitment(op_wire_polys[idx], labels[idx]);
        }
    }
}

/**
 * @brief Compute sorted witness-table accumulator
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_sorted_list_accumulator_round()
{
    // Compute and add eta to relation parameters
    auto eta = transcript.get_challenge("eta");
    relation_parameters.eta = eta;

    // Compute sorted witness-table accumulator and its commitment
    key->sorted_accum = prover_library::compute_sorted_list_accumulator<Flavor>(key, eta);
    queue.add_commitment(key->sorted_accum, commitment_labels.sorted_accum);

    // Finalize fourth wire polynomial by adding lookup memory records, then commit
    prover_library::add_plookup_memory_records_to_wire_4<Flavor>(key, eta);
    queue.add_commitment(key->w_4, commitment_labels.w_4);

    prover_polynomials.sorted_accum_shift = key->sorted_accum.shifted();
    prover_polynomials.sorted_accum = key->sorted_accum;
    prover_polynomials.w_4 = key->w_4;
    prover_polynomials.w_4_shift = key->w_4.shifted();
}

/**
 * @brief Compute permutation and lookup grand product polynomials and commitments
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_grand_product_computation_round()
{
    // Compute and store parameters required by relations in Sumcheck
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    auto public_input_delta =
        compute_public_input_delta<Flavor>(public_inputs, beta, gamma, key->circuit_size, pub_inputs_offset);
    auto lookup_grand_product_delta = compute_lookup_grand_product_delta(beta, gamma, key->circuit_size);

    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = public_input_delta;
    relation_parameters.lookup_grand_product_delta = lookup_grand_product_delta;

    // Compute permutation + lookup grand product and their commitments
    grand_product_library::compute_grand_products<Flavor>(key, prover_polynomials, relation_parameters);

    queue.add_commitment(key->z_perm, commitment_labels.z_perm);
    queue.add_commitment(key->z_lookup, commitment_labels.z_lookup);
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_relation_check_rounds()
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
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_univariatization_round()
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
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_pcs_evaluation_round()
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
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_shplonk_batched_quotient_round()
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
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_shplonk_partial_evaluation_round()
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
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_final_pcs_round()
{
    PCS::compute_opening_proof(pcs_commitment_key, shplonk_output.opening_pair, shplonk_output.witness, transcript);
    // queue.add_commitment(quotient_W, "KZG:W");
}

template <UltraFlavor Flavor> plonk::proof& UltraProver_<Flavor>::export_proof()
{
    proof.proof_data = transcript.proof_data;
    return proof;
}

template <UltraFlavor Flavor> plonk::proof& UltraProver_<Flavor>::construct_proof()
{
    // Add circuit size public input size and public inputs to transcript.
    execute_preamble_round();

    // Compute first three wire commitments
    execute_wire_commitments_round();
    queue.process_queue();

    // Compute sorted list accumulator and commitment
    execute_sorted_list_accumulator_round();
    queue.process_queue();

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
    // Compute PCS opening proof (either KZG quotient commitment or IPA opening proof)
    execute_final_pcs_round();

    return export_proof();
}

template class UltraProver_<honk::flavor::Ultra>;
template class UltraProver_<honk::flavor::UltraGrumpkin>;
template class UltraProver_<honk::flavor::GoblinUltra>;

} // namespace proof_system::honk
