#pragma once
#include "turbo_arithmetic_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboFixedBaseWidget {
  public:
    VerifierTurboFixedBaseWidget();

    static VerifierBaseWidget::challenge_coefficients<Field> append_scalar_multiplication_inputs(
        verification_key*,
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
                                                          const bool use_lineraisation);
};

extern template class VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                   barretenberg::g1::affine_element,
                                                   transcript::StandardTranscript>;

class ProverTurboFixedBaseWidget : public ProverTurboArithmeticWidget {
  public:
    ProverTurboFixedBaseWidget(proving_key* input_key, program_witness* input_witness);
    ProverTurboFixedBaseWidget(const ProverTurboFixedBaseWidget& other);
    ProverTurboFixedBaseWidget(ProverTurboFixedBaseWidget&& other);
    ProverTurboFixedBaseWidget& operator=(const ProverTurboFixedBaseWidget& other);
    ProverTurboFixedBaseWidget& operator=(ProverTurboFixedBaseWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    size_t compute_opening_poly_contribution(const size_t nu_index,
                                             const transcript::Transcript&,
                                             barretenberg::fr*,
                                             barretenberg::fr*,
                                             const bool) override;

    void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_ecc_1;
    barretenberg::polynomial& q_ecc_1_fft;
};
} // namespace waffle
