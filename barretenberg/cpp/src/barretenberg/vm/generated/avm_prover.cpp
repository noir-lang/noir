

#include "avm_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

using Flavor = AvmFlavor;
using FF = Flavor::FF;

/**
 * Create AvmProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
AvmProver::AvmProver(std::shared_ptr<Flavor::ProvingKey> input_key, std::shared_ptr<PCSCommitmentKey> commitment_key)
    : key(input_key)
    , commitment_key(commitment_key)
{
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), key->get_all())) {
        ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) == bb::flavor_get_label(*key, key_poly));
        prover_poly = key_poly.share();
    }
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_shifted(), key->get_to_be_shifted())) {
        ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
               bb::flavor_get_label(*key, key_poly) + "_shift");
        prover_poly = key_poly.shifted();
    }
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
void AvmProver::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);

    transcript->send_to_verifier("circuit_size", circuit_size);
}

/**
 * @brief Compute commitments to all of the witness wires (apart from the logderivative inverse wires)
 *
 */
void AvmProver::execute_wire_commitments_round()
{

    // Commit to all polynomials (apart from logderivative inverse polynomials, which are committed to in the later
    // logderivative phase)
    auto wire_polys = prover_polynomials.get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < wire_polys.size(); ++idx) {
        transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }
}

void AvmProver::execute_log_derivative_inverse_round()
{

    auto [beta, gamm] = transcript->template get_challenges<FF>("beta", "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamm;

    key->compute_logderivative_inverses(relation_parameters);

    // Commit to all logderivative inverse polynomials
    witness_commitments.perm_main_alu = commitment_key->commit(key->perm_main_alu);
    witness_commitments.perm_main_bin = commitment_key->commit(key->perm_main_bin);
    witness_commitments.perm_main_conv = commitment_key->commit(key->perm_main_conv);
    witness_commitments.perm_main_pos2_perm = commitment_key->commit(key->perm_main_pos2_perm);
    witness_commitments.perm_main_pedersen = commitment_key->commit(key->perm_main_pedersen);
    witness_commitments.perm_main_mem_a = commitment_key->commit(key->perm_main_mem_a);
    witness_commitments.perm_main_mem_b = commitment_key->commit(key->perm_main_mem_b);
    witness_commitments.perm_main_mem_c = commitment_key->commit(key->perm_main_mem_c);
    witness_commitments.perm_main_mem_d = commitment_key->commit(key->perm_main_mem_d);
    witness_commitments.perm_main_mem_ind_addr_a = commitment_key->commit(key->perm_main_mem_ind_addr_a);
    witness_commitments.perm_main_mem_ind_addr_b = commitment_key->commit(key->perm_main_mem_ind_addr_b);
    witness_commitments.perm_main_mem_ind_addr_c = commitment_key->commit(key->perm_main_mem_ind_addr_c);
    witness_commitments.perm_main_mem_ind_addr_d = commitment_key->commit(key->perm_main_mem_ind_addr_d);
    witness_commitments.lookup_byte_lengths = commitment_key->commit(key->lookup_byte_lengths);
    witness_commitments.lookup_byte_operations = commitment_key->commit(key->lookup_byte_operations);
    witness_commitments.lookup_opcode_gas = commitment_key->commit(key->lookup_opcode_gas);
    witness_commitments.range_check_l2_gas_hi = commitment_key->commit(key->range_check_l2_gas_hi);
    witness_commitments.range_check_l2_gas_lo = commitment_key->commit(key->range_check_l2_gas_lo);
    witness_commitments.range_check_da_gas_hi = commitment_key->commit(key->range_check_da_gas_hi);
    witness_commitments.range_check_da_gas_lo = commitment_key->commit(key->range_check_da_gas_lo);
    witness_commitments.kernel_output_lookup = commitment_key->commit(key->kernel_output_lookup);
    witness_commitments.lookup_into_kernel = commitment_key->commit(key->lookup_into_kernel);
    witness_commitments.incl_main_tag_err = commitment_key->commit(key->incl_main_tag_err);
    witness_commitments.incl_mem_tag_err = commitment_key->commit(key->incl_mem_tag_err);
    witness_commitments.lookup_mem_rng_chk_lo = commitment_key->commit(key->lookup_mem_rng_chk_lo);
    witness_commitments.lookup_mem_rng_chk_mid = commitment_key->commit(key->lookup_mem_rng_chk_mid);
    witness_commitments.lookup_mem_rng_chk_hi = commitment_key->commit(key->lookup_mem_rng_chk_hi);
    witness_commitments.lookup_pow_2_0 = commitment_key->commit(key->lookup_pow_2_0);
    witness_commitments.lookup_pow_2_1 = commitment_key->commit(key->lookup_pow_2_1);
    witness_commitments.lookup_u8_0 = commitment_key->commit(key->lookup_u8_0);
    witness_commitments.lookup_u8_1 = commitment_key->commit(key->lookup_u8_1);
    witness_commitments.lookup_u16_0 = commitment_key->commit(key->lookup_u16_0);
    witness_commitments.lookup_u16_1 = commitment_key->commit(key->lookup_u16_1);
    witness_commitments.lookup_u16_2 = commitment_key->commit(key->lookup_u16_2);
    witness_commitments.lookup_u16_3 = commitment_key->commit(key->lookup_u16_3);
    witness_commitments.lookup_u16_4 = commitment_key->commit(key->lookup_u16_4);
    witness_commitments.lookup_u16_5 = commitment_key->commit(key->lookup_u16_5);
    witness_commitments.lookup_u16_6 = commitment_key->commit(key->lookup_u16_6);
    witness_commitments.lookup_u16_7 = commitment_key->commit(key->lookup_u16_7);
    witness_commitments.lookup_u16_8 = commitment_key->commit(key->lookup_u16_8);
    witness_commitments.lookup_u16_9 = commitment_key->commit(key->lookup_u16_9);
    witness_commitments.lookup_u16_10 = commitment_key->commit(key->lookup_u16_10);
    witness_commitments.lookup_u16_11 = commitment_key->commit(key->lookup_u16_11);
    witness_commitments.lookup_u16_12 = commitment_key->commit(key->lookup_u16_12);
    witness_commitments.lookup_u16_13 = commitment_key->commit(key->lookup_u16_13);
    witness_commitments.lookup_u16_14 = commitment_key->commit(key->lookup_u16_14);
    witness_commitments.lookup_div_u16_0 = commitment_key->commit(key->lookup_div_u16_0);
    witness_commitments.lookup_div_u16_1 = commitment_key->commit(key->lookup_div_u16_1);
    witness_commitments.lookup_div_u16_2 = commitment_key->commit(key->lookup_div_u16_2);
    witness_commitments.lookup_div_u16_3 = commitment_key->commit(key->lookup_div_u16_3);
    witness_commitments.lookup_div_u16_4 = commitment_key->commit(key->lookup_div_u16_4);
    witness_commitments.lookup_div_u16_5 = commitment_key->commit(key->lookup_div_u16_5);
    witness_commitments.lookup_div_u16_6 = commitment_key->commit(key->lookup_div_u16_6);
    witness_commitments.lookup_div_u16_7 = commitment_key->commit(key->lookup_div_u16_7);

    // Send all commitments to the verifier
    transcript->send_to_verifier(commitment_labels.perm_main_alu, witness_commitments.perm_main_alu);
    transcript->send_to_verifier(commitment_labels.perm_main_bin, witness_commitments.perm_main_bin);
    transcript->send_to_verifier(commitment_labels.perm_main_conv, witness_commitments.perm_main_conv);
    transcript->send_to_verifier(commitment_labels.perm_main_pos2_perm, witness_commitments.perm_main_pos2_perm);
    transcript->send_to_verifier(commitment_labels.perm_main_pedersen, witness_commitments.perm_main_pedersen);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_a, witness_commitments.perm_main_mem_a);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_b, witness_commitments.perm_main_mem_b);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_c, witness_commitments.perm_main_mem_c);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_d, witness_commitments.perm_main_mem_d);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_addr_a,
                                 witness_commitments.perm_main_mem_ind_addr_a);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_addr_b,
                                 witness_commitments.perm_main_mem_ind_addr_b);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_addr_c,
                                 witness_commitments.perm_main_mem_ind_addr_c);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_addr_d,
                                 witness_commitments.perm_main_mem_ind_addr_d);
    transcript->send_to_verifier(commitment_labels.lookup_byte_lengths, witness_commitments.lookup_byte_lengths);
    transcript->send_to_verifier(commitment_labels.lookup_byte_operations, witness_commitments.lookup_byte_operations);
    transcript->send_to_verifier(commitment_labels.lookup_opcode_gas, witness_commitments.lookup_opcode_gas);
    transcript->send_to_verifier(commitment_labels.range_check_l2_gas_hi, witness_commitments.range_check_l2_gas_hi);
    transcript->send_to_verifier(commitment_labels.range_check_l2_gas_lo, witness_commitments.range_check_l2_gas_lo);
    transcript->send_to_verifier(commitment_labels.range_check_da_gas_hi, witness_commitments.range_check_da_gas_hi);
    transcript->send_to_verifier(commitment_labels.range_check_da_gas_lo, witness_commitments.range_check_da_gas_lo);
    transcript->send_to_verifier(commitment_labels.kernel_output_lookup, witness_commitments.kernel_output_lookup);
    transcript->send_to_verifier(commitment_labels.lookup_into_kernel, witness_commitments.lookup_into_kernel);
    transcript->send_to_verifier(commitment_labels.incl_main_tag_err, witness_commitments.incl_main_tag_err);
    transcript->send_to_verifier(commitment_labels.incl_mem_tag_err, witness_commitments.incl_mem_tag_err);
    transcript->send_to_verifier(commitment_labels.lookup_mem_rng_chk_lo, witness_commitments.lookup_mem_rng_chk_lo);
    transcript->send_to_verifier(commitment_labels.lookup_mem_rng_chk_mid, witness_commitments.lookup_mem_rng_chk_mid);
    transcript->send_to_verifier(commitment_labels.lookup_mem_rng_chk_hi, witness_commitments.lookup_mem_rng_chk_hi);
    transcript->send_to_verifier(commitment_labels.lookup_pow_2_0, witness_commitments.lookup_pow_2_0);
    transcript->send_to_verifier(commitment_labels.lookup_pow_2_1, witness_commitments.lookup_pow_2_1);
    transcript->send_to_verifier(commitment_labels.lookup_u8_0, witness_commitments.lookup_u8_0);
    transcript->send_to_verifier(commitment_labels.lookup_u8_1, witness_commitments.lookup_u8_1);
    transcript->send_to_verifier(commitment_labels.lookup_u16_0, witness_commitments.lookup_u16_0);
    transcript->send_to_verifier(commitment_labels.lookup_u16_1, witness_commitments.lookup_u16_1);
    transcript->send_to_verifier(commitment_labels.lookup_u16_2, witness_commitments.lookup_u16_2);
    transcript->send_to_verifier(commitment_labels.lookup_u16_3, witness_commitments.lookup_u16_3);
    transcript->send_to_verifier(commitment_labels.lookup_u16_4, witness_commitments.lookup_u16_4);
    transcript->send_to_verifier(commitment_labels.lookup_u16_5, witness_commitments.lookup_u16_5);
    transcript->send_to_verifier(commitment_labels.lookup_u16_6, witness_commitments.lookup_u16_6);
    transcript->send_to_verifier(commitment_labels.lookup_u16_7, witness_commitments.lookup_u16_7);
    transcript->send_to_verifier(commitment_labels.lookup_u16_8, witness_commitments.lookup_u16_8);
    transcript->send_to_verifier(commitment_labels.lookup_u16_9, witness_commitments.lookup_u16_9);
    transcript->send_to_verifier(commitment_labels.lookup_u16_10, witness_commitments.lookup_u16_10);
    transcript->send_to_verifier(commitment_labels.lookup_u16_11, witness_commitments.lookup_u16_11);
    transcript->send_to_verifier(commitment_labels.lookup_u16_12, witness_commitments.lookup_u16_12);
    transcript->send_to_verifier(commitment_labels.lookup_u16_13, witness_commitments.lookup_u16_13);
    transcript->send_to_verifier(commitment_labels.lookup_u16_14, witness_commitments.lookup_u16_14);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_0, witness_commitments.lookup_div_u16_0);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_1, witness_commitments.lookup_div_u16_1);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_2, witness_commitments.lookup_div_u16_2);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_3, witness_commitments.lookup_div_u16_3);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_4, witness_commitments.lookup_div_u16_4);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_5, witness_commitments.lookup_div_u16_5);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_6, witness_commitments.lookup_div_u16_6);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_7, witness_commitments.lookup_div_u16_7);
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
void AvmProver::execute_relation_check_rounds()
{
    using Sumcheck = SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);

    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));

    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters, alpha, gate_challenges);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
void AvmProver::execute_pcs_rounds()
{
    auto prover_opening_claim = ZeroMorph::prove(key->circuit_size,
                                                 prover_polynomials.get_unshifted(),
                                                 prover_polynomials.get_to_be_shifted(),
                                                 sumcheck_output.claimed_evaluations.get_unshifted(),
                                                 sumcheck_output.claimed_evaluations.get_shifted(),
                                                 sumcheck_output.challenge,
                                                 commitment_key,
                                                 transcript);
    PCS::compute_opening_proof(commitment_key, prover_opening_claim, transcript);
}

HonkProof AvmProver::export_proof()
{
    proof = transcript->proof_data;
    return proof;
}

HonkProof AvmProver::construct_proof()
{
    // Add circuit size public input size and public inputs to transcript.
    execute_preamble_round();

    // Compute wire commitments
    execute_wire_commitments_round();

    // Compute sorted list accumulator and commitment
    execute_log_derivative_inverse_round();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_pcs_rounds();

    return export_proof();
}

} // namespace bb
