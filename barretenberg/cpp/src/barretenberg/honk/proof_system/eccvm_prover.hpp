#pragma once
#include "barretenberg/honk/flavor/ecc_vm.hpp"
#include "barretenberg/honk/pcs/gemini/gemini.hpp"
#include "barretenberg/honk/pcs/shplonk/shplonk.hpp"
#include "barretenberg/honk/proof_system/work_queue.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_output.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"

namespace proof_system::honk {

// We won't compile this class with honk::flavor::Standard, but we will like want to compile it (at least for testing)
// with a flavor that uses the curve Grumpkin, or a flavor that does/does not have zk, etc.
template <ECCVMFlavor Flavor> class ECCVMProver_ {

    using FF = typename Flavor::FF;
    using PCS = typename Flavor::PCS;
    using PCSCommitmentKey = typename Flavor::CommitmentKey;
    using ProvingKey = typename Flavor::ProvingKey;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Curve = typename Flavor::Curve;

  public:
    explicit ECCVMProver_(std::shared_ptr<ProvingKey> input_key, std::shared_ptr<PCSCommitmentKey> commitment_key);

    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_log_derivative_commitments_round();
    void execute_grand_product_computation_round();
    void execute_relation_check_rounds();
    void execute_univariatization_round();
    void execute_pcs_evaluation_round();
    void execute_shplonk_batched_quotient_round();
    void execute_shplonk_partial_evaluation_round();
    void execute_final_pcs_round();

    void compute_wire_commitments();

    plonk::proof& export_proof();
    plonk::proof& construct_proof();

    ProverTranscript<FF> transcript;

    std::vector<FF> public_inputs;

    proof_system::RelationParameters<FF> relation_parameters;

    std::shared_ptr<ProvingKey> key;

    // Container for spans of all polynomials required by the prover (i.e. all multivariates evaluated by Sumcheck).
    ProverPolynomials prover_polynomials;

    CommitmentLabels commitment_labels;

    // Container for d + 1 Fold polynomials produced by Gemini
    std::vector<Polynomial> gemini_polynomials;

    Polynomial batched_quotient_Q; // batched quotient poly computed by Shplonk
    FF nu_challenge;               // needed in both Shplonk rounds

    Polynomial quotient_W;

    work_queue<Curve> queue;

    sumcheck::SumcheckOutput<Flavor> sumcheck_output;
    pcs::gemini::ProverOutput<Curve> gemini_output;
    pcs::shplonk::ProverOutput<Curve> shplonk_output;
    std::shared_ptr<PCSCommitmentKey> pcs_commitment_key;

    using Gemini = pcs::gemini::GeminiProver_<Curve>;
    using Shplonk = pcs::shplonk::ShplonkProver_<Curve>;

  private:
    plonk::proof proof;
};

extern template class ECCVMProver_<honk::flavor::ECCVM>;
extern template class ECCVMProver_<honk::flavor::ECCVMGrumpkin>;

using ECCVMProver = ECCVMProver_<honk::flavor::ECCVM>;

} // namespace proof_system::honk
