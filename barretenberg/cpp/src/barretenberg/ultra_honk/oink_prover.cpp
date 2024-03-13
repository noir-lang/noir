#include "barretenberg/ultra_honk/oink_prover.hpp"

namespace bb {

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(instance->proving_key->circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(instance->proving_key->num_public_inputs);
    transcript->send_to_verifier(domain_separator + "circuit_size", circuit_size);
    transcript->send_to_verifier(domain_separator + "public_input_size", num_public_inputs);
    transcript->send_to_verifier(domain_separator + "pub_inputs_offset",
                                 static_cast<uint32_t>(instance->proving_key->pub_inputs_offset));

    ASSERT(instance->proving_key->num_public_inputs == instance->proving_key->public_inputs.size());

    for (size_t i = 0; i < instance->proving_key->num_public_inputs; ++i) {
        auto public_input_i = instance->proving_key->public_inputs[i];
        transcript->send_to_verifier(domain_separator + "public_input_" + std::to_string(i), public_input_i);
    }
}

/**
 * @brief Commit to the wire polynomials (part of the witness), with the exception of the fourth wire, which is
 * only commited to after adding memory records. In the Goblin Flavor, we also commit to the ECC OP wires and the
 * DataBus columns.
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_wire_commitments_round()
{
    // Commit to the first three wire polynomials of the instance
    // We only commit to the fourth wire polynomial after adding memory recordss
    witness_commitments.w_l = commitment_key->commit(instance->proving_key->w_l);
    witness_commitments.w_r = commitment_key->commit(instance->proving_key->w_r);
    witness_commitments.w_o = commitment_key->commit(instance->proving_key->w_o);

    auto wire_comms = witness_commitments.get_wires();
    auto wire_labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        transcript->send_to_verifier(domain_separator + wire_labels[idx], wire_comms[idx]);
    }

    if constexpr (IsGoblinFlavor<Flavor>) {
        // Commit to Goblin ECC op wires
        witness_commitments.ecc_op_wire_1 = commitment_key->commit(instance->proving_key->ecc_op_wire_1);
        witness_commitments.ecc_op_wire_2 = commitment_key->commit(instance->proving_key->ecc_op_wire_2);
        witness_commitments.ecc_op_wire_3 = commitment_key->commit(instance->proving_key->ecc_op_wire_3);
        witness_commitments.ecc_op_wire_4 = commitment_key->commit(instance->proving_key->ecc_op_wire_4);

        auto op_wire_comms = witness_commitments.get_ecc_op_wires();
        auto labels = commitment_labels.get_ecc_op_wires();
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            transcript->send_to_verifier(domain_separator + labels[idx], op_wire_comms[idx]);
        }
        // Commit to DataBus columns
        witness_commitments.calldata = commitment_key->commit(instance->proving_key->calldata);
        witness_commitments.calldata_read_counts = commitment_key->commit(instance->proving_key->calldata_read_counts);
        transcript->send_to_verifier(domain_separator + commitment_labels.calldata, witness_commitments.calldata);
        transcript->send_to_verifier(domain_separator + commitment_labels.calldata_read_counts,
                                     witness_commitments.calldata_read_counts);
    }
}

/**
 * @brief Compute sorted witness-table accumulator and commit to the resulting polynomials.
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_sorted_list_accumulator_round()
{

    auto eta = transcript->template get_challenge<FF>(domain_separator + "eta");
    instance->compute_sorted_accumulator_polynomials(eta);

    // Commit to the sorted witness-table accumulator and the finalized (i.e. with memory records) fourth wire
    // polynomial
    witness_commitments.sorted_accum = commitment_key->commit(instance->proving_key->sorted_accum);
    witness_commitments.w_4 = commitment_key->commit(instance->proving_key->w_4);

    transcript->send_to_verifier(domain_separator + commitment_labels.sorted_accum, witness_commitments.sorted_accum);
    transcript->send_to_verifier(domain_separator + commitment_labels.w_4, witness_commitments.w_4);
}

/**
 * @brief Compute log derivative inverse polynomial and its commitment, if required
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_log_derivative_inverse_round()
{
    auto [beta, gamma] = transcript->template get_challenges<FF>(domain_separator + "beta", domain_separator + "gamma");
    instance->relation_parameters.beta = beta;
    instance->relation_parameters.gamma = gamma;
    if constexpr (IsGoblinFlavor<Flavor>) {
        // Compute and commit to the logderivative inverse used in DataBus
        instance->compute_logderivative_inverse(beta, gamma);
        witness_commitments.lookup_inverses = commitment_key->commit(instance->proving_key->lookup_inverses);
        transcript->send_to_verifier(domain_separator + commitment_labels.lookup_inverses,
                                     witness_commitments.lookup_inverses);
    }
}

/**
 * @brief Compute permutation and lookup grand product polynomials and their commitments
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_grand_product_computation_round()
{

    instance->compute_grand_product_polynomials(instance->relation_parameters.beta,
                                                instance->relation_parameters.gamma);

    witness_commitments.z_perm = commitment_key->commit(instance->proving_key->z_perm);
    witness_commitments.z_lookup = commitment_key->commit(instance->proving_key->z_lookup);

    transcript->send_to_verifier(domain_separator + commitment_labels.z_perm, witness_commitments.z_perm);
    transcript->send_to_verifier(domain_separator + commitment_labels.z_lookup, witness_commitments.z_lookup);
}

template class OinkProver<UltraFlavor>;
template class OinkProver<GoblinUltraFlavor>;

} // namespace bb