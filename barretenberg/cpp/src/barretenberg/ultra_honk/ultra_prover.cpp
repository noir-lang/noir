#include "ultra_prover.hpp"
#include "barretenberg/honk/proof_system/power_polynomial.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace proof_system::honk {

/**
 * Create UltraProver_ from an instance.
 *
 * @param instance Instance whose proof we want to generate.
 *
 * @tparam a type of UltraFlavor
 * */
template <UltraFlavor Flavor>
UltraProver_<Flavor>::UltraProver_(std::shared_ptr<Instance> inst)
    : instance(std::move(inst))
    , commitment_key(instance->commitment_key)
{
    instance->initialize_prover_polynomials();
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_preamble_round()
{
    auto proving_key = instance->proving_key;
    const auto circuit_size = static_cast<uint32_t>(proving_key->circuit_size);
    const auto num_public_inputs = static_cast<uint32_t>(proving_key->num_public_inputs);

    transcript.send_to_verifier("circuit_size", circuit_size);
    transcript.send_to_verifier("public_input_size", num_public_inputs);
    transcript.send_to_verifier("pub_inputs_offset", static_cast<uint32_t>(instance->pub_inputs_offset));

    for (size_t i = 0; i < proving_key->num_public_inputs; ++i) {
        auto public_input_i = instance->public_inputs[i];
        transcript.send_to_verifier("public_input_" + std::to_string(i), public_input_i);
    }
}

/**
 * @brief Compute commitments to the first three wire polynomials (and ECC op wires if using Goblin).
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_wire_commitments_round()
{
    // Commit to the first three wire polynomials
    // We only commit to the fourth wire polynomial after adding memory records
    auto wire_polys = instance->proving_key->get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < 3; ++idx) {
        transcript.send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }

    if constexpr (IsGoblinFlavor<Flavor>) {
        // Commit to Goblin ECC op wires
        auto op_wire_polys = instance->proving_key->get_ecc_op_wires();
        auto labels = commitment_labels.get_ecc_op_wires();
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            transcript.send_to_verifier(labels[idx], commitment_key->commit(op_wire_polys[idx]));
        }
        // Commit to DataBus columns
        transcript.send_to_verifier(commitment_labels.calldata,
                                    commitment_key->commit(instance->proving_key->calldata));
        transcript.send_to_verifier(commitment_labels.calldata_read_counts,
                                    commitment_key->commit(instance->proving_key->calldata_read_counts));
    }
}

/**
 * @brief Compute sorted witness-table accumulator and commit to the resulting polynomials.
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_sorted_list_accumulator_round()
{
    auto eta = transcript.get_challenge("eta");

    instance->compute_sorted_accumulator_polynomials(eta);

    // Commit to the sorted withness-table accumulator and the finalized (i.e. with memory records) fourth wire
    // polynomial
    auto sorted_accum_commitment = commitment_key->commit(instance->proving_key->sorted_accum);
    auto w_4_commitment = commitment_key->commit(instance->proving_key->w_4);
    transcript.send_to_verifier(commitment_labels.sorted_accum, sorted_accum_commitment);
    transcript.send_to_verifier(commitment_labels.w_4, w_4_commitment);
}

/**
 * @brief Compute log derivative inverse polynomial and its commitment, if required
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_log_derivative_inverse_round()
{
    // Compute and store challenges beta and gamma
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;

    if constexpr (IsGoblinFlavor<Flavor>) {
        instance->compute_logderivative_inverse(beta, gamma);

        auto lookup_inverses_commitment = commitment_key->commit(instance->proving_key->lookup_inverses);
        transcript.send_to_verifier(commitment_labels.lookup_inverses, lookup_inverses_commitment);
    }
}

/**
 * @brief Compute permutation and lookup grand product polynomials and their commitments
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_grand_product_computation_round()
{

    instance->compute_grand_product_polynomials(relation_parameters.beta, relation_parameters.gamma);

    auto z_perm_commitment = commitment_key->commit(instance->proving_key->z_perm);
    auto z_lookup_commitment = commitment_key->commit(instance->proving_key->z_lookup);
    transcript.send_to_verifier(commitment_labels.z_perm, z_perm_commitment);
    transcript.send_to_verifier(commitment_labels.z_lookup, z_lookup_commitment);
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(instance->proving_key->circuit_size, transcript);

    sumcheck_output = sumcheck.prove(instance);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
template <UltraFlavor Flavor> void UltraProver_<Flavor>::execute_zeromorph_rounds()
{
    ZeroMorph::prove(instance->prover_polynomials.get_unshifted(),
                     instance->prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript);
}

template <UltraFlavor Flavor> plonk::proof& UltraProver_<Flavor>::export_proof()
{
    proof.proof_data = transcript.proof_data;
    return proof;
}

template <UltraFlavor Flavor> plonk::proof& UltraProver_<Flavor>::construct_proof()
{
    // Add circuit size public input size and public inputs to transcript.
    execute_preamble_round();

    // Compute first three wire commitments
    execute_wire_commitments_round();

    // Compute sorted list accumulator and commitment
    execute_sorted_list_accumulator_round();

    // Fiat-Shamir: beta & gamma
    execute_log_derivative_inverse_round();

    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_zeromorph_rounds();

    return export_proof();
}

template class UltraProver_<honk::flavor::Ultra>;
template class UltraProver_<honk::flavor::GoblinUltra>;

} // namespace proof_system::honk
