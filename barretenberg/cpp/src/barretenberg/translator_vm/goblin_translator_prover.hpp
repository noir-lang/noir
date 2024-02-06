#pragma once
#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"

namespace bb {

// We won't compile this class with Standard, but we will like want to compile it (at least for testing)
// with a flavor that uses the curve Grumpkin, or a flavor that does/does not have zk, etc.
class GoblinTranslatorProver {

    using Flavor = GoblinTranslatorFlavor;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using Commitment = typename Flavor::Commitment;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using ProvingKey = typename Flavor::ProvingKey;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Curve = typename Flavor::Curve;
    using Transcript = typename Flavor::Transcript;

  public:
    explicit GoblinTranslatorProver(const std::shared_ptr<ProvingKey>& input_key,
                                    const std::shared_ptr<CommitmentKey>& commitment_key,
                                    const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    BB_PROFILE void execute_preamble_round();
    BB_PROFILE void execute_wire_and_sorted_constraints_commitments_round();
    BB_PROFILE void execute_grand_product_computation_round();
    BB_PROFILE void execute_relation_check_rounds();
    BB_PROFILE void execute_zeromorph_rounds();
    HonkProof& export_proof();
    HonkProof& construct_proof();

    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();

    bb::RelationParameters<FF> relation_parameters;

    std::shared_ptr<ProvingKey> key;

    // Container for spans of all polynomials required by the prover (i.e. all multivariates evaluated by Sumcheck).
    ProverPolynomials prover_polynomials;

    CommitmentLabels commitment_labels;

    std::shared_ptr<CommitmentKey> commitment_key;

    SumcheckOutput<Flavor> sumcheck_output;

  private:
    HonkProof proof;
};

} // namespace bb
