#pragma once

#include "./base_widget.hpp"

namespace waffle {
class VerifierTurboRangeWidget : public VerifierBaseWidget {
  public:
    VerifierTurboRangeWidget();

    VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key*,
        const challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars) override;

    barretenberg::fr compute_batch_evaluation_contribution(verification_key*,
                                                                    barretenberg::fr&,
                                                                    const barretenberg::fr& nu_base,
                                                                    const transcript::Transcript&) override;

    barretenberg::fr compute_quotient_evaluation_contribution(verification_key*,
                                                                       const barretenberg::fr&,
                                                                       const transcript::Transcript& transcript,
                                                                       barretenberg::fr&) override;
};

class ProverTurboRangeWidget : public ProverBaseWidget {
  public:
    ProverTurboRangeWidget(proving_key* input_key, program_witness* input_witness);
    ProverTurboRangeWidget(const ProverTurboRangeWidget& other);
    ProverTurboRangeWidget(ProverTurboRangeWidget&& other);
    ProverTurboRangeWidget& operator=(const ProverTurboRangeWidget& other);
    ProverTurboRangeWidget& operator=(ProverTurboRangeWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                            const transcript::Transcript& transcript);
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                          const transcript::Transcript& transcript,
                                                          barretenberg::polynomial& r);
    barretenberg::fr compute_opening_poly_contribution(const barretenberg::fr& nu_base,
                                                                const transcript::Transcript&,
                                                                barretenberg::fr*,
                                                                barretenberg::fr*);

    void compute_transcript_elements(transcript::Transcript& transcript);

    barretenberg::polynomial& q_range;
    barretenberg::polynomial& q_range_fft;
};
} // namespace waffle
