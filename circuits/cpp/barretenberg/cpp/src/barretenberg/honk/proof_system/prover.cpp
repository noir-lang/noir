#include "prover.hpp"
#include <algorithm>
#include <cstddef>
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include <array>
#include "barretenberg/honk/sumcheck/polynomials/univariate.hpp" // will go away
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

namespace honk {

using Fr = barretenberg::fr;
using Commitment = barretenberg::g1::affine_element;
using Polynomial = barretenberg::Polynomial<Fr>;
using POLYNOMIAL = honk::StandardArithmetization::POLYNOMIAL;

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
                         std::shared_ptr<bonk::proving_key> input_key)
    : wire_polynomials(wire_polys)
    , key(input_key)
    , commitment_key(std::make_unique<pcs::kzg::CommitmentKey>(
          input_key->circuit_size,
          "../srs_db/ignition")) // TODO(Cody): Need better constructors for prover.
// , queue(proving_key.get(), &transcript)
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
 * - Commit to wires 1,2,3
 * - Add PI to transcript (I guess PI will stay in w_2 for now?)
 *
 * */
template <typename settings> void Prover<settings>::compute_wire_commitments()
{
    for (size_t i = 0; i < settings::program_width; ++i) {
        auto commitment = commitment_key->commit(wire_polynomials[i]);

        transcript.send_to_verifier("W_" + std::to_string(i + 1), commitment);
    }
}

/**
 * @brief Compute the permutation grand product polynomial Z_perm(X)
 * *
 * @detail (This description assumes program_width 3). Z_perm may be defined in terms of its values
 * on X_i = 0,1,...,n-1 as Z_perm[0] = 1 and for i = 1:n-1
 *
 *                  (w_1(j) + β⋅id_1(j) + γ) ⋅ (w_2(j) + β⋅id_2(j) + γ) ⋅ (w_3(j) + β⋅id_3(j) + γ)
 * Z_perm[i] = ∏ --------------------------------------------------------------------------------
 *                  (w_1(j) + β⋅σ_1(j) + γ) ⋅ (w_2(j) + β⋅σ_2(j) + γ) ⋅ (w_3(j) + β⋅σ_3(j) + γ)
 *
 * where ∏ := ∏_{j=0:i-1} and id_i(X) = id(X) + n*(i-1). These evaluations are constructed over the
 * course of four steps. For expositional simplicity, write Z_perm[i] as
 *
 *                A_1(j) ⋅ A_2(j) ⋅ A_3(j)
 * Z_perm[i] = ∏ --------------------------
 *                B_1(j) ⋅ B_2(j) ⋅ B_3(j)
 *
 * Step 1) Compute the 2*program_width length-n polynomials A_i and B_i
 * Step 2) Compute the 2*program_width length-n polynomials ∏ A_i(j) and ∏ B_i(j)
 * Step 3) Compute the two length-n polynomials defined by
 *          numer[i] = ∏ A_1(j)⋅A_2(j)⋅A_3(j), and denom[i] = ∏ B_1(j)⋅B_2(j)⋅B_3(j)
 * Step 4) Compute Z_perm[i+1] = numer[i]/denom[i] (recall: Z_perm[0] = 1)
 *
 * Note: Step (4) utilizes Montgomery batch inversion to replace n-many inversions with
 * one batch inversion (at the expense of more multiplications)
 */
// TODO(#222)(luke): Parallelize
template <typename settings> Polynomial Prover<settings>::compute_grand_product_polynomial(Fr beta, Fr gamma)
{
    using barretenberg::polynomial_arithmetic::copy_polynomial;
    static const size_t program_width = settings::program_width;

    // Allocate scratch space for accumulators
    std::array<Fr*, program_width> numerator_accumulator;
    std::array<Fr*, program_width> denominator_accumulator;
    for (size_t i = 0; i < program_width; ++i) {
        numerator_accumulator[i] = static_cast<Fr*>(aligned_alloc(64, sizeof(Fr) * key->circuit_size));
        denominator_accumulator[i] = static_cast<Fr*>(aligned_alloc(64, sizeof(Fr) * key->circuit_size));
    }

    // Populate wire and permutation polynomials
    std::array<std::span<const Fr>, program_width> wires;
    std::array<std::span<const Fr>, program_width> sigmas;
    for (size_t i = 0; i < program_width; ++i) {
        std::string sigma_id = "sigma_" + std::to_string(i + 1) + "_lagrange";
        wires[i] = wire_polynomials[i];
        sigmas[i] = key->polynomial_store.get(sigma_id);
    }

    // Step (1)
    // TODO(#222)(kesha): Change the order to engage automatic prefetching and get rid of redundant computation
    for (size_t i = 0; i < key->circuit_size; ++i) {
        for (size_t k = 0; k < program_width; ++k) {
            // Note(luke): this idx could be replaced by proper ID polys if desired
            Fr idx = k * key->circuit_size + i;
            numerator_accumulator[k][i] = wires[k][i] + (idx * beta) + gamma;            // w_k(i) + β.(k*n+i) + γ
            denominator_accumulator[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
        }
    }

    // Step (2)
    for (size_t k = 0; k < program_width; ++k) {
        for (size_t i = 0; i < key->circuit_size - 1; ++i) {
            numerator_accumulator[k][i + 1] *= numerator_accumulator[k][i];
            denominator_accumulator[k][i + 1] *= denominator_accumulator[k][i];
        }
    }

    // Step (3)
    for (size_t i = 0; i < key->circuit_size; ++i) {
        for (size_t k = 1; k < program_width; ++k) {
            numerator_accumulator[0][i] *= numerator_accumulator[k][i];
            denominator_accumulator[0][i] *= denominator_accumulator[k][i];
        }
    }

    // Step (4)
    // Use Montgomery batch inversion to compute z_perm[i+1] = numerator_accumulator[0][i] /
    // denominator_accumulator[0][i]. At the end of this computation, the quotient numerator_accumulator[0] /
    // denominator_accumulator[0] is stored in numerator_accumulator[0].
    Fr* inversion_coefficients = &denominator_accumulator[1][0]; // arbitrary scratch space
    Fr inversion_accumulator = Fr::one();
    for (size_t i = 0; i < key->circuit_size; ++i) {
        inversion_coefficients[i] = numerator_accumulator[0][i] * inversion_accumulator;
        inversion_accumulator *= denominator_accumulator[0][i];
    }
    inversion_accumulator = inversion_accumulator.invert(); // perform single inversion per thread
    for (size_t i = key->circuit_size - 1; i != std::numeric_limits<size_t>::max(); --i) {
        numerator_accumulator[0][i] = inversion_accumulator * inversion_coefficients[i];
        inversion_accumulator *= denominator_accumulator[0][i];
    }

    // Construct permutation polynomial 'z_perm' in lagrange form as:
    // z_perm = [0 numerator_accumulator[0][0] numerator_accumulator[0][1] ... numerator_accumulator[0][n-2] 0]
    Polynomial z_perm(key->circuit_size);
    // We'll need to shift this polynomial to the left by dividing it by X in gemini, so the the 0-th coefficient should
    // stay zero
    copy_polynomial(numerator_accumulator[0], &z_perm[1], key->circuit_size - 1, key->circuit_size - 1);

    // free memory allocated for scratch space
    for (size_t k = 0; k < program_width; ++k) {
        aligned_free(numerator_accumulator[k]);
        aligned_free(denominator_accumulator[k]);
    }

    return z_perm;
}

/**
 * - Add circuit size, public input size, and public inputs to transcript
 *
 * */
template <typename settings> void Prover<settings>::execute_preamble_round()
{
    // queue.flush_queue(); // NOTE: Don't remove; we may reinstate the queue

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
    // queue.flush_queue(); // NOTE: Don't remove; we may reinstate the queue
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
    // queue.flush_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Compute and store parameters required by relations in Sumcheck
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    auto public_input_delta = compute_public_input_delta<Fr>(public_inputs, beta, gamma, key->circuit_size);

    relation_parameters = sumcheck::RelationParameters<Fr>{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    z_permutation = compute_grand_product_polynomial(beta, gamma);

    auto commitment = commitment_key->commit(z_permutation);

    transcript.send_to_verifier("Z_PERM", commitment);

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
    // queue.flush_queue(); // NOTE: Don't remove; we may reinstate the queue

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
    const size_t NUM_POLYNOMIALS = honk::StandardArithmetization::NUM_POLYNOMIALS;
    const size_t NUM_UNSHIFTED_POLYS = honk::StandardArithmetization::NUM_UNSHIFTED_POLYNOMIALS;

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

    // // Reserve space for d+1 Fold polynomials. At the end of this round, the last d-1 polynomials will
    // // correspond to Fold^(i). At the end of the full Gemini prover protocol, the first two will
    // // be the partially evaluated Fold polynomials Fold_{r}^(0) and Fold_{-r}^(0).
    // fold_polynomials.reserve(key->log_circuit_size + 1);
    // fold_polynomials.emplace_back(batched_poly_unshifted);
    // fold_polynomials.emplace_back(batched_poly_to_be_shifted);

    // Compute d-1 polynomials Fold^(i), i = 1, ..., d-1.
    fold_polynomials = Gemini::compute_fold_polynomials(
        sumcheck_output.challenge_point, std::move(batched_poly_unshifted), std::move(batched_poly_to_be_shifted));

    // Compute and add to trasnscript the commitments [Fold^(i)], i = 1, ..., d-1
    for (size_t l = 0; l < key->log_circuit_size - 1; ++l) {
        std::string label = "Gemini:FOLD_" + std::to_string(l + 1);
        auto commitment = commitment_key->commit(fold_polynomials[l + 2]);
        transcript.send_to_verifier(label, commitment);
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
 * - Do Fiat-Shamir to get "z" challenge.
 * - Compute polynomial Q(X) - Q_z(X)
 * */
template <typename settings> void Prover<settings>::execute_shplonk_round()
{
    shplonk_output =
        Shplonk::reduce_prove(commitment_key, gemini_output.opening_pairs, gemini_output.witnesses, transcript);
}

/**
 * - Compute KZG quotient commitment [W]_1.
 *
 * */
template <typename settings> void Prover<settings>::execute_kzg_round()
{
    KZG::reduce_prove(commitment_key, shplonk_output.opening_pair, shplonk_output.witness, transcript);
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
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Compute wire commitments; Add PI to transcript
    execute_wire_commitments_round();
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Currently a no-op; may execute some "random widgets", commit to W_4, do RAM/ROM stuff
    // if this prover structure is kept when we bring tables to Honk.
    // Suggestion: Maybe we shouldn't mix and match proof creation for different systems and
    // instead instatiate construct_proof differently for each?
    execute_tables_round();
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Fiat-Shamir: beta & gamma
    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();
    // // queue currently only handles commitments, not partial multivariate evaluations.
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Fiat-Shamir: rho
    // Compute Fold polynomials and their commitments.
    execute_univariatization_round();
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Fiat-Shamir: r
    // Compute Fold evaluations
    execute_pcs_evaluation_round();

    // Fiat-Shamir: nu
    // Compute Shplonk batched quotient commitment
    execute_shplonk_round();
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // Fiat-Shamir: z
    // Compute KZG quotient commitment
    execute_kzg_round();
    // queue.process_queue(); // NOTE: Don't remove; we may reinstate the queue

    // queue.flush_queue(); // NOTE: Don't remove; we may reinstate the queue

    return export_proof();
}

template class Prover<plonk::standard_settings>;

} // namespace honk
