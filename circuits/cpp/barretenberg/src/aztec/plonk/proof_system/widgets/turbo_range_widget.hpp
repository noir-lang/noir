#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboRangeWidget {
  public:
    inline VerifierTurboRangeWidget();

    inline static VerifierBaseWidget::challenge_coefficients<Field> append_scalar_multiplication_inputs(
        verification_key*,
        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
        const Transcript& transcript,
        std::vector<Group>& points,
        std::vector<Field>& scalars,
        const bool use_linearisation);

    inline static size_t compute_batch_evaluation_contribution(verification_key*,
                                                               Field& batch_eval,
                                                               const size_t nu_base,
                                                               const Transcript& transcript,
                                                               const bool use_linearisation);

    inline static Field compute_quotient_evaluation_contribution(verification_key* key,
                                                                 const Field& alpha_base,
                                                                 const Transcript& transcript,
                                                                 Field& t_eval,
                                                                 const bool use_linearisation);
};

extern template class VerifierTurboRangeWidget<barretenberg::fr,
                                               barretenberg::g1::affine_element,
                                               transcript::StandardTranscript>;

class ProverTurboRangeWidget : public ProverBaseWidget {
  public:
    inline ProverTurboRangeWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverTurboRangeWidget(const ProverTurboRangeWidget& other);
    inline ProverTurboRangeWidget(ProverTurboRangeWidget&& other);
    inline ProverTurboRangeWidget& operator=(const ProverTurboRangeWidget& other);
    inline ProverTurboRangeWidget& operator=(ProverTurboRangeWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                          const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                        const transcript::Transcript& transcript,
                                                        barretenberg::polynomial& r) override;
    inline size_t compute_opening_poly_contribution(const size_t nu_index,
                                                    const transcript::Transcript& transcript,
                                                    barretenberg::fr* poly,
                                                    barretenberg::fr*,
                                                    const bool use_linearisation) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_range;
    barretenberg::polynomial& q_range_fft;
};
} // namespace waffle

#include "turbo_range_widget_impl.hpp"