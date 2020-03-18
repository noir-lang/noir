#pragma once
#include "base_widget.hpp"

namespace waffle {
class VerifierSequentialWidget : public VerifierBaseWidget {
  public:
    VerifierSequentialWidget();

    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key* key,
        const challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars);

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key*,
                                                                  barretenberg::fr&,
                                                                  const barretenberg::fr& nu_base,
                                                                  const transcript::Transcript&)
    {
        return nu_base;
    };

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key*,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::Transcript& transcript,
                                                                     barretenberg::fr& t_eval,
                                                                     const bool use_linearisation);
};

class ProverSequentialWidget : public ProverBaseWidget {
  public:
    ProverSequentialWidget(proving_key* input_key, program_witness* input_witness);
    ProverSequentialWidget(const ProverSequentialWidget& other);
    ProverSequentialWidget(ProverSequentialWidget&& other);
    ProverSequentialWidget& operator=(const ProverSequentialWidget& other);
    ProverSequentialWidget& operator=(ProverSequentialWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript);
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r);

    barretenberg::fr compute_opening_poly_contribution(
        const barretenberg::fr&, const transcript::Transcript&, barretenberg::fr*, barretenberg::fr*, const bool);

    void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_3_next;

    barretenberg::polynomial& q_3_next_fft;
};
} // namespace waffle