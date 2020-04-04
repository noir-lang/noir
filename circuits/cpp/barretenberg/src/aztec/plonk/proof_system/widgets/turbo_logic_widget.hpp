#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboLogicWidget {
  public:
    VerifierTurboLogicWidget();

    static Field append_scalar_multiplication_inputs(verification_key* key,
                                                     const Field& alpha_base,
                                                     const Transcript& transcript,
                                                     std::vector<Group>& points,
                                                     std::vector<Field>& scalars,
                                                     const bool use_linearisation);

    static void compute_batch_evaluation_contribution(verification_key*,
                                                        Field& batch_eval,
                                                        const Transcript& transcript,
                                                        const bool use_linearisation);

    static Field compute_quotient_evaluation_contribution(verification_key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);
};

extern template class VerifierTurboLogicWidget<barretenberg::fr,
                                               barretenberg::g1::affine_element,
                                               transcript::StandardTranscript>;

class ProverTurboLogicWidget : public ProverBaseWidget {
  public:
    ProverTurboLogicWidget(proving_key* input_key, program_witness* input_witness);
    ProverTurboLogicWidget(const ProverTurboLogicWidget& other);
    ProverTurboLogicWidget(ProverTurboLogicWidget&& other);
    ProverTurboLogicWidget& operator=(const ProverTurboLogicWidget& other);
    ProverTurboLogicWidget& operator=(ProverTurboLogicWidget&& other);

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    void compute_opening_poly_contribution(const transcript::Transcript&, const bool) override;

    void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_logic;
    barretenberg::polynomial& q_logic_fft;
    barretenberg::polynomial& q_c;
    barretenberg::polynomial& q_c_fft;
};
} // namespace waffle
