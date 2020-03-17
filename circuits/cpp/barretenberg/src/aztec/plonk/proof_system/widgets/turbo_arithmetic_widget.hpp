#pragma once
#include "base_widget.hpp"

namespace waffle {
class VerifierTurboArithmeticWidget : public VerifierBaseWidget {
  public:
    VerifierTurboArithmeticWidget();

    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key* key,
        const challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars);

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key*,
                                                                  barretenberg::fr& batch_eval,
                                                                  const barretenberg::fr& nu_base,
                                                                  const transcript::Transcript& transcript);

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key*,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::Transcript& transcript,
                                                                     barretenberg::fr&);
};

class ProverTurboArithmeticWidget : public ProverBaseWidget {
  public:
    ProverTurboArithmeticWidget(proving_key* input_key, program_witness* input_witness);
    ProverTurboArithmeticWidget(const ProverTurboArithmeticWidget& other);
    ProverTurboArithmeticWidget(ProverTurboArithmeticWidget&& other);
    ProverTurboArithmeticWidget& operator=(const ProverTurboArithmeticWidget& other);
    ProverTurboArithmeticWidget& operator=(ProverTurboArithmeticWidget&& other);

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

    barretenberg::polynomial& q_1;
    barretenberg::polynomial& q_2;
    barretenberg::polynomial& q_3;
    barretenberg::polynomial& q_4;
    barretenberg::polynomial& q_5;
    barretenberg::polynomial& q_m;
    barretenberg::polynomial& q_c;
    barretenberg::polynomial& q_arith;

    barretenberg::polynomial& q_1_fft;
    barretenberg::polynomial& q_2_fft;
    barretenberg::polynomial& q_3_fft;
    barretenberg::polynomial& q_4_fft;
    barretenberg::polynomial& q_5_fft;
    barretenberg::polynomial& q_m_fft;
    barretenberg::polynomial& q_c_fft;
    barretenberg::polynomial& q_arith_fft;
};
} // namespace waffle