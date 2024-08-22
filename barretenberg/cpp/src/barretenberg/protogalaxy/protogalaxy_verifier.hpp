#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {
template <class VerifierInstances> class ProtoGalaxyVerifier_ {
  public:
    using Flavor = typename VerifierInstances::Flavor;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using Instance = typename VerifierInstances::Instance;
    using VerificationKey = typename Flavor::VerificationKey;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using RelationSeparator = typename Flavor::RelationSeparator;

    static constexpr size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;

    VerifierInstances instances;

    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();

    CommitmentLabels commitment_labels;

    ProtoGalaxyVerifier_(const std::vector<std::shared_ptr<Instance>>& insts)
        : instances(VerifierInstances(insts)){};
    ~ProtoGalaxyVerifier_() = default;

    std::shared_ptr<Instance> get_accumulator() { return instances[0]; }

    /**
     * @brief Instatiate the instances and the transcript.
     *
     * @param fold_data The data transmitted via the transcript by the prover.
     */
    void prepare_for_folding(const std::vector<FF>&);

    /**
     * @brief Process the public data ϕ for the Instances to be folded.
     *
     */
    void receive_and_finalise_instance(const std::shared_ptr<Instance>&, const std::string&);

    /**
     * @brief Run the folding protocol on the verifier side to establish whether the public data ϕ of the new
     * accumulator, received from the prover is the same as that produced by the verifier.
     *
     */
    std::shared_ptr<Instance> verify_folding_proof(const std::vector<FF>&);
};

} // namespace bb
