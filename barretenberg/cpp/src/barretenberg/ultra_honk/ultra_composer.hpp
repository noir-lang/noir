#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/ultra_honk/merge_prover.hpp"
#include "barretenberg/ultra_honk/merge_verifier.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

namespace proof_system::honk {
template <UltraFlavor Flavor> class UltraComposer_ {
  public:
    using CircuitBuilder = typename Flavor::CircuitBuilder;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCS = typename Flavor::PCS;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Instance = ProverInstance_<Flavor>;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;

    static constexpr size_t NUM_FOLDING = 2;
    using ProverInstances = ProverInstances_<Flavor, NUM_FOLDING>;
    using VerifierInstances = VerifierInstances_<Flavor, NUM_FOLDING>;

    // offset due to placing zero wires at the start of execution trace
    static constexpr size_t num_zero_rows = Flavor::has_zero_row ? 1 : 0;
    static constexpr std::string_view NAME_STRING = "UltraHonk";
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<srs::factories::CrsFactory<typename Flavor::Curve>> crs_factory_;
    // The commitment key is passed to the prover but also used herein to compute the verfication key commitments
    std::shared_ptr<CommitmentKey> commitment_key;

    UltraComposer_() { crs_factory_ = barretenberg::srs::get_crs_factory(); }

    explicit UltraComposer_(std::shared_ptr<srs::factories::CrsFactory<typename Flavor::Curve>> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    UltraComposer_(UltraComposer_&& other) noexcept = default;
    UltraComposer_(UltraComposer_ const& other) noexcept = default;
    UltraComposer_& operator=(UltraComposer_&& other) noexcept = default;
    UltraComposer_& operator=(UltraComposer_ const& other) noexcept = default;
    ~UltraComposer_() = default;

    std::shared_ptr<CommitmentKey> compute_commitment_key(size_t circuit_size)
    {
        commitment_key = std::make_shared<CommitmentKey>(circuit_size, crs_factory_);
        return commitment_key;
    };

    std::shared_ptr<Instance> create_instance(CircuitBuilder& circuit);

    UltraProver_<Flavor> create_prover(const std::shared_ptr<Instance>&,
                                       const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    UltraVerifier_<Flavor> create_verifier(
        const std::shared_ptr<Instance>&,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    /**
     * @brief Create Prover for Goblin ECC op queue merge protocol
     *
     * @param op_queue
     * @return MergeProver_<Flavor>
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/804): Goblin should be responsible for constructing
     * merge prover/verifier.
     */
    MergeProver_<Flavor> create_merge_prover(
        const std::shared_ptr<ECCOpQueue>& op_queue,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>())
    {
        // Store the previous aggregate op queue size and update the current one
        op_queue->set_size_data();
        // Merge requires a commitment key with size equal to that of the current op queue transcript T_i since the
        // shift of the current contribution t_i will be of degree equal to deg(T_i)
        auto commitment_key = compute_commitment_key(op_queue->get_current_size());
        return MergeProver_<Flavor>(commitment_key, op_queue, transcript);
    }

    /**
     * @brief Create Verifier for Goblin ECC op queue merge protocol
     *
     * @return MergeVerifier_<Flavor>
     */
    MergeVerifier_<Flavor> create_merge_verifier() { return MergeVerifier_<Flavor>(); }

    ProtoGalaxyProver_<ProverInstances> create_folding_prover(const std::vector<std::shared_ptr<Instance>>& instances)
    {
        ProverInstances insts(instances);
        ProtoGalaxyProver_<ProverInstances> output_state(insts);

        return output_state;
    };
    ProtoGalaxyVerifier_<VerifierInstances> create_folding_verifier(
        const std::vector<std::shared_ptr<Instance>>& instances)
    {
        std::vector<std::shared_ptr<VerificationKey>> vks;
        for (const auto& inst : instances) {
            vks.emplace_back(inst->verification_key);
        }
        VerifierInstances insts(vks);
        ProtoGalaxyVerifier_<VerifierInstances> output_state(insts);

        return output_state;
    };

  private:
    /**
     * @brief Compute the verification key of an Instance, produced from a finalised circuit.
     *
     * @param inst
     */
    void compute_verification_key(const std::shared_ptr<Instance>&);
};
extern template class UltraComposer_<honk::flavor::Ultra>;
extern template class UltraComposer_<honk::flavor::GoblinUltra>;
// TODO(#532): this pattern is weird; is this not instantiating the templates?
using UltraComposer = UltraComposer_<honk::flavor::Ultra>;
using GoblinUltraComposer = UltraComposer_<honk::flavor::GoblinUltra>;
} // namespace proof_system::honk
