#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
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
    VerifierFoldingResult<Flavor> fold_public_parameters(std::vector<uint8_t> fold_data);
};

extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::Ultra, 2>>;
extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::UltraGrumpkin, 2>>;
extern template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk