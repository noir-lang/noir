#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace proof_system::honk {
template <class VerifierInstances> class ProtoGalaxyVerifier_ {
  public:
    using Flavor = typename VerifierInstances::Flavor;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;
    using Instance = typename VerifierInstances::Instance;
    using VerificationKey = typename Flavor::VerificationKey;

    VerifierInstances verifier_instances;
    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();

    ProtoGalaxyVerifier_(VerifierInstances insts)
        : verifier_instances(insts){};
    ~ProtoGalaxyVerifier_() = default;
    /**
     * @brief For a new round challenge δ at each iteration of the ProtoGalaxy protocol, compute the vector
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

    std::shared_ptr<Instance> get_accumulator() { return verifier_instances[0]; }

    /**
     * @brief Instatiate the VerifierInstances and the VerifierTranscript.
     *
     * @param fold_data The data transmitted via the transcript by the prover.
     */
    void prepare_for_folding(std::vector<uint8_t> fold_data);

    /**
     * @brief Run the folding protocol on the verifier side.
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/690): finalise the implementation of this function
     */
    VerifierFoldingResult<Flavor> fold_public_parameters(std::vector<uint8_t> fold_data);
};

extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::Ultra, 2>>;
extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk