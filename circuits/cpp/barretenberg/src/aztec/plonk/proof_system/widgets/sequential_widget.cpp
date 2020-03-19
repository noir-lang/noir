#include "sequential_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
ProverSequentialWidget::ProverSequentialWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
    , q_3_next(key->constraint_selectors.at("q_3_next"))
    , q_3_next_fft(key->constraint_selector_ffts.at("q_3_next_fft"))
{}

ProverSequentialWidget::ProverSequentialWidget(const ProverSequentialWidget& other)
    : ProverBaseWidget(other)
    , q_3_next(key->constraint_selectors.at("q_3_next"))
    , q_3_next_fft(key->constraint_selector_ffts.at("q_3_next_fft"))
{}

ProverSequentialWidget::ProverSequentialWidget(ProverSequentialWidget&& other)
    : ProverBaseWidget(other)
    , q_3_next(key->constraint_selectors.at("q_3_next"))
    , q_3_next_fft(key->constraint_selector_ffts.at("q_3_next_fft"))
{}

ProverSequentialWidget& ProverSequentialWidget::operator=(const ProverSequentialWidget& other)
{
    ProverBaseWidget::operator=(other);

    q_3_next = key->constraint_selectors.at("q_3_next");

    q_3_next_fft = key->constraint_selector_ffts.at("q_3_next_fft");
    return *this;
}

ProverSequentialWidget& ProverSequentialWidget::operator=(ProverSequentialWidget&& other)
{
    ProverBaseWidget::operator=(other);

    q_3_next = key->constraint_selectors.at("q_3_next");

    q_3_next_fft = key->constraint_selector_ffts.at("q_3_next_fft");
    return *this;
}

fr ProverSequentialWidget::compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                         const transcript::Transcript& transcript)
{
    fr alpha = fr::serialize_from_buffer(&transcript.get_challenge("alpha")[0]);

    barretenberg::fr old_alpha = alpha_base * alpha.invert();
    polynomial& w_3_fft = key->wire_ffts.at("w_3_fft");
    polynomial& quotient_mid = key->quotient_mid;
    ITERATE_OVER_DOMAIN_START(key->mid_domain);
    fr T0;
    T0 = w_3_fft.at(2 * i + 4) * q_3_next_fft[i]; // w_l * q_m = rdx
    T0 *= old_alpha;
    quotient_mid[i] += T0;
    ITERATE_OVER_DOMAIN_END;

    return alpha_base;
}

fr ProverSequentialWidget::compute_linear_contribution(const fr& alpha_base,
                                                       const transcript::Transcript& transcript,
                                                       polynomial& r)
{
    fr w_o_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);
    fr alpha = fr::serialize_from_buffer(&transcript.get_challenge("alpha")[0]);

    barretenberg::fr old_alpha = alpha_base * alpha.invert();
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T0;
    T0 = w_o_shifted_eval * q_3_next[i];
    T0 *= old_alpha;
    r[i] += T0;
    ITERATE_OVER_DOMAIN_END;
    return alpha_base;
}

fr ProverSequentialWidget::compute_opening_poly_contribution(
    const fr& nu_base, const transcript::Transcript& transcript, fr* poly, fr*, const bool use_linearisation)
{
    if (use_linearisation) {
        return nu_base;
    }

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_3_next[i] * nu_base);
    ITERATE_OVER_DOMAIN_END;

    return nu_base * nu;
}

void ProverSequentialWidget::compute_transcript_elements(transcript::Transcript& transcript,
                                                         const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }
    fr z = fr::serialize_from_buffer(&transcript.get_challenge("z")[0]);
    transcript.add_element("q_3_omega", q_3_next.evaluate(z, key->small_domain.size).to_buffer());
}

// ###

VerifierSequentialWidget::VerifierSequentialWidget()
    : VerifierBaseWidget()
{}

fr VerifierSequentialWidget::compute_quotient_evaluation_contribution(verification_key*,
                                                                      const fr& alpha_base,
                                                                      const transcript::Transcript& transcript,
                                                                      fr& t_eval,
                                                                      const bool use_linearisation)
{
    const fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    if (use_linearisation) {
        return alpha_base;
    }

    fr w_3_next_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);

    fr q_3_next_eval = fr::serialize_from_buffer(&transcript.get_element("q_3_omega")[0]);

    barretenberg::fr old_alpha = alpha_base * alpha.invert();

    fr T0 = (w_3_next_eval * q_3_next_eval) * old_alpha;
    t_eval += T0;

    return alpha_base;
}

fr VerifierSequentialWidget::compute_batch_evaluation_contribution(verification_key*,
                                                                   fr& batch_eval,
                                                                   const fr& nu_base,
                                                                   const transcript::Transcript& transcript,
                                                                   bool use_linearisation)
{
    if (use_linearisation) {
        return nu_base;
    }

    fr q_3_omega_eval = fr::serialize_from_buffer(&transcript.get_element("q_3_omega")[0]);

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge("nu")[0]);

    batch_eval += (q_3_omega_eval * nu_base);

    return nu_base * nu;
};

VerifierBaseWidget::challenge_coefficients VerifierSequentialWidget::append_scalar_multiplication_inputs(
    verification_key* key,
    const challenge_coefficients& challenge,
    const transcript::Transcript& transcript,
    std::vector<barretenberg::g1::affine_element>& points,
    std::vector<barretenberg::fr>& scalars,
    const bool use_linearisation)
{
    if (use_linearisation) {
        fr w_o_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("w_3_omega")[0]);

        barretenberg::fr old_alpha = (challenge.alpha_base * (challenge.alpha_step.invert()));

        // Q_M term = w_l * w_r * challenge.alpha_base * nu
        fr q_o_next_term;
        q_o_next_term = w_o_shifted_eval * old_alpha;
        q_o_next_term *= challenge.linear_nu;

        if (key->constraint_selectors.at("Q_3_NEXT").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_3_NEXT"));
            scalars.push_back(q_o_next_term);
        }

        return VerifierBaseWidget::challenge_coefficients{
            challenge.alpha_base, challenge.alpha_step, challenge.nu_base, challenge.nu_step, challenge.linear_nu
        };
    }

    if (key->constraint_selectors.at("Q_3_NEXT").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_3_NEXT"));
        scalars.push_back(challenge.nu_base);
    }
    return VerifierBaseWidget::challenge_coefficients{ challenge.alpha_base,
                                                       challenge.alpha_step,
                                                       challenge.nu_base * challenge.nu_step,
                                                       challenge.nu_step,
                                                       challenge.linear_nu };
}
} // namespace waffle