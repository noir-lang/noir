

#include "./Fib_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/Fib_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/vm/generated/Fib_verifier.hpp"

namespace proof_system::honk {

using Flavor = honk::flavor::FibFlavor;
void FibComposer::compute_witness(CircuitConstructor& circuit)
{
    if (computed_witness) {
        return;
    }

    auto polynomials = circuit.compute_polynomials();

    proving_key->Fibonacci_LAST = polynomials.Fibonacci_LAST;
    proving_key->Fibonacci_FIRST = polynomials.Fibonacci_FIRST;
    proving_key->Fibonacci_x = polynomials.Fibonacci_x;
    proving_key->Fibonacci_y = polynomials.Fibonacci_y;

    computed_witness = true;
}

FibProver FibComposer::create_prover(CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);
    compute_commitment_key(circuit_constructor.get_circuit_subgroup_size());

    FibProver output_state(proving_key, commitment_key);

    return output_state;
}

FibVerifier FibComposer::create_verifier(CircuitConstructor& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    FibVerifier output_state(verification_key);

    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size, crs_factory_);

    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

std::shared_ptr<Flavor::ProvingKey> FibComposer::compute_proving_key(CircuitConstructor& circuit_constructor)
{
    if (proving_key) {
        return proving_key;
    }

    // Initialize proving_key
    {
        const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size();
        proving_key = std::make_shared<Flavor::ProvingKey>(subgroup_size, 0);
    }

    proving_key->contains_recursive_proof = false;

    return proving_key;
}

std::shared_ptr<Flavor::VerificationKey> FibComposer::compute_verification_key(CircuitConstructor& circuit_constructor)
{
    if (verification_key) {
        return verification_key;
    }

    if (!proving_key) {
        compute_proving_key(circuit_constructor);
    }

    verification_key =
        std::make_shared<Flavor::VerificationKey>(proving_key->circuit_size, proving_key->num_public_inputs);

    return verification_key;
}

} // namespace proof_system::honk
