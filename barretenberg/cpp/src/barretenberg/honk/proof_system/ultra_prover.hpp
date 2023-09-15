#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
#include "barretenberg/honk/instance/prover_instance.hpp"
#include "barretenberg/honk/pcs/gemini/gemini.hpp"
#include "barretenberg/honk/pcs/shplonk/shplonk.hpp"
#include "barretenberg/honk/proof_system/work_queue.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_output.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"

namespace proof_system::honk {

template <UltraFlavor Flavor> class UltraProver_ {
    using FF = typename Flavor::FF;
    using PCS = typename Flavor::PCS;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using ProvingKey = typename Flavor::ProvingKey;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Curve = typename Flavor::Curve;
    using Instance = ProverInstance_<Flavor>;

  public:
    explicit UltraProver_(std::shared_ptr<Instance>);
    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_sorted_list_accumulator_round();
    void execute_grand_product_computation_round();
    void execute_relation_check_rounds();
    void execute_univariatization_round();
    void execute_pcs_evaluation_round();
    void execute_shplonk_batched_quotient_round();
    void execute_shplonk_partial_evaluation_round();
    void execute_final_pcs_round();

    plonk::proof& export_proof();
    plonk::proof& construct_proof();

    ProverTranscript<FF> transcript;

    std::vector<FF> public_inputs;
    size_t pub_inputs_offset;

    CommitmentLabels commitment_labels;

    // Container for d + 1 Fold polynomials produced by Gemini
    std::vector<Polynomial> gemini_polynomials;

    Polynomial batched_quotient_Q; // batched quotient poly computed by Shplonk
    FF nu_challenge;               // needed in both Shplonk rounds

    Polynomial quotient_W;

    work_queue<Curve> queue;

    std::shared_ptr<Instance> instance;

    sumcheck::SumcheckOutput<Flavor> sumcheck_output;
    pcs::gemini::ProverOutput<Curve> gemini_output;
    pcs::shplonk::ProverOutput<Curve> shplonk_output;
    std::shared_ptr<CommitmentKey> pcs_commitment_key;

    using Gemini = pcs::gemini::GeminiProver_<Curve>;
    using Shplonk = pcs::shplonk::ShplonkProver_<Curve>;

  private:
    plonk::proof proof;
};

extern template class UltraProver_<honk::flavor::Ultra>;
extern template class UltraProver_<honk::flavor::UltraGrumpkin>;
extern template class UltraProver_<honk::flavor::GoblinUltra>;

using UltraProver = UltraProver_<honk::flavor::Ultra>;

} // namespace proof_system::honk
