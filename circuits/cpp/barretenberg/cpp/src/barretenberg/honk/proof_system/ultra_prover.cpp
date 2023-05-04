#include "ultra_prover.hpp"

namespace proof_system::honk {

/**
 * Create UltraHonkProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <UltraFlavor Flavor>
UltraProver_<Flavor>::UltraProver_(std::shared_ptr<typename Flavor::ProvingKey> input_key)
    : key(input_key)
    , queue(input_key->circuit_size, transcript)
{}

template <UltraFlavor Flavor> plonk::proof& UltraProver_<Flavor>::export_proof()
{
    proof.proof_data = transcript.proof_data;
    return proof;
}

template <UltraFlavor Flavor> plonk::proof& UltraProver_<Flavor>::construct_proof()
{
    return export_proof();
}

template class UltraProver_<honk::flavor::Ultra>;

} // namespace proof_system::honk
