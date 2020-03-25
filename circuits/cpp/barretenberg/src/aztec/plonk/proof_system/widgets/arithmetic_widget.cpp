#include "arithmetic_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverArithmeticWidget::ProverArithmeticWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_1(key->constraint_selectors.at("q_1"))
    , q_2(key->constraint_selectors.at("q_2"))
    , q_3(key->constraint_selectors.at("q_3"))
    , q_m(key->constraint_selectors.at("q_m"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_1_fft(key->constraint_selector_ffts.at("q_1_fft"))
    , q_2_fft(key->constraint_selector_ffts.at("q_2_fft"))
    , q_3_fft(key->constraint_selector_ffts.at("q_3_fft"))
    , q_m_fft(key->constraint_selector_ffts.at("q_m_fft"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
{}

ProverArithmeticWidget::ProverArithmeticWidget(const ProverArithmeticWidget& other)
    : ProverBaseWidget(other)
    , q_1(other.q_1)
    , q_2(other.q_2)
    , q_3(other.q_3)
    , q_m(other.q_m)
    , q_c(other.q_c)
    , q_1_fft(other.q_1_fft)
    , q_2_fft(other.q_2_fft)
    , q_3_fft(other.q_3_fft)
    , q_m_fft(other.q_m_fft)
    , q_c_fft(other.q_c_fft)
{}

ProverArithmeticWidget::ProverArithmeticWidget(ProverArithmeticWidget&& other)
    : ProverBaseWidget(other)
    , q_1(other.q_1)
    , q_2(other.q_2)
    , q_3(other.q_3)
    , q_m(other.q_m)
    , q_c(other.q_c)
    , q_1_fft(other.q_1_fft)
    , q_2_fft(other.q_2_fft)
    , q_3_fft(other.q_3_fft)
    , q_m_fft(other.q_m_fft)
    , q_c_fft(other.q_c_fft)
{}

ProverArithmeticWidget& ProverArithmeticWidget::operator=(const ProverArithmeticWidget& other)
{
    ProverBaseWidget::operator=(other);
    q_1 = key->constraint_selectors.at("q_1");
    q_2 = key->constraint_selectors.at("q_2");
    q_3 = key->constraint_selectors.at("q_3");
    q_m = key->constraint_selectors.at("q_m");
    q_c = key->constraint_selectors.at("q_c");

    q_1_fft = key->constraint_selectors.at("q_1_fft");
    q_2_fft = key->constraint_selectors.at("q_2_fft");
    q_3_fft = key->constraint_selectors.at("q_3_fft");
    q_m_fft = key->constraint_selectors.at("q_m_fft");
    q_c_fft = key->constraint_selectors.at("q_c_fft");
    return *this;
}

ProverArithmeticWidget& ProverArithmeticWidget::operator=(ProverArithmeticWidget&& other)
{
    ProverBaseWidget::operator=(other);

    q_1 = key->constraint_selectors.at("q_1");
    q_2 = key->constraint_selectors.at("q_2");
    q_3 = key->constraint_selectors.at("q_3");
    q_m = key->constraint_selectors.at("q_m");
    q_c = key->constraint_selectors.at("q_c");

    q_1_fft = key->constraint_selectors.at("q_1_fft");
    q_2_fft = key->constraint_selectors.at("q_2_fft");
    q_3_fft = key->constraint_selectors.at("q_3_fft");
    q_m_fft = key->constraint_selectors.at("q_m_fft");
    q_c_fft = key->constraint_selectors.at("q_c_fft");

    return *this;
}

fr ProverArithmeticWidget::compute_quotient_contribution(const fr& alpha_base, const transcript::Transcript& transcript)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    polynomial& w_1_fft = key->wire_ffts.at("w_1_fft");
    polynomial& w_2_fft = key->wire_ffts.at("w_2_fft");
    polynomial& w_3_fft = key->wire_ffts.at("w_3_fft");

    polynomial& quotient_mid = key->quotient_mid;

    ITERATE_OVER_DOMAIN_START(key->mid_domain);
    fr T0 = w_1_fft[2 * i] * w_2_fft[2 * i] * q_m_fft[i];
    fr T1 = w_1_fft[2 * i] * q_1_fft[i];
    fr T2 = w_2_fft[2 * i] * q_2_fft[i];
    fr T3 = w_3_fft[2 * i] * q_3_fft[i];
    quotient_mid[i] += ((T0 + T1 + T2 + T3 + q_c_fft[i]) * alpha_base);
    ITERATE_OVER_DOMAIN_END;

    return alpha_base * alpha;
}

fr ProverArithmeticWidget::compute_linear_contribution(const fr& alpha_base,
                                                       const transcript::Transcript& transcript,
                                                       polynomial& r)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_lr = w_l_eval * w_r_eval;
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T0 = w_lr * q_m[i];
    fr T1 = w_l_eval * q_1[i];
    fr T2 = w_r_eval * q_2[i];
    fr T3 = w_o_eval * q_3[i];
    r[i] += ((T0 + T1 + T2 + T3 + q_c[i]) * alpha_base);
    ITERATE_OVER_DOMAIN_END;

    return alpha_base * alpha;
}

size_t ProverArithmeticWidget::compute_opening_poly_contribution(const size_t nu_index,
                                                                 const transcript::Transcript& transcript,
                                                                 barretenberg::fr* poly,
                                                                 barretenberg::fr*,
                                                                 const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_index;
    }

    std::array<barretenberg::fr, 5> nu_challenges;
    nu_challenges[0] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index)[0]);
    nu_challenges[1] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 1)[0]);
    nu_challenges[2] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 2)[0]);
    nu_challenges[3] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 3)[0]);
    nu_challenges[4] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 4)[0]);
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_1[i] * nu_challenges[0]);
    poly[i] += (q_2[i] * nu_challenges[1]);
    poly[i] += (q_3[i] * nu_challenges[2]);
    poly[i] += (q_m[i] * nu_challenges[3]);
    poly[i] += (q_c[i] * nu_challenges[4]);
    ITERATE_OVER_DOMAIN_END;

    return nu_index + 5;
}

void ProverArithmeticWidget::compute_transcript_elements(transcript::Transcript& transcript,
                                                         const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_1", q_1.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_2", q_2.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_3", q_3.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_m", q_m.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_c", q_c.evaluate(z, key->small_domain.size).to_buffer());
}

// ###

template <typename Field, typename Group, typename Transcript>
VerifierArithmeticWidget<Field, Group, Transcript>::VerifierArithmeticWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierArithmeticWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*,
    const Field& alpha_base,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{
    const Field alpha = transcript.get_challenge_field_element("alpha");
    if (use_linearisation) {
        return alpha_base * alpha;
    }

    Field w_l_eval = transcript.get_field_element("w_1");
    Field w_r_eval = transcript.get_field_element("w_2");
    Field w_o_eval = transcript.get_field_element("w_3");
    Field q_1_eval = transcript.get_field_element("q_1");
    Field q_2_eval = transcript.get_field_element("q_2");
    Field q_3_eval = transcript.get_field_element("q_3");
    Field q_m_eval = transcript.get_field_element("q_m");
    Field q_c_eval = transcript.get_field_element("q_c");

    Field T0 = w_l_eval * w_r_eval * q_m_eval;
    Field T1 = w_l_eval * q_1_eval;
    Field T2 = w_r_eval * q_2_eval;
    Field T3 = w_o_eval * q_3_eval;
    t_eval += ((T0 + T1 + T2 + T3 + q_c_eval) * alpha_base);
    return alpha_base * alpha;
}

template <typename Field, typename Group, typename Transcript>
size_t VerifierArithmeticWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*,
    Field& batch_eval,
    const size_t nu_index,
    const Transcript& transcript,
    const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_index;
    }

    std::array<Field, 5> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element("nu", nu_index);
    nu_challenges[1] = transcript.get_challenge_field_element("nu", nu_index + 1);
    nu_challenges[2] = transcript.get_challenge_field_element("nu", nu_index + 2);
    nu_challenges[3] = transcript.get_challenge_field_element("nu", nu_index + 3);
    nu_challenges[4] = transcript.get_challenge_field_element("nu", nu_index + 4);

    Field q_1_eval = transcript.get_field_element("q_1");
    Field q_2_eval = transcript.get_field_element("q_2");
    Field q_3_eval = transcript.get_field_element("q_3");
    Field q_m_eval = transcript.get_field_element("q_m");
    Field q_c_eval = transcript.get_field_element("q_c");

    batch_eval += (q_1_eval * nu_challenges[0]);
    batch_eval += (q_2_eval * nu_challenges[1]);
    batch_eval += (q_3_eval * nu_challenges[2]);
    batch_eval += (q_m_eval * nu_challenges[3]);
    batch_eval += (q_c_eval * nu_challenges[4]);
    return nu_index + 5;
};

template <typename Field, typename Group, typename Transcript>
VerifierBaseWidget::challenge_coefficients<Field> VerifierArithmeticWidget<Field, Group, Transcript>::
    append_scalar_multiplication_inputs(verification_key* key,
                                        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
                                        const Transcript& transcript,
                                        std::vector<Group>& points,
                                        std::vector<Field>& scalars,
                                        const bool use_linearisation)
{
    if (use_linearisation) {
        Field w_l_eval = transcript.get_field_element("w_1");
        Field w_r_eval = transcript.get_field_element("w_2");
        Field w_o_eval = transcript.get_field_element("w_3");

        Field linear_nu = transcript.get_challenge_field_element("nu", challenge.linear_nu_index);

        // Q_M term = w_l * w_r * challenge.alpha_base * nu
        Field q_m_term = w_l_eval * w_r_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_M").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_M"));
            scalars.push_back(q_m_term);
        }

        Field q_l_term = w_l_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_1").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_1"));
            scalars.push_back(q_l_term);
        }

        Field q_r_term = w_r_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_2").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_2"));
            scalars.push_back(q_r_term);
        }

        Field q_o_term = w_o_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_3").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_3"));
            scalars.push_back(q_o_term);
        }

        Field q_c_term = challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_C").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_C"));
            scalars.push_back(q_c_term);
        }

        return VerifierBaseWidget::challenge_coefficients<Field>{ challenge.alpha_base * challenge.alpha_step,
                                                                  challenge.alpha_step,
                                                                  challenge.nu_index,
                                                                  challenge.linear_nu_index };
    }

    std::array<Field, 5> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element("nu", challenge.nu_index);
    nu_challenges[1] = transcript.get_challenge_field_element("nu", challenge.nu_index + 1);
    nu_challenges[2] = transcript.get_challenge_field_element("nu", challenge.nu_index + 2);
    nu_challenges[3] = transcript.get_challenge_field_element("nu", challenge.nu_index + 3);
    nu_challenges[4] = transcript.get_challenge_field_element("nu", challenge.nu_index + 4);

    if (key->constraint_selectors.at("Q_1").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_1"));
        scalars.push_back(nu_challenges[0]);
    }

    if (key->constraint_selectors.at("Q_2").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_2"));
        scalars.push_back(nu_challenges[1]);
    }

    if (key->constraint_selectors.at("Q_3").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_3"));
        scalars.push_back(nu_challenges[2]);
    }

    if (key->constraint_selectors.at("Q_M").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_M"));
        scalars.push_back(nu_challenges[3]);
    }

    if (key->constraint_selectors.at("Q_C").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_C"));
        scalars.push_back(nu_challenges[4]);
    }

    return VerifierBaseWidget::challenge_coefficients<Field>{ challenge.alpha_base * challenge.alpha_step,
                                                              challenge.alpha_step,
                                                              challenge.nu_index + 5,
                                                              challenge.linear_nu_index };
}

template class VerifierArithmeticWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>;

} // namespace waffle