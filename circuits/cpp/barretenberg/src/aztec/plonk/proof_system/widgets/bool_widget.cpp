#include "bool_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverBoolWidget::ProverBoolWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_bl(key->constraint_selectors.at("q_bl"))
    , q_br(key->constraint_selectors.at("q_br"))
    , q_bo(key->constraint_selectors.at("q_bo"))
    , q_bl_fft(key->constraint_selector_ffts.at("q_bl_fft"))
    , q_br_fft(key->constraint_selector_ffts.at("q_br_fft"))
    , q_bo_fft(key->constraint_selector_ffts.at("q_bo_fft"))
{}

ProverBoolWidget::ProverBoolWidget(const ProverBoolWidget& other)
    : ProverBaseWidget(other)
    , q_bl(key->constraint_selectors.at("q_bl"))
    , q_br(key->constraint_selectors.at("q_br"))
    , q_bo(key->constraint_selectors.at("q_bo"))
    , q_bl_fft(key->constraint_selector_ffts.at("q_bl_fft"))
    , q_br_fft(key->constraint_selector_ffts.at("q_br_fft"))
    , q_bo_fft(key->constraint_selector_ffts.at("q_bo_fft"))
{}

ProverBoolWidget::ProverBoolWidget(ProverBoolWidget&& other)
    : ProverBaseWidget(other)
    , q_bl(key->constraint_selectors.at("q_bl"))
    , q_br(key->constraint_selectors.at("q_br"))
    , q_bo(key->constraint_selectors.at("q_bo"))
    , q_bl_fft(key->constraint_selector_ffts.at("q_bl_fft"))
    , q_br_fft(key->constraint_selector_ffts.at("q_br_fft"))
    , q_bo_fft(key->constraint_selector_ffts.at("q_bo_fft"))
{}

ProverBoolWidget& ProverBoolWidget::operator=(const ProverBoolWidget& other)
{
    ProverBaseWidget::operator=(other);

    q_bl = key->constraint_selectors.at("q_bl");
    q_br = key->constraint_selectors.at("q_br");
    q_bo = key->constraint_selectors.at("q_bo");

    q_bl_fft = key->constraint_selectors.at("q_bl_fft");
    q_br_fft = key->constraint_selectors.at("q_br_fft");
    q_bo_fft = key->constraint_selectors.at("q_bo_fft");
    return *this;
}

ProverBoolWidget& ProverBoolWidget::operator=(ProverBoolWidget&& other)
{
    ProverBaseWidget::operator=(other);

    q_bl = key->constraint_selectors.at("q_bl");
    q_br = key->constraint_selectors.at("q_br");
    q_bo = key->constraint_selectors.at("q_bo");

    q_bl_fft = key->constraint_selectors.at("q_bl_fft");
    q_br_fft = key->constraint_selectors.at("q_br_fft");
    q_bo_fft = key->constraint_selectors.at("q_bo_fft");
    return *this;
}

fr ProverBoolWidget::compute_quotient_contribution(const fr& alpha_base, const transcript::Transcript& transcript)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    polynomial& w_1_fft = key->wire_ffts.at("w_1_fft");
    polynomial& w_2_fft = key->wire_ffts.at("w_2_fft");
    polynomial& w_3_fft = key->wire_ffts.at("w_3_fft");

    polynomial& quotient_mid = key->quotient_mid;

    fr alpha_a = alpha_base * alpha;
    fr alpha_b = alpha_a * alpha;
    ITERATE_OVER_DOMAIN_START(key->mid_domain);
    fr T0 = (w_1_fft[i * 2].sqr() - w_1_fft[i * 2]) * q_bl_fft[i] * alpha_base;
    fr T1 = (w_2_fft[i * 2].sqr() - w_2_fft[i * 2]) * q_br_fft[i] * alpha_a;
    fr T2 = (w_3_fft[i * 2].sqr() - w_3_fft[i * 2]) * q_bo_fft[i] * alpha_b;

    quotient_mid[i] += (T0 + T1 + T2);
    ITERATE_OVER_DOMAIN_END;

    return alpha_base * alpha_b * alpha;
}

fr ProverBoolWidget::compute_linear_contribution(const fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 polynomial& r)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);

    fr left = (w_l_eval.sqr() - w_l_eval) * alpha_base;
    fr right = (w_r_eval.sqr() - w_r_eval) * alpha_base * alpha;
    fr out = (w_o_eval.sqr() - w_o_eval) * alpha_base * alpha.sqr();

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] += ((left * q_bl[i]) + (right * q_br[i]) + (out * q_bo[i]));
    ITERATE_OVER_DOMAIN_END;

    return alpha_base * alpha.sqr() * alpha;
}

fr ProverBoolWidget::compute_opening_poly_contribution(
    const fr& nu_base, const transcript::Transcript& transcript, fr* poly, fr*, const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_base;
    }

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    std::array<barretenberg::fr, 3> nu_powers;
    nu_powers[0] = nu_base;
    nu_powers[1] = nu_powers[0] * nu;
    nu_powers[2] = nu_powers[1] * nu;
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_bl[i] * nu_powers[0]);
    poly[i] += (q_br[i] * nu_powers[1]);
    poly[i] += (q_bo[i] * nu_powers[2]);
    ITERATE_OVER_DOMAIN_END;

    return nu_powers[2] * nu;
}

void ProverBoolWidget::compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_bl", q_bl.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_br", q_br.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_bo", q_bo.evaluate(z, key->small_domain.size).to_buffer());
}

// ###

VerifierBoolWidget::VerifierBoolWidget()
    : VerifierBaseWidget()
{}

fr VerifierBoolWidget::compute_quotient_evaluation_contribution(verification_key*,
                                                                const fr& alpha_base,
                                                                const transcript::Transcript& transcript,
                                                                fr& t_eval,
                                                                const bool use_linearisation)
{
    const fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    if (use_linearisation) {
        return alpha_base * alpha;
    }

    fr alpha_a = alpha_base * alpha;
    fr alpha_b = alpha_a * alpha;

    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr q_bl_eval = fr::serialize_from_buffer(&transcript.get_element("q_bl")[0]);
    fr q_br_eval = fr::serialize_from_buffer(&transcript.get_element("q_br")[0]);
    fr q_bo_eval = fr::serialize_from_buffer(&transcript.get_element("q_bo")[0]);

    fr T0 = (w_l_eval.sqr() - w_l_eval) * q_bl_eval * alpha_base;
    fr T1 = (w_r_eval.sqr() - w_r_eval) * q_br_eval * alpha_a;
    fr T2 = (w_o_eval.sqr() - w_o_eval) * q_bo_eval * alpha_b;

    t_eval += (T0 + T1 + T2);

    return alpha_base * alpha;
}

fr VerifierBoolWidget::compute_batch_evaluation_contribution(verification_key*,
                                                             fr& batch_eval,
                                                             const fr& nu_base,
                                                             const transcript::Transcript& transcript,
                                                             const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_base;
    }

    fr q_bl_eval = fr::serialize_from_buffer(&transcript.get_element("q_bl")[0]);
    fr q_br_eval = fr::serialize_from_buffer(&transcript.get_element("q_br")[0]);
    fr q_bo_eval = fr::serialize_from_buffer(&transcript.get_element("q_bo")[0]);

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    std::array<barretenberg::fr, 3> nu_powers;
    nu_powers[0] = nu_base;
    nu_powers[1] = nu_powers[0] * nu;
    nu_powers[2] = nu_powers[1] * nu;

    batch_eval += (q_bl_eval * nu_powers[0]);
    batch_eval += (q_br_eval * nu_powers[1]);
    batch_eval += (q_bo_eval * nu_powers[2]);

    return nu_powers[2] * nu;
};

VerifierBaseWidget::challenge_coefficients VerifierBoolWidget::append_scalar_multiplication_inputs(
    verification_key* key,
    const challenge_coefficients& challenge,
    const transcript::Transcript& transcript,
    std::vector<g1::affine_element>& points,
    std::vector<fr>& scalars,
    const bool use_linearisation)
{
    if (use_linearisation) {
        fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
        fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
        fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);

        fr left_bool_multiplier = (w_l_eval.sqr() - w_l_eval) * challenge.alpha_base * challenge.linear_nu;
        fr right_bool_multiplier =
            (w_r_eval.sqr() - w_r_eval) * challenge.alpha_base * challenge.alpha_step * challenge.linear_nu;
        fr output_bool_multiplier =
            (w_o_eval.sqr() - w_o_eval) * challenge.alpha_base * challenge.alpha_step.sqr() * challenge.linear_nu;

        if (key->constraint_selectors.at("Q_BL").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_BL"));
            scalars.push_back(left_bool_multiplier);
        }
        if (key->constraint_selectors.at("Q_BR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_BR"));
            scalars.push_back(right_bool_multiplier);
        }
        if (key->constraint_selectors.at("Q_BO").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_BO"));
            scalars.push_back(output_bool_multiplier);
        }

        return VerifierBaseWidget::challenge_coefficients{ challenge.alpha_base * challenge.alpha_step.sqr() *
                                                               challenge.alpha_step,
                                                           challenge.alpha_step,
                                                           challenge.nu_base,
                                                           challenge.nu_step,
                                                           challenge.linear_nu };
    }

    std::array<barretenberg::fr, 3> nu_powers;
    nu_powers[0] = challenge.nu_base;
    nu_powers[1] = nu_powers[0] * challenge.nu_step;
    nu_powers[2] = nu_powers[1] * challenge.nu_step;

    if (key->constraint_selectors.at("Q_BL").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_BL"));
        scalars.push_back(nu_powers[0]);
    }
    if (key->constraint_selectors.at("Q_BR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_BR"));
        scalars.push_back(nu_powers[1]);
    }
    if (key->constraint_selectors.at("Q_BO").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_BO"));
        scalars.push_back(nu_powers[2]);
    }

    return VerifierBaseWidget::challenge_coefficients{ challenge.alpha_base * challenge.alpha_step.sqr() *
                                                           challenge.alpha_step,
                                                       challenge.alpha_step,
                                                       nu_powers[2] * challenge.nu_step,
                                                       challenge.nu_step,
                                                       challenge.linear_nu };
}
} // namespace waffle