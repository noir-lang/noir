#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"

namespace bb {

template <IsUltraFlavor Flavor>
std::shared_ptr<ProverInstance_<Flavor>> UltraComposer_<Flavor>::create_prover_instance(CircuitBuilder& circuit)
{
    return std::make_shared<ProverInstance>(circuit);
}

template <IsUltraFlavor Flavor>
std::shared_ptr<VerifierInstance_<Flavor>> UltraComposer_<Flavor>::create_verifier_instance(
    std::shared_ptr<ProverInstance_<Flavor>>& prover_instance)
{
    auto verification_key = std::make_shared<VerificationKey>(prover_instance->proving_key);
    auto instance = std::make_shared<VerifierInstance>(verification_key);
    return instance;
}

template <IsUltraFlavor Flavor>
UltraProver_<Flavor> UltraComposer_<Flavor>::create_prover(const std::shared_ptr<ProverInstance>& instance,
                                                           const std::shared_ptr<Transcript>& transcript)
{
    UltraProver_<Flavor> output_state(instance, transcript);

    return output_state;
}

template <IsUltraFlavor Flavor>
UltraVerifier_<Flavor> UltraComposer_<Flavor>::create_verifier(const std::shared_ptr<VerificationKey>& verification_key,
                                                               const std::shared_ptr<Transcript>& transcript)
{
    return UltraVerifier_<Flavor>(transcript, verification_key);
}

template <IsUltraFlavor Flavor>
DeciderProver_<Flavor> UltraComposer_<Flavor>::create_decider_prover(const std::shared_ptr<ProverInstance>& accumulator,
                                                                     const std::shared_ptr<Transcript>& transcript)
{
    return DeciderProver_<Flavor>(accumulator, transcript);
}

template <IsUltraFlavor Flavor>
DeciderVerifier_<Flavor> UltraComposer_<Flavor>::create_decider_verifier(
    const std::shared_ptr<VerifierInstance>& accumulator, const std::shared_ptr<Transcript>& transcript)
{
    return DeciderVerifier_<Flavor>(transcript, accumulator);
}

template class UltraComposer_<UltraFlavor>;
template class UltraComposer_<GoblinUltraFlavor>;
} // namespace bb
