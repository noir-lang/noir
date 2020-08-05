#pragma once
#include "random_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierPermutationWidget {
  public:
    VerifierPermutationWidget();

    static Field compute_quotient_evaluation_contribution(verification_key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation,
                                                          const bool idpolys = false);

    static Field append_scalar_multiplication_inputs(verification_key*,
                                                     const Field& alpha_base,
                                                     const Transcript& transcript,
                                                     std::map<std::string, Field>& scalars,
                                                     const bool use_linearisation,
                                                     const bool idpolys = false);
};

extern template class VerifierPermutationWidget<barretenberg::fr,
                                                barretenberg::g1::affine_element,
                                                transcript::StandardTranscript>;

template <size_t program_width, bool idpolys = false> class ProverPermutationWidget : public ProverRandomWidget {
  public:
    ProverPermutationWidget(proving_key*, program_witness*);
    ProverPermutationWidget(const ProverPermutationWidget& other);
    ProverPermutationWidget(ProverPermutationWidget&& other);
    ProverPermutationWidget& operator=(const ProverPermutationWidget& other);
    ProverPermutationWidget& operator=(ProverPermutationWidget&& other);

    void compute_round_commitments(transcript::StandardTranscript& transcript,
                                   const size_t round_number,
                                   work_queue& queue) override;

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::StandardTranscript& transcript) override;
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::StandardTranscript& transcript,
                                                 barretenberg::polynomial& r) override;
};

} // namespace waffle

#include "./permutation_widget_impl.hpp"