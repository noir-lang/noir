#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"

namespace bb::honk {

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to the resulting verification key of the Instance.
 * */
template <UltraFlavor Flavor>
void UltraComposer_<Flavor>::compute_verification_key(const std::shared_ptr<ProverInstance_<Flavor>>& instance)
{
    if (instance->verification_key) {
        return;
    }

    auto& proving_key = instance->proving_key;

    auto verification_key =
        std::make_shared<typename Flavor::VerificationKey>(proving_key->circuit_size, proving_key->num_public_inputs);

    // Compute and store commitments to all precomputed polynomials
    verification_key->q_m = commitment_key->commit(proving_key->q_m);
    verification_key->q_l = commitment_key->commit(proving_key->q_l);
    verification_key->q_r = commitment_key->commit(proving_key->q_r);
    verification_key->q_o = commitment_key->commit(proving_key->q_o);
    verification_key->q_c = commitment_key->commit(proving_key->q_c);
    verification_key->sigma_1 = commitment_key->commit(proving_key->sigma_1);
    verification_key->sigma_2 = commitment_key->commit(proving_key->sigma_2);
    verification_key->sigma_3 = commitment_key->commit(proving_key->sigma_3);
    verification_key->id_1 = commitment_key->commit(proving_key->id_1);
    verification_key->id_2 = commitment_key->commit(proving_key->id_2);
    verification_key->id_3 = commitment_key->commit(proving_key->id_3);
    verification_key->lagrange_first = commitment_key->commit(proving_key->lagrange_first);
    verification_key->lagrange_last = commitment_key->commit(proving_key->lagrange_last);

    verification_key->q_4 = commitment_key->commit(proving_key->q_4);
    verification_key->q_arith = commitment_key->commit(proving_key->q_arith);
    verification_key->q_sort = commitment_key->commit(proving_key->q_sort);
    verification_key->q_elliptic = commitment_key->commit(proving_key->q_elliptic);
    verification_key->q_aux = commitment_key->commit(proving_key->q_aux);
    verification_key->q_lookup = commitment_key->commit(proving_key->q_lookup);
    verification_key->sigma_4 = commitment_key->commit(proving_key->sigma_4);
    verification_key->id_4 = commitment_key->commit(proving_key->id_4);
    verification_key->table_1 = commitment_key->commit(proving_key->table_1);
    verification_key->table_2 = commitment_key->commit(proving_key->table_2);
    verification_key->table_3 = commitment_key->commit(proving_key->table_3);
    verification_key->table_4 = commitment_key->commit(proving_key->table_4);

    // TODO(luke): Similar to the lagrange_first/last polynomials, we dont really need to commit to these polynomials
    // due to their simple structure.
    if constexpr (IsGoblinFlavor<Flavor>) {
        verification_key->lagrange_ecc_op = commitment_key->commit(proving_key->lagrange_ecc_op);
        verification_key->q_busread = commitment_key->commit(proving_key->q_busread);
        verification_key->databus_id = commitment_key->commit(proving_key->databus_id);
        verification_key->q_poseidon2_external = commitment_key->commit(proving_key->q_poseidon2_external);
        verification_key->q_poseidon2_internal = commitment_key->commit(proving_key->q_poseidon2_internal);
    }

    instance->verification_key = std::move(verification_key);
}

template <UltraFlavor Flavor>
std::shared_ptr<ProverInstance_<Flavor>> UltraComposer_<Flavor>::create_instance(CircuitBuilder& circuit)
{
    circuit.add_gates_to_ensure_all_polys_are_non_zero();
    circuit.finalize_circuit();
    auto instance = std::make_shared<Instance>(circuit);
    commitment_key = compute_commitment_key(instance->proving_key->circuit_size);

    compute_verification_key(instance);
    return instance;
}

template <UltraFlavor Flavor>
UltraProver_<Flavor> UltraComposer_<Flavor>::create_prover(const std::shared_ptr<Instance>& instance,
                                                           const std::shared_ptr<Transcript>& transcript)
{
    UltraProver_<Flavor> output_state(instance, commitment_key, transcript);

    return output_state;
}

template <UltraFlavor Flavor>
UltraVerifier_<Flavor> UltraComposer_<Flavor>::create_verifier(const std::shared_ptr<Instance>& instance,
                                                               const std::shared_ptr<Transcript>& transcript)
{
    auto& verification_key = instance->verification_key;
    UltraVerifier_<Flavor> output_state(transcript, verification_key);
    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size, crs_factory_);
    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

template <UltraFlavor Flavor>
DeciderProver_<Flavor> UltraComposer_<Flavor>::create_decider_prover(const std::shared_ptr<Instance>& accumulator,
                                                                     const std::shared_ptr<Transcript>& transcript)
{
    commitment_key = compute_commitment_key(accumulator->instance_size);
    DeciderProver_<Flavor> output_state(accumulator, commitment_key, transcript);

    return output_state;
}

template <UltraFlavor Flavor>
DeciderProver_<Flavor> UltraComposer_<Flavor>::create_decider_prover(
    const std::shared_ptr<Instance>& accumulator,
    const std::shared_ptr<CommitmentKey>& commitment_key,
    const std::shared_ptr<Transcript>& transcript)
{
    DeciderProver_<Flavor> output_state(accumulator, commitment_key, transcript);

    return output_state;
}

template <UltraFlavor Flavor>
DeciderVerifier_<Flavor> UltraComposer_<Flavor>::create_decider_verifier(const std::shared_ptr<Instance>& accumulator,
                                                                         const std::shared_ptr<Transcript>& transcript)
{
    auto& verification_key = accumulator->verification_key;
    DeciderVerifier_<Flavor> output_state(transcript, verification_key);
    auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(accumulator->instance_size, crs_factory_);
    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

template class UltraComposer_<honk::flavor::Ultra>;
template class UltraComposer_<honk::flavor::GoblinUltra>;
} // namespace bb::honk
