// Note: this is split up from protogalaxy_prover_impl.hpp for compile performance reasons
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "protogalaxy_prover_impl.hpp"

// TODO(https://github.com/AztecProtocol/barretenberg/issues/1076) Remove this instantiation.
namespace bb {
template class ProtogalaxyProver_<ProverInstances_<UltraFlavor, 2>>;
} // namespace bb