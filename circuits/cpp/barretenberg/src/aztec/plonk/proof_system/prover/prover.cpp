#include "prover.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <chrono>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;

namespace waffle {

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
ProverBase<settings>::ProverBase(std::shared_ptr<proving_key> input_key,
                                 std::shared_ptr<program_witness> input_witness,
                                 const transcript::Manifest& input_manifest)
    : n(input_key == nullptr ? 0 : input_key->n)
    , transcript(input_manifest, settings::hash_type, settings::num_challenge_bytes)
    , key(input_key)
    , witness(input_witness)
    , queue(key.get(), witness.get(), &transcript)
{
    if (input_witness && witness->wires.count("z") == 0) {
        witness->wires.insert({ "z", polynomial(n, n) });
    }
}

template <typename settings>
ProverBase<settings>::ProverBase(ProverBase<settings>&& other)
    : n(other.n)
    , transcript(other.transcript)
    , key(std::move(other.key))
    , witness(std::move(other.witness))
    , commitment_scheme(std::move(other.commitment_scheme))
    , queue(key.get(), witness.get(), &transcript)
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
    n = other.n;

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
    witness = std::move(other.witness);
    commitment_scheme = std::move(other.commitment_scheme);

    queue = work_queue(key.get(), witness.get(), &transcript);
    return *this;
}

/**
 * Compute wire precommitments and add public_inputs from w_2_fft to transcript.
 *
 * @tparam settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::compute_wire_pre_commitments()
{
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        std::string commit_tag = "W_" + std::to_string(i + 1);
        barretenberg::fr* coefficients = witness->wires.at(wire_tag).get_coefficients();
        commitment_scheme->commit(coefficients, commit_tag, barretenberg::fr(0), queue);
    }

    // add public inputs
    const polynomial& public_wires_source = key->wire_ffts.at("w_2_fft");
    std::vector<fr> public_wires;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_wires.push_back(public_wires_source[i]);
    }
    transcript.add_element("public_inputs", ::to_buffer(public_wires));
}

template <typename settings> void ProverBase<settings>::compute_quotient_pre_commitment()
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
    // Since we must cut atleast 4 roots from the vanishing polynomial
    // (refer to ./src/aztec/plonk/proof_system/widgets/random_widgets/permutation_widget_impl.hpp/L247),
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
    // increase according to the type of the composer type we are using. Currently, for TurboPLONK and Ultra-
    // PLONK, the degree of t(X) is (4n - 1) and hence each t_{low}, t_{mid}, t_{high}, t_{higher} each is of
    // degree (n - 1) (and thus contains n coefficients). Therefore, we are on the brink!
    // If we need to cut out more zeros off the vanishing polynomial, sizes of coefficients of individual
    // t_{i} would change and so we will have to ensure the correct size of multi-scalar multiplication in
    // computing the commitments to these polynomials.
    //
    for (size_t i = 0; i < settings::program_width - 1; ++i) {
        const size_t offset = n * i;
        fr* coefficients = &key->quotient_large.get_coefficients()[offset];
        std::string quotient_tag = "T_" + std::to_string(i + 1);
        commitment_scheme->commit(coefficients, quotient_tag, barretenberg::fr(0), queue);
    }

    fr* coefficients = &key->quotient_large.get_coefficients()[(settings::program_width - 1) * n];
    std::string quotient_tag = "T_" + std::to_string(settings::program_width);
    fr program_flag = settings::program_width == 3 ? barretenberg::fr(1) : barretenberg::fr(0);
    commitment_scheme->commit(coefficients, quotient_tag, program_flag, queue);
}

/**
 * Execute preamble round.
 * Execute init round, add randomness to witness polynomials for Honest-Verifier Zero Knowledge.
 * N.B. Maybe we need to refactor this, since before we execute this function wires are in lagrange basis
 * and after they are in monomial form. This is an inconsistency that can mislead developers.
 *
 * @tparam settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::execute_preamble_round()
{
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

    for (size_t i = 0; i < settings::program_width; ++i) {
        // fetch witness wire w_i
        std::string wire_tag = "w_" + std::to_string(i + 1);
        barretenberg::polynomial& wire = witness->wires.at(wire_tag);

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
        // NOTE: In TurboPlonk and UltraPlonk, the witness polynomials are evaluated at 2 points and thus
        // we need to add 3 random scalars in them.
        //
        // We start adding random scalars in `wire` polynomials from index (n - k) upto (n - k + 2).
        // For simplicity, we add 3 random scalars even for standard plonk (recall, just 2 of them are required)
        // since an additional random scalar would not affect things.
        //
        // NOTE: If in future there is a need to cut off more zeros off the vanishing polynomial, this method
        // will not change. This must be changed only if the number of evaluations of witness polynomials
        // change.
        //
        const size_t w_randomness = 3;
        ASSERT(w_randomness < settings::num_roots_cut_out_of_vanishing_polynomial);
        for (size_t k = 0; k < w_randomness; ++k) {
            wire.at(n - settings::num_roots_cut_out_of_vanishing_polynomial + k) = fr::random_element();
        }

        barretenberg::polynomial& wire_fft = key->wire_ffts.at(wire_tag + "_fft");
        barretenberg::polynomial_arithmetic::copy_polynomial(&wire[0], &wire_fft[0], n, n);
        queue.add_to_queue({
            work_queue::WorkType::IFFT,
            nullptr,
            wire_tag,
            barretenberg::fr(0),
            0,
        });
    }
}

/**
 * Execute the first round by computing wire precommitments.
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
    std::cout << "init quotient polys: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    compute_wire_pre_commitments();
    for (auto& widget : random_widgets) {
        widget->compute_round_commitments(transcript, 1, queue);
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire commitments: " << diff.count() << "ms" << std::endl;
#endif
}

/**
 * Execute second round by applying Fiat-Shamir transform on the "eta" challenge
 * and computing random_widgets round commitments that need to be computed at round 2.
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
}
/**
 * Execute third round by applying Fiat-Shamir transform on the "beta" challenge,
 * apply 3rd round random widgets and FFT the wires. For example, standard composer
 * executes permutation widget for z polynomial construction at this round.
 *
 * @tparam settings Program settings.
 * */
template <typename settings> void ProverBase<settings>::execute_third_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("beta");
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute z coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    for (auto& widget : random_widgets) {
        widget->compute_round_commitments(transcript, 3, queue);
    }

    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        queue.add_to_queue({
            work_queue::WorkType::FFT,
            nullptr,
            wire_tag,
            barretenberg::fr(0),
            0,
        });
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute z commitment: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_fourth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("alpha");
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire ffts: " << diff.count() << "ms" << std::endl;
#endif

#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "copy z: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute permutation grand product coeffs: " << diff.count() << "ms" << std::endl;
#endif
    fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    for (auto& widget : random_widgets) {
#ifdef DEBUG_TIMING
        std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
        alpha_base = widget->compute_quotient_contribution(alpha_base, transcript);
#ifdef DEBUG_TIMING
        std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
        std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        std::cout << "widget " << i << " quotient compute time: " << diff.count() << "ms" << std::endl;
#endif
    }
    for (auto& widget : transition_widgets) {
        alpha_base = widget->compute_quotient_contribution(alpha_base, transcript);
    }
    fr* q_mid = &key->quotient_mid[0];
    fr* q_large = &key->quotient_large[0];

#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    if constexpr (settings::uses_quotient_mid) {
        barretenberg::polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(
            key->quotient_mid.get_coefficients(), key->small_domain, key->mid_domain);
    }
    barretenberg::polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(
        key->quotient_large.get_coefficients(), key->small_domain, key->large_domain);
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "divide by vanishing polynomial: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    if (settings::uses_quotient_mid) {
        key->quotient_mid.coset_ifft(key->mid_domain);
    }
    key->quotient_large.coset_ifft(key->large_domain);
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "final inverse fourier transforms: " << diff.count() << "ms" << std::endl;
#endif
    if (settings::uses_quotient_mid) {
        ITERATE_OVER_DOMAIN_START(key->mid_domain);
        q_large[i] += q_mid[i];
        ITERATE_OVER_DOMAIN_END;
    }
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    compute_quotient_pre_commitment();
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute quotient commitment: " << diff.count() << "ms" << std::endl;
#endif
} // namespace waffle

template <typename settings> void ProverBase<settings>::execute_fifth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("z"); // end of 4th round
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
    compute_linearisation_coefficients();
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute linearisation coefficients: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_sixth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("nu");
    commitment_scheme->batch_open(transcript, queue, key, witness);
}

template <typename settings> void ProverBase<settings>::compute_linearisation_coefficients()
{

    fr zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());

    polynomial& r = key->linear_poly;

    commitment_scheme->add_opening_evaluations_to_transcript(transcript, key, witness, false);
    fr t_eval = key->quotient_large.evaluate(zeta, 4 * n);

    if constexpr (settings::use_linearisation) {
        fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

        for (auto& widget : random_widgets) {
            alpha_base = widget->compute_linear_contribution(alpha_base, transcript, r);
        }
        for (auto& widget : transition_widgets) {
            alpha_base = widget->compute_linear_contribution(alpha_base, transcript, &r[0]);
        }
        // The below code adds −Z_H(z) * (t_lo(X) + z^n * t_mid(X) + z^2n * t_hi(X)) term to r(X)
        barretenberg::fr z_pow_n = zeta.pow(key->n);
        barretenberg::fr z_pow_two_n = z_pow_n.sqr();
        // We access Z_H(z) from lagrange_evals
        barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
            barretenberg::polynomial_arithmetic::get_lagrange_evaluations(zeta, key->small_domain);
        ITERATE_OVER_DOMAIN_START(key->small_domain);
        fr quotient_sum = 0, quotient_multiplier = 1;
        for (size_t j = 0; j < settings::program_width; j++) {
            quotient_sum += key->quotient_large[i + key->n * j] * quotient_multiplier;
            quotient_multiplier *= z_pow_n;
        }
        r[i] += -lagrange_evals.vanishing_poly * quotient_sum;
        ITERATE_OVER_DOMAIN_END;
        // For standard Plonk, t_hi(X) has, (n+1) coefficients
        if (settings::program_width == 3) {
            if (r.get_size() < key->n + 1) {
                r.resize(key->n + 1);
            }
            r[key->n] = -key->quotient_large[3 * key->n] * lagrange_evals.vanishing_poly * z_pow_two_n;
        }

        // Assert that r(X) at X = zeta is 0
        const auto size = (settings::program_width == 3) ? key->n + 1 : key->n;
        fr linear_eval = r.evaluate(zeta, size);
        // This condition checks if r(z) = 0 but does not abort.
        if (linear_eval != fr(0)) {
            error("linear_eval is not 0.");
        }
    } else {
        transcript.add_element("t", t_eval.to_buffer());
    }
}

template <typename settings> waffle::plonk_proof& ProverBase<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> waffle::plonk_proof& ProverBase<settings>::construct_proof()
{
    // Execute init round. Randomize witness polynomials.
    execute_preamble_round();
    queue.process_queue();
    // Compute wire precommitments and sometimes random widget round commitments
    execute_first_round();
    queue.process_queue();

    // Fiat-Shamir eta + execute random widgets.
    execute_second_round();
    queue.process_queue();

    // Fiat-Shamir beta, execute random widgets (Permutation widget is executed here)
    // and fft the witnesses
    execute_third_round();
    queue.process_queue();

    execute_fourth_round();
    queue.process_queue();
    execute_fifth_round();
    execute_sixth_round();
    queue.process_queue();
    return export_proof();
}

template <typename settings> void ProverBase<settings>::reset()
{
    transcript::Manifest manifest = transcript.get_manifest();
    transcript = transcript::StandardTranscript(manifest, settings::hash_type, settings::num_challenge_bytes);
}

template class ProverBase<unrolled_standard_settings>;
template class ProverBase<unrolled_turbo_settings>;
template class ProverBase<unrolled_plookup_settings>;
template class ProverBase<standard_settings>;
template class ProverBase<turbo_settings>;
template class ProverBase<plookup_settings>;

} // namespace waffle
