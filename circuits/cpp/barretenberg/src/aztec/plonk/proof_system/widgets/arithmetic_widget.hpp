#pragma once
#include "base_widget.hpp"

namespace waffle {
class VerifierArithmeticWidget : public VerifierBaseWidget {
  public:
    VerifierArithmeticWidget();

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key*,
                                                              const barretenberg::fr& alpha_base,
                                                              const transcript::Transcript& transcript,
                                                              barretenberg::fr& t_eval);

    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key*,
        const challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars);

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key*,
                                                           barretenberg::fr&,
                                                           const barretenberg::fr& nu_base,
                                                           const transcript::Transcript&);
};

class ProverArithmeticWidget : public ProverBaseWidget {
  public:
    ProverArithmeticWidget(proving_key*, program_witness*);
    ProverArithmeticWidget(const ProverArithmeticWidget& other);
    ProverArithmeticWidget(ProverArithmeticWidget&& other);
    ProverArithmeticWidget& operator=(const ProverArithmeticWidget& other);
    ProverArithmeticWidget& operator=(ProverArithmeticWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript);
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r);
    barretenberg::fr compute_opening_poly_contribution(const barretenberg::fr& nu_base,
                                                       const transcript::Transcript&,
                                                       barretenberg::fr*,
                                                       barretenberg::fr*);

    barretenberg::polynomial& q_1;
    barretenberg::polynomial& q_2;
    barretenberg::polynomial& q_3;
    barretenberg::polynomial& q_m;
    barretenberg::polynomial& q_c;

    barretenberg::polynomial& q_1_fft;
    barretenberg::polynomial& q_2_fft;
    barretenberg::polynomial& q_3_fft;
    barretenberg::polynomial& q_m_fft;
    barretenberg::polynomial& q_c_fft;
};
} // namespace waffle
