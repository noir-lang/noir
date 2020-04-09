#pragma once

#include "../proving_key/proving_key.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverMiMCWidget::ProverMiMCWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_mimc_selector(key->constraint_selectors.at("q_mimc_selector"))
    , q_mimc_coefficient(key->constraint_selectors.at("q_mimc_coefficient"))
    , q_mimc_selector_fft(key->constraint_selector_ffts.at("q_mimc_selector_fft"))
    , q_mimc_coefficient_fft(key->constraint_selector_ffts.at("q_mimc_coefficient_fft"))
{}

ProverMiMCWidget::ProverMiMCWidget(const ProverMiMCWidget& other)
    : ProverBaseWidget(other)
    , q_mimc_selector(key->constraint_selectors.at("q_mimc_selector"))
    , q_mimc_coefficient(key->constraint_selectors.at("q_mimc_coefficient"))
    , q_mimc_selector_fft(key->constraint_selector_ffts.at("q_mimc_selector_fft"))
    , q_mimc_coefficient_fft(key->constraint_selector_ffts.at("q_mimc_coefficient_fft"))
{}

ProverMiMCWidget::ProverMiMCWidget(ProverMiMCWidget&& other)
    : ProverBaseWidget(other)
    , q_mimc_selector(key->constraint_selectors.at("q_mimc_selector"))
    , q_mimc_coefficient(key->constraint_selectors.at("q_mimc_coefficient"))
    , q_mimc_selector_fft(key->constraint_selector_ffts.at("q_mimc_selector_fft"))
    , q_mimc_coefficient_fft(key->constraint_selector_ffts.at("q_mimc_coefficient_fft"))
{}

ProverMiMCWidget& ProverMiMCWidget::operator=(const ProverMiMCWidget& other)
{
    ProverBaseWidget::operator=(other);

    q_mimc_selector = key->constraint_selectors.at("q_mimc_selector");
    q_mimc_coefficient = key->constraint_selectors.at("q_mimc_coefficient");

    q_mimc_selector_fft = key->constraint_selector_ffts.at("q_mimc_selector_fft");
    q_mimc_coefficient_fft = key->constraint_selector_ffts.at("q_mimc_coefficient_fft");
    return *this;
}

ProverMiMCWidget& ProverMiMCWidget::operator=(ProverMiMCWidget&& other)
{
    ProverBaseWidget::operator=(other);

    q_mimc_selector = key->constraint_selectors.at("q_mimc_selector");
    q_mimc_coefficient = key->constraint_selectors.at("q_mimc_coefficient");

    q_mimc_selector_fft = key->constraint_selector_ffts.at("q_mimc_selector_fft");
    q_mimc_coefficient_fft = key->constraint_selector_ffts.at("q_mimc_coefficient_fft");
    return *this;
}

fr ProverMiMCWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    polynomial& w_1_fft = key->wire_ffts.at("w_1_fft");
    polynomial& w_2_fft = key->wire_ffts.at("w_2_fft");
    polynomial& w_3_fft = key->wire_ffts.at("w_3_fft");

    polynomial& quotient_large = key->quotient_large;

    ITERATE_OVER_DOMAIN_START(key->large_domain);
    fr T0 = (w_3_fft[i] + w_1_fft[i] + q_mimc_coefficient_fft[i]);
    fr T1 = (T0.sqr() * T0) - w_2_fft[i];
    fr T2 = (w_2_fft[i].sqr() * T0 - w_3_fft[i + 4]) * alpha;
    fr T3 = (T1 + T2) * q_mimc_selector_fft[i] * alpha_base;
    quotient_large[i] += T3;
    ITERATE_OVER_DOMAIN_END;

    return alpha_base * alpha.sqr();
}

void ProverMiMCWidget::compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation)
{
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_mimc_coefficient", q_mimc_coefficient.evaluate(z, key->small_domain.size).to_buffer());
    if (!use_linearisation) {
        transcript.add_element("q_mimc_selector", q_mimc_selector.evaluate(z, key->small_domain.size).to_buffer());
    }
}

fr ProverMiMCWidget::compute_linear_contribution(const fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 polynomial& r)
{
    fr alpha = fr::serialize_from_buffer(&transcript.get_challenge("alpha")[0]);
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_o_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);
    fr q_mimc_coefficient_eval = fr::serialize_from_buffer(&transcript.get_element("q_mimc_coefficient")[0]);

    fr mimc_T0 = w_l_eval + w_o_eval + q_mimc_coefficient_eval;
    fr mimc_a = (mimc_T0.sqr() * mimc_T0) - w_r_eval;
    fr mimc_term = ((w_r_eval.sqr() * mimc_T0 - w_o_shifted_eval) * alpha + mimc_a) * alpha_base;

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] += (mimc_term * q_mimc_selector[i]);
    ITERATE_OVER_DOMAIN_END;
    return alpha_base * alpha.sqr();
}

void ProverMiMCWidget::compute_opening_poly_contribution(const transcript::Transcript& transcript,
                                                         const bool use_linearisation)
{
    polynomial& poly = key->opening_poly;

    if (use_linearisation) {

        fr nu_base = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_mimc_coefficient")[0]);

        ITERATE_OVER_DOMAIN_START(key->small_domain);
        poly[i] += (q_mimc_coefficient[i] * nu_base);
        ITERATE_OVER_DOMAIN_END;

        return;
    }

    std::array<barretenberg::fr, 2> nu_powers;
    nu_powers[0] = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_mimc_coefficient")[0]);
    nu_powers[1] = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_mimc_selector")[0]);
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_mimc_coefficient[i] * nu_powers[0]);
    poly[i] += (q_mimc_selector[i] * nu_powers[1]);
    ITERATE_OVER_DOMAIN_END;
}

// ###

template <typename Field, typename Group, typename Transcript>
VerifierMiMCWidget<Field, Group, Transcript>::VerifierMiMCWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierMiMCWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*,
    const Field& alpha_base,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{
    const Field alpha = transcript.get_challenge_field_element("alpha");

    if (use_linearisation) {
        return alpha_base * alpha.sqr();
    }

    Field w_l_eval = transcript.get_field_element("w_1");
    Field w_r_eval = transcript.get_field_element("w_2");
    Field w_o_eval = transcript.get_field_element("w_3");
    Field w_o_next_eval = transcript.get_field_element("w_3_omega");

    Field q_mimc_coefficient_eval = transcript.get_field_element("q_mimc_coefficient");
    Field q_mimc_selector_eval = transcript.get_field_element("q_mimc_selector");

    Field T0 = (w_o_eval + w_l_eval + q_mimc_coefficient_eval);
    Field T1 = (T0.sqr() * T0) - w_r_eval;
    Field T2 = (w_r_eval.sqr() * T0 - w_o_next_eval) * alpha;
    Field T3 = (T1 + T2) * q_mimc_selector_eval * alpha_base;
    t_eval += T3;

    return alpha_base * alpha.sqr();
}

template <typename Field, typename Group, typename Transcript>
void VerifierMiMCWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(verification_key*,
                                                                                         Field& batch_eval,
                                                                                         const Transcript& transcript,
                                                                                         const bool use_linearisation)
{
    Field q_mimc_coefficient_eval = transcript.get_field_element("q_mimc_coefficient");

    if (use_linearisation) {
        fr nu_base = transcript.get_challenge_field_element_from_map("nu", "q_mimc_coefficient");
        batch_eval += (q_mimc_coefficient_eval * nu_base);
        return;
    }

    Field q_mimc_selector_eval = transcript.get_field_element("q_mimc_selector");

    std::array<Field, 5> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element_from_map("nu", "q_mimc_coefficient");
    nu_challenges[1] = transcript.get_challenge_field_element_from_map("nu", "q_mimc_selector");

    batch_eval += (q_mimc_coefficient_eval * nu_challenges[0]);
    batch_eval += (q_mimc_selector_eval * nu_challenges[1]);

    return;
}

template <typename Field, typename Group, typename Transcript>
Field VerifierMiMCWidget<Field, Group, Transcript>::append_scalar_multiplication_inputs(verification_key* key,
                                                                                        const Field& alpha_base,
                                                                                        const Transcript& transcript,
                                                                                        std::vector<Group>& points,
                                                                                        std::vector<Field>& scalars,
                                                                                        const bool use_linearisation)
{
    Field alpha_step = transcript.get_challenge_field_element("alpha");

    if (use_linearisation) {
        Field nu_base = transcript.get_challenge_field_element_from_map("nu", "q_mimc_coefficient");

        if (key->constraint_selectors.at("Q_MIMC_COEFFICIENT").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_MIMC_COEFFICIENT"));
            scalars.push_back(nu_base);
        }
        Field w_l_eval = transcript.get_field_element("w_1");
        Field w_r_eval = transcript.get_field_element("w_2");
        Field w_o_eval = transcript.get_field_element("w_3");
        Field w_o_shifted_eval = transcript.get_field_element("w_3_omega");
        Field q_mimc_coefficient_eval = transcript.get_field_element("q_mimc_coefficient");

        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");

        Field mimc_T0 = w_l_eval + w_o_eval + q_mimc_coefficient_eval;
        Field mimc_a = (mimc_T0.sqr() * mimc_T0) - w_r_eval;
        Field q_mimc_term = ((w_r_eval.sqr() * mimc_T0 - w_o_shifted_eval) * alpha_step + mimc_a) * alpha_base;
        q_mimc_term = q_mimc_term * linear_nu;

        if (key->constraint_selectors.at("Q_MIMC_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_MIMC_SELECTOR"));
            scalars.push_back(q_mimc_term);
        }

        return alpha_base * alpha_step.sqr();
    }

    std::array<Field, 5> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element_from_map("nu", "q_mimc_coefficient");
    nu_challenges[1] = transcript.get_challenge_field_element_from_map("nu", "q_mimc_selector");

    if (key->constraint_selectors.at("Q_MIMC_COEFFICIENT").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_MIMC_COEFFICIENT"));
        scalars.push_back(nu_challenges[0]);
    }
    if (key->constraint_selectors.at("Q_MIMC_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_MIMC_SELECTOR"));
        scalars.push_back(nu_challenges[1]);
    }

    return alpha_base * alpha_step.sqr();
}

template class VerifierMiMCWidget<barretenberg::fr, barretenberg::g1::affine_element, transcript::StandardTranscript>;

} // namespace waffle