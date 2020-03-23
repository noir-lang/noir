#pragma once
#include "base_widget.hpp"

namespace waffle {
/**
 * ProverBoolWidget : constraint that constrains left and right wire values to be booleans
 *
 **/

template <typename Field, typename Group, typename Transcript> class VerifierBoolWidget {
  public:
    VerifierBoolWidget();

    static Field compute_quotient_evaluation_contribution(
        verification_key*, const Field&, const Transcript&, Field&, const bool);

    static size_t compute_batch_evaluation_contribution(verification_key*,
                                                        Field& batch_eval,
                                                        const size_t nu_index,
                                                        const Transcript& transcript,
                                                        const bool use_linearisation);

    static VerifierBaseWidget::challenge_coefficients<Field> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
        const Transcript& transcript,
        std::vector<Group>& points,
        std::vector<Field>& scalars,
        const bool use_linearisation);
};

extern template class VerifierBoolWidget<barretenberg::fr,
                                         barretenberg::g1::affine_element,
                                         transcript::StandardTranscript>;

class ProverBoolWidget : public ProverBaseWidget {
  public:
    ProverBoolWidget(proving_key* input_key, program_witness* input_witness);
    ProverBoolWidget(const ProverBoolWidget& other);
    ProverBoolWidget(ProverBoolWidget&& other);
    ProverBoolWidget& operator=(const ProverBoolWidget& other);
    ProverBoolWidget& operator=(ProverBoolWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;

    size_t compute_opening_poly_contribution(
        const size_t nu_index, const transcript::Transcript&, barretenberg::fr*, barretenberg::fr*, const bool) override;

    void compute_transcript_elements(transcript::Transcript&, const bool) override;

    barretenberg::polynomial& q_bl;
    barretenberg::polynomial& q_br;
    barretenberg::polynomial& q_bo;

    barretenberg::polynomial& q_bl_fft;
    barretenberg::polynomial& q_br_fft;
    barretenberg::polynomial& q_bo_fft;
};
} // namespace waffle
