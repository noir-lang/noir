#include "eccvm_composer.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"

namespace bb::honk {

/**
 * @brief Compute witness polynomials
 *
 */
template <ECCVMFlavor Flavor> void ECCVMComposer_<Flavor>::compute_witness(CircuitConstructor& circuit_constructor)
{
    if (computed_witness) {
        return;
    }

    auto polynomials = circuit_constructor.compute_polynomials();

    auto key_wires = proving_key->get_wires();
    auto poly_wires = polynomials.get_wires();

    for (size_t i = 0; i < key_wires.size(); ++i) {
        std::copy(poly_wires[i].begin(), poly_wires[i].end(), key_wires[i].begin());
    }

    computed_witness = true;
}

template <ECCVMFlavor Flavor>
ECCVMProver_<Flavor> ECCVMComposer_<Flavor>::create_prover(CircuitConstructor& circuit_constructor,
                                                           const std::shared_ptr<Transcript>& transcript)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);
    compute_commitment_key(proving_key->circuit_size);

    ECCVMProver_<Flavor> output_state(proving_key, commitment_key, transcript);

    return output_state;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */
template <ECCVMFlavor Flavor>
ECCVMVerifier_<Flavor> ECCVMComposer_<Flavor>::create_verifier(CircuitConstructor& circuit_constructor,
                                                               const std::shared_ptr<Transcript>& transcript)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    ECCVMVerifier_<Flavor> output_state(verification_key);

    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size, crs_factory_);

    output_state.pcs_verification_key = std::move(pcs_verification_key);
    output_state.transcript = transcript;

    return output_state;
}

template <ECCVMFlavor Flavor>
std::shared_ptr<typename Flavor::ProvingKey> ECCVMComposer_<Flavor>::compute_proving_key(
    CircuitConstructor& circuit_constructor)
{
    if (proving_key) {
        return proving_key;
    }

    // Initialize proving_key
    {
        const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size(circuit_constructor.get_num_gates());
        proving_key = std::make_shared<typename Flavor::ProvingKey>(subgroup_size, 0);
    }

    // TODO(@zac-williamson): We don't enforce nonzero selectors atm. Will create problems in recursive setting.
    // Fix once we have a stable base to work off of
    // enforce_nonzero_polynomial_selectors(circuit_constructor, proving_key.get());

    compute_first_and_last_lagrange_polynomials<Flavor>(proving_key.get());
    {
        const size_t n = proving_key->circuit_size;
        typename Flavor::Polynomial lagrange_polynomial_second(n);
        lagrange_polynomial_second[1] = 1;
        proving_key->lagrange_second = lagrange_polynomial_second.share();
    }

    proving_key->contains_recursive_proof = false;

    return proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
template <ECCVMFlavor Flavor>
std::shared_ptr<typename Flavor::VerificationKey> ECCVMComposer_<Flavor>::compute_verification_key(
    CircuitConstructor& circuit_constructor)
{
    if (verification_key) {
        return verification_key;
    }

    if (!proving_key) {
        compute_proving_key(circuit_constructor);
    }

    verification_key =
        std::make_shared<typename Flavor::VerificationKey>(proving_key->circuit_size, proving_key->num_public_inputs);

    verification_key->lagrange_first = commitment_key->commit(proving_key->lagrange_first);
    verification_key->lagrange_second = commitment_key->commit(proving_key->lagrange_second);
    verification_key->lagrange_last = commitment_key->commit(proving_key->lagrange_last);
    return verification_key;
}
template class ECCVMComposer_<honk::flavor::ECCVM>;

} // namespace bb::honk
