#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierSequentialWidget {
  public:
    inline VerifierSequentialWidget();

    inline static VerifierBaseWidget::challenge_coefficients<Field> append_scalar_multiplication_inputs(
        verification_key* key,
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

    inline static Field compute_quotient_evaluation_contribution(verification_key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);
};

extern template class VerifierSequentialWidget<barretenberg::fr,
                                               barretenberg::g1::affine_element,
                                               transcript::StandardTranscript>;

class ProverSequentialWidget : public ProverBaseWidget {
  public:
    inline ProverSequentialWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverSequentialWidget(const ProverSequentialWidget& other);
    inline ProverSequentialWidget(ProverSequentialWidget&& other);
    inline ProverSequentialWidget& operator=(const ProverSequentialWidget& other);
    inline ProverSequentialWidget& operator=(ProverSequentialWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;

    inline size_t compute_opening_poly_contribution(
        const size_t, const transcript::Transcript&, barretenberg::fr*, barretenberg::fr*, const bool) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_3_next;

    barretenberg::polynomial& q_3_next_fft;
};
} // namespace waffle

#include "sequential_widget_impl.hpp"