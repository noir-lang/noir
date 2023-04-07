#include "prover.hpp"
#include <algorithm>
#include <cstddef>
#include "barretenberg/honk/proof_system/prover_library.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include <array>
#include "barretenberg/honk/sumcheck/polynomials/univariate.hpp" // will go away
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include <memory>
#include <span>
#include <utility>
#include <vector>
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/honk/sumcheck/relations/arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_computation_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_initialization_relation.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "barretenberg/transcript/transcript_wrappers.hpp"
#include <string>
#include "barretenberg/honk/pcs/claim.hpp"

namespace proof_system::honk {

using Fr = barretenberg::fr;
using Commitment = barretenberg::g1::affine_element;
using Polynomial = barretenberg::Polynomial<Fr>;
using POLYNOMIAL = proof_system::honk::StandardArithmetization::POLYNOMIAL;

/**
 * Create Prover from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <typename settings>
Prover<settings>::Prover(std::vector<barretenberg::polynomial>&& wire_polys,
                         const std::shared_ptr<plonk::proving_key> input_key)
    : wire_polynomials(wire_polys)
    , key(input_key)
    , queue(key, transcript)
{
    // Note(luke): This could be done programmatically with some hacks but this isnt too bad and its nice to see the
    // polys laid out explicitly.
    prover_polynomials[POLYNOMIAL::Q_C] = key->polynomial_store.get("q_c_lagrange");
    prover_polynomials[POLYNOMIAL::Q_L] = key->polynomial_store.get("q_1_lagrange");
    prover_polynomials[POLYNOMIAL::Q_R] = key->polynomial_store.get("q_2_lagrange");
    prover_polynomials[POLYNOMIAL::Q_O] = key->polynomial_store.get("q_3_lagrange");
    prover_polynomials[POLYNOMIAL::Q_M] = key->polynomial_store.get("q_m_lagrange");
    prover_polynomials[POLYNOMIAL::SIGMA_1] = key->polynomial_store.get("sigma_1_lagrange");
    prover_polynomials[POLYNOMIAL::SIGMA_2] = key->polynomial_store.get("sigma_2_lagrange");
    prover_polynomials[POLYNOMIAL::SIGMA_3] = key->polynomial_store.get("sigma_3_lagrange");
    prover_polynomials[POLYNOMIAL::ID_1] = key->polynomial_store.get("id_1_lagrange");
    prover_polynomials[POLYNOMIAL::ID_2] = key->polynomial_store.get("id_2_lagrange");
    prover_polynomials[POLYNOMIAL::ID_3] = key->polynomial_store.get("id_3_lagrange");
    prover_polynomials[POLYNOMIAL::LAGRANGE_FIRST] = key->polynomial_store.get("L_first_lagrange");
    prover_polynomials[POLYNOMIAL::LAGRANGE_LAST] = key->polynomial_store.get("L_last_lagrange");
    prover_polynomials[POLYNOMIAL::W_L] = wire_polynomials[0];
    prover_polynomials[POLYNOMIAL::W_R] = wire_polynomials[1];
    prover_polynomials[POLYNOMIAL::W_O] = wire_polynomials[2];

    // Add public inputs to transcript from the second wire polynomial
    std::span<Fr> public_wires_source = prover_polynomials[POLYNOMIAL::W_R];

    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_inputs.emplace_back(public_wires_source[i]);
    }
}

/**
 * - Add commitment to wires 1,2,3 to work queue
 * - Add PI to transcript (I guess PI will stay in w_2 for now?)
 *
 * */
template <typename settings> void Prover<settings>::compute_wire_commitments()
{
    for (size_t i = 0; i < settings::Arithmetization::num_wires; ++i) {
        queue.add_commitment(wire_polynomials[i], "W_" + std::to_string(i + 1));
    }
}

/**
 * - Add circuit size, public input size, and public inputs to transcript
 *
 * */
template <typename settings> void Prover<settings>::execute_preamble_round()
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
template <typename settings> void Prover<settings>::execute_wire_commitments_round()
{
    compute_wire_commitments();
}

/**
 * For Standard Honk, this is a non-op (just like for Standard/Turbo Plonk).
 * */
template <typename settings> void Prover<settings>::execute_tables_round()
{
    // No operations are needed here for Standard Honk
}

/**
 * - Do Fiat-Shamir to get "beta" challenge (Note: gamma = beta^2)
 * - Compute grand product polynomial (permutation only) and commitment
 * */
template <typename settings> void Prover<settings>::execute_grand_product_computation_round()
{
    // Compute and store parameters required by relations in Sumcheck
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    auto public_input_delta = compute_public_input_delta<Fr>(public_inputs, beta, gamma, key->circuit_size);

    relation_parameters = sumcheck::RelationParameters<Fr>{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    z_permutation =
        prover_library::compute_permutation_grand_product<settings::program_width>(key, wire_polynomials, beta, gamma);

    queue.add_commitment(z_permutation, "Z_PERM");

    prover_polynomials[POLYNOMIAL::Z_PERM] = z_permutation;
    prover_polynomials[POLYNOMIAL::Z_PERM_SHIFT] = z_permutation.shifted();
}

/**
 * - Do Fiat-Shamir to get "alpha" challenge
 * - Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all
 *   evaluations at u being calculated.
 * */
template <typename settings> void Prover<settings>::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::Sumcheck<Fr,
                                        ProverTranscript<Fr>,
                                        sumcheck::ArithmeticRelation,
                                        sumcheck::GrandProductComputationRelation,
                                        sumcheck::GrandProductInitializationRelation>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);

    sumcheck_output = sumcheck.execute_prover(prover_polynomials, relation_parameters);
}

/**
 * - Get rho challenge
 * - Compute d+1 Fold polynomials and their evaluations.
 *
 * */
template <typename settings> void Prover<settings>::execute_univariatization_round()
{
    const size_t NUM_POLYNOMIALS = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;
    const size_t NUM_UNSHIFTED_POLYS = proof_system::honk::StandardArithmetization::NUM_UNSHIFTED_POLYNOMIALS;

    // Generate batching challenge ρ and powers 1,ρ,…,ρᵐ⁻¹
    Fr rho = transcript.get_challenge("rho");
    std::vector<Fr> rhos = Gemini::powers_of_rho(rho, NUM_POLYNOMIALS);

    // Batch the unshifted polynomials and the to-be-shifted polynomials using ρ
    Polynomial batched_poly_unshifted(key->circuit_size); // batched unshifted polynomials
    for (size_t i = 0; i < NUM_UNSHIFTED_POLYS; ++i) {
        batched_poly_unshifted.add_scaled(prover_polynomials[i], rhos[i]);
    }
    Polynomial batched_poly_to_be_shifted(key->circuit_size); // batched to-be-shifted polynomials
    batched_poly_to_be_shifted.add_scaled(prover_polynomials[POLYNOMIAL::Z_PERM], rhos[NUM_UNSHIFTED_POLYS]);

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
template <typename settings> void Prover<settings>::execute_pcs_evaluation_round()
{
    const Fr r_challenge = transcript.get_challenge("Gemini:r");

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
template <typename settings> void Prover<settings>::execute_shplonk_batched_quotient_round()
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
template <typename settings> void Prover<settings>::execute_shplonk_partial_evaluation_round()
{
    const Fr z_challenge = transcript.get_challenge("Shplonk:z");
    shplonk_output = Shplonk::compute_partially_evaluated_batched_quotient(
        gemini_output.opening_pairs, gemini_output.witnesses, std::move(batched_quotient_Q), nu_challenge, z_challenge);
}

/**
 * - Compute KZG quotient commitment [W]_1.
 *
 * */
template <typename settings> void Prover<settings>::execute_kzg_round()
{
    quotient_W = KZG::compute_opening_proof_polynomial(shplonk_output.opening_pair, shplonk_output.witness);
    queue.add_commitment(quotient_W, "KZG:W");
}

template <typename settings> plonk::proof& Prover<settings>::export_proof()
{
    proof.proof_data = transcript.proof_data;
    return proof;
}

template <typename settings> plonk::proof& Prover<settings>::construct_proof()
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
    // Compute KZG quotient commitment
    execute_kzg_round();
    queue.process_queue();

    return export_proof();
}

template class Prover<plonk::standard_settings>;

} // namespace proof_system::honk
