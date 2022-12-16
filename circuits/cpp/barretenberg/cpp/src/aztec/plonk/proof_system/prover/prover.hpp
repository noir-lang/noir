#pragma once
#include "../../transcript/transcript_wrappers.hpp"
#include "../proving_key/proving_key.hpp"
#include "../types/plonk_proof.hpp"
#include "../types/program_settings.hpp"
#include "../widgets/random_widgets/random_widget.hpp"
#include "./work_queue.hpp"
#include "../widgets/transition_widgets/transition_widget.hpp"
#include "../commitment_scheme/commitment_scheme.hpp"
namespace waffle {

template <typename settings> class ProverBase {

  public:
    ProverBase(std::shared_ptr<proving_key> input_key = nullptr,
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

    void add_polynomial_evaluations_to_transcript();
    void compute_batch_opening_polynomials();
    void compute_wire_commitments();
    void compute_quotient_commitments();
    void init_quotient_polynomials();
    void compute_opening_elements();
    void add_plookup_memory_records_to_w_4();

    void compute_linearisation_coefficients();
    void add_blinding_to_quotient_polynomial_parts();
    void compute_lagrange_1_fft();
    waffle::plonk_proof& export_proof();
    waffle::plonk_proof& construct_proof();

    size_t get_circuit_size() const { return n; }

    void flush_queued_work_items() { queue.flush_queue(); }

    work_queue::work_item_info get_queued_work_item_info() const { return queue.get_queued_work_item_info(); }

    barretenberg::fr* get_scalar_multiplication_data(const size_t work_item_number) const
    {
        return queue.get_scalar_multiplication_data(work_item_number);
    }

    size_t get_scalar_multiplication_size(const size_t work_item_number) const
    {
        return queue.get_scalar_multiplication_size(work_item_number);
    }

    barretenberg::fr* get_ifft_data(const size_t work_item_number) const
    {
        return queue.get_ifft_data(work_item_number);
    }

    work_queue::queued_fft_inputs get_fft_data(const size_t work_item_number) const
    {
        return queue.get_fft_data(work_item_number);
    }

    void put_scalar_multiplication_data(const barretenberg::g1::affine_element result, const size_t work_item_number)
    {
        queue.put_scalar_multiplication_data(result, work_item_number);
    }

    void put_fft_data(barretenberg::fr* result, const size_t work_item_number)
    {
        queue.put_fft_data(result, work_item_number);
    }

    void put_ifft_data(barretenberg::fr* result, const size_t work_item_number)
    {
        queue.put_ifft_data(result, work_item_number);
    }

    void reset();

    size_t n;

    std::vector<std::unique_ptr<ProverRandomWidget>> random_widgets;
    std::vector<std::unique_ptr<widget::TransitionWidgetBase<barretenberg::fr>>> transition_widgets;
    transcript::StandardTranscript transcript;

    std::shared_ptr<proving_key> key;
    std::unique_ptr<CommitmentScheme> commitment_scheme;

    work_queue queue;

  private:
    waffle::plonk_proof proof;
};
extern template class ProverBase<unrolled_standard_settings>;
extern template class ProverBase<unrolled_turbo_settings>;
extern template class ProverBase<standard_settings>;
extern template class ProverBase<turbo_settings>;
extern template class ProverBase<ultra_settings>;

typedef ProverBase<unrolled_standard_settings> UnrolledProver;
typedef ProverBase<unrolled_turbo_settings> UnrolledTurboProver;
typedef ProverBase<unrolled_ultra_settings>
    UnrolledUltraProver; // TODO: maybe just return a templated proverbase so that I don't need separate casees for
                         // ultra vs ultra_to_standard...???
typedef ProverBase<unrolled_ultra_to_standard_settings> UnrolledUltraToStandardProver;
typedef ProverBase<unrolled_turbo_settings> UnrolledGenPermProver;

typedef ProverBase<standard_settings> Prover;
typedef ProverBase<turbo_settings> TurboProver;
typedef ProverBase<ultra_settings> UltraProver;
typedef ProverBase<turbo_settings> GenPermProver;

} // namespace waffle
