// Note: this is split up from protogalaxy_prover_impl.hpp for compile performance reasons
#include "protogalaxy_prover_impl.hpp"

namespace bb {
template class ProtoGalaxyProver_<ProverInstances_<UltraFlavor, 2>>;
template class ProtoGalaxyProver_<ProverInstances_<UltraFlavor, 3>>;
template class ProtoGalaxyProver_<ProverInstances_<UltraFlavor, 4>>;
} // namespace bb