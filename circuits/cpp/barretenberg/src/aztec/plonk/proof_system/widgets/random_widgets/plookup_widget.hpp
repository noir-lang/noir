#pragma once
#include "random_widget.hpp"

namespace waffle {
template <typename Field, typename Group, typename Transcript> class VerifierPLookupWidget {
  public:
    VerifierPLookupWidget();

    static Field compute_quotient_evaluation_contribution(verification_key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& t_eval,
                                                          const bool use_linearisation);

    static Field append_scalar_multiplication_inputs(verification_key*,
                                                     const Field& alpha_base,
                                                     const Transcript& transcript,
                                                     std::map<std::string, Field>& scalars,
                                                     const bool use_linearisation);
};

extern template class VerifierPLookupWidget<barretenberg::fr,
                                            barretenberg::g1::affine_element,
                                            transcript::StandardTranscript>;

class ProverPLookupWidget : public ProverRandomWidget {
  public:
    inline ProverPLookupWidget(proving_key*, program_witness*);
    inline ProverPLookupWidget(const ProverPLookupWidget& other);
    inline ProverPLookupWidget(ProverPLookupWidget&& other);
    inline ProverPLookupWidget& operator=(const ProverPLookupWidget& other);
    inline ProverPLookupWidget& operator=(ProverPLookupWidget&& other);

    inline void compute_sorted_list_commitment(transcript::StandardTranscript& transcript);

    inline void compute_grand_product_commitment(transcript::StandardTranscript& transcript);

    inline void compute_round_commitments(transcript::StandardTranscript& transcript,
                                          const size_t round_number,
                                          work_queue& queue) override;

    inline barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                          const transcript::StandardTranscript& transcript) override;
    inline barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                        const transcript::StandardTranscript& transcript,
                                                        barretenberg::polynomial& r) override;
};

} // namespace waffle

#include "./plookup_widget_impl.hpp"