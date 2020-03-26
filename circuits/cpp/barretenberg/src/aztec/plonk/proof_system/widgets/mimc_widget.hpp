#pragma once
#include "base_widget.hpp"

namespace waffle {

template <typename Field, typename Group, typename Transcript> class VerifierMiMCWidget {
  public:
    inline VerifierMiMCWidget();

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

    inline static Field compute_quotient_evaluation_contribution(verification_key* key,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);
};

extern template class VerifierMiMCWidget<barretenberg::fr,
                                         barretenberg::g1::affine_element,
                                         transcript::StandardTranscript>;

class ProverMiMCWidget : public ProverBaseWidget {
  public:
    inline ProverMiMCWidget(proving_key* key, program_witness* witness);
    inline ProverMiMCWidget(const ProverMiMCWidget& other);
    inline ProverMiMCWidget(ProverMiMCWidget&& other);
    inline ProverMiMCWidget& operator=(const ProverMiMCWidget& other);
    inline ProverMiMCWidget& operator=(ProverMiMCWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    inline size_t compute_opening_poly_contribution(const size_t nu_index,
                                             const transcript::Transcript& transcript,
                                             barretenberg::fr* poly,
                                             barretenberg::fr*,
                                             const bool use_linerisation) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_mimc_selector;
    barretenberg::polynomial& q_mimc_coefficient;

    barretenberg::polynomial& q_mimc_selector_fft;
    barretenberg::polynomial& q_mimc_coefficient_fft;
};
} // namespace waffle

#include "mimc_widget_impl.hpp"