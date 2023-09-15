#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
#include "barretenberg/honk/instance/prover_instance.hpp"
#include "barretenberg/honk/proof_system/folding_result.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
namespace proof_system::honk {
template <class Flavor> class ProtoGalaxyProver_ {
  public:
    using FF = typename Flavor::FF;
    using Instance = ProverInstance_<Flavor>;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    std::vector<std::shared_ptr<Instance>> instances;

    ProverTranscript<FF> transcript;

    explicit ProtoGalaxyProver_(std::vector<std::shared_ptr<Instance>>);
    ~ProtoGalaxyProver_() = default;

    void prepare_for_folding();

    ProverFoldingResult<Flavor> fold_instances();
};

extern template class ProtoGalaxyProver_<honk::flavor::Ultra>;
extern template class ProtoGalaxyProver_<honk::flavor::UltraGrumpkin>;
extern template class ProtoGalaxyProver_<honk::flavor::GoblinUltra>;
} // namespace proof_system::honk