#include "ultra_prover.hpp"
#include "barretenberg/honk/pcs/claim.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript_wrappers.hpp"

namespace proof_system::honk {

/**
 * Create UltraProver_ from an instance.
 *
 * @param instance Instance whose proof we want to generate.
 *
 * @tparam a type of UltraFlavor
 * */
template <UltraFlavor Flavor>
UltraProver_<Flavor>::UltraProver_(std::shared_ptr<Instance> inst)
    : queue(inst->commitment_key, transcript)
    , instance(std::move(inst))
    , pcs_commitment_key(instance->commitment_key)
{
    instance->initialise_prover_polynomials();
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_preamble_round()
{
    auto proving_key = instance->proving_key;
    const auto circuit_size = static_cast<uint32_t>(proving_key->circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(proving_key->num_public_inputs);

    transcript.send_to_verifier("circuit_size", circuit_size);
    transcript.send_to_verifier("public_input_size", num_public_inputs);
    transcript.send_to_verifier("pub_inputs_offset", static_cast<uint32_t>(instance->pub_inputs_offset));

    for (size_t i = 0; i < proving_key->num_public_inputs; ++i) {
        auto public_input_i = instance->public_inputs[i];
        transcript.send_to_verifier("public_input_" + std::to_string(i), public_input_i);
    }
}

/**
 * @brief Compute commitments to the first three wire polynomials (and ECC op wires if using Goblin).
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_wire_commitments_round()
{
    // Commit to the first three wire polynomials
    // We only commit to the fourth wire polynomial after adding memory records
    auto wire_polys = instance->proving_key->get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        queue.add_commitment(wire_polys[idx], labels[idx]);
    }

    if constexpr (IsGoblinFlavor<Flavor>) {
        auto op_wire_polys = instance->proving_key->get_ecc_op_wires();
        auto labels = commitment_labels.get_ecc_op_wires();
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            queue.add_commitment(op_wire_polys[idx], labels[idx]);
        }
    }
}

/**
 * @brief Compute sorted witness-table accumulator and commit to the resulting polynomials.
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_sorted_list_accumulator_round()
{
    auto eta = transcript.get_challenge("eta");

    instance->compute_sorted_accumulator_polynomials(eta);

    // Commit to the sorted withness-table accumulator and the finalised (i.e. with memory records) fourth wire
    // polynomial
    queue.add_commitment(instance->proving_key->sorted_accum, commitment_labels.sorted_accum);
    queue.add_commitment(instance->proving_key->w_4, commitment_labels.w_4);
}

/**
 * @brief Compute permutation and lookup grand product polynomials and their commitments
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_grand_product_computation_round()
{
    // Compute and store parameters required by relations in Sumcheck
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    instance->compute_grand_product_polynomials(beta, gamma);

    queue.add_commitment(instance->proving_key->z_perm, commitment_labels.z_perm);
    queue.add_commitment(instance->proving_key->z_lookup, commitment_labels.z_lookup);
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(instance->proving_key->circuit_size, transcript);

    sumcheck_output = sumcheck.prove(instance->prover_polynomials, instance->relation_parameters);
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
    Polynomial batched_poly_unshifted(instance->proving_key->circuit_size); // batched unshifted polynomials
    size_t poly_idx = 0;                                                    // TODO(#391) zip
    for (auto& unshifted_poly : instance->prover_polynomials.get_unshifted()) {
        batched_poly_unshifted.add_scaled(unshifted_poly, rhos[poly_idx]);
        ++poly_idx;
    }

    Polynomial batched_poly_to_be_shifted(instance->proving_key->circuit_size); // batched to-be-shifted polynomials
    for (auto& to_be_shifted_poly : instance->prover_polynomials.get_to_be_shifted()) {
        batched_poly_to_be_shifted.add_scaled(to_be_shifted_poly, rhos[poly_idx]);
        ++poly_idx;
    };

    // Compute d-1 polynomials Fold^(i), i = 1, ..., d-1.
    gemini_polynomials = Gemini::compute_gemini_polynomials(
        sumcheck_output.challenge, std::move(batched_poly_unshifted), std::move(batched_poly_to_be_shifted));

    // Compute and add to trasnscript the commitments [Fold^(i)], i = 1, ..., d-1
    for (size_t l = 0; l < instance->proving_key->log_circuit_size - 1; ++l) {
        queue.add_commitment(gemini_polynomials[l + 2], "Gemini:FOLD_" + std::to_string(l + 1));
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
    univariate_openings = Gemini::compute_fold_polynomial_evaluations(
        sumcheck_output.challenge, std::move(gemini_polynomials), r_challenge);

    for (size_t l = 0; l < instance->proving_key->log_circuit_size; ++l) {
        std::string label = "Gemini:a_" + std::to_string(l);
        const auto& evaluation = univariate_openings.opening_pairs[l + 1].evaluation;
        transcript.send_to_verifier(label, evaluation);
    }
}

/**
 * @brief Prove proper construction of the aggregate Goblin ECC op queue polynomials T_i^(j), j = 1,2,3,4.
 * @details Let T_i^(j) be the jth column of the aggregate op queue after incorporating the contribution from the
 * present circuit. T_{i-1}^(j) corresponds to the aggregate op queue at the previous stage and $t_i^(j)$ represents
 * the contribution from the present circuit only. For each j, we have the relationship T_i = T_{i-1} + right_shift(t_i,
 * M_{i-1}), where the shift magnitude M_{i-1} is the length of T_{i-1}. This stage of the protocol demonstrates that
 * the aggregate op queue has been constructed correctly.
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_op_queue_transcript_aggregation_round()
{
    if constexpr (IsGoblinFlavor<Flavor>) {
        // Extract size M_{i-1} of T_{i-1} from op_queue
        size_t prev_op_queue_size = instance->proving_key->op_queue->get_previous_size(); // M_{i-1}
        // TODO(#723): Cannot currently support an empty T_{i-1}. Need to be able to properly handle zero commitment.
        ASSERT(prev_op_queue_size > 0);

        auto circuit_size = instance->proving_key->circuit_size;

        // TODO(#723): The below assert ensures that M_{i-1} + m_i < n, i.e. the right shifted result can be expressed
        // as a size n polynomial. If this is not the case then we should still be able to proceed without increasing
        // the circuit size but need to handle with care.
        ASSERT(prev_op_queue_size + instance->proving_key->num_ecc_op_gates < circuit_size); // M_{i-1} + m_i < n

        // Construct right-shift of op wires t_i^{shift} so that T_i(X) = T_{i-1}(X) + t_i^{shift}(X).
        // Note: The op_wire polynomials (like all others) have constant coefficient equal to zero. Thus to obtain
        // t_i^{shift} we must left-shift by 1 then right-shift by M_{i-1}, or equivalently, right-shift by
        // M_{i-1} - 1.
        std::array<Polynomial, Flavor::NUM_WIRES> right_shifted_op_wires;
        auto op_wires = instance->proving_key->get_ecc_op_wires();
        for (size_t i = 0; i < op_wires.size(); ++i) {
            // Right shift by M_{i-1} - 1.
            right_shifted_op_wires[i].set_to_right_shifted(op_wires[i], prev_op_queue_size - 1);
        }

        // Compute/get commitments [t_i^{shift}], [T_{i-1}], and [T_i] and add to transcript
        std::array<Commitment, Flavor::NUM_WIRES> prev_aggregate_op_queue_commitments;
        std::array<Commitment, Flavor::NUM_WIRES> shifted_op_wire_commitments;
        std::array<Commitment, Flavor::NUM_WIRES> aggregate_op_queue_commitments;
        for (size_t idx = 0; idx < right_shifted_op_wires.size(); ++idx) {
            // Get previous transcript commitment [T_{i-1}] from op queue
            prev_aggregate_op_queue_commitments[idx] = instance->proving_key->op_queue->ultra_ops_commitments[idx];
            // Compute commitment [t_i^{shift}] directly
            shifted_op_wire_commitments[idx] = pcs_commitment_key->commit(right_shifted_op_wires[idx]);
            // Compute updated aggregate transcript commitmen as [T_i] = [T_{i-1}] + [t_i^{shift}]
            aggregate_op_queue_commitments[idx] =
                prev_aggregate_op_queue_commitments[idx] + shifted_op_wire_commitments[idx];

            std::string suffix = std::to_string(idx + 1);
            transcript.send_to_verifier("PREV_AGG_OP_QUEUE_" + suffix, prev_aggregate_op_queue_commitments[idx]);
            transcript.send_to_verifier("SHIFTED_OP_WIRE_" + suffix, shifted_op_wire_commitments[idx]);
            transcript.send_to_verifier("AGG_OP_QUEUE_" + suffix, aggregate_op_queue_commitments[idx]);
        }

        // Store the commitments [T_{i}] (to be used later in subsequent iterations as [T_{i-1}]).
        instance->proving_key->op_queue->set_commitment_data(aggregate_op_queue_commitments);

        // Compute evaluations T_i(\kappa), T_{i-1}(\kappa), t_i^{shift}(\kappa), add to transcript. For each polynomial
        // we add a univariate opening claim {(\kappa, p(\kappa)), p(X)} to the set of claims to be combined in the
        // batch univariate polynomial Q in Shplonk. (The other univariate claims come from the output of Gemini).
        // TODO(#729): It should be possible to reuse the opening challenge from Gemini rather than generate a new one.
        auto kappa = transcript.get_challenge("kappa");
        auto prev_aggregate_ecc_op_transcript = instance->proving_key->op_queue->get_previous_aggregate_transcript();
        auto aggregate_ecc_op_transcript = instance->proving_key->op_queue->get_aggregate_transcript();
        std::array<FF, Flavor::NUM_WIRES> prev_agg_op_queue_evals;
        std::array<FF, Flavor::NUM_WIRES> right_shifted_op_wire_evals;
        std::array<FF, Flavor::NUM_WIRES> agg_op_queue_evals;
        std::array<Polynomial, Flavor::NUM_WIRES> prev_agg_op_queue_polynomials;
        std::array<Polynomial, Flavor::NUM_WIRES> agg_op_queue_polynomials;
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            std::string suffix = std::to_string(idx + 1);

            // Compute evaluation T_{i-1}(\kappa)
            prev_agg_op_queue_polynomials[idx] = Polynomial(prev_aggregate_ecc_op_transcript[idx]);
            prev_agg_op_queue_evals[idx] = prev_agg_op_queue_polynomials[idx].evaluate(kappa);
            transcript.send_to_verifier("prev_agg_op_queue_eval_" + suffix, prev_agg_op_queue_evals[idx]);

            // Compute evaluation t_i^{shift}(\kappa)
            right_shifted_op_wire_evals[idx] = right_shifted_op_wires[idx].evaluate(kappa);
            transcript.send_to_verifier("op_wire_eval_" + suffix, right_shifted_op_wire_evals[idx]);

            // Compute evaluation T_i(\kappa)
            agg_op_queue_polynomials[idx] = Polynomial(aggregate_ecc_op_transcript[idx]);
            agg_op_queue_evals[idx] = agg_op_queue_polynomials[idx].evaluate(kappa);
            transcript.send_to_verifier("agg_op_queue_eval_" + suffix, agg_op_queue_evals[idx]);
        }

        // Add univariate opening claims for each polynomial.
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            univariate_openings.opening_pairs.emplace_back(OpenPair{ kappa, prev_agg_op_queue_evals[idx] });
            univariate_openings.witnesses.emplace_back(std::move(prev_agg_op_queue_polynomials[idx]));
        }
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            univariate_openings.opening_pairs.emplace_back(OpenPair{ kappa, right_shifted_op_wire_evals[idx] });
            univariate_openings.witnesses.emplace_back(std::move(right_shifted_op_wires[idx]));
        }
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            univariate_openings.opening_pairs.emplace_back(OpenPair{ kappa, agg_op_queue_evals[idx] });
            univariate_openings.witnesses.emplace_back(std::move(agg_op_queue_polynomials[idx]));
        }
    }
}

/**
 * - Do Fiat-Shamir to get "nu" challenge.
 * - Compute commitment [Q]_1
 * */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_shplonk_batched_quotient_round()
{
    nu_challenge = transcript.get_challenge("Shplonk:nu");

    batched_quotient_Q = Shplonk::compute_batched_quotient(
        univariate_openings.opening_pairs, univariate_openings.witnesses, nu_challenge);

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

    shplonk_output = Shplonk::compute_partially_evaluated_batched_quotient(univariate_openings.opening_pairs,
                                                                           univariate_openings.witnesses,
                                                                           std::move(batched_quotient_Q),
                                                                           nu_challenge,
                                                                           z_challenge);
}
/**
 * - Compute final PCS opening proof:
 * - For KZG, this is the quotient commitmecnt [W]_1
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

    // ECC op queue transcript aggregation
    execute_op_queue_transcript_aggregation_round();

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
