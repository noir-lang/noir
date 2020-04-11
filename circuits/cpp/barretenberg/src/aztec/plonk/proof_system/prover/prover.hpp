#pragma once
#include "../../transcript/transcript_wrappers.hpp"
#include "../proving_key/proving_key.hpp"
#include "../types/plonk_proof.hpp"
#include "../types/program_settings.hpp"
#include "../types/program_witness.hpp"
#include "../widgets/base_widget.hpp"
#include "./work_queue.hpp"

namespace waffle {

template <typename settings> class ProverBase {

  public:
    ProverBase(std::shared_ptr<proving_key> input_key = nullptr,
               std::shared_ptr<program_witness> input_witness = nullptr,
               const transcript::Manifest& manifest = transcript::Manifest({}));
    ProverBase(ProverBase&& other);
    ProverBase(const ProverBase& other) = delete;
    ProverBase& operator=(const ProverBase& other) = delete;
    ProverBase& operator=(ProverBase&& other);

    void execute_preamble_round();
    void execute_first_round();
    void execute_second_round();
    void execute_third_round();
    void execute_fourth_round();
    void execute_fifth_round();
    void execute_sixth_round();

    void compute_wire_pre_commitments();
    void compute_quotient_pre_commitment();
    void init_quotient_polynomials();
    void compute_opening_elements();

    barretenberg::fr compute_linearisation_coefficients();
    waffle::plonk_proof& export_proof();
    waffle::plonk_proof& construct_proof();

    size_t get_circuit_size() const { return n; }

    size_t get_num_queued_scalar_multiplications() const { return queue.get_num_queued_scalar_multiplications(); }

    barretenberg::fr* get_scalar_multiplication_data(const size_t work_item_number) const
    {
        return queue.get_scalar_multiplication_data(work_item_number);
    }

    void put_scalar_multiplication_data(const barretenberg::g1::affine_element result, const size_t work_item_number)
    {
        queue.put_scalar_multiplication_data(result, work_item_number);
    }

    void reset();

    size_t n;

    std::vector<uint32_t> sigma_1_mapping;
    std::vector<uint32_t> sigma_2_mapping;
    std::vector<uint32_t> sigma_3_mapping;

    std::vector<std::unique_ptr<ProverBaseWidget>> widgets;
    transcript::StandardTranscript transcript;

    std::shared_ptr<proving_key> key;
    std::shared_ptr<program_witness> witness;

    work_queue queue;
    bool uses_quotient_mid;

  private:
    waffle::plonk_proof proof;
};
extern template class ProverBase<unrolled_standard_settings>;
extern template class ProverBase<unrolled_turbo_settings>;
extern template class ProverBase<standard_settings>;
extern template class ProverBase<turbo_settings>;

typedef ProverBase<unrolled_standard_settings> UnrolledProver;
typedef ProverBase<unrolled_turbo_settings> UnrolledTurboProver;
typedef ProverBase<standard_settings> Prover;
typedef ProverBase<turbo_settings> TurboProver;
typedef ProverBase<turbo_settings> PLookupProver;
typedef ProverBase<unrolled_turbo_settings> UnrolledPLookupProver;

} // namespace waffle
