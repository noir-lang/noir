#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierArithmeticWidget {
  public:
    inline VerifierArithmeticWidget();

    inline static Field compute_quotient_evaluation_contribution(verification_key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);

    inline static VerifierBaseWidget::challenge_coefficients<Field> append_scalar_multiplication_inputs(
        verification_key*,
        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
        const Transcript& transcript,
        std::vector<Group>& points,
        std::vector<Field>& scalars,
        const bool use_linearisation);

    inline static size_t compute_batch_evaluation_contribution(verification_key*,
                                                        Field& batch_eval,
                                                        const size_t nu_index,
                                                        const Transcript& transcript,
                                                        const bool use_linearisation);
};

extern template class VerifierArithmeticWidget<barretenberg::fr,
                                               barretenberg::g1::affine_element,
                                               transcript::StandardTranscript>;

class ProverArithmeticWidget : public ProverBaseWidget {
  public:
    inline ProverArithmeticWidget(proving_key*, program_witness*);
    inline ProverArithmeticWidget(const ProverArithmeticWidget& other);
    inline ProverArithmeticWidget(ProverArithmeticWidget&& other);
    inline ProverArithmeticWidget& operator=(const ProverArithmeticWidget& other);
    inline ProverArithmeticWidget& operator=(ProverArithmeticWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    inline size_t compute_opening_poly_contribution(const size_t nu_index,
                                             const transcript::Transcript&,
                                             barretenberg::fr*,
                                             barretenberg::fr*,
                                             const bool) override;

    inline void compute_transcript_elements(transcript::Transcript&, const bool) override;

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

#include "arithmetic_widget_impl.hpp"