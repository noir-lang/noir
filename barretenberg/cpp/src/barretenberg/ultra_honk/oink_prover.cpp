#include "barretenberg/ultra_honk/oink_prover.hpp"
#include "barretenberg/plonk_honk_shared/instance_inspector.hpp"
#include "barretenberg/relations/logderiv_lookup_relation.hpp"

namespace bb {

/**
 * @brief Oink Prover function that runs all the rounds of the verifier
 * @details Returns the witness commitments and relation_parameters
 * @tparam Flavor
 * @return OinkProverOutput<Flavor>
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::prove()
{
    {
        ZoneScopedN("execute_preamble_round");
        // Add circuit size public input size and public inputs to transcript->
        execute_preamble_round();
    }
    {
        ZoneScopedN("execute_wire_commitments_round");
        // Compute first three wire commitments
        execute_wire_commitments_round();
    }
    {
        ZoneScopedN("execute_sorted_list_accumulator_round");
        // Compute sorted list accumulator and commitment
        execute_sorted_list_accumulator_round();
    }

    {
        ZoneScopedN("execute_log_derivative_inverse_round");
        // Fiat-Shamir: beta & gamma
        execute_log_derivative_inverse_round();
    }

    {
        ZoneScopedN("execute_grand_product_computation_round");
        // Compute grand product(s) and commitments.
        execute_grand_product_computation_round();
    }

    // Generate relation separators alphas for sumcheck/combiner computation
    instance->alphas = generate_alphas_round();
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(instance->proving_key.circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(instance->proving_key.num_public_inputs);
    transcript->send_to_verifier(domain_separator + "circuit_size", circuit_size);
    transcript->send_to_verifier(domain_separator + "public_input_size", num_public_inputs);
    transcript->send_to_verifier(domain_separator + "pub_inputs_offset",
                                 static_cast<uint32_t>(instance->proving_key.pub_inputs_offset));

    ASSERT(instance->proving_key.num_public_inputs == instance->proving_key.public_inputs.size());

    for (size_t i = 0; i < instance->proving_key.num_public_inputs; ++i) {
        auto public_input_i = instance->proving_key.public_inputs[i];
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
    {
        BB_OP_COUNT_TIME_NAME("COMMIT::wires");
        witness_commitments.w_l = commitment_key->commit(instance->proving_key.polynomials.w_l);
        witness_commitments.w_r = commitment_key->commit(instance->proving_key.polynomials.w_r);
        witness_commitments.w_o = commitment_key->commit(instance->proving_key.polynomials.w_o);
    }

    auto wire_comms = witness_commitments.get_wires();
    auto wire_labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        transcript->send_to_verifier(domain_separator + wire_labels[idx], wire_comms[idx]);
    }

    if constexpr (IsGoblinFlavor<Flavor>) {

        // Commit to Goblin ECC op wires
        for (auto [commitment, polynomial, label] : zip_view(witness_commitments.get_ecc_op_wires(),
                                                             instance->proving_key.polynomials.get_ecc_op_wires(),
                                                             commitment_labels.get_ecc_op_wires())) {
            {
                BB_OP_COUNT_TIME_NAME("COMMIT::ecc_op_wires");
                commitment = commitment_key->commit_sparse(polynomial);
            }
            transcript->send_to_verifier(domain_separator + label, commitment);
        }

        // Commit to DataBus related polynomials
        for (auto [commitment, polynomial, label] : zip_view(witness_commitments.get_databus_entities(),
                                                             instance->proving_key.polynomials.get_databus_entities(),
                                                             commitment_labels.get_databus_entities())) {
            {
                BB_OP_COUNT_TIME_NAME("COMMIT::databus");
                commitment = commitment_key->commit_sparse(polynomial);
            }
            transcript->send_to_verifier(domain_separator + label, commitment);
        }
    }
}

/**
 * @brief Compute sorted witness-table accumulator and commit to the resulting polynomials.
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_sorted_list_accumulator_round()
{
    // Get eta challenges
    auto [eta, eta_two, eta_three] = transcript->template get_challenges<FF>(
        domain_separator + "eta", domain_separator + "eta_two", domain_separator + "eta_three");
    instance->relation_parameters.eta = eta;
    instance->relation_parameters.eta_two = eta_two;
    instance->relation_parameters.eta_three = eta_three;

    instance->proving_key.add_ram_rom_memory_records_to_wire_4(eta, eta_two, eta_three);

    // Commit to lookup argument polynomials and the finalized (i.e. with memory records) fourth wire polynomial
    {
        BB_OP_COUNT_TIME_NAME("COMMIT::lookup_counts_tags");
        witness_commitments.lookup_read_counts =
            commitment_key->commit(instance->proving_key.polynomials.lookup_read_counts);
        witness_commitments.lookup_read_tags =
            commitment_key->commit(instance->proving_key.polynomials.lookup_read_tags);
    }
    {
        BB_OP_COUNT_TIME_NAME("COMMIT::wires");
        witness_commitments.w_4 = commitment_key->commit(instance->proving_key.polynomials.w_4);
    }

    transcript->send_to_verifier(domain_separator + commitment_labels.lookup_read_counts,
                                 witness_commitments.lookup_read_counts);
    transcript->send_to_verifier(domain_separator + commitment_labels.lookup_read_tags,
                                 witness_commitments.lookup_read_tags);
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

    // Compute the inverses used in log-derivative lookup relations
    instance->proving_key.compute_logderivative_inverses(instance->relation_parameters);

    {
        BB_OP_COUNT_TIME_NAME("COMMIT::lookup_inverses");
        witness_commitments.lookup_inverses = commitment_key->commit(instance->proving_key.polynomials.lookup_inverses);
    }
    transcript->send_to_verifier(domain_separator + commitment_labels.lookup_inverses,
                                 witness_commitments.lookup_inverses);

    // If Mega, commit to the databus inverse polynomials and send
    if constexpr (IsGoblinFlavor<Flavor>) {
        for (auto [commitment, polynomial, label] : zip_view(witness_commitments.get_databus_inverses(),
                                                             instance->proving_key.polynomials.get_databus_inverses(),
                                                             commitment_labels.get_databus_inverses())) {
            {
                BB_OP_COUNT_TIME_NAME("COMMIT::databus_inverses");
                commitment = commitment_key->commit_sparse(polynomial);
            }
            transcript->send_to_verifier(domain_separator + label, commitment);
        }
    }
}

/**
 * @brief Compute permutation and lookup grand product polynomials and their commitments
 *
 */
template <IsUltraFlavor Flavor> void OinkProver<Flavor>::execute_grand_product_computation_round()
{
    instance->proving_key.compute_grand_product_polynomials(instance->relation_parameters);

    {
        BB_OP_COUNT_TIME_NAME("COMMIT::z_perm");
        witness_commitments.z_perm = commitment_key->commit(instance->proving_key.polynomials.z_perm);
    }
    transcript->send_to_verifier(domain_separator + commitment_labels.z_perm, witness_commitments.z_perm);
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
template class OinkProver<UltraKeccakFlavor>;
template class OinkProver<MegaFlavor>;

} // namespace bb