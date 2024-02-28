#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"

namespace bb {

template <IsUltraFlavor Flavor>
std::shared_ptr<ProverInstance_<Flavor>> UltraComposer_<Flavor>::create_instance(CircuitBuilder& circuit)
{
    return std::make_shared<Instance>(circuit);
}

template <IsUltraFlavor Flavor>
UltraProver_<Flavor> UltraComposer_<Flavor>::create_prover(const std::shared_ptr<Instance>& instance,
                                                           const std::shared_ptr<Transcript>& transcript)
{
    UltraProver_<Flavor> output_state(instance, transcript);

    return output_state;
}

template <IsUltraFlavor Flavor>
UltraVerifier_<Flavor> UltraComposer_<Flavor>::create_verifier(const std::shared_ptr<Instance>& instance,
                                                               const std::shared_ptr<Transcript>& transcript)
{
    auto& verification_key = instance->verification_key;
    UltraVerifier_<Flavor> output_state(transcript, verification_key);
    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size, crs_factory_);
    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

template <IsUltraFlavor Flavor>
DeciderProver_<Flavor> UltraComposer_<Flavor>::create_decider_prover(const std::shared_ptr<Instance>& accumulator,
                                                                     const std::shared_ptr<Transcript>& transcript)
{
    commitment_key = compute_commitment_key(accumulator->instance_size);
    DeciderProver_<Flavor> output_state(accumulator, commitment_key, transcript);

    return output_state;
}

template <IsUltraFlavor Flavor>
DeciderProver_<Flavor> UltraComposer_<Flavor>::create_decider_prover(
    const std::shared_ptr<Instance>& accumulator,
    const std::shared_ptr<CommitmentKey>& commitment_key,
    const std::shared_ptr<Transcript>& transcript)
{
    DeciderProver_<Flavor> output_state(accumulator, commitment_key, transcript);

    return output_state;
}

template <IsUltraFlavor Flavor>
DeciderVerifier_<Flavor> UltraComposer_<Flavor>::create_decider_verifier(const std::shared_ptr<Instance>& accumulator,
                                                                         const std::shared_ptr<Transcript>& transcript)
{
    auto& verification_key = accumulator->verification_key;
    DeciderVerifier_<Flavor> output_state(transcript, verification_key);
    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(accumulator->instance_size, crs_factory_);
    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

template class UltraComposer_<UltraFlavor>;
template class UltraComposer_<GoblinUltraFlavor>;
} // namespace bb
