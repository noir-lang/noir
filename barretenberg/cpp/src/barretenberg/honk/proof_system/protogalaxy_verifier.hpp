#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
#include "barretenberg/honk/instance/verifier_instance.hpp"
#include "barretenberg/honk/proof_system/folding_result.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"

namespace proof_system::honk {
template <class Flavor> class ProtoGalaxyVerifier_ {
  public:
    using FF = typename Flavor::FF;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierInstance = VerifierInstance_<Flavor>;
    std::vector<VerifierInstance> verifier_instances;
    VerifierTranscript<FF> transcript;

    explicit ProtoGalaxyVerifier_(std::vector<std::shared_ptr<VerificationKey>> vks);
    ~ProtoGalaxyVerifier_() = default;
    VerifierFoldingResult<Flavor> fold_public_parameters(std::vector<uint8_t> fold_data);
};

extern template class ProtoGalaxyVerifier_<honk::flavor::Ultra>;
extern template class ProtoGalaxyVerifier_<honk::flavor::UltraGrumpkin>;
extern template class ProtoGalaxyVerifier_<honk::flavor::GoblinUltra>;
} // namespace proof_system::honk