// Note: this is split up from protogalaxy_prover_impl.hpp for compile performance reasons
#include "protogalaxy_prover_impl.hpp"

// TODO(https://github.com/AztecProtocol/barretenberg/issues/1076) Remove this instantiation.
namespace bb {
template class ProtoGalaxyProver_<ProverInstances_<UltraFlavor, 2>>;
} // namespace bb