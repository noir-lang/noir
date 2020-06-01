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
    for (auto& widget : random_widgets) {
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
    transcript.apply_fiat_shamir("eta");
    for (auto& widget : random_widgets) {
        widget->compute_round_commitments(transcript, 2, queue);
    }
}

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
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif

    compute_batch_opening_polynomials();
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute base opening poly contribution: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif

#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute widget opening poly contributions: " << diff.count() << "ms" << std::endl;
#endif
    const auto zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    const auto zeta_omega = zeta * key->small_domain.root;
    polynomial& opening_poly = key->opening_poly;
    polynomial& shifted_opening_poly = key->shifted_opening_poly;

#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    opening_poly.compute_kate_opening_coefficients(zeta);

    shifted_opening_poly.compute_kate_opening_coefficients(zeta_omega);
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

template <typename settings> void ProverBase<settings>::compute_batch_opening_polynomials()
{
    std::vector<std::pair<fr*, fr>> opened_polynomials_at_zeta;
    std::vector<std::pair<fr*, fr>> opened_polynomials_at_zeta_omega;

    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& info = key->polynomial_manifest[i];
        const std::string poly_label(info.polynomial_label);
        fr* poly = nullptr;
        switch (info.source) {
        case PolynomialSource::WITNESS: {
            poly = &witness->wires.at(poly_label)[0];
            break;
        }
        case PolynomialSource::SELECTOR: {
            poly = &key->constraint_selectors.at(poly_label)[0];
            break;
        }
        case PolynomialSource::PERMUTATION: {
            poly = &key->permutation_selectors.at(poly_label)[0];
            break;
        }
        }
        if (!info.is_linearised || !settings::use_linearisation) {
            const fr nu_challenge = transcript.get_challenge_field_element_from_map("nu", poly_label);
            opened_polynomials_at_zeta.push_back({ poly, nu_challenge });
        }
        if (info.requires_shifted_evaluation) {
            const auto nu_challenge = transcript.get_challenge_field_element_from_map("nu", poly_label + "_omega");
            opened_polynomials_at_zeta_omega.push_back({ poly, nu_challenge });
        }
    }

    const auto zeta = transcript.get_challenge_field_element("z");

    for (size_t i = 1; i < settings::program_width; ++i) {
        const size_t offset = i * key->small_domain.size;
        const fr scalar = zeta.pow(static_cast<uint64_t>(offset));
        opened_polynomials_at_zeta.push_back({ &key->quotient_large[offset], scalar });
    }

    if constexpr (settings::use_linearisation) {
        const fr linear_challenge = transcript.get_challenge_field_element_from_map("nu", "r");
        opened_polynomials_at_zeta.push_back({ &key->linear_poly[0], linear_challenge });
    }

    polynomial& opening_poly = key->opening_poly;
    polynomial& shifted_opening_poly = key->shifted_opening_poly;

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    opening_poly[i] = key->quotient_large[i];
    for (const auto& [poly, challenge] : opened_polynomials_at_zeta) {
        opening_poly[i] += poly[i] * challenge;
    }
    shifted_opening_poly[i] = 0;
    for (const auto& [poly, challenge] : opened_polynomials_at_zeta_omega) {
        shifted_opening_poly[i] += poly[i] * challenge;
    }
    ITERATE_OVER_DOMAIN_END;
}

template <typename settings> void ProverBase<settings>::add_polynomial_evaluations_to_transcript()
{
    fr zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr shifted_z = zeta * key->small_domain.root;

    const auto get_polynomial = [key = this->key, witness = this->witness](const auto& poly_info,
                                                                           const auto& poly_label) -> polynomial& {
        switch (poly_info.source) {
        case PolynomialSource::WITNESS: {
            return witness->wires.at(poly_label);
        }
        case PolynomialSource::SELECTOR: {
            return key->constraint_selectors.at(poly_label);
        }
        case PolynomialSource::PERMUTATION: {
            return key->permutation_selectors.at(poly_label);
        }
        default: {
            barretenberg::errors::throw_or_abort("invalid polynomial source");
            return witness->wires.at("w_1");
        }
        }
    };
    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& info = key->polynomial_manifest[i];
        std::string label(info.polynomial_label);
        polynomial& poly = get_polynomial(info, label);

        if (!info.is_linearised || !settings::use_linearisation) {
            transcript.add_element(label, poly.evaluate(zeta, key->small_domain.size).to_buffer());
        }
        if (info.requires_shifted_evaluation) {
            transcript.add_element(label + "_omega", poly.evaluate(shifted_z, key->small_domain.size).to_buffer());
        }
    }
}

template <typename settings> barretenberg::fr ProverBase<settings>::compute_linearisation_coefficients()
{

    fr zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());

    polynomial& r = key->linear_poly;

    add_polynomial_evaluations_to_transcript();
    fr t_eval = key->quotient_large.evaluate(zeta, 4 * n);

    if constexpr (settings::use_linearisation) {
        fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

        for (auto& widget : random_widgets) {
            alpha_base = widget->compute_linear_contribution(alpha_base, transcript, r);
        }
        for (auto& widget : transition_widgets) {
            alpha_base = widget->compute_linear_contribution(alpha_base, transcript, &r[0]);
        }
        fr linear_eval = r.evaluate(zeta, n);
        transcript.add_element("r", linear_eval.to_buffer());
    }
    transcript.add_element("t", t_eval.to_buffer());
    return t_eval;
}

template <typename settings> waffle::plonk_proof& ProverBase<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> waffle::plonk_proof& ProverBase<settings>::construct_proof()
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
