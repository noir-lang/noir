#include "barretenberg/ultra_honk/oink_prover.hpp"

namespace bb {

/**
 * @brief Oink Prover function that runs all the rounds of the verifier
 * @details Returns the witness commitments and relation_parameters
 * @tparam Flavor
 * @return OinkProverOutput<Flavor>
 */
template <IsUltraFlavor Flavor> OinkProverOutput<Flavor> OinkProver<Flavor>::prove()
{
    // Add circuit size public input size and public inputs to transcript->
    execute_preamble_round();

    // Compute first three wire commitments
    execute_wire_commitments_round();

    // Compute sorted list accumulator and commitment
    execute_sorted_list_accumulator_round();

    // Fiat-Shamir: beta & gamma
    execute_log_derivative_inverse_round();

    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();

    // Generate relation separators alphas for sumcheck/combiner computation
    RelationSeparator alphas = generate_alphas_round();

    return OinkProverOutput<Flavor>{
        .proving_key = std::move(proving_key),
        .relation_parameters = std::move(relation_parameters),
        .alphas = std::move(alphas),
    };
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(proving_key.circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(proving_key.num_public_inputs);
    transcript->send_to_verifier(domain_separator + "circuit_size", circuit_size);
    transcript->send_to_verifier(domain_separator + "public_input_size", num_public_inputs);
    transcript->send_to_verifier(domain_separator + "pub_inputs_offset",
                                 static_cast<uint32_t>(proving_key.pub_inputs_offset));

    ASSERT(proving_key.num_public_inputs == proving_key.public_inputs.size());

    for (size_t i = 0; i < proving_key.num_public_inputs; ++i) {
        auto public_input_i = proving_key.public_inputs[i];
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
    witness_commitments.w_l = commitment_key->commit(proving_key.w_l);
    witness_commitments.w_r = commitment_key->commit(proving_key.w_r);
    witness_commitments.w_o = commitment_key->commit(proving_key.w_o);

    auto wire_comms = witness_commitments.get_wires();
    auto wire_labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        transcript->send_to_verifier(domain_separator + wire_labels[idx], wire_comms[idx]);
    }

    if constexpr (IsGoblinFlavor<Flavor>) {
        // Commit to Goblin ECC op wires
        witness_commitments.ecc_op_wire_1 = commitment_key->commit(proving_key.ecc_op_wire_1);
        witness_commitments.ecc_op_wire_2 = commitment_key->commit(proving_key.ecc_op_wire_2);
        witness_commitments.ecc_op_wire_3 = commitment_key->commit(proving_key.ecc_op_wire_3);
        witness_commitments.ecc_op_wire_4 = commitment_key->commit(proving_key.ecc_op_wire_4);

        auto op_wire_comms = witness_commitments.get_ecc_op_wires();
        auto labels = commitment_labels.get_ecc_op_wires();
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            transcript->send_to_verifier(domain_separator + labels[idx], op_wire_comms[idx]);
        }

        // Commit to DataBus columns and corresponding read counts
        witness_commitments.calldata = commitment_key->commit(proving_key.calldata);
        witness_commitments.calldata_read_counts = commitment_key->commit(proving_key.calldata_read_counts);
        transcript->send_to_verifier(domain_separator + commitment_labels.calldata, witness_commitments.calldata);
        transcript->send_to_verifier(domain_separator + commitment_labels.calldata_read_counts,
                                     witness_commitments.calldata_read_counts);
        witness_commitments.return_data = commitment_key->commit(proving_key.return_data);
        witness_commitments.return_data_read_counts = commitment_key->commit(proving_key.return_data_read_counts);
        transcript->send_to_verifier(domain_separator + commitment_labels.return_data, witness_commitments.return_data);
        transcript->send_to_verifier(domain_separator + commitment_labels.return_data_read_counts,
                                     witness_commitments.return_data_read_counts);
    }
}

/**
 * @brief Compute sorted witness-table accumulator and commit to the resulting polynomials.
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_sorted_list_accumulator_round()
{

    auto [eta, eta_two, eta_three] = transcript->template get_challenges<FF>(
        domain_separator + "eta", domain_separator + "eta_two", domain_separator + "eta_three");
    relation_parameters.eta = eta;
    relation_parameters.eta_two = eta_two;
    relation_parameters.eta_three = eta_three;

    proving_key.compute_sorted_accumulator_polynomials(
        relation_parameters.eta, relation_parameters.eta_two, relation_parameters.eta_three);
    // Commit to the sorted witness-table accumulator and the finalized (i.e. with memory records) fourth wire
    // polynomial
    witness_commitments.sorted_accum = commitment_key->commit(proving_key.sorted_accum);
    witness_commitments.w_4 = commitment_key->commit(proving_key.w_4);

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
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;
    if constexpr (IsGoblinFlavor<Flavor>) {
        // Compute and commit to the logderivative inverse used in DataBus
        proving_key.compute_logderivative_inverse(relation_parameters);
        witness_commitments.calldata_inverses = commitment_key->commit(proving_key.calldata_inverses);
        witness_commitments.return_data_inverses = commitment_key->commit(proving_key.return_data_inverses);
        transcript->send_to_verifier(domain_separator + commitment_labels.calldata_inverses,
                                     witness_commitments.calldata_inverses);
        transcript->send_to_verifier(domain_separator + commitment_labels.return_data_inverses,
                                     witness_commitments.return_data_inverses);
    }
}

/**
 * @brief Compute permutation and lookup grand product polynomials and their commitments
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_grand_product_computation_round()
{

    proving_key.compute_grand_product_polynomials(relation_parameters);

    witness_commitments.z_perm = commitment_key->commit(proving_key.z_perm);
    witness_commitments.z_lookup = commitment_key->commit(proving_key.z_lookup);

    transcript->send_to_verifier(domain_separator + commitment_labels.z_perm, witness_commitments.z_perm);
    transcript->send_to_verifier(domain_separator + commitment_labels.z_lookup, witness_commitments.z_lookup);
}

template <IsUltraFlavor Flavor> typename Flavor::RelationSeparator OinkProver<Flavor>::generate_alphas_round()
{
    RelationSeparator alphas;
    for (size_t idx = 0; idx < alphas.size(); idx++) {
        alphas[idx] = transcript->template get_challenge<FF>(domain_separator + "alpha_" + std::to_string(idx));
    }
    return alphas;
}

template class OinkProver<UltraFlavor>;
template class OinkProver<GoblinUltraFlavor>;

} // namespace bb