#pragma once
#include "random_widget.hpp"

namespace waffle {
template <typename Field,
          typename Group,
          typename Transcript,
          const size_t num_roots_cut_out_of_vanishing_polynomial = 4>
class VerifierPlookupWidget {
  public:
    VerifierPlookupWidget();

    static Field compute_quotient_evaluation_contribution(typename Transcript::Key*,
                                                          const Field& alpha_base,
                                                          const Transcript& transcript,
                                                          Field& r_0,
                                                          const bool use_linearisation);

    static Field append_scalar_multiplication_inputs(typename Transcript::Key*,
                                                     const Field& alpha_base,
                                                     const Transcript& transcript,
                                                     std::map<std::string, Field>& scalars,
                                                     const bool use_linearisation);
};

extern template class VerifierPlookupWidget<barretenberg::fr,
                                            barretenberg::g1::affine_element,
                                            transcript::StandardTranscript>;

template <const size_t num_roots_cut_out_of_vanishing_polynomial = 4>
class ProverPlookupWidget : public ProverRandomWidget {
  public:
    inline ProverPlookupWidget(proving_key*);
    inline ProverPlookupWidget(const ProverPlookupWidget& other);
    inline ProverPlookupWidget(ProverPlookupWidget&& other);
    inline ProverPlookupWidget& operator=(const ProverPlookupWidget& other);
    inline ProverPlookupWidget& operator=(ProverPlookupWidget&& other);

    inline void compute_sorted_list_polynomial(transcript::StandardTranscript& transcript);

    inline void compute_grand_product_polynomial(transcript::StandardTranscript& transcript);

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