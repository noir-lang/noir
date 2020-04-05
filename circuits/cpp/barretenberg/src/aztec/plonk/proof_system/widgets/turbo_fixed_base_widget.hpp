#pragma once
#include "turbo_arithmetic_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboFixedBaseWidget {
  public:
    inline VerifierTurboFixedBaseWidget();

    inline static Field append_scalar_multiplication_inputs(verification_key*,
                                                     const Field& alpha_base,
                                                     const Transcript& transcript,
                                                     std::vector<Group>& points,
                                                     std::vector<Field>& scalars,
                                                     const bool use_linearisation);

    inline static void compute_batch_evaluation_contribution(verification_key*,
                                                      Field& batch_eval,
                                                      const Transcript& transcript,
                                                      const bool use_linearisation);

    inline static Field compute_quotient_evaluation_contribution(verification_key*,
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
    inline ProverTurboFixedBaseWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverTurboFixedBaseWidget(const ProverTurboFixedBaseWidget& other);
    inline ProverTurboFixedBaseWidget(ProverTurboFixedBaseWidget&& other);
    inline ProverTurboFixedBaseWidget& operator=(const ProverTurboFixedBaseWidget& other);
    inline ProverTurboFixedBaseWidget& operator=(ProverTurboFixedBaseWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    inline void compute_opening_poly_contribution(const transcript::Transcript&, const bool) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_ecc_1;
    barretenberg::polynomial& q_ecc_1_fft;
};
} // namespace waffle

#include "turbo_fixed_base_widget_impl.hpp"