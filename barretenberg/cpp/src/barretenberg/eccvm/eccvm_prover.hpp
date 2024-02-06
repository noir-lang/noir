#pragma once
#include "barretenberg/commitment_schemes/gemini/gemini.hpp"
#include "barretenberg/commitment_schemes/shplonk/shplonk.hpp"
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/goblin/translation_evaluations.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

// We won't compile this class with Standard, but we will like want to compile it (at least for testing)
// with a flavor that uses the curve Grumpkin, or a flavor that does/does not have zk, etc.
template <IsECCVMFlavor Flavor> class ECCVMProver_ {

    using FF = typename Flavor::FF;
    using PCS = typename Flavor::PCS;
    using PCSCommitmentKey = typename Flavor::CommitmentKey;
    using ProvingKey = typename Flavor::ProvingKey;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Curve = typename Flavor::Curve;
    using Transcript = typename Flavor::Transcript;
    using TranslationEvaluations = bb::TranslationEvaluations;

  public:
    explicit ECCVMProver_(const std::shared_ptr<ProvingKey>& input_key,
                          const std::shared_ptr<PCSCommitmentKey>& commitment_key,
                          const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    BB_PROFILE void execute_preamble_round();
    BB_PROFILE void execute_wire_commitments_round();
    BB_PROFILE void execute_log_derivative_commitments_round();
    BB_PROFILE void execute_grand_product_computation_round();
    BB_PROFILE void execute_relation_check_rounds();
    BB_PROFILE void execute_univariatization_round();
    BB_PROFILE void execute_pcs_evaluation_round();
    BB_PROFILE void execute_shplonk_batched_quotient_round();
    BB_PROFILE void execute_shplonk_partial_evaluation_round();
    BB_PROFILE void execute_final_pcs_round();
    BB_PROFILE void execute_transcript_consistency_univariate_opening_round();

    HonkProof& export_proof();
    HonkProof& construct_proof();

    std::shared_ptr<Transcript> transcript;

    TranslationEvaluations translation_evaluations;

    std::vector<FF> public_inputs;

    bb::RelationParameters<FF> relation_parameters;

    std::shared_ptr<ProvingKey> key;

    // Container for spans of all polynomials required by the prover (i.e. all multivariates evaluated by Sumcheck).
    ProverPolynomials prover_polynomials;

    CommitmentLabels commitment_labels;

    // Container for d + 1 Fold polynomials produced by Gemini
    std::vector<Polynomial> gemini_polynomials;

    Polynomial batched_quotient_Q; // batched quotient poly computed by Shplonk
    FF nu_challenge;               // needed in both Shplonk rounds

    Polynomial quotient_W;

    FF evaluation_challenge_x;
    FF translation_batching_challenge_v; // to be rederived by the translator verifier

    SumcheckOutput<Flavor> sumcheck_output;
    GeminiProverOutput<Curve> gemini_output;
    ShplonkProverOutput<Curve> shplonk_output;
    std::shared_ptr<PCSCommitmentKey> commitment_key;

    using Gemini = GeminiProver_<Curve>;
    using Shplonk = ShplonkProver_<Curve>;

  private:
    HonkProof proof;
};

} // namespace bb
