#pragma once
#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"

namespace proof_system::honk {

// We won't compile this class with honk::flavor::Standard, but we will like want to compile it (at least for testing)
// with a flavor that uses the curve Grumpkin, or a flavor that does/does not have zk, etc.
class GoblinTranslatorProver {

    using Flavor = honk::flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using ProvingKey = typename Flavor::ProvingKey;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Curve = typename Flavor::Curve;

    static size_t constexpr MINI_CIRCUIT_SIZE = Flavor::MINI_CIRCUIT_SIZE;
    static size_t constexpr FULL_CIRCUIT_SIZE = Flavor::FULL_CIRCUIT_SIZE;

  public:
    explicit GoblinTranslatorProver(std::shared_ptr<ProvingKey> input_key,
                                    std::shared_ptr<CommitmentKey> commitment_key);

    void execute_preamble_round();
    void execute_wire_and_sorted_constraints_commitments_round();
    void execute_grand_product_computation_round();
    void execute_relation_check_rounds();
    void execute_zeromorph_rounds();
    plonk::proof& export_proof();
    plonk::proof& construct_proof();

    BaseTranscript<FF> transcript;

    proof_system::RelationParameters<FF> relation_parameters;

    std::shared_ptr<ProvingKey> key;

    // Container for spans of all polynomials required by the prover (i.e. all multivariates evaluated by Sumcheck).
    ProverPolynomials prover_polynomials;

    CommitmentLabels commitment_labels;

    std::shared_ptr<CommitmentKey> commitment_key;

    sumcheck::SumcheckOutput<Flavor> sumcheck_output;

  private:
    plonk::proof proof;
};

} // namespace proof_system::honk
