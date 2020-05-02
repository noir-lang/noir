#pragma once

#include "../proving_key/proving_key.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

namespace waffle {
ProverTurboFixedBaseWidget::ProverTurboFixedBaseWidget(proving_key* input_key, program_witness* input_witness)
    : ProverTurboArithmeticWidget(input_key, input_witness)
    , q_ecc_1(key->constraint_selectors.at("q_ecc_1"))
    , q_ecc_1_fft(key->constraint_selector_ffts.at("q_ecc_1_fft"))
{}

ProverTurboFixedBaseWidget::ProverTurboFixedBaseWidget(const ProverTurboFixedBaseWidget& other)
    : ProverTurboArithmeticWidget(other)
    , q_ecc_1(key->constraint_selectors.at("q_ecc_1"))
    , q_ecc_1_fft(key->constraint_selector_ffts.at("q_ecc_1_fft"))
{}

ProverTurboFixedBaseWidget::ProverTurboFixedBaseWidget(ProverTurboFixedBaseWidget&& other)
    : ProverTurboArithmeticWidget(other)
    , q_ecc_1(key->constraint_selectors.at("q_ecc_1"))
    , q_ecc_1_fft(key->constraint_selector_ffts.at("q_ecc_1_fft"))
{}

ProverTurboFixedBaseWidget& ProverTurboFixedBaseWidget::operator=(const ProverTurboFixedBaseWidget& other)
{
    ProverTurboArithmeticWidget::operator=(other);
    q_ecc_1 = key->constraint_selectors.at("q_ecc_1");
    q_ecc_1_fft = key->constraint_selector_ffts.at("q_ecc_1_fft");
    return *this;
}

ProverTurboFixedBaseWidget& ProverTurboFixedBaseWidget::operator=(ProverTurboFixedBaseWidget&& other)
{
    ProverTurboArithmeticWidget::operator=(other);
    q_ecc_1 = key->constraint_selectors.at("q_ecc_1");
    q_ecc_1_fft = key->constraint_selector_ffts.at("q_ecc_1_fft");
    return *this;
}

barretenberg::fr ProverTurboFixedBaseWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                             const transcript::Transcript& transcript)
{
    fr new_alpha_base = ProverTurboArithmeticWidget::compute_quotient_contribution(alpha_base, transcript);

    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    fr alpha_a = new_alpha_base;
    fr alpha_b = alpha_a * alpha;
    fr alpha_c = alpha_b * alpha;
    fr alpha_d = alpha_c * alpha;
    fr alpha_e = alpha_d * alpha;
    fr alpha_f = alpha_e * alpha;
    fr alpha_g = alpha_f * alpha;

    fr* w_1_fft = &key->wire_ffts.at("w_1_fft")[0];
    fr* w_2_fft = &key->wire_ffts.at("w_2_fft")[0];
    fr* w_3_fft = &key->wire_ffts.at("w_3_fft")[0];
    fr* w_4_fft = &key->wire_ffts.at("w_4_fft")[0];

    fr* quotient_large = &key->quotient_large[0];
    // selector renaming:
    // q_1 = q_x_1
    // q_2 = q_x_2
    // q_3 = q_y_1
    // q_ecc_1 = q_y_2
    // q_4 = q_x_init_1
    // q_5 = q_x_init_2
    // q_m = q_y_init_1
    // q_c = q_y_init_2
    constexpr fr minus_nine = -fr(9);
    constexpr fr minus_one = -fr(1);

    ITERATE_OVER_DOMAIN_START(key->large_domain);

    // accumulator_delta = d(Xw) - 4d(X)
    // accumulator_delta tracks the current round's scalar multiplier
    // which should be one of {-3, -1, 1, 3}
    fr accumulator_delta = w_4_fft[i] + w_4_fft[i];
    accumulator_delta += accumulator_delta;
    accumulator_delta = w_4_fft[i + 4] - accumulator_delta;

    fr accumulator_delta_squared = accumulator_delta.sqr();

    // y_alpha represents the point that we're adding into our accumulator point at the current round
    // q_3 and q_ecc_1 are selector polynomials that describe two different y-coordinates
    // the value of y-alpha is one of these two points, or their inverses
    // y_alpha = delta * (x_alpha * q_3 + q_ecc_1)
    // (we derive x_alpha from y_alpha, with `delta` conditionally flipping the sign of the output)
    // q_3 and q_ecc_1 are not directly equal to the 2 potential y-coordinates.
    // let's use `x_beta`, `x_gamma`, `y_beta`, `y_gamma` to refer to the two points in our lookup table
    // y_alpha = [(x_alpha - x_gamma) / (x_beta - x_gamma)].y_beta.delta + [(x_alpha - x_beta) / 3.(x_gamma -
    // x_beta)].y_gamma.delta
    // => q_3 = (3.y_beta - y_gamma) / 3.(x_beta - x_gamma)
    // => q_ecc_1 = (3.x_beta.y_gamma - x_gammay_beta) / 3.(x_beta - x_gammma)
    fr y_alpha = w_3_fft[i + 4] * q_3_fft[i];
    y_alpha += q_ecc_1_fft[i];
    y_alpha *= accumulator_delta;

    fr T0 = accumulator_delta_squared + minus_one;
    fr T1 = accumulator_delta_squared + minus_nine;

    // scalar accumulator consistency check
    // (delta - 1)(delta - 3)(delta + 1)(delta + 3).q_ecc_1 = 0 mod Z_H
    fr scalar_accumulator_identity = T0 * T1;
    scalar_accumulator_identity *= alpha_a;

    // x_alpha consistency check
    // (delta^2.q_1 + q_2 - x_alpha).q_ecc = 0 mod Z_H
    // x_alpha is the x-coordinate of the point we're adding into our accumulator point.
    // We use a w_o(X) to track x_alpha, to reduce the number of required selector polynomials
    fr x_alpha_identity = accumulator_delta_squared * q_1_fft[i];
    x_alpha_identity += q_2_fft[i];
    x_alpha_identity -= w_3_fft[i + 4];
    x_alpha_identity *= alpha_b;

    // x-accumulator consistency check
    // ((x_2 + x_1 + x_alpha)(x_alpha - x_1)^2 - (y_alpha - y_1)^2).q_ecc = 0 mod Z_H
    // we use the fact that y_alpha^2 = x_alpha^3 + grumpkin::g1::element::curve_b
    fr x_alpha_minus_x_1 = w_3_fft[i + 4] - (w_1_fft[i]);

    T0 = y_alpha * w_2_fft[i];
    T0 += T0;

    T1 = x_alpha_minus_x_1.sqr();
    fr T2 = w_1_fft[i + 4] + w_1_fft[i]; // T1 = (x_alpha - x_1)^2
    T2 += w_3_fft[i + 4];                // T2 = (x_2 + x_1 + x_alpha)
    T1 *= T2;
    T2 = w_2_fft[i].sqr(); // T1 = y_1^2
    T2 += grumpkin::g1::element::curve_b;
    fr x_accumulator_identity = T0 + T1;
    x_accumulator_identity -= T2;
    T0 = w_3_fft[i + 4].sqr(); // y_alpha^2 = x_alpha^3 + b
    T0 *= w_3_fft[i + 4];
    x_accumulator_identity -= T0;
    x_accumulator_identity *= alpha_c;

    // y-accumulator consistency check
    // ((y_2 + y_1)(x_alpha - x_1) - (y_alpha - y_1)(x_1 - x_2)).q_ecc = 0 mod Z_H
    T0 = w_2_fft[i] + w_2_fft[i + 4];
    T0 *= x_alpha_minus_x_1;

    T1 = y_alpha - w_2_fft[i];

    T2 = w_1_fft[i] - w_1_fft[i + 4];
    T1 *= T2;

    fr y_accumulator_identity = T0 - T1;
    y_accumulator_identity *= alpha_d;

    // accumlulator-init consistency check
    // at the start of our scalar multiplication ladder, we want to validate that
    // the initial values of (x_1, y_1) and scalar accumulator a_1 are correctly set
    // We constrain a_1 to be either 0 or the value in w_o (which should be correctly initialized to (1 / 4^n) via a
    // copy constraint) We constraint (x_1, y_1) to be one of 4^n.[1] or (4^n + 1).[1]
    fr w_4_minus_one = w_4_fft[i] + minus_one;
    T1 = w_4_minus_one - w_3_fft[i];
    fr accumulator_init_identity = w_4_minus_one * T1;
    accumulator_init_identity *= alpha_e;

    // // x-init consistency check
    T0 = q_4_fft[i] - w_1_fft[i];
    T0 *= w_3_fft[i];
    T1 = w_4_minus_one * q_5_fft[i];
    fr x_init_identity = T0 - T1;
    x_init_identity *= alpha_f;

    // // y-init consistency check
    T0 = q_m_fft[i] - w_2_fft[i];
    T0 *= w_3_fft[i];
    T1 = w_4_minus_one * q_c_fft[i];
    fr y_init_identity = T0 - T1;
    y_init_identity *= alpha_g;

    fr gate_identity = accumulator_init_identity + x_init_identity;
    gate_identity += y_init_identity;
    gate_identity *= q_c_fft[i];
    gate_identity += scalar_accumulator_identity;
    gate_identity += x_alpha_identity;
    gate_identity += x_accumulator_identity;
    gate_identity += y_accumulator_identity;
    gate_identity *= q_ecc_1_fft[i];

    quotient_large[i] += gate_identity;
    ITERATE_OVER_DOMAIN_END;

    return alpha_g * alpha;
}

void ProverTurboFixedBaseWidget::compute_transcript_elements(transcript::Transcript& transcript,
                                                             const bool use_linearisation)
{
    ProverTurboArithmeticWidget::compute_transcript_elements(transcript, use_linearisation);
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_ecc_1", q_ecc_1.evaluate(z, key->small_domain.size).to_buffer());
    if (use_linearisation) {
        transcript.add_element("q_c", q_c.evaluate(z, key->small_domain.size).to_buffer());
    }
}

barretenberg::fr ProverTurboFixedBaseWidget::compute_linear_contribution(const fr& alpha_base,
                                                           const transcript::Transcript& transcript,
                                                           barretenberg::polynomial& r)
{
    fr new_alpha_base = ProverTurboArithmeticWidget::compute_linear_contribution(alpha_base, transcript, r);
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr w_1_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_2_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_3_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr w_1_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_1_omega")[0]);
    fr w_3_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);

    fr w_4_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_4_omega")[0]);

    fr q_ecc_1_eval = fr::serialize_from_buffer(&transcript.get_element("q_ecc_1")[0]);
    fr q_c_eval = fr::serialize_from_buffer(&transcript.get_element("q_c")[0]);

    fr alpha_b = new_alpha_base * (alpha);
    fr alpha_c = alpha_b * alpha;
    fr alpha_d = alpha_c * alpha;
    fr alpha_e = alpha_d * alpha;
    fr alpha_f = alpha_e * alpha;
    fr alpha_g = alpha_f * alpha;

    fr delta = w_4_omega_eval - (w_4_eval + w_4_eval + w_4_eval + w_4_eval);

    fr delta_squared = delta.sqr();

    fr q_1_multiplicand = delta_squared * q_ecc_1_eval * alpha_b;

    fr q_2_multiplicand = alpha_b * q_ecc_1_eval;

    fr q_3_multiplicand = (w_1_omega_eval - w_1_eval) * delta * w_3_omega_eval * alpha_d * q_ecc_1_eval;
    fr T1 = delta * w_3_omega_eval * w_2_eval * alpha_c;
    q_3_multiplicand = q_3_multiplicand + (T1 + T1) * q_ecc_1_eval;

    fr q_4_multiplicand = w_3_eval * q_ecc_1_eval * q_c_eval * alpha_f;

    fr q_5_multiplicand = (fr::one() - w_4_eval) * q_ecc_1_eval * q_c_eval * alpha_f;

    fr q_m_multiplicand = w_3_eval * q_ecc_1_eval * q_c_eval * alpha_g;

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T2 = q_1_multiplicand * q_1[i];
    fr T3 = q_2_multiplicand * q_2[i];
    fr T4 = q_3_multiplicand * q_3[i];
    fr T5 = q_4_multiplicand * q_4[i];
    fr T6 = q_5_multiplicand * q_5[i];
    fr T7 = q_m_multiplicand * q_m[i];
    r[i] += (T2 + T3 + T4 + T5 + T6 + T7);
    ITERATE_OVER_DOMAIN_END;

    return alpha_g * alpha;
}

void ProverTurboFixedBaseWidget::compute_opening_poly_contribution(const transcript::Transcript& transcript,
                                                                   const bool use_linearisation)
{

    ProverTurboArithmeticWidget::compute_opening_poly_contribution(transcript, use_linearisation);

    polynomial& poly = key->opening_poly;

    fr nu_a = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_ecc_1")[0]);

    if (use_linearisation) {
        fr nu_b = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_c")[0]);

        ITERATE_OVER_DOMAIN_START(key->small_domain);
        fr T0 = q_ecc_1[i] * nu_a;
        fr T1 = q_c[i] * nu_b;
        T0 += T1;
        poly[i] += T0;
        ITERATE_OVER_DOMAIN_END;
        return;
    }

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += q_ecc_1[i] * nu_a;
    ITERATE_OVER_DOMAIN_END;
}

// ###

template <typename Field, typename Group, typename Transcript>
VerifierTurboFixedBaseWidget<Field, Group, Transcript>::VerifierTurboFixedBaseWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierTurboFixedBaseWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key* key,
    const Field& alpha_base,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{
    Field new_alpha_base =
        VerifierTurboArithmeticWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, t_eval, use_linearisation);
    Field w_1_eval = transcript.get_field_element("w_1");
    Field w_2_eval = transcript.get_field_element("w_2");
    Field w_3_eval = transcript.get_field_element("w_3");
    Field w_4_eval = transcript.get_field_element("w_4");
    Field w_1_omega_eval = transcript.get_field_element("w_1_omega");
    Field w_2_omega_eval = transcript.get_field_element("w_2_omega");
    Field w_3_omega_eval = transcript.get_field_element("w_3_omega");
    Field w_4_omega_eval = transcript.get_field_element("w_4_omega");

    Field q_ecc_1_eval = transcript.get_field_element("q_ecc_1");
    Field q_c_eval = transcript.get_field_element("q_c");

    Field alpha = transcript.get_challenge_field_element("alpha");
    Field alpha_a = new_alpha_base;
    Field alpha_b = alpha_a * alpha;
    Field alpha_c = alpha_b * alpha;
    Field alpha_d = alpha_c * alpha;
    Field alpha_e = alpha_d * alpha;
    Field alpha_f = alpha_e * alpha;
    Field alpha_g = alpha_f * alpha;
    const Field grumpkin_curve_b = -Field(17);

    if (use_linearisation) {
        Field delta = w_4_omega_eval - (w_4_eval + w_4_eval + w_4_eval + w_4_eval);
        const Field three = Field(3);
        Field T1 = (delta + Field(1));
        Field T2 = (delta + three);
        Field T3 = (delta - Field(1));
        Field T4 = (delta - three);

        Field accumulator_identity = T1 * T2 * T3 * T4 * alpha_a;

        Field x_alpha_identity = -(w_3_omega_eval * alpha_b);

        Field T0 = w_1_omega_eval + w_1_eval + w_3_omega_eval;
        T1 = (w_3_omega_eval - w_1_eval).sqr();
        T0 = T0 * T1;

        T1 = w_3_omega_eval.sqr() * w_3_omega_eval;
        T2 = w_2_eval.sqr();
        T1 = T1 + T2;
        T1 = -(T1 + grumpkin_curve_b);

        T2 = delta * w_2_eval * q_ecc_1_eval;
        T2 = T2 + T2;

        Field x_accumulator_identity = (T0 + T1 + T2) * alpha_c;

        T0 = (w_2_omega_eval + w_2_eval) * (w_3_omega_eval - w_1_eval);

        T1 = w_1_eval - w_1_omega_eval;
        T2 = w_2_eval - (q_ecc_1_eval * delta);
        T1 = T1 * T2;

        Field y_accumulator_identity = (T0 + T1) * alpha_d;

        T0 = w_4_eval - Field(1);
        T1 = T0 - w_3_eval;
        Field accumulator_init_identity = T0 * T1 * alpha_e;

        Field x_init_identity = -(w_1_eval * w_3_eval) * alpha_f;

        T0 = Field(1) - w_4_eval;
        T0 = T0 * q_c_eval;
        T1 = w_2_eval * w_3_eval;
        Field y_init_identity = (T0 - T1) * alpha_g;

        Field gate_identity = accumulator_init_identity + x_init_identity + y_init_identity;
        gate_identity = gate_identity * q_c_eval;
        gate_identity =
            gate_identity + accumulator_identity + x_alpha_identity + x_accumulator_identity + y_accumulator_identity;
        gate_identity = gate_identity * q_ecc_1_eval;

        t_eval = t_eval + gate_identity;

    } else {
        Field q_1_eval = transcript.get_field_element("q_1");
        Field q_2_eval = transcript.get_field_element("q_2");
        Field q_3_eval = transcript.get_field_element("q_3");
        Field q_4_eval = transcript.get_field_element("q_4");
        Field q_5_eval = transcript.get_field_element("q_5");
        Field q_m_eval = transcript.get_field_element("q_m");

        const Field minus_nine = -Field(9);
        const Field minus_one = -Field(1);

        Field accumulator_delta = w_4_eval + w_4_eval;
        accumulator_delta += accumulator_delta;
        accumulator_delta = w_4_omega_eval - accumulator_delta;

        Field accumulator_delta_squared = accumulator_delta.sqr();

        // y_alpha represents the point that we're adding into our accumulator point at the current round
        // q_3 and q_ecc_1 are selector polynomials that describe two different y-coordinates
        // the value of y-alpha is one of these two points, or their inverses
        // y_alpha = delta * (x_alpha * q_3 + q_ecc_1)
        // (we derive x_alpha from y_alpha, with `delta` conditionally flipping the sign of the output)
        // q_3 and q_ecc_1 are not directly equal to the 2 potential y-coordinates.
        // let's use `x_beta`, `x_gamma`, `y_beta`, `y_gamma` to refer to the two points in our lookup table
        // y_alpha = [(x_alpha - x_gamma) / (x_beta - x_gamma)].y_beta.delta + [(x_alpha - x_beta) / 3.(x_gamma -
        // x_beta)].y_gamma.delta
        // => q_3 = (3.y_beta - y_gamma) / 3.(x_beta - x_gamma)
        // => q_ecc_1 = (3.x_beta.y_gamma - x_gammay_beta) / 3.(x_beta - x_gammma)
        Field y_alpha = w_3_omega_eval * q_3_eval;
        y_alpha += q_ecc_1_eval;
        y_alpha *= accumulator_delta;

        Field T0 = accumulator_delta_squared + minus_one;
        Field T1 = accumulator_delta_squared + minus_nine;

        // scalar accumulator consistency check
        // (delta - 1)(delta - 3)(delta + 1)(delta + 3).q_ecc_1 = 0 mod Z_H
        Field scalar_accumulator_identity = T0 * T1;
        scalar_accumulator_identity *= alpha_a;

        // x_alpha consistency check
        // (delta^2.q_1 + q_2 - x_alpha).q_ecc = 0 mod Z_H
        // x_alpha is the x-coordinate of the point we're adding into our accumulator point.
        // We use a w_o(X) to track x_alpha, to reduce the number of required selector polynomials
        Field x_alpha_identity = accumulator_delta_squared * q_1_eval;
        x_alpha_identity += q_2_eval;
        x_alpha_identity -= w_3_omega_eval;
        x_alpha_identity *= alpha_b;

        // x-accumulator consistency check
        // ((x_2 + x_1 + x_alpha)(x_alpha - x_1)^2 - (y_alpha - y_1)^2).q_ecc = 0 mod Z_H
        // we use the fact that y_alpha^2 = x_alpha^3 + grumpkin::g1::element::curve_b
        Field x_alpha_minus_x_1 = w_3_omega_eval - (w_1_eval);

        T0 = y_alpha * w_2_eval;
        T0 += T0;

        T1 = x_alpha_minus_x_1.sqr();
        Field T2 = w_1_omega_eval + w_1_eval; // T1 = (x_alpha - x_1)^2
        T2 += w_3_omega_eval;                 // T2 = (x_2 + x_1 + x_alpha)
        T1 *= T2;
        T2 = w_2_eval.sqr(); // T1 = y_1^2
        T2 += grumpkin_curve_b;
        Field x_accumulator_identity = T0 + T1;
        x_accumulator_identity -= T2;
        T0 = w_3_omega_eval.sqr(); // y_alpha^2 = x_alpha^3 + b
        T0 *= w_3_omega_eval;
        x_accumulator_identity -= T0;
        x_accumulator_identity *= alpha_c;

        // y-accumulator consistency check
        // ((y_2 + y_1)(x_alpha - x_1) - (y_alpha - y_1)(x_1 - x_2)).q_ecc = 0 mod Z_H
        T0 = w_2_eval + w_2_omega_eval;
        T0 *= x_alpha_minus_x_1;

        T1 = y_alpha - w_2_eval;

        T2 = w_1_eval - w_1_omega_eval;
        T1 *= T2;

        Field y_accumulator_identity = T0 - T1;
        y_accumulator_identity *= alpha_d;

        // accumlulator-init consistency check
        // at the start of our scalar multiplication ladder, we want to validate that
        // the initial values of (x_1, y_1) and scalar accumulator a_1 are correctly set
        // We constrain a_1 to be either 0 or the value in w_o (which should be correctly initialized to (1 / 4^n) via a
        // copy constraint) We constraint (x_1, y_1) to be one of 4^n.[1] or (4^n + 1).[1]
        Field w_4_minus_one = w_4_eval + minus_one;
        T1 = w_4_minus_one - w_3_eval;
        Field accumulator_init_identity = w_4_minus_one * T1;
        accumulator_init_identity *= alpha_e;

        // // x-init consistency check
        T0 = q_4_eval - w_1_eval;
        T0 *= w_3_eval;
        T1 = w_4_minus_one * q_5_eval;
        Field x_init_identity = T0 - T1;
        x_init_identity *= alpha_f;

        // // y-init consistency check
        T0 = q_m_eval - w_2_eval;
        T0 *= w_3_eval;
        T1 = w_4_minus_one * q_c_eval;
        Field y_init_identity = T0 - T1;
        y_init_identity *= alpha_g;

        Field gate_identity = accumulator_init_identity + x_init_identity;
        gate_identity += y_init_identity;
        gate_identity *= q_c_eval;
        gate_identity += scalar_accumulator_identity;
        gate_identity += x_alpha_identity;
        gate_identity += x_accumulator_identity;
        gate_identity += y_accumulator_identity;
        gate_identity *= q_ecc_1_eval;

        t_eval += gate_identity;
    }
    return alpha_g * alpha;
}

template <typename Field, typename Group, typename Transcript>
void VerifierTurboFixedBaseWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key* key, Field& batch_eval, const Transcript& transcript, const bool use_linearisation)
{
    Field q_arith_eval = transcript.get_field_element("q_arith");
    Field q_ecc_1_eval = transcript.get_field_element("q_ecc_1");

    if (use_linearisation) {
        std::array<Field, 3> nu_challenges;
        nu_challenges[0] = transcript.get_challenge_field_element_from_map("nu", "q_arith");
        nu_challenges[1] = transcript.get_challenge_field_element_from_map("nu", "q_ecc_1");
        nu_challenges[2] = transcript.get_challenge_field_element_from_map("nu", "q_c");
        Field q_c_eval = transcript.get_field_element("q_c");

        Field T0 = q_arith_eval * nu_challenges[0];
        Field T1 = q_ecc_1_eval * nu_challenges[1];
        Field T2 = q_c_eval * nu_challenges[2];

        batch_eval = batch_eval + T0 + T1 + T2;

        return;
    }
    VerifierTurboArithmeticWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
        key, batch_eval, transcript, use_linearisation);
    Field nu_base = transcript.get_challenge_field_element_from_map("nu", "q_ecc_1");

    batch_eval += q_ecc_1_eval * nu_base;
}

template <typename Field, typename Group, typename Transcript>
Field VerifierTurboFixedBaseWidget<Field, Group, Transcript>::append_scalar_multiplication_inputs(
    verification_key* key,
    const Field& alpha_base,
    const Transcript& transcript,
    std::vector<Group>& points,
    std::vector<Field>& scalars,
    const bool use_linearisation)
{
    Field alpha_step = transcript.get_challenge_field_element("alpha");

    Field alpha_a = alpha_base * alpha_step.sqr();
    Field alpha_b = alpha_a * alpha_step;
    Field alpha_c = alpha_b * alpha_step;
    Field alpha_d = alpha_c * alpha_step;
    Field alpha_e = alpha_d * alpha_step;
    Field alpha_f = alpha_e * alpha_step;
    Field alpha_g = alpha_f * alpha_step;
    if (use_linearisation) {
        Field q_arith_eval = transcript.get_field_element("q_arith");
        Field q_ecc_1_eval = transcript.get_field_element("q_ecc_1");

        Field w_1_eval = transcript.get_field_element("w_1");
        Field w_2_eval = transcript.get_field_element("w_2");
        Field w_3_eval = transcript.get_field_element("w_3");
        Field w_4_eval = transcript.get_field_element("w_4");
        Field w_1_omega_eval = transcript.get_field_element("w_1_omega");
        Field w_3_omega_eval = transcript.get_field_element("w_3_omega");
        Field w_4_omega_eval = transcript.get_field_element("w_4_omega");

        Field q_c_eval = transcript.get_field_element("q_c");

        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");

        std::array<Field, 3> nu_challenges;
        nu_challenges[0] = transcript.get_challenge_field_element_from_map("nu", "q_arith");
        nu_challenges[1] = transcript.get_challenge_field_element_from_map("nu", "q_ecc_1");
        nu_challenges[2] = transcript.get_challenge_field_element_from_map("nu", "q_c");

        Field delta = w_4_omega_eval - (w_4_eval + w_4_eval + w_4_eval + w_4_eval);

        Field delta_squared = delta.sqr();

        Field q_l_term_ecc = delta_squared * q_ecc_1_eval * alpha_b;

        Field q_l_term_arith = w_1_eval * alpha_base * q_arith_eval;

        Field q_l_term = (q_l_term_arith + q_l_term_ecc) * linear_nu;
        if (key->constraint_selectors.at("Q_1").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_1"));
            scalars.push_back(q_l_term);
        }

        Field q_r_term_ecc = alpha_b * q_ecc_1_eval;

        Field q_r_term_arith = w_2_eval * alpha_base * q_arith_eval;

        Field q_r_term = (q_r_term_ecc + q_r_term_arith) * linear_nu;
        if (key->constraint_selectors.at("Q_2").on_curve()) {
            key->scalar_multiplication_indices.insert({ "Q_2", scalars.size() });
            points.push_back(key->constraint_selectors.at("Q_2"));
            scalars.push_back(q_r_term);
        }

        Field T0 = (w_1_omega_eval - w_1_eval) * delta * w_3_omega_eval * alpha_d;
        Field T1 = delta * w_3_omega_eval * w_2_eval;
        T1 = T1 + T1;
        T1 = T1 * alpha_c;

        Field q_o_term_ecc = (T0 + T1) * q_ecc_1_eval;
        T0 = w_1_omega_eval - w_1_eval;

        Field q_o_term_arith = w_3_eval * alpha_base * q_arith_eval;

        Field q_o_term = (q_o_term_ecc + q_o_term_arith) * linear_nu;
        if (key->constraint_selectors.at("Q_3").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_3"));
            scalars.push_back(q_o_term);
        }

        Field q_4_term_ecc = w_3_eval * q_ecc_1_eval * q_c_eval * alpha_f;

        Field q_4_term_arith = w_4_eval * alpha_base * q_arith_eval;

        Field q_4_term = (q_4_term_ecc + q_4_term_arith) * linear_nu;
        if (key->constraint_selectors.at("Q_4").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_4"));
            scalars.push_back(q_4_term);
        }

        Field q_5_term_ecc = (Field(1) - w_4_eval) * q_ecc_1_eval * q_c_eval * alpha_f;

        const Field minus_two = -Field(2);
        Field q_5_term_arith =
            (w_4_eval.sqr() - w_4_eval) * (w_4_eval + minus_two) * alpha_base * alpha_step * q_arith_eval;

        Field q_5_term = (q_5_term_ecc + q_5_term_arith) * linear_nu;
        if (key->constraint_selectors.at("Q_5").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_5"));
            scalars.push_back(q_5_term);
        }

        // Q_M term = w_l * w_r * alpha_base * nu
        Field q_m_term_ecc = w_3_eval * q_ecc_1_eval * q_c_eval * alpha_g;

        Field q_m_term_arith = w_1_eval * w_2_eval * alpha_base * q_arith_eval;

        Field q_m_term = (q_m_term_ecc + q_m_term_arith) * linear_nu;
        if (key->constraint_selectors.at("Q_M").on_curve()) {
            key->scalar_multiplication_indices.insert({ "Q_M", scalars.size() });
            points.push_back(key->constraint_selectors.at("Q_M"));
            scalars.push_back(q_m_term);
        }

        Field q_c_term = alpha_base * linear_nu * q_arith_eval;
        if (key->constraint_selectors.at("Q_C").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_C"));

            // TODO: ROLL ARITHMETIC EXPRESSION INVOLVING Q_C INTO BATCH EVALUATION OF T(X)
            q_c_term = q_c_term + nu_challenges[2];
            scalars.push_back(q_c_term);
        }

        if (key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR"));
            scalars.push_back(nu_challenges[0]);
        }

        if (key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR"));
            scalars.push_back((nu_challenges[1]));
        }

        return alpha_g * alpha_step;
    }

    std::array<Field, 9> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element_from_map("nu", "q_1");
    nu_challenges[1] = transcript.get_challenge_field_element_from_map("nu", "q_2");
    nu_challenges[2] = transcript.get_challenge_field_element_from_map("nu", "q_3");
    nu_challenges[3] = transcript.get_challenge_field_element_from_map("nu", "q_4");
    nu_challenges[4] = transcript.get_challenge_field_element_from_map("nu", "q_5");
    nu_challenges[5] = transcript.get_challenge_field_element_from_map("nu", "q_m");
    nu_challenges[6] = transcript.get_challenge_field_element_from_map("nu", "q_c");
    nu_challenges[7] = transcript.get_challenge_field_element_from_map("nu", "q_arith");
    nu_challenges[8] = transcript.get_challenge_field_element_from_map("nu", "q_ecc_1");

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
    if (key->constraint_selectors.at("Q_4").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_4"));
        scalars.push_back(nu_challenges[3]);
    }
    if (key->constraint_selectors.at("Q_5").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_5"));
        scalars.push_back(nu_challenges[4]);
    }
    if (key->constraint_selectors.at("Q_M").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_M"));
        scalars.push_back(nu_challenges[5]);
    }
    if (key->constraint_selectors.at("Q_C").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_C"));
        scalars.push_back(nu_challenges[6]);
    }
    if (key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR"));
        scalars.push_back(nu_challenges[7]);
    }
    if (key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR"));
        scalars.push_back(nu_challenges[8]);
    }

    return alpha_g * alpha_step;
}

template class VerifierTurboFixedBaseWidget<barretenberg::fr,
                                            barretenberg::g1::affine_element,
                                            transcript::StandardTranscript>;

} // namespace waffle