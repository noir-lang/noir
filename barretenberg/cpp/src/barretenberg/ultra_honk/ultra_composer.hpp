#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/protogalaxy/decider_prover.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

namespace bb {
template <IsUltraFlavor Flavor_> class UltraComposer_ {
  public:
    using Flavor = Flavor_;
    using CircuitBuilder = typename Flavor::CircuitBuilder;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCS = typename Flavor::PCS;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using ProverInstance = ProverInstance_<Flavor>;
    using VerifierInstance = VerifierInstance_<Flavor>;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;
    using CRSFactory = srs::factories::CrsFactory<typename Flavor::Curve>;

    static constexpr size_t NUM_FOLDING = 2;
    using ProverInstances = ProverInstances_<Flavor, NUM_FOLDING>;
    using VerifierInstances = VerifierInstances_<Flavor, NUM_FOLDING>;

    // offset due to placing zero wires at the start of execution trace
    static constexpr size_t num_zero_rows = Flavor::has_zero_row ? 1 : 0;
    static constexpr std::string_view NAME_STRING = "UltraHonk";
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<CRSFactory> crs_factory_;
    // The commitment key is passed to the prover but also used herein to compute the verfication key commitments
    std::shared_ptr<CommitmentKey> commitment_key;

    UltraComposer_() { crs_factory_ = bb::srs::get_bn254_crs_factory(); }

    explicit UltraComposer_(std::shared_ptr<CRSFactory> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    UltraComposer_(UltraComposer_&& other) noexcept = default;
    UltraComposer_(UltraComposer_ const& other) noexcept = default;
    UltraComposer_& operator=(UltraComposer_&& other) noexcept = default;
    UltraComposer_& operator=(UltraComposer_ const& other) noexcept = default;
    ~UltraComposer_() = default;

    std::shared_ptr<CommitmentKey> compute_commitment_key(size_t circuit_size)
    {
        commitment_key = std::make_shared<CommitmentKey>(circuit_size + 1);
        return commitment_key;
    };

    std::shared_ptr<ProverInstance> create_prover_instance(CircuitBuilder&);

    /**
     * @brief Create a verifier instance object.
     *
     * @details Currently use prover instance
     */
    std::shared_ptr<VerifierInstance> create_verifier_instance(std::shared_ptr<ProverInstance>&);

    UltraProver_<Flavor> create_prover(const std::shared_ptr<ProverInstance>&,
                                       const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    UltraVerifier_<Flavor> create_verifier(
        const std::shared_ptr<VerificationKey>&,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    DeciderProver_<Flavor> create_decider_prover(
        const std::shared_ptr<ProverInstance>&,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    DeciderProver_<Flavor> create_decider_prover(
        const std::shared_ptr<ProverInstance>&,
        const std::shared_ptr<CommitmentKey>&,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    DeciderVerifier_<Flavor> create_decider_verifier(
        const std::shared_ptr<VerifierInstance>&,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    UltraVerifier_<Flavor> create_verifier(CircuitBuilder& circuit);

    UltraVerifier_<Flavor> create_ultra_with_keccak_verifier(CircuitBuilder& circuit);

    ProtoGalaxyProver_<ProverInstances> create_folding_prover(
        const std::vector<std::shared_ptr<ProverInstance>>& instances)
    {
        ProtoGalaxyProver_<ProverInstances> output_state(instances);

        return output_state;
    };

    ProtoGalaxyVerifier_<VerifierInstances> create_folding_verifier(
        const std::vector<std::shared_ptr<VerifierInstance>>& instances)
    {
        ProtoGalaxyVerifier_<VerifierInstances> output_state(instances);

        return output_state;
    };
};

// TODO(#532): this pattern is weird; is this not instantiating the templates?
using UltraComposer = UltraComposer_<UltraFlavor>;
using GoblinUltraComposer = UltraComposer_<GoblinUltraFlavor>;
} // namespace bb
