#pragma once
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

template <IsUltraFlavor Flavor_> class UltraProver_ {
  public:
    using Flavor = Flavor_;
    using FF = typename Flavor::FF;
    using Builder = typename Flavor::CircuitBuilder;
    using Commitment = typename Flavor::Commitment;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using PCS = typename Flavor::PCS;
    using ProverInstance = ProverInstance_<Flavor>;
    using Instance = ProverInstance;
    using Transcript = typename Flavor::Transcript;
    using ZeroMorph = ZeroMorphProver_<PCS>;

    std::shared_ptr<Instance> instance;

    std::shared_ptr<Transcript> transcript;

    bb::RelationParameters<FF> relation_parameters;

    Polynomial quotient_W;

    SumcheckOutput<Flavor> sumcheck_output;

    std::shared_ptr<CommitmentKey> commitment_key;

    explicit UltraProver_(const std::shared_ptr<Instance>&,
                          const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    explicit UltraProver_(Builder&);

    BB_PROFILE void generate_gate_challenges();

    HonkProof export_proof();
    HonkProof construct_proof();

  private:
    HonkProof proof;
};

using UltraProver = UltraProver_<UltraFlavor>;
using UltraKeccakProver = UltraProver_<UltraKeccakFlavor>;
using MegaProver = UltraProver_<MegaFlavor>;

} // namespace bb
