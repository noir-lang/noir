#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/instance/instances.hpp"
#include "barretenberg/honk/proof_system/folding_result.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"

namespace proof_system::honk {
template <class VerifierInstances> class ProtoGalaxyVerifier_ {
  public:
    using Flavor = typename VerifierInstances::Flavor;
    using FF = typename Flavor::FF;
    using Instance = typename VerifierInstances::Instance;
    using VerificationKey = typename Flavor::VerificationKey;
    VerifierInstances verifier_instances;
    VerifierTranscript<FF> transcript;

    // should the PG verifier be given the VerifierInstances, nah this makes sense yo me
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

    VerifierFoldingResult<Flavor> fold_public_parameters(std::vector<uint8_t> fold_data);
};

extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::Ultra, 2>>;
extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk