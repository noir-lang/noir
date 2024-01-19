#include "prover.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "barretenberg/polynomials/iterate_over_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include <chrono>

using namespace bb;

namespace bb::plonk {

/**
 * Create ProverBase from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_witness Witness containing witness polynomials.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <typename settings>
ProverBase<settings>::ProverBase(std::shared_ptr<proving_key> input_key, const transcript::Manifest& input_manifest)
    : circuit_size(input_key == nullptr ? 0 : input_key->circuit_size)
    , transcript(input_manifest, settings::hash_type, settings::num_challenge_bytes)
    , key(input_key)
    , queue(key.get(), &transcript)
{}

template <typename settings>
ProverBase<settings>::ProverBase(ProverBase<settings>&& other)
    : circuit_size(other.circuit_size)
    , transcript(other.transcript)
    , key(std::move(other.key))
    , commitment_scheme(std::move(other.commitment_scheme))
    , queue(key.get(), &transcript)
{
    for (size_t i = 0; i < other.random_widgets.size(); ++i) {
        random_widgets.emplace_back(std::move(other.random_widgets[i]));
    }
    for (size_t i = 0; i < other.transition_widgets.size(); ++i) {
        transition_widgets.emplace_back(std::move(other.transition_widgets[i]));
    }
}

template <typename settings> ProverBase<settings>& ProverBase<settings>::operator=(ProverBase<settings>&& other)
{
    circuit_size = other.circuit_size;

    random_widgets.resize(0);
    transition_widgets.resize(0);
    for (size_t i = 0; i < other.random_widgets.size(); ++i) {
        random_widgets.emplace_back(std::move(other.random_widgets[i]));
    }
    for (size_t i = 0; i < other.transition_widgets.size(); ++i) {
        transition_widgets.emplace_back(std::move(other.transition_widgets[i]));
    }
    transcript = other.transcript;
    key = std::move(other.key);
    commitment_scheme = std::move(other.commitment_scheme);

    queue = work_queue(key.get(), &transcript);
    return *this;
}

/**
 * - Compute wire commitments and add them to the transcript.
 * - Add public_inputs from w_2_fft to transcript.
 *
 * @tparam settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::compute_wire_commitments()
{
    // Compute wire commitments
    const size_t end = settings::is_plookup ? (settings::program_width - 1) : settings::program_width;
    for (size_t i = 0; i < end; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        std::string commit_tag = "W_" + std::to_string(i + 1);
        auto poly = key->polynomial_store.get(wire_tag);
        auto coefficients = poly.data();

        // This automatically saves the computed point to the transcript
        fr domain_size_flag = i > 2 ? key->circuit_size : (key->circuit_size + 1);
        commitment_scheme->commit(coefficients, commit_tag, domain_size_flag, queue);
    }

    // add public inputs
    const polynomial& public_wires_source = key->polynomial_store.get("w_2_lagrange");
    std::vector<fr> public_wires;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_wires.push_back(public_wires_source[i]);
    }
    transcript.add_element("public_inputs", ::to_buffer(public_wires));
}

template <typename settings> void ProverBase<settings>::compute_quotient_commitments()
{
    // In this method, we compute the commitments to polynomials t_{low}(X), t_{mid}(X) and t_{high}(X).
    // Recall, the quotient polynomial t(X) = t_{low}(X) + t_{mid}(X).X^n + t_{high}(X).X^{2n}
    //
    // The reason we split t(X) into three degree-n polynomials is because:
    //  (i) We want the opening proof polynomials bounded by degree n as the opening algorithm of the
    //      polynomial commitment scheme results in O(n) prover computation.
    // (ii) The size of the srs restricts us to compute commitments to polynomials of degree n
    //      (and disallows for degree 2n and 3n for large n).
    //
    // The degree of t(X) is determined by the term:
    // ((a(X) + βX + γ) (b(X) + βk_1X + γ) (c(X) + βk_2X + γ)z(X)) / Z*_H(X).
    //
    // Let k = num_roots_cut_out_of_vanishing_polynomial, we have
    // deg(t) = (n - 1) * (program_width + 1) - (n - k)
    //        = n * program_width - program_width - 1 + k
    //
    // Since we must cut at least 4 roots from the vanishing polynomial
    // (refer to ./src/barretenberg/plonk/proof_system/widgets/random_widgets/permutation_widget_impl.hpp/L247),
    // k = 4 => deg(t) = n * program_width - program_width + 3
    //
    // For standard plonk, program_width = 3 and thus, deg(t) = 3n. This implies that there would be
    // (3n + 1) coefficients of t(X). Now, splitting them into t_{low}(X), t_{mid}(X) and t_{high}(X),
    // t_{high} will have (n+1) coefficients while t_{low} and t_{mid} will have n coefficients.
    // This means that to commit t_{high}, we need a multi-scalar multiplication of size (n+1).
    // Thus, we first compute the commitments to t_{low}(X), t_{mid}(X) using n multi-scalar multiplications
    // each and separately compute commitment to t_{high} which is of size (n + 1).
    // Note that this must be done only when program_width = 3.
    //
    //
    // NOTE: If in future there is a need to cut off more zeros off the vanishing polynomial, the degree of
    // the quotient polynomial t(X) will increase, so the degrees of t_{high}, t_{mid}, t_{low} could also
    // increase according to the type of the composer type we are using. Currently, for Ultra-
    // PLONK, the degree of t(X) is (4n - 1) and hence each t_{low}, t_{mid}, t_{high}, t_{higher} each is of
    // degree (n - 1) (and thus contains n coefficients). Therefore, we are on the brink!
    // If we need to cut out more zeros off the vanishing polynomial, sizes of coefficients of individual
    // t_{i} would change and so we will have to ensure the correct size of multi-scalar multiplication in
    // computing the commitments to these polynomials.
    //
    for (size_t i = 0; i < settings::program_width; ++i) {
        auto coefficients = key->quotient_polynomial_parts[i].data();
        std::string quotient_tag = "T_" + std::to_string(i + 1);
        // Set flag that determines domain size (currently n or n+1) in pippenger (see process_queue()).
        // Note: After blinding, all t_i have size n+1 representation (degree n) except t_4 in Ultra.
        fr domain_size_flag = i > 2 ? key->circuit_size : (key->circuit_size + 1);
        commitment_scheme->commit(coefficients, quotient_tag, domain_size_flag, queue);
    }
}

/**
 * Execute preamble round.
 * - Execute init round
 * - Add randomness to the wire witness polynomials for Honest-Verifier Zero Knowledge.
 *
 * N.B. Maybe we need to refactor this, since before we execute this function wires are in lagrange basis
 * and after they are in monomial form. This is an inconsistency that can mislead developers.
 *
 * @tparam settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::execute_preamble_round()
{
    queue.flush_queue();

    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(circuit_size >> 24),
                             static_cast<uint8_t>(circuit_size >> 16),
                             static_cast<uint8_t>(circuit_size >> 8),
                             static_cast<uint8_t>(circuit_size) });

    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs >> 24),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs) });

    transcript.apply_fiat_shamir("init");

    // If this is a plookup proof, do not queue up an ifft on W_4 - we can only finish computing
    // the lagrange-base values in W_4 once eta has been generated.
    // This is because of the RAM/ROM subprotocol, which adds witnesses into W_4 that depend on eta
    const size_t end = settings::is_plookup ? (settings::program_width - 1) : settings::program_width;
    for (size_t i = 0; i < end; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        auto wire_lagrange = key->polynomial_store.get(wire_tag + "_lagrange");

        /*
        Adding zero knowledge to the witness polynomials.
        */
        // To ensure that PLONK is honest-verifier zero-knowledge, we need to ensure that the witness polynomials
        // and the permutation polynomial look uniformly random to an adversary. To make the witness polynomials
        // a(X), b(X) and c(X) uniformly random, we need to add 2 random blinding factors into each of them.
        // i.e. a'(X) = a(X) + (r_1X + r_2)
        // where r_1 and r_2 are uniformly random scalar field elements. A natural question is:
        // Why do we need 2 random scalars in witness polynomials? The reason is: our witness polynomials are
        // evaluated at only 1 point (\scripted{z}), so adding a random degree-1 polynomial suffices.
        //
        // NOTE: In UltraPlonk, the witness polynomials are evaluated at 2 points and thus
        // we need to add 3 random scalars in them.
        //
        // We start adding random scalars in `wire` polynomials from index (n - k) upto (n - k + 2).
        // For simplicity, we add 3 random scalars even for standard plonk (recall, just 2 of them are required)
        // since an additional random scalar would not affect things.
        //
        // NOTE: If in future there is a need to cut off more zeros off the vanishing polynomial, this method
        // will not change. This must be changed only if the number of evaluations of witness polynomials
        // change.
        const size_t w_randomness = 3;
        ASSERT(w_randomness < settings::num_roots_cut_out_of_vanishing_polynomial);
        for (size_t k = 0; k < w_randomness; ++k) {
            wire_lagrange.at(circuit_size - settings::num_roots_cut_out_of_vanishing_polynomial + k) =
                fr::random_element();
        }

        key->polynomial_store.put(wire_tag + "_lagrange", std::move(wire_lagrange));
    }

    // perform an IFFT so that the "w_i" polynomial cache will contain the monomial form
    for (size_t i = 0; i < end; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        queue.add_to_queue({
            .work_type = work_queue::WorkType::IFFT,
            .mul_scalars = nullptr,
            .tag = wire_tag,
            .constant = 0,
            .index = 0,
        });
    }
}

/**
 * Execute the first round:
 * - Compute wire commitments.
 * - Add public input values to the transcript
 *
 * N.B. Random widget precommitments aren't actually being computed, since we are using permutation widget
 * which only does computation in compute_random_commitments function if the round is 3.
 *
 * @tname settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::execute_first_round()
{
    queue.flush_queue();
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cerr << "init quotient polys: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cerr << "compute wire coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    compute_wire_commitments();

    for (auto& widget : random_widgets) {
        widget->compute_round_commitments(transcript, 1, queue);
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cerr << "compute wire commitments: " << diff.count() << "ms" << std::endl;
#endif
}

/**
 * Execute second round:
 * - Apply Fiat-Shamir transform to generate the "eta" challenge
 * - Compute the random_widgets' round commitments that need to be computed at round 2.
 * - If using plookup, we compute some w_4 values here (for gates which access "memory"), and apply blinding factors,
 * before finally committing to w_4.
 *
 * @tname settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::execute_second_round()
{
    queue.flush_queue();

    transcript.apply_fiat_shamir("eta");

    for (auto& widget : random_widgets) {
        widget->compute_round_commitments(transcript, 2, queue);
    }

    // RAM/ROM memory subprotocol requires eta is generated before w_4 is comitted
    if (settings::is_plookup) {
        add_plookup_memory_records_to_w_4();
        std::string wire_tag = "w_4";
        auto w_4_lagrange = key->polynomial_store.get(wire_tag + "_lagrange");

        // add randomness to w_4_lagrange
        const size_t w_randomness = 3;
        ASSERT(w_randomness < settings::num_roots_cut_out_of_vanishing_polynomial);
        for (size_t k = 0; k < w_randomness; ++k) {
            // Blinding
            w_4_lagrange.at(circuit_size - settings::num_roots_cut_out_of_vanishing_polynomial + k) =
                fr::random_element();
        }

        // compute poly w_4 from w_4_lagrange and add it to the cache
        bb::polynomial w_4(key->circuit_size);
        bb::polynomial_arithmetic::copy_polynomial(&w_4_lagrange[0], &w_4[0], circuit_size, circuit_size);
        w_4.ifft(key->small_domain);
        key->polynomial_store.put(wire_tag, std::move(w_4));
        key->polynomial_store.put(wire_tag + "_lagrange", std::move(w_4_lagrange));

        // commit to w_4 using the monomial srs.
        queue.add_to_queue({
            .work_type = work_queue::WorkType::SCALAR_MULTIPLICATION,
            .mul_scalars = key->polynomial_store.get(wire_tag).data(),
            .tag = "W_4",
            .constant = key->circuit_size + 1,
            .index = 0,
        });
    }
}

/**
 * Execute third round:
 * - Apply Fiat-Shamir transform on the "beta" challenge
 * - Apply 3rd round random widgets*
 * - FFT the wires.
 *
 * *For example, standard composer executes permutation widget for z polynomial construction at this round.
 *
 * @tparam settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::execute_third_round()
{
    queue.flush_queue();

    transcript.apply_fiat_shamir("beta");

    for (auto& widget : random_widgets) {
        widget->compute_round_commitments(transcript, 3, queue);
    }

    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        queue.add_to_queue({
            .work_type = work_queue::WorkType::FFT,
            .mul_scalars = nullptr,
            .tag = wire_tag,
            .constant = bb::fr(0),
            .index = 0,
        });
    }
}

/**
 * @brief Computes the quotient polynomial, then commits to its degree-n split parts.
 */
template <typename settings> void ProverBase<settings>::execute_fourth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("alpha");
    fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    // Compute FFT of lagrange polynomial L_1 (needed in random widgets only)
    compute_lagrange_1_fft();

    for (auto& widget : random_widgets) {
        alpha_base = widget->compute_quotient_contribution(alpha_base, transcript);
    }

    for (auto& widget : transition_widgets) {
        alpha_base = widget->compute_quotient_contribution(alpha_base, transcript);
    }

    // The parts of the quotient polynomial t(X) are stored as 4 separate polynomials in
    // the code. However, operations such as dividing by the pseudo vanishing polynomial
    // as well as iFFT (coset) are to be performed on the polynomial t(X) as a whole.
    // We avoid redundant copy of the parts t_1, t_2, t_3, t_4 and instead just tweak the
    // relevant functions to work on quotient polynomial parts.
    std::vector<fr*> quotient_poly_parts;
    quotient_poly_parts.push_back(&key->quotient_polynomial_parts[0][0]);
    quotient_poly_parts.push_back(&key->quotient_polynomial_parts[1][0]);
    quotient_poly_parts.push_back(&key->quotient_polynomial_parts[2][0]);
    quotient_poly_parts.push_back(&key->quotient_polynomial_parts[3][0]);
    bb::polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(
        quotient_poly_parts, key->small_domain, key->large_domain);

    polynomial_arithmetic::coset_ifft(quotient_poly_parts, key->large_domain);

    // Manually copy the (n + 1)th coefficient of t_3 for StandardPlonk from t_4.
    // This is because the degree of t_3 for StandardPlonk is n.
    if (settings::program_width == 3) {
        key->quotient_polynomial_parts[2][circuit_size] = key->quotient_polynomial_parts[3][0];
        key->quotient_polynomial_parts[3][0] = 0;
    }

    add_blinding_to_quotient_polynomial_parts();

    compute_quotient_commitments();
} // namespace bb::plonk

template <typename settings> void ProverBase<settings>::execute_fifth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("z"); // end of 4th round
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
    compute_quotient_evaluation();
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cerr << "compute quotient evaluation: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_sixth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("nu");
    commitment_scheme->batch_open(transcript, queue, key);
}

template <typename settings> void ProverBase<settings>::compute_quotient_evaluation()
{

    fr zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());

    commitment_scheme->add_opening_evaluations_to_transcript(transcript, key, false);

    fr t_eval = polynomial_arithmetic::evaluate({ &key->quotient_polynomial_parts[0][0],
                                                  &key->quotient_polynomial_parts[1][0],
                                                  &key->quotient_polynomial_parts[2][0],
                                                  &key->quotient_polynomial_parts[3][0] },
                                                zeta,
                                                4 * circuit_size);

    fr zeta_pow_n = zeta.pow(key->circuit_size);
    fr scalar = zeta_pow_n;
    // Adjust the evaluation to consider the (n + 1)th coefficient when needed (note that width 3 is just an avatar for
    // StandardPlonkComposer here)
    const size_t num_deg_n_poly = settings::program_width == 3 ? settings::program_width : settings::program_width - 1;
    for (size_t j = 0; j < num_deg_n_poly; j++) {
        t_eval += key->quotient_polynomial_parts[j][key->circuit_size] * scalar;
        scalar *= zeta_pow_n;
    }

    transcript.add_element("t", t_eval.to_buffer());
}

// Add blinding to the components in such a way that the full quotient would be unchanged if reconstructed
template <typename settings> void ProverBase<settings>::add_blinding_to_quotient_polynomial_parts()
{
    // Construct blinded quotient polynomial parts t_i by adding randomness to the unblinded parts t_i' in
    // such a way that the full quotient polynomial t is unchanged upon reconstruction, i.e.
    //
    //        t = t_1' + X^n*t_2' + X^2n*t_3' + X^3n*t_4' = t_1 + X^n*t_2 + X^2n*t_3 + X^3n*t_4
    //
    // Blinding is done as follows, where b_i are random field elements:
    //
    //              t_1 = t_1' +       b_0*X^n
    //              t_2 = t_2' - b_0 + b_1*X^n
    //              t_3 = t_3' - b_1 + b_2*X^n
    //              t_4 = t_4' - b_2
    //
    // For details, please head to: https://hackmd.io/JiyexiqRQJW55TMRrBqp1g.
    for (size_t i = 0; i < settings::program_width - 1; i++) {
        // Note that only program_width-1 random elements are required for full blinding
        fr quotient_randomness = fr::random_element();

        key->quotient_polynomial_parts[i][key->circuit_size] +=
            quotient_randomness;                                         // update coefficient of X^n'th term
        key->quotient_polynomial_parts[i + 1][0] -= quotient_randomness; // update constant coefficient
    }
}

// Compute FFT of lagrange polynomial L_1 needed in random widgets only
template <typename settings> void ProverBase<settings>::compute_lagrange_1_fft()
{
    polynomial lagrange_1_fft(4 * circuit_size + 8);
    polynomial_arithmetic::compute_lagrange_polynomial_fft(
        lagrange_1_fft.data().get(), key->small_domain, key->large_domain);
    for (size_t i = 0; i < 8; i++) {
        lagrange_1_fft[4 * circuit_size + i] = lagrange_1_fft[i];
    }
    key->polynomial_store.put("lagrange_1_fft", std::move(lagrange_1_fft));
}

template <typename settings> plonk::proof& ProverBase<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> plonk::proof& ProverBase<settings>::construct_proof()
{
    // Execute init round. Randomize witness polynomials.
    // info("preamble");
    execute_preamble_round();
    queue.process_queue();

    // Compute wire precommitments and sometimes random widget round commitments
    // info("first");
    execute_first_round();
    queue.process_queue();

    // Fiat-Shamir eta + execute random widgets.
    // info("second");
    execute_second_round();
    queue.process_queue();

    // Fiat-Shamir beta & gamma, execute random widgets (Permutation widget is executed here)
    // and fft the witnesses
    // info("third");
    execute_third_round();
    queue.process_queue();

    // Fiat-Shamir alpha, compute & commit to quotient polynomial.
    // info("fourth");
    execute_fourth_round();
    queue.process_queue();

    // info("fifth");
    execute_fifth_round();

    // info("sixth");
    execute_sixth_round();
    queue.process_queue();

    queue.flush_queue();

    return export_proof();
}

template <typename settings> void ProverBase<settings>::reset()
{
    transcript::Manifest manifest = transcript.get_manifest();
    transcript = transcript::StandardTranscript(manifest, settings::hash_type, settings::num_challenge_bytes);
}

template <typename settings> void ProverBase<settings>::add_plookup_memory_records_to_w_4()
{
    // We can only compute memory record values once W_1, W_2, W_3 have been comitted to,
    // due to the dependence on the `eta` challenge.

    const fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());

    // We need the lagrange-base forms of the first 3 wires to compute the plookup memory record
    // value. w4 = w3 * eta^3 + w2 * eta^2 + w1 * eta + read_write_flag;
    // a RAM write. See plookup_auxiliary_widget.hpp for details)
    auto w_1 = key->polynomial_store.get("w_1_lagrange");
    auto w_2 = key->polynomial_store.get("w_2_lagrange");
    auto w_3 = key->polynomial_store.get("w_3_lagrange");
    auto w_4 = key->polynomial_store.get("w_4_lagrange");
    for (const auto& gate_idx : key->memory_read_records) {
        w_4[gate_idx] += w_3[gate_idx];
        w_4[gate_idx] *= eta;
        w_4[gate_idx] += w_2[gate_idx];
        w_4[gate_idx] *= eta;
        w_4[gate_idx] += w_1[gate_idx];
        w_4[gate_idx] *= eta;
    }
    for (const auto& gate_idx : key->memory_write_records) {
        w_4[gate_idx] += w_3[gate_idx];
        w_4[gate_idx] *= eta;
        w_4[gate_idx] += w_2[gate_idx];
        w_4[gate_idx] *= eta;
        w_4[gate_idx] += w_1[gate_idx];
        w_4[gate_idx] *= eta;
        w_4[gate_idx] += 1;
    }
    key->polynomial_store.put("w_4_lagrange", std::move(w_4));
}

template class ProverBase<standard_settings>;
template class ProverBase<ultra_settings>;
template class ProverBase<ultra_to_standard_settings>;
template class ProverBase<ultra_with_keccak_settings>;

} // namespace bb::plonk
