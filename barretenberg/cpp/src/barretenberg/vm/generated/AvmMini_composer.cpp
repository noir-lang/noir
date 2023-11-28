

#include "./AvmMini_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/AvmMini_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/vm/generated/AvmMini_verifier.hpp"

namespace proof_system::honk {

using Flavor = honk::flavor::AvmMiniFlavor;
void AvmMiniComposer::compute_witness(CircuitConstructor& circuit)
{
    if (computed_witness) {
        return;
    }

    auto polynomials = circuit.compute_polynomials();

    proving_key->avmMini_clk = polynomials.avmMini_clk;
    proving_key->avmMini_positive = polynomials.avmMini_positive;
    proving_key->avmMini_first = polynomials.avmMini_first;
    proving_key->avmMini_subop = polynomials.avmMini_subop;
    proving_key->avmMini_ia = polynomials.avmMini_ia;
    proving_key->avmMini_ib = polynomials.avmMini_ib;
    proving_key->avmMini_ic = polynomials.avmMini_ic;
    proving_key->avmMini_mem_op_a = polynomials.avmMini_mem_op_a;
    proving_key->avmMini_mem_op_b = polynomials.avmMini_mem_op_b;
    proving_key->avmMini_mem_op_c = polynomials.avmMini_mem_op_c;
    proving_key->avmMini_rwa = polynomials.avmMini_rwa;
    proving_key->avmMini_rwb = polynomials.avmMini_rwb;
    proving_key->avmMini_rwc = polynomials.avmMini_rwc;
    proving_key->avmMini_mem_idx_a = polynomials.avmMini_mem_idx_a;
    proving_key->avmMini_mem_idx_b = polynomials.avmMini_mem_idx_b;
    proving_key->avmMini_mem_idx_c = polynomials.avmMini_mem_idx_c;
    proving_key->avmMini_last = polynomials.avmMini_last;
    proving_key->avmMini_m_clk = polynomials.avmMini_m_clk;
    proving_key->avmMini_m_sub_clk = polynomials.avmMini_m_sub_clk;
    proving_key->avmMini_m_addr = polynomials.avmMini_m_addr;
    proving_key->avmMini_m_val = polynomials.avmMini_m_val;
    proving_key->avmMini_m_lastAccess = polynomials.avmMini_m_lastAccess;
    proving_key->avmMini_m_rw = polynomials.avmMini_m_rw;

    computed_witness = true;
}

AvmMiniProver AvmMiniComposer::create_prover(CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);
    compute_commitment_key(circuit_constructor.get_circuit_subgroup_size());

    AvmMiniProver output_state(proving_key, commitment_key);

    return output_state;
}

AvmMiniVerifier AvmMiniComposer::create_verifier(CircuitConstructor& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    AvmMiniVerifier output_state(verification_key);

    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size, crs_factory_);

    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

std::shared_ptr<Flavor::ProvingKey> AvmMiniComposer::compute_proving_key(CircuitConstructor& circuit_constructor)
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

std::shared_ptr<Flavor::VerificationKey> AvmMiniComposer::compute_verification_key(
    CircuitConstructor& circuit_constructor)
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
