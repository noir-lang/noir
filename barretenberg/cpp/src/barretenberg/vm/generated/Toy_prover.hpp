

#pragma once
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/flavor/generated/Toy_flavor.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::honk {

class ToyProver {

    using Flavor = honk::flavor::ToyFlavor;
    using FF = Flavor::FF;
    using PCS = Flavor::PCS;
    using PCSCommitmentKey = Flavor::CommitmentKey;
    using ProvingKey = Flavor::ProvingKey;
    using Polynomial = Flavor::Polynomial;
    using ProverPolynomials = Flavor::ProverPolynomials;
    using CommitmentLabels = Flavor::CommitmentLabels;
    using Curve = Flavor::Curve;
    using Transcript = Flavor::Transcript;

  public:
    explicit ToyProver(std::shared_ptr<ProvingKey> input_key, std::shared_ptr<PCSCommitmentKey> commitment_key);

    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_relation_check_rounds();
    void execute_zeromorph_rounds();

    plonk::proof& export_proof();
    plonk::proof& construct_proof();

    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();

    std::vector<FF> public_inputs;

    bb::RelationParameters<FF> relation_parameters;

    std::shared_ptr<ProvingKey> key;

    // Container for spans of all polynomials required by the prover (i.e. all multivariates evaluated by Sumcheck).
    ProverPolynomials prover_polynomials;

    CommitmentLabels commitment_labels;

    Polynomial quotient_W;

    sumcheck::SumcheckOutput<Flavor> sumcheck_output;

    std::shared_ptr<PCSCommitmentKey> commitment_key;

    using ZeroMorph = pcs::zeromorph::ZeroMorphProver_<Curve>;

  private:
    plonk::proof proof;
};

} // namespace bb::honk
