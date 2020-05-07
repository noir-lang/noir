#pragma once
#include "base_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierTurboArithmeticWidget {
  public:
    inline VerifierTurboArithmeticWidget();

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

extern template class VerifierTurboArithmeticWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>;

class ProverTurboArithmeticWidget : public ProverBaseWidget {
  public:
    inline ProverTurboArithmeticWidget(proving_key* input_key, program_witness* input_witness);
    inline ProverTurboArithmeticWidget(const ProverTurboArithmeticWidget& other);
    inline ProverTurboArithmeticWidget(ProverTurboArithmeticWidget&& other);
    inline ProverTurboArithmeticWidget& operator=(const ProverTurboArithmeticWidget& other);
    inline ProverTurboArithmeticWidget& operator=(ProverTurboArithmeticWidget&& other);

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                          const transcript::Transcript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                        const transcript::Transcript& transcript,
                                                        barretenberg::polynomial& r) override;
    inline void compute_opening_poly_contribution(const transcript::Transcript&, const bool) override;

    inline void compute_transcript_elements(transcript::Transcript& transcript, const bool use_linearisation) override;

    barretenberg::polynomial& q_1;
    barretenberg::polynomial& q_2;
    barretenberg::polynomial& q_3;
    barretenberg::polynomial& q_4;
    barretenberg::polynomial& q_5;
    barretenberg::polynomial& q_m;
    barretenberg::polynomial& q_c;
    barretenberg::polynomial& q_arith;

    barretenberg::polynomial& q_1_fft;
    barretenberg::polynomial& q_2_fft;
    barretenberg::polynomial& q_3_fft;
    barretenberg::polynomial& q_4_fft;
    barretenberg::polynomial& q_5_fft;
    barretenberg::polynomial& q_m_fft;
    barretenberg::polynomial& q_c_fft;
    barretenberg::polynomial& q_arith_fft;
};
} // namespace waffle

#include "turbo_arithmetic_widget_impl.hpp"