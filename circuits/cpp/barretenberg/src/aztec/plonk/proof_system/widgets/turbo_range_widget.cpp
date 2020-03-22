#include "turbo_range_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverTurboRangeWidget::ProverTurboRangeWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_range(key->constraint_selectors.at("q_range"))
    , q_range_fft(key->constraint_selector_ffts.at("q_range_fft"))
{}

ProverTurboRangeWidget::ProverTurboRangeWidget(const ProverTurboRangeWidget& other)
    : ProverBaseWidget(other)
    , q_range(key->constraint_selectors.at("q_range"))
    , q_range_fft(key->constraint_selector_ffts.at("q_range_fft"))
{}

ProverTurboRangeWidget::ProverTurboRangeWidget(ProverTurboRangeWidget&& other)
    : ProverBaseWidget(other)
    , q_range(key->constraint_selectors.at("q_range"))
    , q_range_fft(key->constraint_selector_ffts.at("q_range_fft"))
{}

ProverTurboRangeWidget& ProverTurboRangeWidget::operator=(const ProverTurboRangeWidget& other)
{
    ProverBaseWidget::operator=(other);
    q_range = key->constraint_selectors.at("q_range");
    q_range_fft = key->constraint_selector_ffts.at("q_range_fft");
    return *this;
}

ProverTurboRangeWidget& ProverTurboRangeWidget::operator=(ProverTurboRangeWidget&& other)
{
    ProverBaseWidget::operator=(other);
    q_range = key->constraint_selectors.at("q_range");
    q_range_fft = key->constraint_selector_ffts.at("q_range_fft");
    return *this;
}

/*
 * The range constraint accumulates base 4 values into a sum.
 * We do this by evaluating a kind of 'raster scan', where we compare adjacent elements
 * and validate that their differences map to a base for value  *
 * Let's say that we want to perform a 32-bit range constraint in 'x'.
 * We can represent x via 16 constituent base-4 'quads' {q_0, ..., q_15}:
 *
 *      15
 *      ===
 *      \          i
 * x =  /    q  . 4
 *      ===   i
 *     i = 0
 *
 * In program memory, we place an accumulating base-4 sum of x {a_0, ..., a_15}, where
 *
 *         i
 *        ===
 *        \                  j
 * a   =  /    q         .  4
 *  i     ===   (15 - j)
 *       j = 0
 *
 *
 * From this, we can use our range transition constraint to validate that
 *
 *
 *  a      - 4 . a  ϵ [0, 1, 2, 3]
 *   i + 1        i
 *
 *
 * We place our accumulating sums in program memory in the following sequence:
 *
 * +-----+-----+-----+-----+
 * |  A  |  B  |  C  |  D  |
 * +-----+-----+-----+-----+
 * | a3  | a2  | a1  | 0   |
 * | a7  | a6  | a5  | a4  |
 * | a11 | a10 | a9  | a8  |
 * | a15 | a14 | a13 | a12 |
 * | --- | --- | --- | a16 |
 * +-----+-----+-----+-----+
 *
 * Our range transition constraint on row 'i'
 * performs our base-4 range check on the follwing pairs:
 *
 * (D_{i}, C_{i}), (C_{i}, B_{i}), (B_{i}, A_{i}), (A_{i}, D_{i+1})
 *
 * We need to start our raster scan at zero, so we simplify matters and just force the first value
 * to be zero.
 *
 * The output will be in the 4th column of an otherwise unused row. Assuming this row can
 * be used for a width-3 standard gate, the total number of gates for an n-bit range constraint
 * is (n / 8) gates
 *
 **/
fr ProverTurboRangeWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                         const transcript::Transcript& transcript)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    fr alpha_a = alpha_base;
    fr alpha_b = alpha_a * alpha;
    fr alpha_c = alpha_b * alpha;
    fr alpha_d = alpha_c * alpha;

    fr* w_1_fft = &key->wire_ffts.at("w_1_fft")[0];
    fr* w_2_fft = &key->wire_ffts.at("w_2_fft")[0];
    fr* w_3_fft = &key->wire_ffts.at("w_3_fft")[0];
    fr* w_4_fft = &key->wire_ffts.at("w_4_fft")[0];

    fr* quotient_large = &key->quotient_large[0];

    constexpr fr minus_two = -fr(2);
    constexpr fr minus_three = -fr(3);

    ITERATE_OVER_DOMAIN_START(key->large_domain);

    fr delta_1 = w_4_fft[i] + w_4_fft[i];
    delta_1 += delta_1;
    delta_1 = w_3_fft[i] - delta_1;

    fr delta_2 = w_3_fft[i] + w_3_fft[i];
    delta_2 += delta_2;
    delta_2 = w_2_fft[i] - delta_2;

    fr delta_3 = w_2_fft[i] + w_2_fft[i];
    delta_3 += delta_3;
    delta_3 = w_1_fft[i] - delta_3;

    fr delta_4 = w_1_fft[i] + w_1_fft[i];
    delta_4 += delta_4;
    delta_4 = w_4_fft[i + 4] - delta_4;

    // D(D - 1)(D - 2)(D - 3).alpha
    fr T0 = delta_1.sqr();
    T0 -= delta_1;
    fr T1 = delta_1 + minus_two;
    T0 *= T1;
    T1 = delta_1 + minus_three;
    T0 *= T1;
    fr range_accumulator = T0 * alpha_a;

    T0 = delta_2.sqr();
    T0 -= delta_2;
    T1 = delta_2 + minus_two;
    T0 *= T1;
    T1 = delta_2 + minus_three;
    T0 *= T1;
    T0 *= alpha_b;
    range_accumulator += T0;

    T0 = delta_3.sqr();
    T0 -= delta_3;
    T1 = delta_3 + minus_two;
    T0 *= T1;
    T1 = delta_3 + minus_three;
    T0 *= T1;
    T0 *= alpha_c;
    range_accumulator += T0;

    T0 = delta_4.sqr();
    T0 -= delta_4;
    T1 = delta_4 + minus_two;
    T0 *= T1;
    T1 = delta_4 + minus_three;
    T0 *= T1;
    T0 *= alpha_d;
    range_accumulator += T0;

    range_accumulator *= q_range_fft[i];
    quotient_large[i] += range_accumulator;
    ITERATE_OVER_DOMAIN_END;

    return alpha_d * alpha;
}

void ProverTurboRangeWidget::compute_transcript_elements(transcript::Transcript& transcript,
                                                         const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_range", q_range.evaluate(z, key->small_domain.size).to_buffer());
}

fr ProverTurboRangeWidget::compute_linear_contribution(const fr& alpha_base,
                                                       const transcript::Transcript& transcript,
                                                       barretenberg::polynomial& r)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr w_1_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_2_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_3_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_4_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_4_omega")[0]);

    constexpr fr minus_two = -fr(2);
    constexpr fr minus_three = -fr(3);

    fr alpha_a = alpha_base;
    fr alpha_b = alpha_a * alpha;
    fr alpha_c = alpha_b * alpha;
    fr alpha_d = alpha_c * alpha;

    fr delta_1 = w_4_eval + w_4_eval;
    delta_1 += delta_1;
    delta_1 = w_3_eval - delta_1;

    fr delta_2 = w_3_eval + w_3_eval;
    delta_2 += delta_2;
    delta_2 = w_2_eval - delta_2;

    fr delta_3 = w_2_eval + w_2_eval;
    delta_3 += delta_3;
    delta_3 = w_1_eval - delta_3;

    fr delta_4 = w_1_eval + w_1_eval;
    delta_4 += delta_4;
    delta_4 = w_4_omega_eval - delta_4;

    // D(D - 1)(D - 2)(D - 3).alpha
    fr T0 = delta_1.sqr();
    T0 -= delta_1;
    fr T1 = delta_1 + minus_two;
    T0 *= T1;
    T1 = delta_1 + minus_three;
    T0 *= T1;
    fr range_accumulator = T0 * alpha_a;

    T0 = delta_2.sqr();
    T0 -= delta_2;
    T1 = delta_2 + minus_two;
    T0 *= T1;
    T1 = delta_2 + minus_three;
    T0 *= T1;
    T0 *= alpha_b;
    range_accumulator += T0;

    T0 = delta_3.sqr();
    T0 -= delta_3;
    T1 = delta_3 + minus_two;
    T0 *= T1;
    T1 = delta_3 + minus_three;
    T0 *= T1;
    T0 *= alpha_c;
    range_accumulator += T0;

    T0 = delta_4.sqr();
    T0 -= delta_4;
    T1 = delta_4 + minus_two;
    T0 *= T1;
    T1 = delta_4 + minus_three;
    T0 *= T1;
    T0 *= alpha_d;
    range_accumulator += T0;

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] += (range_accumulator * q_range[i]);
    ITERATE_OVER_DOMAIN_END;

    return alpha_d * alpha;
}

size_t ProverTurboRangeWidget::compute_opening_poly_contribution(
    const size_t nu_index, const transcript::Transcript& transcript, fr* poly, fr*, const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_index;
    }

    fr nu_base = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index)[0]);

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_range[i] * nu_base);
    ITERATE_OVER_DOMAIN_END;

    return nu_index + 1;
}

// ###

template <typename Field, typename Group, typename Transcript>
VerifierTurboRangeWidget<Field, Group, Transcript>::VerifierTurboRangeWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierTurboRangeWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*,
    const Field& alpha_base,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{
    Field alpha = transcript.get_challenge_field_element("alpha");

    if (use_linearisation) {
        return alpha_base * alpha.sqr().sqr();
    }

    Field alpha_a = alpha_base;
    Field alpha_b = alpha_a * alpha;
    Field alpha_c = alpha_b * alpha;
    Field alpha_d = alpha_c * alpha;

    Field w_1_eval = transcript.get_field_element("w_1");
    Field w_2_eval = transcript.get_field_element("w_2");
    Field w_3_eval = transcript.get_field_element("w_3");
    Field w_4_eval = transcript.get_field_element("w_4");
    Field w_4_omega_eval = transcript.get_field_element("w_4_omega");

    Field q_range_eval = transcript.get_field_element("q_range");

    constexpr Field minus_two = -Field(2);
    constexpr Field minus_three = -Field(3);

    Field delta_1 = w_4_eval + w_4_eval;
    delta_1 += delta_1;
    delta_1 = w_3_eval - delta_1;

    Field delta_2 = w_3_eval + w_3_eval;
    delta_2 += delta_2;
    delta_2 = w_2_eval - delta_2;

    Field delta_3 = w_2_eval + w_2_eval;
    delta_3 += delta_3;
    delta_3 = w_1_eval - delta_3;

    Field delta_4 = w_1_eval + w_1_eval;
    delta_4 += delta_4;
    delta_4 = w_4_omega_eval - delta_4;

    // D(D - 1)(D - 2)(D - 3).alpha
    Field T0 = delta_1.sqr();
    T0 -= delta_1;
    Field T1 = delta_1 + minus_two;
    T0 *= T1;
    T1 = delta_1 + minus_three;
    T0 *= T1;
    Field range_accumulator = T0 * alpha_a;

    T0 = delta_2.sqr();
    T0 -= delta_2;
    T1 = delta_2 + minus_two;
    T0 *= T1;
    T1 = delta_2 + minus_three;
    T0 *= T1;
    T0 *= alpha_b;
    range_accumulator += T0;

    T0 = delta_3.sqr();
    T0 -= delta_3;
    T1 = delta_3 + minus_two;
    T0 *= T1;
    T1 = delta_3 + minus_three;
    T0 *= T1;
    T0 *= alpha_c;
    range_accumulator += T0;

    T0 = delta_4.sqr();
    T0 -= delta_4;
    T1 = delta_4 + minus_two;
    T0 *= T1;
    T1 = delta_4 + minus_three;
    T0 *= T1;
    T0 *= alpha_d;
    range_accumulator += T0;

    range_accumulator *= q_range_eval;

    t_eval += range_accumulator;

    return alpha_d * alpha;
}

template <typename Field, typename Group, typename Transcript>
size_t VerifierTurboRangeWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*,
    Field& batch_eval,
    const size_t nu_index,
    const Transcript& transcript,
    const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_index;
    }

    Field q_range_eval = transcript.get_field_element("q_range");

    Field nu_base = transcript.get_challenge_field_element("nu", nu_index);

    batch_eval += (q_range_eval * nu_base);

    return nu_index + 1;
}

template <typename Field, typename Group, typename Transcript>
VerifierBaseWidget::challenge_coefficients<Field> VerifierTurboRangeWidget<Field, Group, Transcript>::
    append_scalar_multiplication_inputs(verification_key* key,
                                        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
                                        const Transcript& transcript,
                                        std::vector<Group>& points,
                                        std::vector<Field>& scalars,
                                        const bool use_linearisation)
{
    if (use_linearisation) {
        Field w_4_eval = transcript.get_field_element("w_4");
        Field w_1_eval = transcript.get_field_element("w_1");
        Field w_2_eval = transcript.get_field_element("w_2");
        Field w_3_eval = transcript.get_field_element("w_3");
        Field w_4_omega_eval = transcript.get_field_element("w_4_omega");

        Field linear_nu = transcript.get_challenge_field_element("nu", challenge.linear_nu_index);

        constexpr Field minus_two = -Field(2);
        constexpr Field minus_three = -Field(3);

        Field alpha_a = challenge.alpha_base;
        Field alpha_b = alpha_a * challenge.alpha_step;
        Field alpha_c = alpha_b * challenge.alpha_step;
        Field alpha_d = alpha_c * challenge.alpha_step;

        Field delta_1 = w_4_eval + w_4_eval;
        delta_1 += delta_1;
        delta_1 = w_3_eval - delta_1;

        Field delta_2 = w_3_eval + w_3_eval;
        delta_2 += delta_2;
        delta_2 = w_2_eval - delta_2;

        Field delta_3 = w_2_eval + w_2_eval;
        delta_3 += delta_3;
        delta_3 = w_1_eval - delta_3;

        Field delta_4 = w_1_eval + w_1_eval;
        delta_4 += delta_4;
        delta_4 = w_4_omega_eval - delta_4;

        // D(D - 1)(D - 2)(D - 3).alpha
        Field T0 = delta_1.sqr();
        T0 -= delta_1;
        Field T1 = delta_1 + minus_two;
        T0 *= T1;
        T1 = delta_1 + minus_three;
        T0 *= T1;
        Field range_accumulator = T0 * alpha_a;

        T0 = delta_2.sqr();
        T0 -= delta_2;
        T1 = delta_2 + minus_two;
        T0 *= T1;
        T1 = delta_2 + minus_three;
        T0 *= T1;
        T0 *= alpha_b;
        range_accumulator += T0;

        T0 = delta_3.sqr();
        T0 -= delta_3;
        T1 = delta_3 + minus_two;
        T0 *= T1;
        T1 = delta_3 + minus_three;
        T0 *= T1;
        T0 *= alpha_c;
        range_accumulator += T0;

        T0 = delta_4.sqr();
        T0 -= delta_4;
        T1 = delta_4 + minus_two;
        T0 *= T1;
        T1 = delta_4 + minus_three;
        T0 *= T1;
        T0 *= alpha_d;
        range_accumulator += T0;

        range_accumulator *= linear_nu;

        if (key->constraint_selectors.at("Q_RANGE_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_RANGE_SELECTOR"));
            scalars.push_back(range_accumulator);
        }

        return VerifierBaseWidget::challenge_coefficients<Field>{
            alpha_d * challenge.alpha_step, challenge.alpha_step, challenge.nu_index, challenge.linear_nu_index
        };
    }
    Field nu_base = transcript.get_challenge_field_element("nu", challenge.nu_index);
    if (key->constraint_selectors.at("Q_RANGE_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_RANGE_SELECTOR"));
        scalars.push_back(nu_base);
    }
    Field alpha_a = challenge.alpha_base;
    Field alpha_b = alpha_a * challenge.alpha_step;
    Field alpha_c = alpha_b * challenge.alpha_step;
    Field alpha_d = alpha_c * challenge.alpha_step;

    return VerifierBaseWidget::challenge_coefficients<Field>{
        alpha_d * challenge.alpha_step, challenge.alpha_step, challenge.nu_index + 1, challenge.linear_nu_index
    };
}

template class VerifierTurboRangeWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>;
} // namespace waffle