#include "mimc_widget.hpp"
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

fr ProverMiMCWidget::compute_opening_poly_contribution(
    const fr& nu_base, const transcript::Transcript& transcript, fr* poly, fr*, const bool use_linearisation)
{
    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    if (use_linearisation) {
        ITERATE_OVER_DOMAIN_START(key->small_domain);
        poly[i] += (q_mimc_coefficient[i] * nu_base);
        ITERATE_OVER_DOMAIN_END;

        return nu_base * nu;
    }

    std::array<barretenberg::fr, 2> nu_powers;
    nu_powers[0] = nu_base;
    nu_powers[1] = nu_powers[0] * nu;
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_mimc_coefficient[i] * nu_powers[0]);
    poly[i] += (q_mimc_selector[i] * nu_powers[1]);
    ITERATE_OVER_DOMAIN_END;

    return nu_powers[1] * nu;
}

// ###

VerifierMiMCWidget::VerifierMiMCWidget()
    : VerifierBaseWidget()
{}

fr VerifierMiMCWidget::compute_quotient_evaluation_contribution(verification_key*,
                                                                const fr& alpha_base,
                                                                const transcript::Transcript& transcript,
                                                                fr& t_eval,
                                                                const bool use_linearisation)
{
    const fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    if (use_linearisation) {
        return alpha_base * alpha.sqr();
    }

    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_o_next_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);

    fr q_mimc_coefficient_eval = fr::serialize_from_buffer(&transcript.get_element("q_mimc_coefficient")[0]);
    fr q_mimc_selector_eval = fr::serialize_from_buffer(&transcript.get_element("q_mimc_selector")[0]);

    fr T0 = (w_o_eval + w_l_eval + q_mimc_coefficient_eval);
    fr T1 = (T0.sqr() * T0) - w_r_eval;
    fr T2 = (w_r_eval.sqr() * T0 - w_o_next_eval) * alpha;
    fr T3 = (T1 + T2) * q_mimc_selector_eval * alpha_base;
    t_eval += T3;

    return alpha_base * alpha.sqr();
}

barretenberg::fr VerifierMiMCWidget::compute_batch_evaluation_contribution(verification_key*,
                                                                           barretenberg::fr& batch_eval,
                                                                           const barretenberg::fr& nu_base,
                                                                           const transcript::Transcript& transcript,
                                                                           const bool use_linearisation)
{
    fr q_mimc_coefficient_eval = fr::serialize_from_buffer(&transcript.get_element("q_mimc_coefficient")[0]);
    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    if (use_linearisation) {
        batch_eval += (q_mimc_coefficient_eval * nu_base);

        return nu_base * nu;
    }

    fr q_mimc_selector_eval = fr::serialize_from_buffer(&transcript.get_element("q_mimc_selector")[0]);

    std::array<barretenberg::fr, 2> nu_powers;
    nu_powers[0] = nu_base;
    nu_powers[1] = nu_powers[0] * nu;
    batch_eval += (q_mimc_coefficient_eval * nu_powers[0]);
    batch_eval += (q_mimc_selector_eval * nu_powers[1]);

    return nu_powers[1] * nu;
}

VerifierBaseWidget::challenge_coefficients VerifierMiMCWidget::append_scalar_multiplication_inputs(
    verification_key* key,
    const VerifierBaseWidget::challenge_coefficients& challenge,
    const transcript::Transcript& transcript,
    std::vector<barretenberg::g1::affine_element>& points,
    std::vector<barretenberg::fr>& scalars,
    const bool use_linearisation)
{
    if (use_linearisation) {
        if (key->constraint_selectors.at("Q_MIMC_COEFFICIENT").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_MIMC_COEFFICIENT"));
            scalars.push_back(challenge.nu_base);
        }
        fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
        fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
        fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
        fr w_o_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);
        fr q_mimc_coefficient_eval = fr::serialize_from_buffer(&transcript.get_element("q_mimc_coefficient")[0]);

        fr mimc_T0 = w_l_eval + w_o_eval + q_mimc_coefficient_eval;
        fr mimc_a = (mimc_T0.sqr() * mimc_T0) - w_r_eval;
        fr q_mimc_term =
            ((w_r_eval.sqr() * mimc_T0 - w_o_shifted_eval) * challenge.alpha_step + mimc_a) * challenge.alpha_base;
        q_mimc_term = q_mimc_term * challenge.linear_nu;

        if (key->constraint_selectors.at("Q_MIMC_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_MIMC_SELECTOR"));
            scalars.push_back(q_mimc_term);
        }

        return VerifierBaseWidget::challenge_coefficients{ challenge.alpha_base * challenge.alpha_step.sqr(),
                                                           challenge.alpha_step,
                                                           challenge.nu_base * challenge.nu_step,
                                                           challenge.nu_step,
                                                           challenge.linear_nu };
    }

    std::array<barretenberg::fr, 2> nu_powers;
    nu_powers[0] = challenge.nu_base;
    nu_powers[1] = nu_powers[0] * challenge.nu_step;

    if (key->constraint_selectors.at("Q_MIMC_COEFFICIENT").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_MIMC_COEFFICIENT"));
        scalars.push_back(nu_powers[0]);
    }
    if (key->constraint_selectors.at("Q_MIMC_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_MIMC_SELECTOR"));
        scalars.push_back(nu_powers[1]);
    }

    return VerifierBaseWidget::challenge_coefficients{ challenge.alpha_base * challenge.alpha_step.sqr(),
                                                       challenge.alpha_step,
                                                       nu_powers[1] * challenge.nu_step,
                                                       challenge.nu_step,
                                                       challenge.linear_nu };
}
} // namespace waffle