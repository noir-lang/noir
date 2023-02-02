#pragma once
#include <proof_system/proving_key/proving_key.hpp>
#include <honk/pcs/commitment_key.hpp>
#include <plonk/proof_system/types/plonk_proof.hpp>
#include <plonk/proof_system/types/program_settings.hpp>
#include <honk/pcs/gemini/gemini.hpp>
#include <honk/pcs/shplonk/shplonk_single.hpp>
#include <honk/pcs/kzg/kzg.hpp>

namespace honk {

template <typename settings> class Prover {

  public:
    // TODO(luke): update this appropriately to work with Honk Manifest
    Prover(std::shared_ptr<waffle::proving_key> input_key = nullptr,
           const transcript::Manifest& manifest = transcript::Manifest());

    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_tables_round();
    void execute_grand_product_computation_round();
    void execute_relation_check_rounds();
    void execute_univariatization_round();
    void execute_pcs_evaluation_round();
    void execute_shplonk_round();
    void execute_kzg_round();

    void compute_wire_commitments();

    void compute_grand_product_polynomial(barretenberg::fr beta, barretenberg::fr gamma);

    waffle::plonk_proof& export_proof();
    waffle::plonk_proof& construct_proof();

    size_t get_circuit_size() const { return circuit_size; }

    // TODO(luke): Eventually get rid of this but leave it for convenience for now
    const size_t circuit_size;

    // No more widgets. The new 'relations' may be owned by Sumcheck rather than Prover...
    // std::vector<std::unique_ptr<ProverRandomWidget>> random_widgets;
    // std::vector<std::unique_ptr<widget::TransitionWidgetBase<barretenberg::fr>>> transition_widgets;

    // TODO(luke): maybe pointer instead?
    transcript::StandardTranscript transcript;

    std::shared_ptr<waffle::proving_key> key;

    std::shared_ptr<pcs::kzg::CommitmentKey> commitment_key;

    // Honk only needs a small portion of the functionality but may be fine to use existing work_queue
    // NOTE: this is not currently in use, but it may well be used in the future.
    // TODO(Adrian): Uncomment when we need this again.
    // waffle::work_queue queue;
    // void flush_queued_work_items() { queue.flush_queue(); }
    // waffle::work_queue::work_item_info get_queued_work_item_info() const {
    //     return queue.get_queued_work_item_info();
    // }
    // size_t get_scalar_multiplication_size(const size_t work_item_number) const
    // {
    //     return queue.get_scalar_multiplication_size(work_item_number);
    // }

    // This makes 'settings' accesible from Prover
    using settings_ = settings;

    pcs::gemini::ProverOutput<pcs::kzg::Params> gemini_output;
    pcs::shplonk::ProverOutput<pcs::kzg::Params> shplonk_output;

  private:
    waffle::plonk_proof proof;
};

// TODO(luke): need equivalent notion of settings for Honk
extern template class Prover<waffle::standard_settings>;

using StandardProver = Prover<waffle::standard_settings>; // TODO(Cody): Delete?
using StandardUnrolledProver = Prover<waffle::standard_settings>;

} // namespace honk
