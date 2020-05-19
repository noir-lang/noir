#pragma once
#include "base_widget.hpp"

namespace waffle {
/**
 * ProverBoolWidget : constraint that constrains left and right wire values to be booleans
 *
 **/

template <typename Field, typename Group, typename Transcript> class VerifierBoolWidget {
  public:
    inline VerifierBoolWidget();

    inline static Field compute_quotient_evaluation_contribution(
        verification_key*, const Field&, const Transcript&, Field&, const bool);

    inline static void compute_batch_evaluation_contribution(verification_key*,
                                                             Field& batch_eval,
                                                             const Transcript& transcript,
                                                             const bool use_linearisation);

    inline static Field append_scalar_multiplication_inputs(verification_key* key,
                                                            const Field& alpha_base,
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
    inline ProverBoolWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverBoolWidget(const ProverBoolWidget& other);
    inline ProverBoolWidget(ProverBoolWidget&& other);
    inline ProverBoolWidget& operator=(const ProverBoolWidget& other);
    inline ProverBoolWidget& operator=(ProverBoolWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                          const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                        const transcript::Transcript& transcript,
                                                        barretenberg::polynomial& r) override;

    inline void compute_opening_poly_contribution(const transcript::Transcript&, const bool) override;

    inline void compute_transcript_elements(transcript::Transcript&, const bool) override;

    barretenberg::polynomial& q_bl;
    barretenberg::polynomial& q_br;
    barretenberg::polynomial& q_bo;

    barretenberg::polynomial& q_bl_fft;
    barretenberg::polynomial& q_br_fft;
    barretenberg::polynomial& q_bo_fft;
};
} // namespace waffle

#include "bool_widget_impl.hpp"