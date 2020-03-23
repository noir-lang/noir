#pragma once
#include "base_widget.hpp"

namespace waffle {

template <typename Field, typename Group, typename Transcript> class VerifierMiMCWidget {
  public:
    VerifierMiMCWidget();

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

    static Field compute_quotient_evaluation_contribution(verification_key* key,
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
    ProverMiMCWidget(proving_key* key, program_witness* witness);
    ProverMiMCWidget(const ProverMiMCWidget& other);
    ProverMiMCWidget(ProverMiMCWidget&& other);
    ProverMiMCWidget& operator=(const ProverMiMCWidget& other);
    ProverMiMCWidget& operator=(ProverMiMCWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript);
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    size_t compute_opening_poly_contribution(const size_t nu_index,
                                             const transcript::Transcript& transcript,
                                             barretenberg::fr* poly,
                                             barretenberg::fr*,
                                             const bool use_linerisation) override;

    void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_mimc_selector;
    barretenberg::polynomial& q_mimc_coefficient;

    barretenberg::polynomial& q_mimc_selector_fft;
    barretenberg::polynomial& q_mimc_coefficient_fft;
};
} // namespace waffle
