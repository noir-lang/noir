#pragma once
#include "base_widget.hpp"

namespace waffle {
class VerifierTurboLogicWidget : public VerifierBaseWidget {
  public:
    VerifierTurboLogicWidget();

    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key* key,
        const challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars);

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key*,
                                                                  barretenberg::fr&,
                                                                  const barretenberg::fr& nu_base,
                                                                  const transcript::Transcript&);

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key*,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::Transcript& transcript,
                                                                     barretenberg::fr& t_eval,
                                                                     const bool use_linearisation);
};

class ProverTurboLogicWidget : public ProverBaseWidget {
  public:
    ProverTurboLogicWidget(proving_key* input_key, program_witness* input_witness);
    ProverTurboLogicWidget(const ProverTurboLogicWidget& other);
    ProverTurboLogicWidget(ProverTurboLogicWidget&& other);
    ProverTurboLogicWidget& operator=(const ProverTurboLogicWidget& other);
    ProverTurboLogicWidget& operator=(ProverTurboLogicWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript);
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r);
    barretenberg::fr compute_opening_poly_contribution(const barretenberg::fr& nu_base,
                                                       const transcript::Transcript&,
                                                       barretenberg::fr*,
                                                       barretenberg::fr*,
                                                       const bool);

    void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_logic;
    barretenberg::polynomial& q_logic_fft;
    barretenberg::polynomial& q_c;
    barretenberg::polynomial& q_c_fft;
};
} // namespace waffle
