#include "turbo_arithmetic_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverTurboArithmeticWidget::ProverTurboArithmeticWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_1(key->constraint_selectors.at("q_1"))
    , q_2(key->constraint_selectors.at("q_2"))
    , q_3(key->constraint_selectors.at("q_3"))
    , q_4(key->constraint_selectors.at("q_4"))
    , q_5(key->constraint_selectors.at("q_5"))
    , q_m(key->constraint_selectors.at("q_m"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_arith(key->constraint_selectors.at("q_arith"))
    , q_1_fft(key->constraint_selector_ffts.at("q_1_fft"))
    , q_2_fft(key->constraint_selector_ffts.at("q_2_fft"))
    , q_3_fft(key->constraint_selector_ffts.at("q_3_fft"))
    , q_4_fft(key->constraint_selector_ffts.at("q_4_fft"))
    , q_5_fft(key->constraint_selector_ffts.at("q_5_fft"))
    , q_m_fft(key->constraint_selector_ffts.at("q_m_fft"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
    , q_arith_fft(key->constraint_selector_ffts.at("q_arith_fft"))
{}

ProverTurboArithmeticWidget::ProverTurboArithmeticWidget(const ProverTurboArithmeticWidget& other)
    : ProverBaseWidget(other)
    , q_1(key->constraint_selectors.at("q_1"))
    , q_2(key->constraint_selectors.at("q_2"))
    , q_3(key->constraint_selectors.at("q_3"))
    , q_4(key->constraint_selectors.at("q_4"))
    , q_5(key->constraint_selectors.at("q_5"))
    , q_m(key->constraint_selectors.at("q_m"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_arith(key->constraint_selectors.at("q_arith"))
    , q_1_fft(key->constraint_selector_ffts.at("q_1_fft"))
    , q_2_fft(key->constraint_selector_ffts.at("q_2_fft"))
    , q_3_fft(key->constraint_selector_ffts.at("q_3_fft"))
    , q_4_fft(key->constraint_selector_ffts.at("q_4_fft"))
    , q_5_fft(key->constraint_selector_ffts.at("q_5_fft"))
    , q_m_fft(key->constraint_selector_ffts.at("q_m_fft"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
    , q_arith_fft(key->constraint_selector_ffts.at("q_arith_fft"))
{}

ProverTurboArithmeticWidget::ProverTurboArithmeticWidget(ProverTurboArithmeticWidget&& other)
    : ProverBaseWidget(other)
    , q_1(key->constraint_selectors.at("q_1"))
    , q_2(key->constraint_selectors.at("q_2"))
    , q_3(key->constraint_selectors.at("q_3"))
    , q_4(key->constraint_selectors.at("q_4"))
    , q_5(key->constraint_selectors.at("q_5"))
    , q_m(key->constraint_selectors.at("q_m"))
    , q_c(key->constraint_selectors.at("q_c"))
    , q_arith(key->constraint_selectors.at("q_arith"))
    , q_1_fft(key->constraint_selector_ffts.at("q_1_fft"))
    , q_2_fft(key->constraint_selector_ffts.at("q_2_fft"))
    , q_3_fft(key->constraint_selector_ffts.at("q_3_fft"))
    , q_4_fft(key->constraint_selector_ffts.at("q_4_fft"))
    , q_5_fft(key->constraint_selector_ffts.at("q_5_fft"))
    , q_m_fft(key->constraint_selector_ffts.at("q_m_fft"))
    , q_c_fft(key->constraint_selector_ffts.at("q_c_fft"))
    , q_arith_fft(key->constraint_selector_ffts.at("q_arith_fft"))
{}

ProverTurboArithmeticWidget& ProverTurboArithmeticWidget::operator=(const ProverTurboArithmeticWidget& other)
{
    ProverBaseWidget::operator=(other);
    q_1 = key->constraint_selectors.at("q_1");
    q_2 = key->constraint_selectors.at("q_2");
    q_3 = key->constraint_selectors.at("q_3");
    q_4 = key->constraint_selectors.at("q_4");
    q_5 = key->constraint_selectors.at("q_5");
    q_m = key->constraint_selectors.at("q_m");
    q_c = key->constraint_selectors.at("q_c");
    q_arith = key->constraint_selectors.at("q_arith");

    q_1_fft = key->constraint_selectors.at("q_1_fft");
    q_2_fft = key->constraint_selectors.at("q_2_fft");
    q_3_fft = key->constraint_selectors.at("q_3_fft");
    q_4_fft = key->constraint_selectors.at("q_4_fft");
    q_5_fft = key->constraint_selectors.at("q_5_fft");
    q_m_fft = key->constraint_selectors.at("q_m_fft");
    q_c_fft = key->constraint_selectors.at("q_c_fft");
    q_arith_fft = key->constraint_selectors.at("q_arith_fft");
    return *this;
}

ProverTurboArithmeticWidget& ProverTurboArithmeticWidget::operator=(ProverTurboArithmeticWidget&& other)
{
    ProverBaseWidget::operator=(other);
    q_1 = key->constraint_selectors.at("q_1");
    q_2 = key->constraint_selectors.at("q_2");
    q_3 = key->constraint_selectors.at("q_3");
    q_4 = key->constraint_selectors.at("q_4");
    q_5 = key->constraint_selectors.at("q_5");
    q_m = key->constraint_selectors.at("q_m");
    q_c = key->constraint_selectors.at("q_c");
    q_arith = key->constraint_selectors.at("q_arith");

    q_1_fft = key->constraint_selectors.at("q_1_fft");
    q_2_fft = key->constraint_selectors.at("q_2_fft");
    q_3_fft = key->constraint_selectors.at("q_3_fft");
    q_4_fft = key->constraint_selectors.at("q_4_fft");
    q_5_fft = key->constraint_selectors.at("q_5_fft");
    q_m_fft = key->constraint_selectors.at("q_m_fft");
    q_c_fft = key->constraint_selectors.at("q_c_fft");
    q_arith_fft = key->constraint_selectors.at("q_arith_fft");
    return *this;
}

fr ProverTurboArithmeticWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                              const transcript::Transcript& transcript)
{
    const fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    const fr* w_1_fft = &key->wire_ffts.at("w_1_fft")[0];
    const fr* w_2_fft = &key->wire_ffts.at("w_2_fft")[0];
    const fr* w_3_fft = &key->wire_ffts.at("w_3_fft")[0];
    const fr* w_4_fft = &key->wire_ffts.at("w_4_fft")[0];

    fr* quotient_large = &key->quotient_large[0];

    constexpr fr minus_two = -fr(2);
    constexpr fr minus_seven = -fr(7);
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
        const size_t start = j * key->large_domain.thread_size;
        const size_t end = (j + 1) * key->large_domain.thread_size;
        fr T0;
        fr T1;
        fr T2;
        fr T3;
        fr T4;
        fr T5;
        fr T6;
        for (size_t i = start; i < end; ++i) {

            T0 = w_1_fft[i] * q_m_fft[i];
            T0 *= w_2_fft[i];
            T1 = w_1_fft[i] * q_1_fft[i];
            T2 = w_2_fft[i] * q_2_fft[i];
            T3 = w_3_fft[i] * q_3_fft[i];
            T4 = w_4_fft[i] * q_4_fft[i];
            T5 = w_4_fft[i].sqr();
            T5 -= w_4_fft[i];
            T6 = w_4_fft[i] + minus_two;
            T5 *= T6;
            T5 *= q_5_fft[i];
            T5 *= alpha;

            T0 += T1;
            T0 += T2;
            T0 += T3;
            T0 += T4;
            T0 += T5;
            T0 += q_c_fft[i];
            T0 *= q_arith_fft[i];

            /**
             * quad extraction term
             *
             * We evaluate ranges using the turbo_range_widget, which generates a sequence
             * of accumulating sums - each sum aggregates a base-4 value.
             *
             * We sometimes need to extract individual bits from our quads, the following
             * term will extrat the high bit from two accumulators, and add it into the
             * arithmetic identity.
             *
             * This term is only active when q_arith[i] is set to 2
             **/
            T1 = q_arith_fft[i].sqr();
            T1 -= q_arith_fft[i];

            T2 = w_4_fft[i] + w_4_fft[i];
            T2 += T2;
            T2 = w_3_fft[i] - T2;

            T3 = T2.sqr();
            T3 += T3;

            T4 = T2 + T2;
            T4 += T2;
            T5 = T4 + T4;
            T4 += T5;

            T4 -= T3;
            T4 += minus_seven;

            // T2 = 6 iff delta is 2 or 3
            // T2 = 0 iff delta is 0 or 1 (extracts high bit)
            T2 *= T4;

            T1 *= T2;

            T0 += T1;
            T0 *= alpha_base;

            quotient_large[i] += T0;
        }
    }
    return alpha_base * alpha.sqr();
}

void ProverTurboArithmeticWidget::compute_transcript_elements(transcript::Transcript& transcript,
                                                              const bool use_linearisation)
{
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);

    transcript.add_element("q_arith", q_arith.evaluate(z, key->small_domain.size).to_buffer());

    if (use_linearisation) {
        return;
    }
    transcript.add_element("q_1", q_1.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_2", q_2.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_3", q_3.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_4", q_4.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_5", q_5.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_m", q_m.evaluate(z, key->small_domain.size).to_buffer());
    transcript.add_element("q_c", q_c.evaluate(z, key->small_domain.size).to_buffer());
}

fr ProverTurboArithmeticWidget::compute_linear_contribution(const fr& alpha_base,
                                                            const transcript::Transcript& transcript,
                                                            barretenberg::polynomial& r)
{

    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr w_l_eval = fr::serialize_from_buffer(&transcript.get_element("w_1")[0]);
    fr w_r_eval = fr::serialize_from_buffer(&transcript.get_element("w_2")[0]);
    fr w_o_eval = fr::serialize_from_buffer(&transcript.get_element("w_3")[0]);
    fr w_4_eval = fr::serialize_from_buffer(&transcript.get_element("w_4")[0]);
    fr q_arith_eval = fr::serialize_from_buffer(&transcript.get_element("q_arith")[0]);

    fr neg_two = -fr(2);
    fr w_lr = w_l_eval * w_r_eval;
    fr is_w_4_bool = (w_4_eval.sqr() - w_4_eval) * (w_4_eval + neg_two) * alpha;
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T0 = w_lr * q_m[i];
    fr T1 = w_l_eval * q_1[i];
    fr T2 = w_r_eval * q_2[i];
    fr T3 = w_o_eval * q_3[i];
    fr T4 = w_4_eval * q_4[i];
    fr T5 = is_w_4_bool * q_5[i];
    r[i] += ((T0 + T1 + T2 + T3 + T4 + T5 + q_c[i]) * q_arith_eval * alpha_base);
    ITERATE_OVER_DOMAIN_END;

    return alpha_base * alpha.sqr();
}

size_t ProverTurboArithmeticWidget::compute_opening_poly_contribution(
    const size_t nu_index, const transcript::Transcript& transcript, fr* poly, fr*, const bool use_linearisation)
{
    if (use_linearisation) {

        fr nu_base = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index)[0]);

        ITERATE_OVER_DOMAIN_START(key->small_domain);
        poly[i] += (q_arith[i] * nu_base);
        ITERATE_OVER_DOMAIN_END;

        return nu_index + 1;
    }

    std::array<barretenberg::fr, 8> nu_challenges;
    nu_challenges[0] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index)[0]);
    nu_challenges[1] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 1)[0]);
    nu_challenges[2] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 2)[0]);
    nu_challenges[3] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 3)[0]);
    nu_challenges[4] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 4)[0]);
    nu_challenges[5] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 5)[0]);
    nu_challenges[6] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 6)[0]);
    nu_challenges[7] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + 7)[0]);
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_1[i] * nu_challenges[0]);
    poly[i] += (q_2[i] * nu_challenges[1]);
    poly[i] += (q_3[i] * nu_challenges[2]);
    poly[i] += (q_4[i] * nu_challenges[3]);
    poly[i] += (q_5[i] * nu_challenges[4]);
    poly[i] += (q_m[i] * nu_challenges[5]);
    poly[i] += (q_c[i] * nu_challenges[6]);
    poly[i] += (q_arith[i] * nu_challenges[7]);
    ITERATE_OVER_DOMAIN_END;

    return nu_index + 8;
} // namespace waffle

// ###

template <typename Field, typename Group, typename Transcript>
VerifierTurboArithmeticWidget<Field, Group, Transcript>::VerifierTurboArithmeticWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierTurboArithmeticWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*, const Field& alpha_base, const Transcript& transcript, Field& t_eval, bool use_linearisation)
{
    const Field alpha = transcript.get_challenge_field_element("alpha");
    if (use_linearisation) {
        const Field q_arith_eval = transcript.get_field_element("q_arith");

        const Field w_3_eval = transcript.get_field_element("w_3");
        const Field w_4_eval = transcript.get_field_element("w_4");

        Field T1;
        Field T2;
        Field T3;
        Field T4;
        Field T5;
        constexpr Field minus_seven = -fr(7);

        T1 = q_arith_eval.sqr() - q_arith_eval;

        T2 = w_4_eval + w_4_eval;
        T2 = T2 + T2;
        T2 = w_3_eval - T2;

        T3 = T2.sqr();
        T3 = T3 + T3;

        T4 = T2 + T2 + T2;
        T5 = T4 + T4;
        T4 = T4 + T5;
        T4 = T4 - T3;
        T4 = T4 + minus_seven;

        T2 = T2 * T4;

        T1 = T1 * T2;
        T1 = T1 * alpha_base;

        t_eval = t_eval + T1;

        return alpha_base * alpha.sqr();
    }

    const Field q_1_eval = transcript.get_field_element("q_1");
    const Field q_2_eval = transcript.get_field_element("q_2");
    const Field q_3_eval = transcript.get_field_element("q_3");
    const Field q_4_eval = transcript.get_field_element("q_4");
    const Field q_5_eval = transcript.get_field_element("q_5");
    const Field q_m_eval = transcript.get_field_element("q_m");
    const Field q_c_eval = transcript.get_field_element("q_c");
    const Field q_arith_eval = transcript.get_field_element("q_arith");

    const Field w_1_eval = transcript.get_field_element("w_1");
    const Field w_2_eval = transcript.get_field_element("w_2");
    const Field w_3_eval = transcript.get_field_element("w_3");
    const Field w_4_eval = transcript.get_field_element("w_4");

    constexpr Field minus_two = -fr(2);
    constexpr Field minus_seven = -fr(7);

    Field T0;
    Field T1;
    Field T2;
    Field T3;
    Field T4;
    Field T5;
    Field T6;

    T0 = w_1_eval * q_m_eval;
    T0 *= w_2_eval;
    T1 = w_1_eval * q_1_eval;
    T2 = w_2_eval * q_2_eval;
    T3 = w_3_eval * q_3_eval;
    T4 = w_4_eval * q_4_eval;
    T5 = w_4_eval.sqr();
    T5 -= w_4_eval;
    T6 = w_4_eval + minus_two;
    T5 *= T6;
    T5 *= q_5_eval;
    T5 *= alpha;

    T0 += T1;
    T0 += T2;
    T0 += T3;
    T0 += T4;
    T0 += T5;
    T0 += q_c_eval;
    T0 *= q_arith_eval;

    /**
     * quad extraction term
     *
     * We evaluate ranges using the turbo_range_widget, which generates a sequence
     * of accumulating sums - each sum aggregates a base-4 value.
     *
     * We sometimes need to extract individual bits from our quads, the following
     * term will extrat the high bit from two accumulators, and add it into the
     * arithmetic identity.
     *
     * This term is only active when q_arith[i] is set to 2
     **/
    T1 = q_arith_eval.sqr();
    T1 -= q_arith_eval;

    T2 = w_4_eval + w_4_eval;
    T2 += T2;
    T2 = w_3_eval - T2;

    T3 = T2.sqr();
    T3 += T3;

    T4 = T2 + T2;
    T4 += T2;
    T5 = T4 + T4;
    T4 += T5;

    T4 -= T3;
    T4 += minus_seven;

    // T2 = 6 iff delta is 2 or 3
    // T2 = 0 iff delta is 0 or 1 (extracts high bit)
    T2 *= T4;

    T1 *= T2;

    T0 += T1;
    T0 *= alpha_base;
    t_eval += T0;

    return alpha_base * alpha.sqr();
}

template <typename Field, typename Group, typename Transcript>
size_t VerifierTurboArithmeticWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*,
    Field& batch_eval,
    const size_t nu_index,
    const Transcript& transcript,
    const bool use_linearisation)
{
    Field q_arith_eval = transcript.get_field_element("q_arith");

    Field nu_base = transcript.get_challenge_field_element("nu", nu_index);

    if (use_linearisation) {
        batch_eval = batch_eval + (nu_base * q_arith_eval);
        return nu_index + 1;
    }

    const Field q_1_eval = transcript.get_field_element("q_1");
    const Field q_2_eval = transcript.get_field_element("q_2");
    const Field q_3_eval = transcript.get_field_element("q_3");
    const Field q_4_eval = transcript.get_field_element("q_4");
    const Field q_5_eval = transcript.get_field_element("q_5");
    const Field q_m_eval = transcript.get_field_element("q_m");
    const Field q_c_eval = transcript.get_field_element("q_c");

    std::array<Field, 8> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element("nu", nu_index);
    nu_challenges[1] = transcript.get_challenge_field_element("nu", nu_index + 1);
    nu_challenges[2] = transcript.get_challenge_field_element("nu", nu_index + 2);
    nu_challenges[3] = transcript.get_challenge_field_element("nu", nu_index + 3);
    nu_challenges[4] = transcript.get_challenge_field_element("nu", nu_index + 4);
    nu_challenges[5] = transcript.get_challenge_field_element("nu", nu_index + 5);
    nu_challenges[6] = transcript.get_challenge_field_element("nu", nu_index + 6);
    nu_challenges[7] = transcript.get_challenge_field_element("nu", nu_index + 7);

    batch_eval += (q_1_eval * nu_challenges[0]);
    batch_eval += (q_2_eval * nu_challenges[1]);
    batch_eval += (q_3_eval * nu_challenges[2]);
    batch_eval += (q_4_eval * nu_challenges[3]);
    batch_eval += (q_5_eval * nu_challenges[4]);
    batch_eval += (q_m_eval * nu_challenges[5]);
    batch_eval += (q_c_eval * nu_challenges[6]);
    batch_eval += (q_arith_eval * nu_challenges[7]);
    return nu_index + 8;
}

template <typename Field, typename Group, typename Transcript>
VerifierBaseWidget::challenge_coefficients<Field> VerifierTurboArithmeticWidget<Field, Group, Transcript>::
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
        Field w_4_eval = transcript.get_field_element("w_4");

        Field q_arith_eval = transcript.get_field_element("q_arith");

        Field linear_nu = transcript.get_challenge_field_element("nu", challenge.linear_nu_index);
        Field nu_base = transcript.get_challenge_field_element("nu", challenge.nu_index);

        Field q_l_term = w_l_eval * q_arith_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_1").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_1"));
            scalars.push_back(q_l_term);
        }

        Field q_r_term = w_r_eval * q_arith_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_2").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_2"));
            scalars.push_back(q_r_term);
        }

        Field q_o_term = w_o_eval * q_arith_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_3").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_3"));
            scalars.push_back(q_o_term);
        }

        Field q_4_term = w_4_eval * q_arith_eval * challenge.alpha_base * linear_nu;
        if (key->constraint_selectors.at("Q_4").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_4"));
            scalars.push_back(q_4_term);
        }

        constexpr Field minus_two = -Field(2);
        Field q_5_term = (w_4_eval.sqr() - w_4_eval) * (w_4_eval + minus_two) * challenge.alpha_base *
                         challenge.alpha_step * linear_nu * q_arith_eval;
        if (key->constraint_selectors.at("Q_5").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_5"));
            scalars.push_back(q_5_term);
        }

        // Q_M term = w_l * w_r * challenge.alpha_base * nu
        Field q_m_term = w_l_eval * w_r_eval * challenge.alpha_base * linear_nu * q_arith_eval;
        if (key->constraint_selectors.at("Q_M").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_M"));
            scalars.push_back(q_m_term);
        }

        Field q_c_term = challenge.alpha_base * linear_nu * q_arith_eval;
        if (key->constraint_selectors.at("Q_C").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_C"));
            scalars.push_back(q_c_term);
        }

        if (key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR"));
            scalars.push_back(nu_base);
        }

        return VerifierBaseWidget::challenge_coefficients<Field>{ challenge.alpha_base * challenge.alpha_step.sqr(),
                                                                  challenge.alpha_step,
                                                                  challenge.nu_index + 1,
                                                                  challenge.linear_nu_index };
    }

    std::array<Field, 8> nu_challenges;
    nu_challenges[0] = transcript.get_challenge_field_element("nu", challenge.nu_index);
    nu_challenges[1] = transcript.get_challenge_field_element("nu", challenge.nu_index + 1);
    nu_challenges[2] = transcript.get_challenge_field_element("nu", challenge.nu_index + 2);
    nu_challenges[3] = transcript.get_challenge_field_element("nu", challenge.nu_index + 3);
    nu_challenges[4] = transcript.get_challenge_field_element("nu", challenge.nu_index + 4);
    nu_challenges[5] = transcript.get_challenge_field_element("nu", challenge.nu_index + 5);
    nu_challenges[6] = transcript.get_challenge_field_element("nu", challenge.nu_index + 6);
    nu_challenges[7] = transcript.get_challenge_field_element("nu", challenge.nu_index + 7);

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

    return VerifierBaseWidget::challenge_coefficients<Field>{ challenge.alpha_base * challenge.alpha_step.sqr(),
                                                              challenge.alpha_step,
                                                              challenge.nu_index + 8,
                                                              challenge.linear_nu_index };
}

template class VerifierTurboArithmeticWidget<barretenberg::fr,
                                             barretenberg::g1::affine_element,
                                             transcript::StandardTranscript>;

} // namespace waffle