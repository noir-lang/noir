#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierGenPermRangeWidget {
  public:
    inline VerifierGenPermRangeWidget();

    inline static Field append_scalar_multiplication_inputs(verification_key* key,
                                                     const Field& alpha_base,
                                                     const Transcript& transcript,
                                                     std::vector<Group>& points,
                                                     std::vector<Field>& scalars,
                                                     const bool use_linearisation);

    inline static void compute_batch_evaluation_contribution(verification_key*,
                                                        Field& batch_eval,
                                                        const Transcript& transcript,
                                                        const bool use_linearisation);

    inline static Field compute_quotient_evaluation_contribution(verification_key* key,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);
};

extern template class VerifierGenPermRangeWidget<barretenberg::fr,
                                               barretenberg::g1::affine_element,
                                               transcript::StandardTranscript>;

class ProverGenPermRangeWidget : public ProverBaseWidget {
  public:
    inline ProverGenPermRangeWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverGenPermRangeWidget(const ProverGenPermRangeWidget& other);
    inline ProverGenPermRangeWidget(ProverGenPermRangeWidget&& other);
    inline ProverGenPermRangeWidget& operator=(const ProverGenPermRangeWidget& other);
    inline ProverGenPermRangeWidget& operator=(ProverGenPermRangeWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    inline void compute_opening_poly_contribution(const transcript::Transcript& transcript,
                                           const bool use_linearisation) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_range;
    barretenberg::polynomial& q_range_fft;
};
} // namespace waffle

#include "turbo_range_widget_impl.hpp"