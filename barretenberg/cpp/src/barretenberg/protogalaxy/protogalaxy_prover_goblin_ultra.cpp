// Note: this is split up from protogalaxy_prover_impl.hpp for compile performance reasons
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/ultra_honk/oink_prover.hpp"
#include "protogalaxy_prover_impl.hpp"
namespace bb {

template class ProtoGalaxyProver_<ProverInstances_<GoblinUltraFlavor, 2>>;
template class ProtoGalaxyProver_<ProverInstances_<GoblinUltraFlavor, 3>>;
template class ProtoGalaxyProver_<ProverInstances_<GoblinUltraFlavor, 4>>;
} // namespace bb