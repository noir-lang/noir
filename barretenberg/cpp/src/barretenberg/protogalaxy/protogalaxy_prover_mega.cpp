// Note: this is split up from protogalaxy_prover_impl.hpp for compile performance reasons
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/ultra_honk/oink_prover.hpp"
#include "protogalaxy_prover_impl.hpp"
namespace bb {

template class ProtogalaxyProver_<ProverInstances_<MegaFlavor, 2>>;
} // namespace bb