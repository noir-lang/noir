#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::honk {
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

    ProtoGalaxyVerifier_(VerifierInstances insts)
        : instances(insts){};
    ~ProtoGalaxyVerifier_() = default;
    /**
     * @brief Given a new round challenge δ for each iteration of the full ProtoGalaxy protocol, compute the vector
     * [δ, δ^2,..., δ^t] where t = logn and n is the size of the instance.
     */
    static std::vector<FF> compute_round_challenge_pows(size_t log_instance_size, FF round_challenge)
    {
        std::vector<FF> pows(log_instance_size);
        pows[0] = round_challenge;
        for (size_t i = 1; i < log_instance_size; i++) {
            pows[i] = pows[i - 1].sqr();
        }
        return pows;
    }

    static std::vector<FF> update_gate_challenges(const FF perturbator_challenge,
                                                  const std::vector<FF>& gate_challenges,
                                                  const std::vector<FF>& round_challenges)
    {
        auto log_instance_size = gate_challenges.size();
        std::vector<FF> next_gate_challenges(log_instance_size);

        for (size_t idx = 0; idx < log_instance_size; idx++) {
            next_gate_challenges[idx] = gate_challenges[idx] + perturbator_challenge * round_challenges[idx];
        }
        return next_gate_challenges;
    }

    std::shared_ptr<Instance> get_accumulator() { return instances[0]; }

    /**
     * @brief Instatiate the instances and the transcript.
     *
     * @param fold_data The data transmitted via the transcript by the prover.
     */
    void prepare_for_folding(const std::vector<uint8_t>&);

    /**
     * @brief Instantiatied the accumulator (i.e. the relaxed instance) from the transcript.
     *
     */
    void receive_accumulator(const std::shared_ptr<Instance>&, const std::string&);

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
    bool verify_folding_proof(std::vector<uint8_t>);
};

} // namespace bb::honk
