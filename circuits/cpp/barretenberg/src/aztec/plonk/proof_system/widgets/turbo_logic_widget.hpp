#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboLogicWidget {
  public:
    inline VerifierTurboLogicWidget();

    inline static Field append_scalar_multiplication_inputs(verification_key* key,
                                                            const Field& alpha_base,
                                                            const Transcript& transcript,
                                                            std::map<std::string, Field>& scalars,
                                                            const bool use_linearisation);

    inline static void compute_batch_evaluation_contribution(verification_key*,
                                                             Field& batch_eval,
                                                             const Transcript& transcript,
                                                             const bool use_linearisation);

    inline static Field compute_quotient_evaluation_contribution(verification_key*,
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
    inline ProverTurboLogicWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverTurboLogicWidget(const ProverTurboLogicWidget& other);
    inline ProverTurboLogicWidget(ProverTurboLogicWidget&& other);
    inline ProverTurboLogicWidget& operator=(const ProverTurboLogicWidget& other);
    inline ProverTurboLogicWidget& operator=(ProverTurboLogicWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                          const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                        const transcript::Transcript& transcript,
                                                        barretenberg::polynomial& r) override;
    inline void compute_opening_poly_contribution(const transcript::Transcript&, const bool) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_logic;
    barretenberg::polynomial& q_logic_fft;
    barretenberg::polynomial& q_c;
    barretenberg::polynomial& q_c_fft;
};
} // namespace waffle

#include "turbo_logic_widget_impl.hpp"