#include "ultra_prover.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/ultra_honk/decider_prover.hpp"
#include "barretenberg/ultra_honk/oink_prover.hpp"
namespace bb {

/**
 * Create UltraProver_ from an instance.
 *
 * @param instance Instance whose proof we want to generate.
 *
 * @tparam a type of UltraFlavor
 * */
template <IsUltraFlavor Flavor>
UltraProver_<Flavor>::UltraProver_(const std::shared_ptr<Instance>& inst, const std::shared_ptr<Transcript>& transcript)
    : instance(std::move(inst))
    , transcript(transcript)
    , commitment_key(instance->proving_key.commitment_key)
{}

/**
 * Create UltraProver_ from a circuit.
 *
 * @param instance Circuit with witnesses whose validity we'd like to prove.
 *
 * @tparam a type of UltraFlavor
 * */
template <IsUltraFlavor Flavor>
UltraProver_<Flavor>::UltraProver_(Builder& circuit)
    : instance(std::make_shared<ProverInstance>(circuit))
    , transcript(std::make_shared<Transcript>())
    , commitment_key(instance->proving_key.commitment_key)
{}

template <IsUltraFlavor Flavor> HonkProof UltraProver_<Flavor>::export_proof()
{
    proof = transcript->proof_data;
    return proof;
}
template <IsUltraFlavor Flavor> void UltraProver_<Flavor>::generate_gate_challenges()
{
    std::vector<FF> gate_challenges(numeric::get_msb(instance->proving_key.circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    instance->gate_challenges = gate_challenges;
}

template <IsUltraFlavor Flavor> HonkProof UltraProver_<Flavor>::construct_proof()
{
    OinkProver<Flavor> oink_prover(instance->proving_key, transcript);
    auto [proving_key, relation_params, alphas] = oink_prover.prove();
    instance->proving_key = std::move(proving_key);
    instance->relation_parameters = std::move(relation_params);
    instance->alphas = alphas;

    generate_gate_challenges();

    DeciderProver_<Flavor> decider_prover(instance, transcript);
    return decider_prover.construct_proof();
}

template class UltraProver_<UltraFlavor>;
template class UltraProver_<UltraKeccakFlavor>;
template class UltraProver_<MegaFlavor>;

} // namespace bb
