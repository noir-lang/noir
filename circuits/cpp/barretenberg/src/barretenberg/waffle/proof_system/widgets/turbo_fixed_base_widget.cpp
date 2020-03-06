#include "./turbo_fixed_base_widget.hpp"

#include "../../../curves/grumpkin/grumpkin.hpp"

#include "../../../curves/bn254/scalar_multiplication/scalar_multiplication.hpp"
#include "../../../polynomials/evaluation_domain.hpp"
#include "../../../transcript/transcript.hpp"
#include "../../../types.hpp"

#include "../transcript_helpers.hpp"

#include "../proving_key/proving_key.hpp"
#include "../verification_key/verification_key.hpp"

using namespace barretenberg;

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

fr ProverTurboFixedBaseWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
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

void ProverTurboFixedBaseWidget::compute_transcript_elements(transcript::Transcript& transcript)
{
    ProverTurboArithmeticWidget::compute_transcript_elements(transcript);
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_ecc_1",
                           transcript_helpers::convert_field_element(q_ecc_1.evaluate(z, key->small_domain.size)));
    transcript.add_element("q_c", transcript_helpers::convert_field_element(q_c.evaluate(z, key->small_domain.size)));
}

fr ProverTurboFixedBaseWidget::compute_linear_contribution(const fr& alpha_base,
                                                           const transcript::Transcript& transcript,
                                                           barretenberg::polynomial& r)
{
    fr new_alpha_base = ProverTurboArithmeticWidget::compute_linear_contribution(alpha_base, transcript, r);
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr w_l_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_1_omega")[0]);
    fr w_o_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);

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

    fr q_3_multiplicand = (w_l_omega_eval - w_l_eval) * delta * w_o_omega_eval * alpha_d * q_ecc_1_eval;
    fr T1 = delta * w_o_omega_eval * w_r_eval * alpha_c;
    q_3_multiplicand = q_3_multiplicand + (T1 + T1) * q_ecc_1_eval;

    fr q_4_multiplicand = w_o_eval * q_ecc_1_eval * q_c_eval * alpha_f;

    fr q_5_multiplicand = (fr::one() - w_4_eval) * q_ecc_1_eval * q_c_eval * alpha_f;

    fr q_m_multiplicand = w_o_eval * q_ecc_1_eval * q_c_eval * alpha_g;

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

fr ProverTurboFixedBaseWidget::compute_opening_poly_contribution(const fr& nu_base,
                                                                 const transcript::Transcript& transcript,
                                                                 fr* poly,
                                                                 fr* shifted_poly)
{
    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);
    fr new_nu_base =
        ProverTurboArithmeticWidget::compute_opening_poly_contribution(nu_base, transcript, poly, shifted_poly);
    fr nu_b = new_nu_base * nu;
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T0 = q_ecc_1[i] * new_nu_base;
    fr T1 = q_c[i] * nu_b;
    T0 += T1;
    poly[i] += T0;
    ITERATE_OVER_DOMAIN_END;

    return nu_b * nu;
}

// ###

VerifierTurboFixedBaseWidget::VerifierTurboFixedBaseWidget()
    : VerifierTurboArithmeticWidget()
{}

barretenberg::fr VerifierTurboFixedBaseWidget::compute_quotient_evaluation_contribution(
    verification_key* key, const fr& alpha_base, const transcript::Transcript& transcript, fr& t_eval)
{
    fr new_alpha_base =
        VerifierTurboArithmeticWidget::compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval);
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr w_l_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_1_omega")[0]);
    fr w_r_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_2_omega")[0]);
    fr w_o_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);
    fr w_4_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_4_omega")[0]);

    fr q_ecc_1_eval = fr::serialize_from_buffer(&transcript.get_element("q_ecc_1")[0]);
    fr q_c_eval = fr::serialize_from_buffer(&transcript.get_element("q_c")[0]);

    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr alpha_a = new_alpha_base;
    fr alpha_b = alpha_a * alpha;
    fr alpha_c = alpha_b * alpha;
    fr alpha_d = alpha_c * alpha;
    fr alpha_e = alpha_d * alpha;
    fr alpha_f = alpha_e * alpha;
    fr alpha_g = alpha_f * alpha;

    fr delta = w_4_omega_eval - (w_4_eval + w_4_eval + w_4_eval + w_4_eval);

    constexpr fr three = fr{ 3, 0, 0, 0 }.to_montgomery_form();

    fr T1 = (delta + fr::one());
    fr T2 = (delta + three);
    fr T3 = (delta - fr::one());
    fr T4 = (delta - three);

    fr accumulator_identity = T1 * T2 * T3 * T4 * alpha_a;

    fr x_alpha_identity = -(w_o_omega_eval * alpha_b);

    fr T0 = w_l_omega_eval + w_l_eval + w_o_omega_eval;
    T1 = (w_o_omega_eval - w_l_eval).sqr();
    T0 = T0 * T1;

    T1 = w_o_omega_eval.sqr() * w_o_omega_eval;
    T2 = w_r_eval.sqr();
    T1 = T1 + T2;
    T1 = -(T1 + grumpkin::g1::element::curve_b);

    T2 = delta * w_r_eval * q_ecc_1_eval;
    T2 = T2 + T2;

    fr x_accumulator_identity = (T0 + T1 + T2) * alpha_c;

    T0 = (w_r_omega_eval + w_r_eval) * (w_o_omega_eval - w_l_eval);

    T1 = w_l_eval - w_l_omega_eval;
    T2 = w_r_eval - (q_ecc_1_eval * delta);
    T1 = T1 * T2;

    fr y_accumulator_identity = (T0 + T1) * alpha_d;

    T0 = w_4_eval - fr::one();
    T1 = T0 - w_o_eval;
    fr accumulator_init_identity = T0 * T1 * alpha_e;

    fr x_init_identity = -(w_l_eval * w_o_eval) * alpha_f;

    T0 = fr::one() - w_4_eval;
    T0 = T0 * q_c_eval;
    T1 = w_r_eval * w_o_eval;
    fr y_init_identity = (T0 - T1) * alpha_g;

    fr gate_identity = accumulator_init_identity + x_init_identity + y_init_identity;
    gate_identity = gate_identity * q_c_eval;
    gate_identity =
        gate_identity + accumulator_identity + x_alpha_identity + x_accumulator_identity + y_accumulator_identity;
    gate_identity = gate_identity * q_ecc_1_eval;

    t_eval = t_eval + gate_identity;

    return alpha_g * alpha;
}

barretenberg::fr VerifierTurboFixedBaseWidget::compute_batch_evaluation_contribution(
    verification_key*,
    barretenberg::fr& batch_eval,
    const barretenberg::fr& nu_base,
    const transcript::Transcript& transcript)
{
    fr q_c_eval = fr::serialize_from_buffer(&transcript.get_element("q_c")[0]);
    fr q_arith_eval = fr::serialize_from_buffer(&transcript.get_element("q_arith")[0]);
    fr q_ecc_1_eval = fr::serialize_from_buffer(&transcript.get_element("q_ecc_1")[0]);

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    fr nu_a = nu_base * nu;
    fr nu_b = nu_a * nu;

    fr T0 = q_arith_eval * nu_base;
    fr T1 = q_ecc_1_eval * nu_a;
    fr T2 = q_c_eval * nu_b;

    batch_eval = batch_eval + T0 + T1 + T2;

    return nu_b * nu;
}

VerifierBaseWidget::challenge_coefficients VerifierTurboFixedBaseWidget::append_scalar_multiplication_inputs(
    verification_key* key,
    const challenge_coefficients& challenge,
    const transcript::Transcript& transcript,
    std::vector<barretenberg::g1::affine_element>& points,
    std::vector<barretenberg::fr>& scalars)
{
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr w_l_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_1_omega")[0]);
    fr w_o_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);
    fr w_4_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_4_omega")[0]);

    fr q_arith_eval = fr::serialize_from_buffer(&transcript.get_element("q_arith")[0]);
    fr q_ecc_1_eval = fr::serialize_from_buffer(&transcript.get_element("q_ecc_1")[0]);
    fr q_c_eval = fr::serialize_from_buffer(&transcript.get_element("q_c")[0]);

    fr alpha_a = challenge.alpha_base * challenge.alpha_step.sqr();
    fr alpha_b = alpha_a * challenge.alpha_step;
    fr alpha_c = alpha_b * challenge.alpha_step;
    fr alpha_d = alpha_c * challenge.alpha_step;
    fr alpha_e = alpha_d * challenge.alpha_step;
    fr alpha_f = alpha_e * challenge.alpha_step;
    fr alpha_g = alpha_f * challenge.alpha_step;

    fr delta = w_4_omega_eval - (w_4_eval + w_4_eval + w_4_eval + w_4_eval);

    fr delta_squared = delta.sqr();

    fr q_l_term_ecc = delta_squared * q_ecc_1_eval * alpha_b;

    fr q_l_term_arith = w_l_eval * challenge.alpha_base * q_arith_eval;

    fr q_l_term = (q_l_term_arith + q_l_term_ecc) * challenge.linear_nu;
    if (key->constraint_selectors.at("Q_1").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_1"));
        scalars.push_back(q_l_term);
    }

    fr q_r_term_ecc = alpha_b * q_ecc_1_eval;

    fr q_r_term_arith = w_r_eval * challenge.alpha_base * q_arith_eval;

    fr q_r_term = (q_r_term_ecc + q_r_term_arith) * challenge.linear_nu;
    if (key->constraint_selectors.at("Q_2").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_2"));
        scalars.push_back(q_r_term);
    }

    fr T0 = (w_l_omega_eval - w_l_eval) * delta * w_o_omega_eval * alpha_d;
    fr T1 = delta * w_o_omega_eval * w_r_eval;
    T1 = T1 + T1;
    T1 = T1 * alpha_c;

    fr q_o_term_ecc = (T0 + T1) * q_ecc_1_eval;
    T0 = w_l_omega_eval - w_l_eval;

    fr q_o_term_arith = w_o_eval * challenge.alpha_base * q_arith_eval;

    fr q_o_term = (q_o_term_ecc + q_o_term_arith) * challenge.linear_nu;
    if (key->constraint_selectors.at("Q_3").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_3"));
        scalars.push_back(q_o_term);
    }

    fr q_4_term_ecc = w_o_eval * q_ecc_1_eval * q_c_eval * alpha_f;

    fr q_4_term_arith = w_4_eval * challenge.alpha_base * q_arith_eval;

    fr q_4_term = (q_4_term_ecc + q_4_term_arith) * challenge.linear_nu;
    if (key->constraint_selectors.at("Q_4").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_4"));
        scalars.push_back(q_4_term);
    }

    fr q_5_term_ecc = (fr::one() - w_4_eval) * q_ecc_1_eval * q_c_eval * alpha_f;

    constexpr fr minus_two = -fr(2);
    fr q_5_term_arith = (w_4_eval.sqr() - w_4_eval) * (w_4_eval + minus_two) * challenge.alpha_base *
                        challenge.alpha_step * q_arith_eval;

    fr q_5_term = (q_5_term_ecc + q_5_term_arith) * challenge.linear_nu;
    if (key->constraint_selectors.at("Q_5").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_5"));
        scalars.push_back(q_5_term);
    }

    // Q_M term = w_l * w_r * challenge.alpha_base * nu
    fr q_m_term_ecc = w_o_eval * q_ecc_1_eval * q_c_eval * alpha_g;

    fr q_m_term_arith = w_l_eval * w_r_eval * challenge.alpha_base * q_arith_eval;

    fr q_m_term = (q_m_term_ecc + q_m_term_arith) * challenge.linear_nu;
    if (key->constraint_selectors.at("Q_M").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_M"));
        scalars.push_back(q_m_term);
    }

    fr q_c_term = challenge.alpha_base * challenge.linear_nu * q_arith_eval;
    if (key->constraint_selectors.at("Q_C").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_C"));

        // TODO: ROLL ARITHMETIC EXPRESSION INVOLVING Q_C INTO BATCH EVALUATION OF T(X)
        fr blah_nu = challenge.nu_base * challenge.nu_step.sqr();
        q_c_term = q_c_term + blah_nu;
        scalars.push_back(q_c_term);
    }

    if (key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR"));
        scalars.push_back(challenge.nu_base);
    }

    if (key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR"));
        scalars.push_back((challenge.nu_base * challenge.nu_step));
    }

    return VerifierBaseWidget::challenge_coefficients{ alpha_g * challenge.alpha_step,
                                                       challenge.alpha_step,
                                                       challenge.nu_base * challenge.nu_step.sqr() * challenge.nu_step,
                                                       challenge.nu_step,
                                                       challenge.linear_nu };
}
} // namespace waffle