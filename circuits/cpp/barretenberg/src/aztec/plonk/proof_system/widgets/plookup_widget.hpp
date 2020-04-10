#pragma once
#include "base_widget.hpp"

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
                                                     std::vector<Group>& points,
                                                     std::vector<Field>& scalars,
                                                     const bool use_linearisation);

    static void compute_batch_evaluation_contribution(verification_key*,
                                                      Field& batch_eval,
                                                      const Transcript& transcript,
                                                      const bool use_linearisation);
};

extern template class VerifierPLookupWidget<barretenberg::fr,
                                            barretenberg::g1::affine_element,
                                            transcript::StandardTranscript>;

class ProverPLookupWidget : public ProverBaseWidget {
  public:
    ProverPLookupWidget(proving_key*, program_witness*);
    ProverPLookupWidget(const ProverPLookupWidget& other);
    ProverPLookupWidget(ProverPLookupWidget&& other);
    ProverPLookupWidget& operator=(const ProverPLookupWidget& other);
    ProverPLookupWidget& operator=(ProverPLookupWidget&& other);

    void compute_sorted_list_commitment(transcript::Transcript& transcript);

    void compute_grand_product_commitment(transcript::Transcript& transcript);

    void compute_round_commitments(transcript::Transcript& transcript,
                                   const size_t round_number,
                                   work_queue& queue) override;

    barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                   const transcript::Transcript& transcript) override;
    barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                 const transcript::Transcript& transcript,
                                                 barretenberg::polynomial& r) override;
    void compute_opening_poly_contribution(const transcript::Transcript&, const bool) override;

    void compute_transcript_elements(transcript::Transcript&, const bool) override;
};

} // namespace waffle

#include "./plookup_widget_impl.hpp"