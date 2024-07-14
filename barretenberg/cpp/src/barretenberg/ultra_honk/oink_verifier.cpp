#include "barretenberg/ultra_honk/oink_verifier.hpp"

namespace bb {

/**
 * @brief Oink Verifier function that runs all the rounds of the verifier
 * @details Returns the witness commitments and relation_parameters
 * @tparam Flavor
 * @return OinkOutput<Flavor>
 */
template <IsUltraFlavor Flavor> OinkOutput<Flavor> OinkVerifier<Flavor>::verify()
{
    // Execute the Verifier rounds
    execute_preamble_round();
    execute_wire_commitments_round();
    execute_sorted_list_accumulator_round();
    execute_log_derivative_inverse_round();
    execute_grand_product_computation_round();
    RelationSeparator alphas = generate_alphas_round();

    return OinkOutput<Flavor>{ .relation_parameters = relation_parameters,
                               .commitments = std::move(witness_comms),
                               .public_inputs = public_inputs,
                               .alphas = alphas };
}

/**
 * @brief Get circuit size, public input size, and public inputs from transcript
 *
 */
template <IsUltraFlavor Flavor> void OinkVerifier<Flavor>::execute_preamble_round()
{
    // TODO(Adrian): Change the initialization of the transcript to take the VK hash?
    const auto circuit_size = transcript->template receive_from_prover<uint32_t>(domain_separator + "circuit_size");
    const auto public_input_size =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "public_input_size");
    const auto pub_inputs_offset =
        transcript->template receive_from_prover<uint32_t>(domain_separator + "pub_inputs_offset");

    ASSERT(circuit_size == key->circuit_size);
    ASSERT(public_input_size == key->num_public_inputs);
    ASSERT(pub_inputs_offset == key->pub_inputs_offset);

    for (size_t i = 0; i < public_input_size; ++i) {
        auto public_input_i =
            transcript->template receive_from_prover<FF>(domain_separator + "public_input_" + std::to_string(i));
        public_inputs.emplace_back(public_input_i);
    }
}

/**
 * @brief Get the wire polynomials (part of the witness), with the exception of the fourth wire, which is
 * only received after adding memory records. In the Goblin Flavor, we also receive the ECC OP wires and the
 * DataBus columns.
 */
template <IsUltraFlavor Flavor> void OinkVerifier<Flavor>::execute_wire_commitments_round()
{
    // Get commitments to first three wire polynomials
    witness_comms.w_l = transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.w_l);
    witness_comms.w_r = transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.w_r);
    witness_comms.w_o = transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.w_o);

    // If Goblin, get commitments to ECC op wire polynomials and DataBus columns
    if constexpr (IsGoblinFlavor<Flavor>) {
        // Receive ECC op wire commitments
        for (auto [commitment, label] : zip_view(witness_comms.get_ecc_op_wires(), comm_labels.get_ecc_op_wires())) {
            commitment = transcript->template receive_from_prover<Commitment>(domain_separator + label);
        }

        // Receive DataBus related polynomial commitments
        for (auto [commitment, label] :
             zip_view(witness_comms.get_databus_entities(), comm_labels.get_databus_entities())) {
            commitment = transcript->template receive_from_prover<Commitment>(domain_separator + label);
        }
    }
}

/**
 * @brief Get sorted witness-table accumulator and fourth wire commitments
 *
 */
template <IsUltraFlavor Flavor> void OinkVerifier<Flavor>::execute_sorted_list_accumulator_round()
{
    // Get eta challenges
    auto [eta, eta_two, eta_three] = transcript->template get_challenges<FF>(
        domain_separator + "eta", domain_separator + "eta_two", domain_separator + "eta_three");
    relation_parameters.eta = eta;
    relation_parameters.eta_two = eta_two;
    relation_parameters.eta_three = eta_three;

    // Get commitments to lookup argument polynomials and fourth wire
    witness_comms.lookup_read_counts =
        transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.lookup_read_counts);
    witness_comms.lookup_read_tags =
        transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.lookup_read_tags);
    witness_comms.w_4 = transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.w_4);
}

/**
 * @brief Get log derivative inverse polynomial and its commitment, if GoblinFlavor
 *
 */
template <IsUltraFlavor Flavor> void OinkVerifier<Flavor>::execute_log_derivative_inverse_round()
{
    // Get permutation challenges
    auto [beta, gamma] = transcript->template get_challenges<FF>(domain_separator + "beta", domain_separator + "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;

    witness_comms.lookup_inverses =
        transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.lookup_inverses);

    // If Goblin (i.e. using DataBus) receive commitments to log-deriv inverses polynomials
    if constexpr (IsGoblinFlavor<Flavor>) {
        witness_comms.calldata_inverses =
            transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.calldata_inverses);
        witness_comms.return_data_inverses =
            transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.return_data_inverses);
    }
}

/**
 * @brief Compute lookup grand product delta and get permutation and lookup grand product commitments
 *
 */
template <IsUltraFlavor Flavor> void OinkVerifier<Flavor>::execute_grand_product_computation_round()
{
    const FF public_input_delta = compute_public_input_delta<Flavor>(public_inputs,
                                                                     relation_parameters.beta,
                                                                     relation_parameters.gamma,
                                                                     key->circuit_size,
                                                                     static_cast<size_t>(key->pub_inputs_offset));

    relation_parameters.public_input_delta = public_input_delta;

    // Get commitment to permutation and lookup grand products
    witness_comms.z_perm = transcript->template receive_from_prover<Commitment>(domain_separator + comm_labels.z_perm);
}

template <IsUltraFlavor Flavor> typename Flavor::RelationSeparator OinkVerifier<Flavor>::generate_alphas_round()
{
    // Get the relation separation challenges for sumcheck/combiner computation
    RelationSeparator alphas;
    for (size_t idx = 0; idx < alphas.size(); idx++) {
        alphas[idx] = transcript->template get_challenge<FF>(domain_separator + "alpha_" + std::to_string(idx));
    }
    return alphas;
}

template class OinkVerifier<UltraFlavor>;
template class OinkVerifier<MegaFlavor>;

} // namespace bb