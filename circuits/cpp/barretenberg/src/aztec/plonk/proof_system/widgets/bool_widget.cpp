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

size_t ProverBoolWidget::compute_opening_poly_contribution(
    const size_t nu_index, const transcript::Transcript& transcript, fr* poly, fr*, const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_index;
    }

    std::array<barretenberg::fr, 3> nu_powers;
    nu_powers[0] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index)[0]);

    nu_powers[1] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 1)[0]);

    nu_powers[2] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 2)[0]);

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_bl[i] * nu_powers[0]);
    poly[i] += (q_br[i] * nu_powers[1]);
    poly[i] += (q_bo[i] * nu_powers[2]);
    ITERATE_OVER_DOMAIN_END;

    return nu_index + 3;
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

template <typename Field, typename Group, typename Transcript>
VerifierBoolWidget<Field, Group, Transcript>::VerifierBoolWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierBoolWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*,
    const Field& alpha_base,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{
    const Field alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    if (use_linearisation) {
        return alpha_base * alpha;
    }

    Field alpha_a = alpha_base * alpha;
    Field alpha_b = alpha_a * alpha;

    Field w_l_eval = transcript.get_field_element("w_1");
    Field w_r_eval = transcript.get_field_element("w_2");
    Field w_o_eval = transcript.get_field_element("w_3");
    Field q_bl_eval = transcript.get_field_element("q_bl");
    Field q_br_eval = transcript.get_field_element("q_br");
    Field q_bo_eval = transcript.get_field_element("q_bo");

    Field T0 = (w_l_eval.sqr() - w_l_eval) * q_bl_eval * alpha_base;
    Field T1 = (w_r_eval.sqr() - w_r_eval) * q_br_eval * alpha_a;
    Field T2 = (w_o_eval.sqr() - w_o_eval) * q_bo_eval * alpha_b;

    t_eval += (T0 + T1 + T2);

    return alpha_base * alpha;
}

template <typename Field, typename Group, typename Transcript>
size_t VerifierBoolWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(verification_key*,
                                                                                           Field& batch_eval,
                                                                                           const size_t nu_index,
                                                                                           const Transcript& transcript,
                                                                                           const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_index;
    }

    Field q_bl_eval = transcript.get_field_element("q_bl");
    Field q_br_eval = transcript.get_field_element("q_br");
    Field q_bo_eval = transcript.get_field_element("q_bo");

    std::array<Field, 3> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element("nu", nu_index);
    nu_challenges[1] = transcript.get_challenge_field_element("nu", nu_index + 1);
    nu_challenges[2] = transcript.get_challenge_field_element("nu", nu_index + 2);

    batch_eval += (q_bl_eval * nu_challenges[0]);
    batch_eval += (q_br_eval * nu_challenges[1]);
    batch_eval += (q_bo_eval * nu_challenges[2]);

    return nu_index + 3;
};

template <typename Field, typename Group, typename Transcript>
VerifierBaseWidget::challenge_coefficients<Field> VerifierBoolWidget<Field, Group, Transcript>::
    append_scalar_multiplication_inputs(verification_key* key,
                                        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
                                        const Transcript& transcript,
                                        std::vector<Group>& points,
                                        std::vector<Field>& scalars,
                                        const bool use_linearisation)
{
    if (use_linearisation) {

        fr linear_nu = transcript.get_challenge_field_element("nu", challenge.linear_nu_index);

        fr w_l_eval = transcript.get_field_element("w_1");
        fr w_r_eval = transcript.get_field_element("w_2");
        fr w_o_eval = transcript.get_field_element("w_3");

        fr left_bool_multiplier = (w_l_eval.sqr() - w_l_eval) * challenge.alpha_base * linear_nu;
        fr right_bool_multiplier =
            (w_r_eval.sqr() - w_r_eval) * challenge.alpha_base * challenge.alpha_step * linear_nu;
        fr output_bool_multiplier =
            (w_o_eval.sqr() - w_o_eval) * challenge.alpha_base * challenge.alpha_step.sqr() * linear_nu;

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

        return VerifierBaseWidget::challenge_coefficients<Field>{ challenge.alpha_base * challenge.alpha_step.sqr() *
                                                                      challenge.alpha_step,
                                                                  challenge.alpha_step,
                                                                  challenge.nu_index,
                                                                  challenge.linear_nu_index };
    }

    std::array<Field, 3> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element("nu", challenge.nu_index);
    nu_challenges[1] = transcript.get_challenge_field_element("nu", challenge.nu_index + 1);
    nu_challenges[2] = transcript.get_challenge_field_element("nu", challenge.nu_index + 2);

    if (key->constraint_selectors.at("Q_BL").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_BL"));
        scalars.push_back(nu_challenges[0]);
    }
    if (key->constraint_selectors.at("Q_BR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_BR"));
        scalars.push_back(nu_challenges[1]);
    }
    if (key->constraint_selectors.at("Q_BO").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_BO"));
        scalars.push_back(nu_challenges[2]);
    }

    return VerifierBaseWidget::challenge_coefficients<Field>{ challenge.alpha_base * challenge.alpha_step.sqr() *
                                                                  challenge.alpha_step,
                                                              challenge.alpha_step,
                                                              challenge.nu_index + 3,
                                                              challenge.linear_nu_index };
}

template class VerifierBoolWidget<barretenberg::fr, barretenberg::g1::affine_element, transcript::StandardTranscript>;

} // namespace waffle