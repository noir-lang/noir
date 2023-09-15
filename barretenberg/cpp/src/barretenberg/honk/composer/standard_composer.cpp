#include "standard_composer.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"

#include <cstddef>
#include <cstdint>
#include <string>

namespace proof_system::honk {

template <StandardFlavor Flavor>
std::shared_ptr<ProverInstance_<Flavor>> StandardComposer_<Flavor>::create_instance(CircuitBuilder& circuit)
{
    auto instance = std::make_shared<Instance>(circuit);
    instance->commitment_key = compute_commitment_key(instance->proving_key->circuit_size);
    return instance;
}

template <StandardFlavor Flavor>
StandardVerifier_<Flavor> StandardComposer_<Flavor>::create_verifier(std::shared_ptr<Instance> instance)
{
    auto verification_key = instance->compute_verification_key();
    StandardVerifier_<Flavor> output_state(verification_key);
    auto pcs_verification_key =
        std::make_unique<typename Flavor::VerifierCommitmentKey>(verification_key->circuit_size, crs_factory_);
    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

template <StandardFlavor Flavor>
StandardProver_<Flavor> StandardComposer_<Flavor>::create_prover(std::shared_ptr<Instance> instance)
{
    StandardProver_<Flavor> output_state(instance);

    return output_state;
}

template class StandardComposer_<honk::flavor::Standard>;
template class StandardComposer_<honk::flavor::StandardGrumpkin>;
} // namespace proof_system::honk