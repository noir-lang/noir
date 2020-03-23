#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboArithmeticWidget {
  public:
    VerifierTurboArithmeticWidget();

    static VerifierBaseWidget::challenge_coefficients<Field> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
        const Transcript& transcript,
        std::vector<Group>& points,
        std::vector<Field>& scalars,
        const bool use_linearisation);

    static size_t compute_batch_evaluation_contribution(verification_key*,
                                                        Field& batch_eval,
                                                        const size_t nu_index,
                                                        const Transcript& transcript,
                                                        const bool use_linearisation);

    static Field compute_quotient_evaluation_contribution(verification_key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);
};

extern template class VerifierTurboArithmeticWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>;

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
                                                 barretenberg::polynomial& r) override;
    size_t compute_opening_poly_contribution(
        const size_t nu_index, const transcript::Transcript&, barretenberg::fr*, barretenberg::fr*, const bool) override;

    void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

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