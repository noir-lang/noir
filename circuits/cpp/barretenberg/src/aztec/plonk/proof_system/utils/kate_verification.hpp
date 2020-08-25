#pragma once

#include <map>

#include "../verification_key/verification_key.hpp"
namespace waffle {

template <typename Field, typename Transcript, typename program_settings>
Field compute_kate_batch_evaluation(typename Transcript::Key* key, const Transcript& transcript)
{
    Field batch_eval(0);

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

template <typename Field, typename Group, typename Transcript, typename program_settings>
void populate_kate_element_map(verification_key* key,
                               const Transcript& transcript,
                               std::map<std::string, Group>& kate_g1_elements,
                               std::map<std::string, Field>& kate_fr_elements)
{
    const auto separator_challenge = transcript.get_challenge_field_element("separator", 0);

    const auto& polynomial_manifest = key->polynomial_manifest;
    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& item = polynomial_manifest[i];
        const std::string label(item.commitment_label);
        const std::string poly_label(item.polynomial_label);
        switch (item.source) {
        case PolynomialSource::WITNESS: {
            const auto element = transcript.get_group_element(label);
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
        Field kate_fr_scalar(0);
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

    Field z_pow_n = zeta;
    const size_t log2_n = numeric::get_msb(key->n);
    for (size_t j = 0; j < log2_n; ++j) {
        z_pow_n = z_pow_n.sqr();
    }
    Field z_power = 1;
    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string quotient_label = "T_" + std::to_string(i + 1);
        const auto element = transcript.get_group_element(quotient_label);

        kate_g1_elements.insert({ quotient_label, element });
        kate_fr_elements.insert({ quotient_label, quotient_nu * z_power });
        z_power *= z_pow_n;
    }
}
} // namespace waffle