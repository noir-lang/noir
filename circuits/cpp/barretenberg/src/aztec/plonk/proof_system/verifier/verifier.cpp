#include "./verifier.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;

namespace waffle {
template <typename program_settings>
VerifierBase<program_settings>::VerifierBase(std::shared_ptr<verification_key> verifier_key,
                                             const transcript::Manifest& input_manifest)
    : manifest(input_manifest)
    , key(verifier_key)
{}

template <typename program_settings>
VerifierBase<program_settings>::VerifierBase(VerifierBase&& other)
    : manifest(other.manifest)
    , key(other.key)
{}

template <typename program_settings>
VerifierBase<program_settings>& VerifierBase<program_settings>::operator=(VerifierBase&& other)
{
    key = other.key;
    manifest = other.manifest;
    kate_g1_elements.clear();
    kate_fr_elements.clear();
    return *this;
}

template <typename program_settings>
barretenberg::fr VerifierBase<program_settings>::compute_non_linear_kate_batch_evaluation(
    const transcript::StandardTranscript& transcript)
{
    barretenberg::fr batch_eval(0);

    const auto separator_challenge = transcript.get_challenge_field_element("separator", 0);
    const auto& polynomial_manifest = key->polynomial_manifest;
    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& item = polynomial_manifest[i];

        if ((item.is_linearised && program_settings::use_linearisation) && !item.requires_shifted_evaluation) {
            continue;
        }

        const std::string poly_label(item.polynomial_label);

        bool has_evaluation = !item.is_linearised || !program_settings::use_linearisation;
        bool has_shifted_evaluation = item.requires_shifted_evaluation;

        if (has_evaluation) {
            const auto nu_challenge = transcript.get_challenge_field_element_from_map("nu", poly_label);
            const auto poly_at_zeta = transcript.get_field_element(poly_label);
            batch_eval += nu_challenge * poly_at_zeta;
        }
        if (has_shifted_evaluation) {
            const auto nu_challenge = transcript.get_challenge_field_element_from_map("nu", poly_label + "_omega");
            const auto poly_at_zeta_omega = transcript.get_field_element(poly_label + "_omega");
            batch_eval += separator_challenge * nu_challenge * poly_at_zeta_omega;
        }
    }

    if constexpr (program_settings::use_linearisation) {
        const auto linear_eval = transcript.get_field_element("r");
        const auto linear_challenge = transcript.get_challenge_field_element_from_map("nu", "r");
        batch_eval += (linear_challenge * linear_eval);
    }

    const auto quotient_eval = transcript.get_field_element("t");
    const auto quotient_challenge = transcript.get_challenge_field_element_from_map("nu", "t");

    batch_eval += (quotient_eval * quotient_challenge);
    return batch_eval;
}

template <typename program_settings>
void VerifierBase<program_settings>::populate_kate_element_map(const transcript::StandardTranscript& transcript)
{
    const auto separator_challenge = transcript.get_challenge_field_element("separator", 0);

    const auto& polynomial_manifest = key->polynomial_manifest;
    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& item = polynomial_manifest[i];
        const std::string label(item.commitment_label);
        const std::string poly_label(item.polynomial_label);
        switch (item.source) {
        case PolynomialSource::WITNESS: {
            const auto element = g1::affine_element::serialize_from_buffer(&transcript.get_element(label)[0]);
            ASSERT(element.on_curve());
            kate_g1_elements.insert({ label, element });
            break;
        }
        case PolynomialSource::SELECTOR: {
            const auto element = key->constraint_selectors.at(label);
            kate_g1_elements.insert({ label, element });
            break;
        }
        case PolynomialSource::PERMUTATION: {
            const auto element = key->permutation_selectors.at(label);
            kate_g1_elements.insert({ label, element });
            break;
        }
        }
        barretenberg::fr kate_fr_scalar(0);
        if (item.requires_shifted_evaluation) {
            const auto challenge = transcript.get_challenge_field_element_from_map("nu", poly_label + "_omega");
            kate_fr_scalar += (separator_challenge * challenge);
        }
        if (!item.is_linearised || !program_settings::use_linearisation) {
            const auto challenge = transcript.get_challenge_field_element_from_map("nu", poly_label);
            kate_fr_scalar += challenge;
        }
        kate_fr_elements.insert({ label, kate_fr_scalar });
    }

    const auto zeta = transcript.get_challenge_field_element("z", 0);
    const auto quotient_nu = transcript.get_challenge_field_element_from_map("nu", "t");

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string quotient_label = "T_" + std::to_string(i + 1);
        const auto element = g1::affine_element::serialize_from_buffer(&transcript.get_element(quotient_label)[0]);
        const auto scalar = quotient_nu * zeta.pow(static_cast<uint64_t>(i * key->domain.size));

        kate_g1_elements.insert({ quotient_label, element });
        kate_fr_elements.insert({ quotient_label, scalar });
    }
}

template <typename program_settings> bool VerifierBase<program_settings>::validate_commitments()
{
    // TODO
    return true;
}

template <typename program_settings> bool VerifierBase<program_settings>::validate_scalars()
{
    // TODO
    return true;
}

template <typename program_settings> bool VerifierBase<program_settings>::verify_proof(const waffle::plonk_proof& proof)
{
    key->program_width = program_settings::program_width;
    transcript::StandardTranscript transcript = transcript::StandardTranscript(
        proof.proof_data, manifest, program_settings::hash_type, program_settings::num_challenge_bytes);
    g1::affine_element PI_Z = g1::affine_element::serialize_from_buffer(&transcript.get_element("PI_Z")[0]);
    g1::affine_element PI_Z_OMEGA = g1::affine_element::serialize_from_buffer(&transcript.get_element("PI_Z_OMEGA")[0]);

    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(key->n >> 24),
                             static_cast<uint8_t>(key->n >> 16),
                             static_cast<uint8_t>(key->n >> 8),
                             static_cast<uint8_t>(key->n) });
    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs >> 24),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs) });
    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("eta");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    transcript.apply_fiat_shamir("z");

    const auto alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    const auto zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    const auto lagrange_evals = barretenberg::polynomial_arithmetic::get_lagrange_evaluations(zeta, key->domain);

    fr t_eval(0);

    program_settings::compute_quotient_evaluation_contribution(key.get(), alpha, transcript, t_eval);
    t_eval *= lagrange_evals.vanishing_poly.invert();
    transcript.add_element("t", t_eval.to_buffer());

    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("separator");
    const auto separator_challenge = fr::serialize_from_buffer(transcript.get_challenge("separator").begin());

    fr batch_evaluation = compute_non_linear_kate_batch_evaluation(transcript);

    // program_settings::compute_batch_evaluation_contribution(key.get(), batch_evaluation, transcript);

    kate_g1_elements.insert({ "BATCH_EVALUATION", g1::affine_one });
    kate_fr_elements.insert({ "BATCH_EVALUATION", -batch_evaluation });

    kate_g1_elements.insert({ "PI_Z_OMEGA", PI_Z_OMEGA });
    kate_fr_elements.insert({ "PI_Z_OMEGA", zeta * key->domain.root * separator_challenge });

    kate_g1_elements.insert({ "PI_Z", PI_Z });
    kate_fr_elements.insert({ "PI_Z", zeta });

    populate_kate_element_map(transcript);

    program_settings::append_scalar_multiplication_inputs(key.get(), alpha, transcript, kate_fr_elements);

    validate_commitments();
    validate_scalars();

    std::vector<fr> scalars;
    std::vector<g1::affine_element> elements;

    for (const auto& [key, value] : kate_g1_elements) {
        if (value.on_curve()) {
            scalars.emplace_back(kate_fr_elements.at(key));
            elements.emplace_back(value);
        }
    }

    size_t num_elements = elements.size();
    elements.resize(num_elements * 2);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(&elements[0], &elements[0], num_elements);
    scalar_multiplication::pippenger_runtime_state state(num_elements);

    g1::element P[2];

    P[0] = barretenberg::scalar_multiplication::pippenger(&scalars[0], &elements[0], num_elements, state);
    P[1] = -(g1::element(PI_Z_OMEGA) * separator_challenge + PI_Z);
    g1::element::batch_normalize(P, 2);

    g1::affine_element P_affine[2]{
        { P[0].x, P[0].y },
        { P[1].x, P[1].y },
    };

    barretenberg::fq12 result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
        P_affine, key->reference_string->get_precomputed_g2_lines(), 2);

    return (result == barretenberg::fq12::one());
}

template class VerifierBase<unrolled_standard_verifier_settings>;
template class VerifierBase<unrolled_turbo_verifier_settings>;
template class VerifierBase<unrolled_plookup_verifier_settings>;
template class VerifierBase<standard_verifier_settings>;
template class VerifierBase<mimc_verifier_settings>;
template class VerifierBase<turbo_verifier_settings>;
template class VerifierBase<plookup_verifier_settings>;

} // namespace waffle