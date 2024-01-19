#pragma once
#include "../../../polynomials/polynomial.hpp"
#include "../../../polynomials/polynomial_arithmetic.hpp"
#include "../types/commitment_open_proof.hpp"
#include "../types/program_settings.hpp"
#include "barretenberg/plonk/work_queue/work_queue.hpp"

namespace bb::plonk {

class CommitmentScheme {
  public:
    using fr = bb::fr;

    // Constructors for CommitmentScheme
    CommitmentScheme() {}

    virtual ~CommitmentScheme() {}

    virtual void commit(std::shared_ptr<fr[]> coefficients, std::string tag, fr item_constant, work_queue& queue) = 0;

    virtual void compute_opening_polynomial(const fr* src, fr* dest, const fr& z, const size_t n) = 0;

    virtual void generic_batch_open(const fr* src,
                                    fr* dest,
                                    const size_t num_polynomials,
                                    const fr* z_points,
                                    const size_t num_z_points,
                                    const fr* challenges,
                                    const size_t n,
                                    std::string* tags,
                                    fr* item_constants,
                                    work_queue& queue) = 0;

    virtual void batch_open(const transcript::StandardTranscript& transcript,
                            work_queue& queue,
                            std::shared_ptr<plonk::proving_key> input_key = nullptr) = 0;

    virtual void batch_verify(const transcript::StandardTranscript& transcript,
                              std::map<std::string, g1::affine_element>& kate_g1_elements,
                              std::map<std::string, fr>& kate_fr_elements,
                              std::shared_ptr<plonk::verification_key> input_key = nullptr) = 0;

    virtual void add_opening_evaluations_to_transcript(transcript::StandardTranscript& trancript,
                                                       std::shared_ptr<plonk::proving_key> input_key = nullptr,
                                                       bool in_lagrange_form = false) = 0;
};

} // namespace bb::plonk
