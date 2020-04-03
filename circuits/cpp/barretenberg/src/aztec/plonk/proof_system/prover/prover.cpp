#include "prover.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <chrono>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;

namespace waffle {

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
    if (witness->wires.count("z") == 0) {
        witness->wires.insert({ "z", polynomial(n, n) });
    }
}

template <typename settings>
ProverBase<settings>::ProverBase(ProverBase<settings>&& other)
    : n(other.n)
    , transcript(other.transcript)
    , key(std::move(other.key))
    , witness(std::move(other.witness))
    , queue(key.get(), witness.get(), &transcript)
{
    for (size_t i = 0; i < other.widgets.size(); ++i) {
        widgets.emplace_back(std::move(other.widgets[i]));
    }
}

template <typename settings> ProverBase<settings>& ProverBase<settings>::operator=(ProverBase<settings>&& other)
{
    n = other.n;

    widgets.resize(0);
    for (size_t i = 0; i < other.widgets.size(); ++i) {
        widgets.emplace_back(std::move(other.widgets[i]));
    }

    transcript = other.transcript;
    key = std::move(other.key);
    witness = std::move(other.witness);
    queue = work_queue(key.get(), witness.get(), &transcript);
    return *this;
}

template <typename settings> void ProverBase<settings>::compute_wire_pre_commitments()
{
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            witness->wires.at(wire_tag).get_coefficients(),
            "W_" + std::to_string(i + 1),
            barretenberg::fr(0),
            0,
        });
    }

    // add public inputs
    const polynomial& public_wires_source = key->wire_ffts.at("w_2_fft");
    std::vector<fr> public_wires;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_wires.push_back(public_wires_source[i]);
    }
    transcript.add_element("public_inputs", fr::to_buffer(public_wires));
}

template <typename settings> void ProverBase<settings>::compute_quotient_pre_commitment()
{
    for (size_t i = 0; i < settings::program_width; ++i) {
        const size_t offset = n * i;
        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            &key->quotient_large.get_coefficients()[offset],
            "T_" + std::to_string(i + 1),
            barretenberg::fr(0),
            0,
        });
    }
}

template <typename settings> void ProverBase<settings>::execute_preamble_round()
{
    queue.flush_queue();
    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(n),
                             static_cast<uint8_t>(n >> 8),
                             static_cast<uint8_t>(n >> 16),
                             static_cast<uint8_t>(n >> 24) });
    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 24) });
    transcript.apply_fiat_shamir("init");

    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        barretenberg::polynomial& wire = witness->wires.at(wire_tag);
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
    for (auto& widget : widgets) {
        widget->compute_round_commitments(transcript, 1, queue);
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire commitments: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_second_round()
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
    for (auto& widget : widgets) {
        widget->compute_round_commitments(transcript, 2, queue);
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

template <typename settings> void ProverBase<settings>::execute_third_round()
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
    // fr alpha_base = alpha.sqr().sqr();

    for (size_t i = 0; i < widgets.size(); ++i) {
#ifdef DEBUG_TIMING
        std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
        alpha_base = widgets[i]->compute_quotient_contribution(alpha_base, transcript);
#ifdef DEBUG_TIMING
        std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
        std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        std::cout << "widget " << i << " quotient compute time: " << diff.count() << "ms" << std::endl;
#endif
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
}

template <typename settings> void ProverBase<settings>::execute_fourth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("z"); // end of 3rd round
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

template <typename settings> void ProverBase<settings>::execute_fifth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("nu");
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
    std::vector<fr> nu_challenges;
    for (size_t i = 0; i < transcript.get_num_challenges("nu"); ++i) {
        nu_challenges.emplace_back(fr::serialize_from_buffer(transcript.get_challenge("nu", i).begin()));
    }
    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr* r = key->linear_poly.get_coefficients();

    std::array<fr*, settings::program_width> wires;
    for (size_t i = 0; i < settings::program_width; ++i) {
        wires[i] = &witness->wires.at("w_" + std::to_string(i + 1))[0];
    }

    // Next step: compute the two Kate polynomial commitments, and associated opening proofs
    // We have two evaluation points: z and z.omega
    // We need to create random linear combinations of each individual polynomial and combine them

    polynomial& opening_poly = key->opening_poly;
    polynomial& shifted_opening_poly = key->shifted_opening_poly;

    std::array<fr, settings::program_width> z_powers;
    z_powers[0] = z_challenge;
    for (size_t i = 1; i < settings::program_width; ++i) {
        z_powers[i] = z_challenge.pow(static_cast<uint64_t>(n * i));
    }

    polynomial& quotient_large = key->quotient_large;

    constexpr size_t nu_offset = (settings::use_linearisation ? 1 : 0);
    constexpr size_t nu_z_offset =
        (settings::use_linearisation) ? 2 * settings::program_width : 2 * settings::program_width + 1;

    ITERATE_OVER_DOMAIN_START(key->small_domain);

    fr T0;
    fr quotient_temp = fr::zero();
    if constexpr (settings::use_linearisation) {
        quotient_temp = r[i] * nu_challenges[0];
    }
    for (size_t k = 1; k < settings::program_width; ++k) {
        T0 = quotient_large[i + (k * n)] * z_powers[k];
        quotient_temp += T0;
    }
    for (size_t k = 0; k < settings::program_width; ++k) {
        T0 = wires[k][i] * nu_challenges[k + nu_offset];
        quotient_temp += T0;
    }
    shifted_opening_poly[i] = 0;

    opening_poly[i] = quotient_large[i] + quotient_temp;

    ITERATE_OVER_DOMAIN_END;

    constexpr size_t shifted_nu_offset = nu_z_offset + 1;
    if constexpr (settings::wire_shift_settings > 0) {
        ITERATE_OVER_DOMAIN_START(key->small_domain);
        size_t nu_ptr = shifted_nu_offset;
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 0)) {
            fr T0;
            T0 = nu_challenges[nu_ptr++] * wires[0][i];
            shifted_opening_poly[i] += T0;
        }
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 1)) {
            fr T0;
            T0 = nu_challenges[nu_ptr++] * wires[1][i];
            shifted_opening_poly[i] += T0;
        }
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 2)) {
            fr T0;
            T0 = nu_challenges[nu_ptr++] * wires[2][i];
            shifted_opening_poly[i] += T0;
        }
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 3)) {
            fr T0;
            T0 = nu_challenges[nu_ptr++] * wires[3][i];
            shifted_opening_poly[i] += T0;
        }
        for (size_t k = 4; k < settings::program_width; ++k) {
            if (settings::requires_shifted_wire(settings::wire_shift_settings, k)) {
                fr T0;
                T0 = nu_challenges[nu_ptr++] * wires[k][i];
                shifted_opening_poly[i] += T0;
            }
        }
        ITERATE_OVER_DOMAIN_END;
    }

    size_t nu_ptr = settings::program_width + nu_offset;

#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute base opening poly contribution: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    // Currently code assumes permutation_widget is first.
    for (size_t i = 0; i < widgets.size(); ++i) {
        nu_ptr = widgets[i]->compute_opening_poly_contribution(
            nu_ptr, transcript, &opening_poly[0], &shifted_opening_poly[0], settings::use_linearisation);
        if (i == 0) {
            for (size_t i = 0; i < settings::program_width; ++i) {
                if (settings::requires_shifted_wire(settings::wire_shift_settings, i)) {
                    ++nu_ptr;
                }
            }
        }
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute widget opening poly contributions: " << diff.count() << "ms" << std::endl;
#endif
    fr shifted_z;
    shifted_z = z_challenge * key->small_domain.root;
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    opening_poly.compute_kate_opening_coefficients(z_challenge);

    shifted_opening_poly.compute_kate_opening_coefficients(shifted_z);
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute kate opening poly coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif

    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        opening_poly.get_coefficients(),
        "PI_Z",
        barretenberg::fr(0),
        0,
    });
    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        shifted_opening_poly.get_coefficients(),
        "PI_Z_OMEGA",
        barretenberg::fr(0),
        0,
    });
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute opening commitment: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> barretenberg::fr ProverBase<settings>::compute_linearisation_coefficients()
{

    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr shifted_z = z_challenge * key->small_domain.root;

    polynomial& r = key->linear_poly;
    // ok... now we need to evaluate polynomials. Jeepers

    // evaluate the prover and instance polynomials.
    // (we don't need to evaluate the quotient polynomial, that can be derived by the verifier)
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_key = "w_" + std::to_string(i + 1);
        const polynomial& wire = witness->wires.at(wire_key);
        fr wire_eval;
        wire_eval = wire.evaluate(z_challenge, n);
        transcript.add_element(wire_key, wire_eval.to_buffer());

        if (settings::requires_shifted_wire(settings::wire_shift_settings, i)) {
            fr shifted_wire_eval;
            shifted_wire_eval = wire.evaluate(shifted_z, n);
            transcript.add_element(wire_key + "_omega", shifted_wire_eval.to_buffer());
        }
    }

    for (size_t i = 0; i < widgets.size(); ++i) {
        widgets[i]->compute_transcript_elements(transcript, settings::use_linearisation);
    }

    fr t_eval = key->quotient_large.evaluate(z_challenge, 4 * n);

    if constexpr (settings::use_linearisation) {
        fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
        for (size_t i = 0; i < widgets.size(); ++i) {
            alpha_base = widgets[i]->compute_linear_contribution(alpha_base, transcript, r);
        }

        fr linear_eval = r.evaluate(z_challenge, n);
        transcript.add_element("r", linear_eval.to_buffer());
    }
    transcript.add_element("t", t_eval.to_buffer());

    return t_eval;
}

template <typename settings> waffle::plonk_proof ProverBase<settings>::export_proof()
{
    waffle::plonk_proof result;
    result.proof_data = transcript.export_transcript();
    return result;
}

template <typename settings> waffle::plonk_proof ProverBase<settings>::construct_proof()
{
    execute_preamble_round();
    queue.process_queue();
    execute_first_round();
    queue.process_queue();
    execute_second_round();
    queue.process_queue();
    execute_third_round();
    queue.process_queue();
    execute_fourth_round();
    execute_fifth_round();
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
template class ProverBase<standard_settings>;
template class ProverBase<turbo_settings>;

} // namespace waffle
