#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
#include "barretenberg/honk/instance/instances.hpp"
#include "barretenberg/honk/proof_system/folding_result.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
namespace proof_system::honk {
template <class ProverInstances> class ProtoGalaxyProver_ {
  public:
    using Flavor = typename ProverInstances::Flavor;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    ProverInstances instances;

    ProverTranscript<FF> transcript;

    ProtoGalaxyProver_(ProverInstances insts)
        : instances(insts){};
    ~ProtoGalaxyProver_() = default;

    void prepare_for_folding();

    ProverFoldingResult<Flavor> fold_instances();
};

extern template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::Ultra, 2>>;
extern template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::UltraGrumpkin, 2>>;
extern template class ProtoGalaxyProver_<ProverInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk