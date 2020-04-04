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

void ProverSequentialWidget::compute_opening_poly_contribution(const transcript::Transcript& transcript,
                                                               const bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }

    polynomial& poly = key->opening_poly;

    fr nu = fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "q_3_next")[0]);

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    poly[i] += (q_3_next[i] * nu);
    ITERATE_OVER_DOMAIN_END;
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

template <typename Field, typename Group, typename Transcript>
VerifierSequentialWidget<Field, Group, Transcript>::VerifierSequentialWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierSequentialWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*,
    const Field& alpha_base,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{
    const Field alpha = transcript.get_challenge_field_element("alpha");

    if (use_linearisation) {
        return alpha_base;
    }

    Field w_3_next_eval = transcript.get_field_element("w_3_omega");

    Field q_3_next_eval = transcript.get_field_element("q_3_omega");

    Field old_alpha = alpha_base * alpha.invert();

    Field T0 = (w_3_next_eval * q_3_next_eval) * old_alpha;
    t_eval += T0;

    return alpha_base;
}

template <typename Field, typename Group, typename Transcript>
void VerifierSequentialWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*, Field& batch_eval, const Transcript& transcript, bool use_linearisation)
{
    if (use_linearisation) {
        return;
    }

    Field q_3_omega_eval = transcript.get_field_element("q_3_omega");

    Field nu = transcript.get_challenge_field_element_from_map("nu", "q_3_omega");

    batch_eval += (q_3_omega_eval * nu);
};

template <typename Field, typename Group, typename Transcript>
Field VerifierSequentialWidget<Field, Group, Transcript>::append_scalar_multiplication_inputs(
    verification_key* key,
    const Field& alpha_base,
    const Transcript& transcript,
    std::vector<Group>& points,
    std::vector<Field>& scalars,
    const bool use_linearisation)
{
    Field alpha_step = transcript.get_challenge_field_element("alpha");

    if (use_linearisation) {
        Field w_o_shifted_eval = transcript.get_field_element("w_3_omega");

        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");

        Field old_alpha = (alpha_base * (alpha_step.invert()));

        // Q_M term = w_l * w_r * alpha_base * nu
        Field q_o_next_term;
        q_o_next_term = w_o_shifted_eval * old_alpha;
        q_o_next_term *= linear_nu;

        if (key->constraint_selectors.at("Q_3_NEXT").on_curve()) {
            points.push_back(key->constraint_selectors.at("Q_3_NEXT"));
            scalars.push_back(q_o_next_term);
        }

        return alpha_base;
    }

    Field nu_base = transcript.get_challenge_field_element_from_map("nu", "q_3_next");

    if (key->constraint_selectors.at("Q_3_NEXT").on_curve()) {
        points.push_back(key->constraint_selectors.at("Q_3_NEXT"));
        scalars.push_back(nu_base);
    }
    return alpha_base;
}

template class VerifierSequentialWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>;

} // namespace waffle