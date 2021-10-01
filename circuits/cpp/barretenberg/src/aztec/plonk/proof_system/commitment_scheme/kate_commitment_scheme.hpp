#pragma once
#include "commitment_scheme.hpp"

namespace waffle {

template <typename settings> class KateCommitmentScheme : public CommitmentScheme {
  public:
    KateCommitmentScheme();

    void commit(fr* coefficients, std::string tag, fr item_constant, work_queue& queue) override;

    void compute_opening_polynomial(const fr* src,
                                    fr* dest,
                                    const fr& z,
                                    const size_t n,
                                    std::string tag,
                                    fr item_constant,
                                    work_queue& queue) override;

    void generic_batch_open(const fr* src,
                            fr* dest,
                            const size_t num_polynomials,
                            const fr* z_points,
                            const size_t num_z_points,
                            const fr* challenges,
                            const size_t n,
                            std::string* tags,
                            fr* item_constants,
                            work_queue& queue) override;

    void batch_open(const transcript::StandardTranscript& transcript,
                    work_queue& queue,
                    std::shared_ptr<proving_key> input_key = nullptr,
                    std::shared_ptr<program_witness> witness = nullptr) override;

    void batch_verify(const transcript::StandardTranscript& transcript,
                      std::map<std::string, g1::affine_element>& kate_g1_elements,
                      std::map<std::string, fr>& kate_fr_elements,
                      std::shared_ptr<verification_key> input_key = nullptr,
                      const barretenberg::fr& r_0 = 0) override;

    void add_opening_evaluations_to_transcript(transcript::StandardTranscript& transcript,
                                               std::shared_ptr<proving_key> input_key = nullptr,
                                               std::shared_ptr<program_witness> witness = nullptr,
                                               bool in_lagrange_form = false) override;

  private:
    waffle::commitment_open_proof kate_open_proof;
};

extern template class KateCommitmentScheme<unrolled_standard_settings>;
extern template class KateCommitmentScheme<unrolled_turbo_settings>;
extern template class KateCommitmentScheme<unrolled_plookup_settings>;
extern template class KateCommitmentScheme<standard_settings>;
extern template class KateCommitmentScheme<turbo_settings>;
extern template class KateCommitmentScheme<plookup_settings>;

} // namespace waffle