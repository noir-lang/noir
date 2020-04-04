#include "turbo_logic_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverTurboLogicWidget::ProverTurboLogicWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_logic(key->constraint_selectors.at("q_logic"))
    , q_logic_fft(key->constraint_selector_ffts.at("q_logic_fft"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
{}

ProverTurboLogicWidget::ProverTurboLogicWidget(const ProverTurboLogicWidget& other)
    : ProverBaseWidget(other)
    , q_logic(key->constraint_selectors.at("q_logic"))
    , q_logic_fft(key->constraint_selector_ffts.at("q_logic_fft"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
{}

ProverTurboLogicWidget::ProverTurboLogicWidget(ProverTurboLogicWidget&& other)
    : ProverBaseWidget(other)
    , q_logic(key->constraint_selectors.at("q_logic"))
    , q_logic_fft(key->constraint_selector_ffts.at("q_logic_fft"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
{}

ProverTurboLogicWidget& ProverTurboLogicWidget::operator=(const ProverTurboLogicWidget& other)
{
    ProverBaseWidget::operator=(other);
    q_c = key->constraint_selectors.at("q_c");
    q_c_fft = key->constraint_selector_ffts.at("q_c_fft");
    q_logic = key->constraint_selectors.at("q_logic");
    q_logic_fft = key->constraint_selector_ffts.at("q_logic_fft");
    return *this;
}

ProverTurboLogicWidget& ProverTurboLogicWidget::operator=(ProverTurboLogicWidget&& other)
{
    ProverBaseWidget::operator=(other);
    q_c = key->constraint_selectors.at("q_c");
    q_c_fft = key->constraint_selector_ffts.at("q_c_fft");
    q_logic = key->constraint_selectors.at("q_logic");
    q_logic_fft = key->constraint_selector_ffts.at("q_logic_fft");
    return *this;
}

/*
 * Hoo boy, AND and XOR polynomials!
 * This transition constraint evaluates either an AND or an XOR relationship (but not an or in sight) between the
 *accumulating sums of three base-4 variables...
 *
 * Ok, so we want to evaluate a ^ b = c OR a & b = c . We can create a | b from a | b = (a ^ b) + (a & b)
 *
 * We also want the output memory cell to represent the actual result of the AND / XOR operation,
 * instead of a collection of bits / quads that need to be summed together. Who has time for that?
 *
 * We can also be super sneaky and evaluate both AND and XOR operations with a single selector polynomial.
 *
 * Let's call this selector 'S', it takes values in { -1, 0, 1}
 *
 * If S = -1, we're evaluating a XOR op
 * If S = 1, we're evaluating an AND op
 * If S = 0, we're evaluating nothing! This constraint is turned off
 *
 * We use 3 columns of program memory to represent accumulating sums of a, b, c.
 *
 * For example, we can represent a 32-bit 'A' via its quads
 *
 *      15
 *      ===
 *      \          i
 * A =  /    a  . 4
 *      ===   i
 *     i = 0
 *
 * In program memory, we place an accumulating base-4 sum of A {A_0, ..., A_15}, where
 *
 *         i
 *        ===
 *        \                  j
 * A   =  /    a         .  4
 *  i     ===   (15 - j)
 *       j = 0
 *
 *
 * From this, we can extract a quad by validating that
 *
 *
 *  A      - 4 . A  ϵ [0, 1, 2, 3]
 *   i + 1        i
 *
 * Once we have validated the above, we can then extract an accumulator's implicit quad via:
 *
 *  a  =  A      - 4 . A  ϵ [0, 1, 2, 3]
 *   i     i + 1        i
 *
 *
 * But of course it's not so simple! An AND/XOR polynomial identity with two input quads (plus selector) has a degree
 *of 8. To constrain the degree of our quotient polynomial T(X) we want our identity to have a degree of 5
 *
 * We also have a spare column to work with, which we can use to store
 *
 *
 *  w = a  * b
 *       i    i
 *
 * For the polynomial identity, we use the following notation:
 *
 *  (1) 'a' is the current round quad attributed to our operand a
 *  (2) 'b' is the current round quad attributed to our operand b
 *  (3) 'c' is the current round quad attributed to our output c
 *  (4) 'w' = a * b
 *  (5) 's' is the AND/XOR selector polynomial round value.
 *
 * The polynomial identity we're evaluating is... wait for it...
 *
 *                                                                                             2    2
 * s ⋅ (s ⋅ (9 ⋅ c - 3 ⋅ (a + b)) + 3 ⋅ (c + a + b) + w ⋅ (w ⋅ (4 ⋅ w - 18 ⋅ (a + b) + 81) + 18 ⋅ (a  + b ) - 81 ⋅ (a +
 *b) + 83))
 *
 * =
 *
 * 0 mod Z_H
 *
 * To simplify things, we *could* frankenstein integers out of the 4th roots of unity to make this simpler,
 * but then integer multiplication would be horrible.
 * So really, it's a question of picking ones poison, and blaming the Babylonians
 * for creating their number system out of the integers instead of a nice cyclic group.
 *
 * In addition to this nonsense, we also need to verify the following:
 *
 *  (1) a is in the set { 0, 1, 2, 3 }
 *  (2) b is in the set { 0, 1, 2, 3 }
 *  (3) c is in the set { 0, 1, 2, 3 }
 *  (4) w = a * c
 *
 *
 * We place our accumulating sums (A, B, C) in program memory in the following sequence:
 *
 *                  +-----+-----+-----+-----+
 *                  |  1  |  2  |  3  |  4  |
 *                  +-----+-----+-----+-----+
 * you are here --> | 0   | 0   | w1  | 0   |
 *                  | A1  | B1  | w2  | C1  |
 *                  | A2  | B2  | w3  | C2  |
 *                  | ... | ... | ... | ... |
 *                  | An  | Bn  | --- | Cn  | --> exit
 *                  +-----+-----+-----+-----+
 *
 *
 **/
fr ProverTurboLogicWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
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

    constexpr fr six = fr{ 6, 0, 0, 0 }.to_montgomery_form();
    constexpr fr eighty_one = fr{ 81, 0, 0, 0 }.to_montgomery_form();
    constexpr fr eighty_three = fr{ 83, 0, 0, 0 }.to_montgomery_form();

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
        size_t start = j * key->large_domain.thread_size;
        size_t end = (j + 1) * key->large_domain.thread_size;

        fr delta_sum;
        fr delta_squared_sum;
        fr T0;
        fr T1;
        fr T2;
        fr T3;
        fr T4;
        fr identity;
        for (size_t i = start; i < end; ++i) {
            // T0 = a
            T0 = w_1_fft[i] + w_1_fft[i];
            T0 += T0;
            T0 = w_1_fft[i + 4] - T0;

            // T1 = b
            T1 = w_2_fft[i] + w_2_fft[i];
            T1 += T1;
            T1 = w_2_fft[i + 4] - T1;

            // delta_sum = a + b
            delta_sum = T0 + T1;

            // T2 = a^2, T3 = b^2
            T2 = T0.sqr();
            T3 = T1.sqr();

            delta_squared_sum = T2 + T3;

            // identity = a^2 + b^2 + 2ab
            identity = delta_sum.sqr();
            // identity = 2ab
            identity -= delta_squared_sum;

            // identity = 2(ab - w)
            T4 = w_3_fft[i] + w_3_fft[i];
            identity -= T4;
            identity *= alpha;

            // T4 = 4w
            T4 += T4;

            // T2 = a^2 - a
            T2 -= T0;

            // T0 = a^2 - 5a + 6
            T0 += T0;
            T0 += T0;
            T0 = T2 - T0;
            T0 += six;

            // identity = (identity + a(a - 1)(a - 2)(a - 3)) * alpha
            T0 *= T2;
            identity += T0;
            identity *= alpha;

            // T3 = b^2 - b
            T3 -= T1;

            // T1 = b^2 - 5b + 6
            T1 += T1;
            T1 += T1;
            T1 = T3 - T1;
            T1 += six;

            // identity = (identity + b(b - 1)(b - 2)(b - 3)) * alpha
            T1 *= T3;
            identity += T1;
            identity *= alpha;

            // T0 = 3(a + b)
            T0 = delta_sum + delta_sum;
            T0 += delta_sum;

            // T1 = 9(a + b)
            T1 = T0 + T0;
            T1 += T0;

            // delta_sum = 18(a + b)
            delta_sum = T1 + T1;

            // T1 = 81(a + b)
            T2 = delta_sum + delta_sum;
            T2 += T2;
            T1 += T2;

            // delta_squared_sum = 18(a^2 + b^2)
            T2 = delta_squared_sum + delta_squared_sum;
            T2 += delta_squared_sum;
            delta_squared_sum = T2 + T2;
            delta_squared_sum += T2;
            delta_squared_sum += delta_squared_sum;

            // delta_sum = w(4w - 18(a + b) + 81)
            delta_sum = T4 - delta_sum;
            delta_sum += eighty_one;
            delta_sum *= w_3_fft[i];

            // T1 = 18(a^2 + b^2) - 81(a + b) + 83
            T1 = delta_squared_sum - T1;
            T1 += eighty_three;

            // delta_sum = w ( w ( 4w - 18(a + b) + 81) + 18(a^2 + b^2) - 81(a + b) + 83)
            delta_sum += T1;
            delta_sum *= w_3_fft[i];

            // T2 = 3c
            T2 = w_4_fft[i] + w_4_fft[i];
            T2 += T2;
            T2 = w_4_fft[i + 4] - T2;
            T3 = T2 + T2;
            T2 += T3;

            // T3 = 9c
            T3 = T2 + T2;
            T3 += T2;

            // T3 = q_c * (9c - 3(a + b))
            T3 -= T0;
            T3 *= q_c_fft[i];

            // T2 = 3c + 3(a + b) - 2 * delta_sum
            T2 += T0;
            delta_sum += delta_sum;
            T2 -= delta_sum;

            // T2 = T2 + T3
            T2 += T3;

            // identity = q_logic * alpha_base * (identity + T2)
            identity += T2;
            identity *= alpha_base;
            identity *= q_logic_fft[i];

            quotient_large[i] += identity;
        }
    }

    return alpha_d * alpha;
}

void ProverTurboLogicWidget::compute_transcript_elements(transcript::Transcript& transcript,
                                                         const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_logic", q_logic.evaluate(z, key->small_domain.size).to_buffer());
}

fr ProverTurboLogicWidget::compute_linear_contribution(const fr& alpha_base,
                                                       const transcript::Transcript& transcript,
                                                       barretenberg::polynomial& r)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr w_1_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_2_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_3_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_1_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_1_omega")[0]);
    fr w_2_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_2_omega")[0]);
    fr w_4_omega_eval = fr::serialize_from_buffer(&transcript.get_element("w_4_omega")[0]);
    fr q_c_eval = fr::serialize_from_buffer(&transcript.get_element("q_c")[0]);

    fr alpha_a = alpha_base;
    fr alpha_b = alpha_a * alpha;
    fr alpha_c = alpha_b * alpha;
    fr alpha_d = alpha_c * alpha;

    constexpr fr six = fr{ 6, 0, 0, 0 }.to_montgomery_form();
    constexpr fr eighty_one = fr{ 81, 0, 0, 0 }.to_montgomery_form();
    constexpr fr eighty_three = fr{ 83, 0, 0, 0 }.to_montgomery_form();

    fr delta_sum;
    fr delta_squared_sum;
    fr T0;
    fr T1;
    fr T2;
    fr T3;
    fr T4;
    fr identity;

    T0 = w_1_eval + w_1_eval;
    T0 += T0;
    T0 = w_1_omega_eval - T0;

    // T1 = b
    T1 = w_2_eval + w_2_eval;
    T1 += T1;
    T1 = w_2_omega_eval - T1;

    // delta_sum = a + b
    delta_sum = T0 + T1;

    // T2 = a^2, T3 = b^2
    T2 = T0.sqr();
    T3 = T1.sqr();

    delta_squared_sum = T2 + T3;

    // identity = a^2 + b^2 + 2ab
    identity = delta_sum.sqr();
    // identity = 2ab
    identity -= delta_squared_sum;

    // identity = 2(ab - w)
    T4 = w_3_eval + w_3_eval;
    identity -= T4;
    identity *= alpha;

    // T4 = 4w
    T4 += T4;

    // T2 = a^2 - a
    T2 -= T0;

    // T0 = a^2 - 5a + 6
    T0 += T0;
    T0 += T0;
    T0 = T2 - T0;
    T0 += six;

    // identity = (identity + a(a - 1)(a - 2)(a - 3)) * alpha
    T0 *= T2;
    identity += T0;
    identity *= alpha;

    // T3 = b^2 - b
    T3 -= T1;

    // T1 = b^2 - 5b + 6
    T1 += T1;
    T1 += T1;
    T1 = T3 - T1;
    T1 += six;

    // identity = (identity + b(b - 1)(b - 2)(b - 3)) * alpha
    T1 *= T3;
    identity += T1;
    identity *= alpha;

    // T0 = 3(a + b)
    T0 = delta_sum + delta_sum;
    T0 += delta_sum;

    // T1 = 9(a + b)
    T1 = T0 + T0;
    T1 += T0;

    // delta_sum = 18(a + b)
    delta_sum = T1 + T1;

    // T1 = 81(a + b)
    T2 = delta_sum + delta_sum;
    T2 += T2;
    T1 += T2;

    // delta_squared_sum = 18(a^2 + b^2)
    T2 = delta_squared_sum + delta_squared_sum;
    T2 += delta_squared_sum;
    delta_squared_sum = T2 + T2;
    delta_squared_sum += T2;
    delta_squared_sum += delta_squared_sum;

    // delta_sum = w(4w - 18(a + b) + 81)
    delta_sum = T4 - delta_sum;
    delta_sum += eighty_one;
    delta_sum *= w_3_eval;

    // T1 = 18(a^2 + b^2) - 81(a + b) + 83
    T1 = delta_squared_sum - T1;
    T1 += eighty_three;

    // delta_sum = w ( w ( 4w - 18(a + b) + 81) + 18(a^2 + b^2) - 81(a + b) + 83)
    delta_sum += T1;
    delta_sum *= w_3_eval;

    // T2 = 3c
    T2 = w_4_eval + w_4_eval;
    T2 += T2;
    T2 = w_4_omega_eval - T2;
    T3 = T2 + T2;
    T2 += T3;

    // T3 = 9c
    T3 = T2 + T2;
    T3 += T2;

    // T3 = q_c * (9c - 3(a + b))
    T3 -= T0;
    T3 *= q_c_eval;

    // T2 = 3c + 3(a + b) - 2 * delta_sum
    T2 += T0;
    delta_sum += delta_sum;
    T2 -= delta_sum;

    // T2 = T2 + T3
    T2 += T3;

    // identity = q_logic * alpha_base * (identity + T2)
    identity += T2;
    identity *= alpha_base;

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] += (identity * q_logic[i]);
    ITERATE_OVER_DOMAIN_END;

    return alpha_d * alpha;
}

void ProverTurboLogicWidget::compute_opening_poly_contribution(const transcript::Transcript& transcript,
                                                               const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }

    polynomial& poly = key->opening_poly;

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_logic")[0]);

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_logic[i] * nu);
    ITERATE_OVER_DOMAIN_END;
}
// ###

template <typename Field, typename Group, typename Transcript>
VerifierTurboLogicWidget<Field, Group, Transcript>::VerifierTurboLogicWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierTurboLogicWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
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

    Field w_1_eval = transcript.get_field_element("w_1");
    Field w_2_eval = transcript.get_field_element("w_2");
    Field w_3_eval = transcript.get_field_element("w_3");
    Field w_4_eval = transcript.get_field_element("w_4");
    Field w_1_omega_eval = transcript.get_field_element("w_1_omega");
    Field w_2_omega_eval = transcript.get_field_element("w_2_omega");
    Field w_4_omega_eval = transcript.get_field_element("w_4_omega");

    Field q_logic_eval = transcript.get_field_element("q_logic");
    Field q_c_eval = transcript.get_field_element("q_c");

    constexpr Field six = Field(6);
    constexpr Field eighty_one = Field(81);
    constexpr Field eighty_three = Field(83);

    Field delta_sum;
    Field delta_squared_sum;
    Field T0;
    Field T1;
    Field T2;
    Field T3;
    Field T4;
    Field identity;
    // T0 = a
    T0 = w_1_eval + w_1_eval;
    T0 += T0;
    T0 = w_1_omega_eval - T0;

    // T1 = b
    T1 = w_2_eval + w_2_eval;
    T1 += T1;
    T1 = w_2_omega_eval - T1;

    // delta_sum = a + b
    delta_sum = T0 + T1;

    // T2 = a^2, T3 = b^2
    T2 = T0.sqr();
    T3 = T1.sqr();

    delta_squared_sum = T2 + T3;

    // identity = a^2 + b^2 + 2ab
    identity = delta_sum.sqr();
    // identity = 2ab
    identity -= delta_squared_sum;

    // identity = 2(ab - w)
    T4 = w_3_eval + w_3_eval;
    identity -= T4;
    identity *= alpha;

    // T4 = 4w
    T4 += T4;

    // T2 = a^2 - a
    T2 -= T0;

    // T0 = a^2 - 5a + 6
    T0 += T0;
    T0 += T0;
    T0 = T2 - T0;
    T0 += six;

    // identity = (identity + a(a - 1)(a - 2)(a - 3)) * alpha
    T0 *= T2;
    identity += T0;
    identity *= alpha;

    // T3 = b^2 - b
    T3 -= T1;

    // T1 = b^2 - 5b + 6
    T1 += T1;
    T1 += T1;
    T1 = T3 - T1;
    T1 += six;

    // identity = (identity + b(b - 1)(b - 2)(b - 3)) * alpha
    T1 *= T3;
    identity += T1;
    identity *= alpha;

    // T0 = 3(a + b)
    T0 = delta_sum + delta_sum;
    T0 += delta_sum;

    // T1 = 9(a + b)
    T1 = T0 + T0;
    T1 += T0;

    // delta_sum = 18(a + b)
    delta_sum = T1 + T1;

    // T1 = 81(a + b)
    T2 = delta_sum + delta_sum;
    T2 += T2;
    T1 += T2;

    // delta_squared_sum = 18(a^2 + b^2)
    T2 = delta_squared_sum + delta_squared_sum;
    T2 += delta_squared_sum;
    delta_squared_sum = T2 + T2;
    delta_squared_sum += T2;
    delta_squared_sum += delta_squared_sum;

    // delta_sum = w(4w - 18(a + b) + 81)
    delta_sum = T4 - delta_sum;
    delta_sum += eighty_one;
    delta_sum *= w_3_eval;

    // T1 = 18(a^2 + b^2) - 81(a + b) + 83
    T1 = delta_squared_sum - T1;
    T1 += eighty_three;

    // delta_sum = w ( w ( 4w - 18(a + b) + 81) + 18(a^2 + b^2) - 81(a + b) + 83)
    delta_sum += T1;
    delta_sum *= w_3_eval;

    // T2 = 3c
    T2 = w_4_eval + w_4_eval;
    T2 += T2;
    T2 = w_4_omega_eval - T2;
    T3 = T2 + T2;
    T2 += T3;

    // T3 = 9c
    T3 = T2 + T2;
    T3 += T2;

    // T3 = q_c * (9c - 3(a + b))
    T3 -= T0;
    T3 *= q_c_eval;

    // T2 = 3c + 3(a + b) - 2 * delta_sum
    T2 += T0;
    delta_sum += delta_sum;
    T2 -= delta_sum;

    // T2 = T2 + T3
    T2 += T3;

    // identity = q_logic * alpha_base * (identity + T2)
    identity += T2;
    identity *= alpha_base;
    identity *= q_logic_eval;

    t_eval += identity;
    return alpha_base * alpha.sqr().sqr();
}

template <typename Field, typename Group, typename Transcript>
void VerifierTurboLogicWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*, Field& batch_eval, const Transcript& transcript, const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }

    Field q_logic_eval = transcript.get_field_element("q_logic");

    Field nu_base = transcript.get_challenge_field_element_from_map("nu", "q_logic");

    batch_eval += (q_logic_eval * nu_base);
}

template <typename Field, typename Group, typename Transcript>
Field VerifierTurboLogicWidget<Field, Group, Transcript>::append_scalar_multiplication_inputs(
    verification_key* key,
    const Field& alpha_base,
    const Transcript& transcript,
    std::vector<Group>& points,
    std::vector<Field>& scalars,
    const bool use_linearisation)
{
    Field alpha_step = transcript.get_challenge_field_element("alpha");

    if (use_linearisation) {
        Field w_4_eval = transcript.get_field_element("w_4");
        Field w_1_eval = transcript.get_field_element("w_1");
        Field w_2_eval = transcript.get_field_element("w_2");
        Field w_3_eval = transcript.get_field_element("w_3");
        Field w_1_omega_eval = transcript.get_field_element("w_1_omega");
        Field w_2_omega_eval = transcript.get_field_element("w_2_omega");
        Field w_4_omega_eval = transcript.get_field_element("w_4_omega");
        Field q_c_eval = transcript.get_field_element("q_c");

        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");

        constexpr Field six = Field(6);
        constexpr Field eighty_one = Field(81);
        constexpr Field eighty_three = Field(83);

        Field alpha_a = alpha_base;
        Field alpha_b = alpha_a * alpha_step;
        Field alpha_c = alpha_b * alpha_step;
        Field alpha_d = alpha_c * alpha_step;

        Field delta_sum;
        Field delta_squared_sum;
        Field T0;
        Field T1;
        Field T2;
        Field T3;
        Field T4;
        Field identity;
        T0 = w_1_eval + w_1_eval;
        T0 += T0;
        T0 = w_1_omega_eval - T0;

        // T1 = b
        T1 = w_2_eval + w_2_eval;
        T1 += T1;
        T1 = w_2_omega_eval - T1;

        // delta_sum = a + b
        delta_sum = T0 + T1;

        // T2 = a^2, T3 = b^2
        T2 = T0.sqr();
        T3 = T1.sqr();

        delta_squared_sum = T2 + T3;

        // identity = a^2 + b^2 + 2ab
        identity = delta_sum.sqr();
        // identity = 2ab
        identity -= delta_squared_sum;

        // identity = 2(ab - w)
        T4 = w_3_eval + w_3_eval;
        identity -= T4;
        identity *= alpha_step;

        // T4 = 4w
        T4 += T4;

        // T2 = a^2 - a
        T2 -= T0;

        // T0 = a^2 - 5a + 6
        T0 += T0;
        T0 += T0;
        T0 = T2 - T0;
        T0 += six;

        // identity = (identity + a(a - 1)(a - 2)(a - 3)) * alpha
        T0 *= T2;
        identity += T0;
        identity *= alpha_step;

        // T3 = b^2 - b
        T3 -= T1;

        // T1 = b^2 - 5b + 6
        T1 += T1;
        T1 += T1;
        T1 = T3 - T1;
        T1 += six;

        // identity = (identity + b(b - 1)(b - 2)(b - 3)) * alpha
        T1 *= T3;
        identity += T1;
        identity *= alpha_step;

        // T0 = 3(a + b)
        T0 = delta_sum + delta_sum;
        T0 += delta_sum;

        // T1 = 9(a + b)
        T1 = T0 + T0;
        T1 += T0;

        // delta_sum = 18(a + b)
        delta_sum = T1 + T1;

        // T1 = 81(a + b)
        T2 = delta_sum + delta_sum;
        T2 += T2;
        T1 += T2;

        // delta_squared_sum = 18(a^2 + b^2)
        T2 = delta_squared_sum + delta_squared_sum;
        T2 += delta_squared_sum;
        delta_squared_sum = T2 + T2;
        delta_squared_sum += T2;
        delta_squared_sum += delta_squared_sum;

        // delta_sum = w(4w - 18(a + b) + 81)
        delta_sum = T4 - delta_sum;
        delta_sum += eighty_one;
        delta_sum *= w_3_eval;

        // T1 = 18(a^2 + b^2) - 81(a + b) + 83
        T1 = delta_squared_sum - T1;
        T1 += eighty_three;

        // delta_sum = w ( w ( 4w - 18(a + b) + 81) + 18(a^2 + b^2) - 81(a + b) + 83)
        delta_sum += T1;
        delta_sum *= w_3_eval;

        // T2 = 3c
        T2 = w_4_eval + w_4_eval;
        T2 += T2;
        T2 = w_4_omega_eval - T2;
        T3 = T2 + T2;
        T2 += T3;

        // T3 = 9c
        T3 = T2 + T2;
        T3 += T2;

        // T3 = q_c * (9c - 3(a + b))
        T3 -= T0;
        T3 *= q_c_eval;

        // T2 = 3c + 3(a + b) - 2 * delta_sum
        T2 += T0;
        delta_sum += delta_sum;
        T2 -= delta_sum;

        // T2 = T2 + T3
        T2 += T3;

        // identity = q_logic * alpha_base * (identity + T2)
        identity += T2;
        identity *= alpha_base;
        identity *= linear_nu;

        if (key->constraint_selectors.at("Q_LOGIC_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_LOGIC_SELECTOR"));
            scalars.push_back(identity);
        }
        return alpha_d * alpha_step;
    }

    Field nu_base = transcript.get_challenge_field_element_from_map("nu", "q_logic");

    if (key->constraint_selectors.at("Q_LOGIC_SELECTOR").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_LOGIC_SELECTOR"));
        scalars.push_back(nu_base);
    }
    Field alpha_a = alpha_base;
    Field alpha_b = alpha_a * alpha_step;
    Field alpha_c = alpha_b * alpha_step;
    Field alpha_d = alpha_c * alpha_step;

    return alpha_d * alpha_step;
}

template class VerifierTurboLogicWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>;
} // namespace waffle