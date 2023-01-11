#include "prover.hpp"
// #include <honk/sumcheck/sumcheck.hpp> // will need
#include <honk/sumcheck/polynomials/univariate.hpp> // will go away
#include <vector>
#include "ecc/curves/bn254/fr.hpp"
#include "ecc/curves/bn254/g1.hpp"
#include "proof_system/flavor/flavor.hpp"

namespace honk {

/**
 * Create Prover from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <typename settings>
Prover<settings>::Prover(std::shared_ptr<waffle::proving_key> input_key, const transcript::Manifest& input_manifest)
    : n(input_key == nullptr ? 0 : input_key->n)
    , transcript(input_manifest, settings::hash_type, settings::num_challenge_bytes)
    , key(input_key)
    , queue(key.get(), &transcript)
{}

/**
 * For Plonk systems:
 * - Compute commitments to wires 1,2,3
 * - Get public inputs (which were stored in w_2_lagrange) and add to transcript
 *
 * For Honk, we should
 * - Commit to wires 1,2,3
 * - Add PI to transcript (I guess PI will stay in w_2 for now?)
 *
 * */
template <typename settings> void Prover<settings>::compute_wire_commitments()
{
    // TODO(luke): Compute wire commitments
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        std::string commit_tag = "W_" + std::to_string(i + 1);
        // barretenberg::fr* coefficients = key->polynomial_cache.get(wire_tag).get_coefficients();
        // This automatically saves the computed point to the transcript
        // commitment_scheme->commit(coefficients, commit_tag, work_queue::MSMSize::N, queue);
        transcript.add_element(commit_tag, barretenberg::g1::affine_one.to_buffer()); // TODO(Cody): unmock.
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
template <typename settings> void Prover<settings>::compute_grand_product_polynomial()
{
    // TODO: Fr to become template param
    using Fr = barretenberg::fr;
    using barretenberg::polynomial;
    using barretenberg::polynomial_arithmetic::copy_polynomial;
    static const size_t program_width = settings::program_width;

    // Allocate scratch space for accumulators
    Fr* numererator_accum[program_width];
    Fr* denominator_accum[program_width];
    for (size_t i = 0; i < program_width; ++i) {
        numererator_accum[i] = static_cast<Fr*>(aligned_alloc(64, sizeof(Fr) * key->n));
        denominator_accum[i] = static_cast<Fr*>(aligned_alloc(64, sizeof(Fr) * key->n));
    }

    // Populate wire and permutation polynomials
    std::array<const Fr*, program_width> wires;
    std::array<const Fr*, program_width> sigmas;
    for (size_t i = 0; i < program_width; ++i) {
        std::string wire_id = "wire_" + std::to_string(i + 1) + "_lagrange";
        std::string sigma_id = "sigma_" + std::to_string(i + 1) + "_lagrange";
        wires[i] = key->polynomial_cache.get(wire_id).get_coefficients();
        sigmas[i] = key->polynomial_cache.get(sigma_id).get_coefficients();
    }

    // Get random challenges (TODO(luke): to be obtained from transcript)
    Fr beta = Fr::one();
    Fr gamma = Fr::one();

    // Step (1)
    for (size_t i = 0; i < key->n; ++i) {
        for (size_t k = 0; k < program_width; ++k) {
            // TODO(luke): maybe this idx is replaced by proper ID polys in the future
            Fr idx = k * key->n + i;
            numererator_accum[k][i] = wires[k][i] + (idx * beta) + gamma;          // w_k(i) + β.(k*n+i) + γ
            denominator_accum[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
        }
    }

    // Step (2)
    for (size_t k = 0; k < program_width; ++k) {
        for (size_t i = 0; i < key->n - 1; ++i) {
            numererator_accum[k][i + 1] *= numererator_accum[k][i];
            denominator_accum[k][i + 1] *= denominator_accum[k][i];
        }
    }

    // Step (3)
    for (size_t i = 0; i < key->n; ++i) {
        for (size_t k = 1; k < program_width; ++k) {
            numererator_accum[0][i] *= numererator_accum[k][i];
            denominator_accum[0][i] *= denominator_accum[k][i];
        }
    }

    // Step (4)
    // Use Montgomery batch inversion to compute z_perm[i+1] = numererator_accum[0][i] / denominator_accum[0][i]. At the
    // end of this computation, the quotient numererator_accum[0] / denominator_accum[0] is stored in
    // numererator_accum[0].
    Fr* inversion_coefficients = &denominator_accum[1][0]; // arbitrary scratch space
    Fr inversion_accumulator = Fr::one();
    for (size_t i = 0; i < key->n; ++i) {
        inversion_coefficients[i] = numererator_accum[0][i] * inversion_accumulator;
        inversion_accumulator *= denominator_accum[0][i];
    }
    inversion_accumulator = inversion_accumulator.invert(); // perform single inversion per thread
    for (size_t i = key->n - 1; i != size_t(0) - 1; --i) {
        // TODO(luke): What needs to be done Re the comment below:
        // We can avoid fully reducing z_perm[i + 1] as the inverse fft will take care of that for us
        numererator_accum[0][i] = inversion_accumulator * inversion_coefficients[i];
        inversion_accumulator *= denominator_accum[0][i];
    }

    // Construct permutation polynomial 'z_perm' in lagrange form as:
    // z_perm = [1 numererator_accum[0][0] numererator_accum[0][1] ... numererator_accum[0][n-2]]
    polynomial z_perm(key->n, key->n);
    z_perm[0] = Fr::one();
    copy_polynomial(numererator_accum[0], &z_perm[1], key->n - 1, key->n - 1);

    // free memory allocated for scratch space
    for (size_t k = 0; k < program_width; ++k) {
        aligned_free(numererator_accum[k]);
        aligned_free(denominator_accum[k]);
    }

    // TODO(luke): Commit to z_perm here? This would match Plonk but maybe best to do separately?

    key->polynomial_cache.put("z_perm", std::move(z_perm));
}

/**
 * For Plonk systems:
 * - added some initial data to transcript: circuit size and PI size
 * - added randomness to lagrange wires
 * - performed ifft to get monomial wires
 *
 * For Honk:
 * - Add circuit size and PI size to transcript. That's it?
 *
 * */
template <typename settings> void Prover<settings>::execute_preamble_round()
{
    // Add some initial data to transcript (circuit size and PI size)
    queue.flush_queue();

    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(n >> 24),
                             static_cast<uint8_t>(n >> 16),
                             static_cast<uint8_t>(n >> 8),
                             static_cast<uint8_t>(n) });

    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs >> 24),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs) });

    transcript.apply_fiat_shamir("init");
}

/**
 * For Plonk systems:
 * - compute wire commitments
 * - add public inputs to transcript (done in compute_wire_commitments() for some reason)
 *
 * For Honk:
 * - compute wire commitments
 * - add public inputs to transcript (done explicitly in execute_first_round())
 * */
template <typename settings> void Prover<settings>::execute_wire_commitments_round()
{
    queue.flush_queue();

    compute_wire_commitments();

    // Add public inputs to transcript
    const barretenberg::polynomial& public_wires_source = key->polynomial_cache.get("w_2_lagrange");
    std::vector<barretenberg::fr> public_wires;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_wires.push_back(public_wires_source[i]);
    }
    transcript.add_element("public_inputs", ::to_buffer(public_wires));
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "eta" challenge (done regardless of arithmetization but only required for Ultra)
 * - does stuff related only to lookups (compute 's' etc and do some RAM/ROM stuff with w_4).
 *
 * For Standard Honk, this is a non-op (just like for Standard/Turbo Plonk).
 * */
template <typename settings> void Prover<settings>::execute_tables_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("eta");

    // No operations are needed here for Standard Honk
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "beta" challenge
 * - Compute grand product polynomials (permutation and lookup) and commitments
 * - Compute wire polynomial coset FFTs
 *
 * For Honk:
 * - Do Fiat-Shamir to get "beta" challenge (Note: gamma = beta^2)
 * - Compute grand product polynomial (permutation only) and commitment
 * */
template <typename settings> void Prover<settings>::execute_grand_product_computation_round()
{
    queue.flush_queue();

    // Compute beta/gamma challenge (Note: gamma = beta^2)
    transcript.apply_fiat_shamir("beta");

    // TODO(luke): compute_grand_product_polynomial
    // TODO(luke): compute_grand_product_polynomial_commitment
    transcript.add_element("Z_PERM", barretenberg::g1::affine_one.to_buffer()); // TODO(Cody): unmock.
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "alpha" challenge
 * - Compute coset_fft(L_1)
 * - Compute quotient polynomial (with blinding)
 * - Compute quotient polynomial commitment
 *
 * For Honk
 * - Do Fiat-Shamir to get "alpha" challenge
 * - Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all
 *   evaluations at u being calculated.
 * */
template <typename settings> void Prover<settings>::execute_relation_check_rounds()
{
    queue.flush_queue();

    // Compute alpha challenge
    transcript.apply_fiat_shamir("alpha");

    // TODO(luke): Run Sumcheck. For now, mock univariates.
    for (size_t round_idx = 0; round_idx < key->log_n; round_idx++) {
        honk::sumcheck::Univariate<barretenberg::fr, honk::StandardHonk::MAX_RELATION_LENGTH> round_univariate;
        for (auto eval : round_univariate.evaluations) {
            eval = round_idx;
        }
        transcript.add_element("univariate_" + std::to_string(key->log_n - round_idx), round_univariate.to_buffer());
        transcript.apply_fiat_shamir("u_" + std::to_string(key->log_n - round_idx));
    }

    transcript.add_element("w_1", barretenberg::fr(100).to_buffer());
    transcript.add_element("w_2", barretenberg::fr(101).to_buffer());
    transcript.add_element("w_3", barretenberg::fr(102).to_buffer());
    transcript.add_element("sigma_1", barretenberg::fr(103).to_buffer());
    transcript.add_element("sigma_2", barretenberg::fr(104).to_buffer());
    transcript.add_element("sigma_3", barretenberg::fr(105).to_buffer());
    transcript.add_element("q_1", barretenberg::fr(106).to_buffer());
    transcript.add_element("q_2", barretenberg::fr(107).to_buffer());
    transcript.add_element("q_3", barretenberg::fr(108).to_buffer());
    transcript.add_element("q_m", barretenberg::fr(109).to_buffer());
    transcript.add_element("q_c", barretenberg::fr(110).to_buffer());
    transcript.add_element("z_perm", barretenberg::fr(111).to_buffer());
}

/**
 * For Plonk: the polynomials are univariate, so this is a no-op.
 * For Honk:
 * - Get rho challenge
 * - Compute Fold polynomials and commitments.
 *
 * */
template <typename settings> void Prover<settings>::execute_univariatization_round()
{
    transcript.apply_fiat_shamir("rho");
    // TODO(Cody): Implement
    for (size_t round_idx = 1; round_idx < key->log_n; round_idx++) {
        transcript.add_element("FOLD_" + std::to_string(round_idx), barretenberg::g1::affine_one.to_buffer());
    }
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "frak-z" challenge
 * - Compute linearization or evaluation of quotient polynomial.
 *
 * For Honk:
 * - Do Fiat-Shamir to get "r" challenge
 * - Compute evaluations of folded polynomials.
 * */
template <typename settings> void Prover<settings>::execute_pcs_evaluation_round()
{
    transcript.apply_fiat_shamir("r");
    // TODO(Cody): Implement
    for (size_t round_idx = 0; round_idx < key->log_n; round_idx++) {
        transcript.add_element("a_" + std::to_string(round_idx), barretenberg::fr(round_idx + 1000).to_buffer());
    }
}

/**
 * For Plonk: Batching is combined with generation of opening proof polynomial commitments.
 *
 * For Honk:
 * - Do Fiat-Shamir to get "nu" challenge.
 * - Compute Shplonk batched quotient commitment [Q]_1.
 * */
template <typename settings> void Prover<settings>::execute_shplonk_round()
{
    // TODO(luke): Do Fiat-Shamir to get "nu" challenge.
    // TODO(luke): Get Shplonk opening point [Q]_1
    transcript.apply_fiat_shamir("nu");
    transcript.add_element("Q", barretenberg::g1::affine_one.to_buffer());
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "nu" challenge
 * - Compute KZG batch opening polynomial commitments.
 *
 * For Honk:
 * - Get "z" challenge.
 * - Compute KZG quotient [W]_1.
 *
 * */
template <typename settings> void Prover<settings>::execute_kzg_round()
{
    transcript.apply_fiat_shamir("z");
    // TODO(luke): Do Fiat-Shamir to get "z" challenge.
    // TODO(luke): Get KZG opening point [W]_1
    transcript.add_element("W", barretenberg::g1::affine_one.to_buffer());
    // transcript.apply_fiat_shamir("separator");
}
template <typename settings> waffle::plonk_proof& Prover<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> waffle::plonk_proof& Prover<settings>::construct_proof()
{
    // Add circuit size and public input size to transcript.
    execute_preamble_round();
    queue.process_queue();

    // Compute wire commitments; Add PI to transcript
    execute_wire_commitments_round();
    queue.process_queue();

    // Currently a no-op; may execute some "random widgets", commit to W_4, do RAM/ROM stuff
    // if this prover structure is kept when we bring tables to Honk.
    execute_tables_round();
    queue.process_queue();

    // Fiat-Shamir: beta & gamma
    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();
    queue.process_queue();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();
    // // queue currently only handles commitments, not partial multivariate evaluations.
    // queue.process_queue();

    // Fiat-Shamir: rho
    // Compute Fold polynomials and their commitments.
    execute_univariatization_round();
    queue.process_queue();

    // Fiat-Shamir: r
    // Compute Fold evaluations
    execute_pcs_evaluation_round();

    // Fiat-Shamir: nu
    // Compute Shplonk batched quotient commitment
    execute_shplonk_round();
    queue.process_queue();

    // Fiat-Shamir: z
    // Compute KZG quotient commitment
    execute_kzg_round();
    queue.process_queue();

    queue.flush_queue();

    return export_proof();
}

// TODO(luke): Need to define a 'standard_settings' analog for Standard Honk
template class Prover<waffle::standard_settings>;

} // namespace honk
