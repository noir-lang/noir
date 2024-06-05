

#include "./avm_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

AvmVerifier::AvmVerifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

AvmVerifier::AvmVerifier(AvmVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

AvmVerifier& AvmVerifier::operator=(AvmVerifier&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

using FF = AvmFlavor::FF;

// Evaluate the given public input column over the multivariate challenge points
[[maybe_unused]] inline FF evaluate_public_input_column(const std::vector<FF>& points,
                                                        const size_t circuit_size,
                                                        std::vector<FF> challenges)
{

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6361): we pad the points to the circuit size in order
    // to get the correct evaluation. This is not efficient, and will not be valid in production.
    std::vector<FF> new_points(circuit_size, 0);
    std::copy(points.begin(), points.end(), new_points.data());

    Polynomial<FF> polynomial(new_points);
    return polynomial.evaluate_mle(challenges);
}

/**
 * @brief This function verifies an Avm Honk proof for given program settings.
 *
 */
bool AvmVerifier::verify_proof(const HonkProof& proof, const std::vector<std::vector<FF>>& public_inputs)
{
    using Flavor = AvmFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using PCS = Flavor::PCS;
    // using ZeroMorph = ZeroMorphVerifier_<PCS>;
    using VerifierCommitments = Flavor::VerifierCommitments;
    using CommitmentLabels = Flavor::CommitmentLabels;

    RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof);

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    commitments.avm_alu_a_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_a_hi);
    commitments.avm_alu_a_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_a_lo);
    commitments.avm_alu_alu_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_alu_sel);
    commitments.avm_alu_b_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_b_hi);
    commitments.avm_alu_b_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_b_lo);
    commitments.avm_alu_borrow = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_borrow);
    commitments.avm_alu_cf = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_cf);
    commitments.avm_alu_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_clk);
    commitments.avm_alu_cmp_rng_ctr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_cmp_rng_ctr);
    commitments.avm_alu_cmp_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_cmp_sel);
    commitments.avm_alu_div_rng_chk_selector =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_rng_chk_selector);
    commitments.avm_alu_div_u16_r0 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r0);
    commitments.avm_alu_div_u16_r1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r1);
    commitments.avm_alu_div_u16_r2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r2);
    commitments.avm_alu_div_u16_r3 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r3);
    commitments.avm_alu_div_u16_r4 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r4);
    commitments.avm_alu_div_u16_r5 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r5);
    commitments.avm_alu_div_u16_r6 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r6);
    commitments.avm_alu_div_u16_r7 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_div_u16_r7);
    commitments.avm_alu_divisor_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_divisor_hi);
    commitments.avm_alu_divisor_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_divisor_lo);
    commitments.avm_alu_ff_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ff_tag);
    commitments.avm_alu_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ia);
    commitments.avm_alu_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ib);
    commitments.avm_alu_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ic);
    commitments.avm_alu_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_in_tag);
    commitments.avm_alu_op_add = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_add);
    commitments.avm_alu_op_cast =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_cast);
    commitments.avm_alu_op_cast_prev =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_cast_prev);
    commitments.avm_alu_op_div = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_div);
    commitments.avm_alu_op_div_a_lt_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_div_a_lt_b);
    commitments.avm_alu_op_div_std =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_div_std);
    commitments.avm_alu_op_eq = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_eq);
    commitments.avm_alu_op_eq_diff_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_eq_diff_inv);
    commitments.avm_alu_op_lt = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_lt);
    commitments.avm_alu_op_lte = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_lte);
    commitments.avm_alu_op_mul = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_mul);
    commitments.avm_alu_op_not = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_not);
    commitments.avm_alu_op_shl = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_shl);
    commitments.avm_alu_op_shr = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_shr);
    commitments.avm_alu_op_sub = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_sub);
    commitments.avm_alu_p_a_borrow =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_p_a_borrow);
    commitments.avm_alu_p_b_borrow =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_p_b_borrow);
    commitments.avm_alu_p_sub_a_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_p_sub_a_hi);
    commitments.avm_alu_p_sub_a_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_p_sub_a_lo);
    commitments.avm_alu_p_sub_b_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_p_sub_b_hi);
    commitments.avm_alu_p_sub_b_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_p_sub_b_lo);
    commitments.avm_alu_partial_prod_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_partial_prod_hi);
    commitments.avm_alu_partial_prod_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_partial_prod_lo);
    commitments.avm_alu_quotient_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_quotient_hi);
    commitments.avm_alu_quotient_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_quotient_lo);
    commitments.avm_alu_remainder =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_remainder);
    commitments.avm_alu_res_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_res_hi);
    commitments.avm_alu_res_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_res_lo);
    commitments.avm_alu_rng_chk_lookup_selector =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_rng_chk_lookup_selector);
    commitments.avm_alu_rng_chk_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_rng_chk_sel);
    commitments.avm_alu_shift_lt_bit_len =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_shift_lt_bit_len);
    commitments.avm_alu_shift_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_shift_sel);
    commitments.avm_alu_t_sub_s_bits =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_t_sub_s_bits);
    commitments.avm_alu_two_pow_s =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_two_pow_s);
    commitments.avm_alu_two_pow_t_sub_s =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_two_pow_t_sub_s);
    commitments.avm_alu_u128_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u128_tag);
    commitments.avm_alu_u16_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r0);
    commitments.avm_alu_u16_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r1);
    commitments.avm_alu_u16_r10 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r10);
    commitments.avm_alu_u16_r11 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r11);
    commitments.avm_alu_u16_r12 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r12);
    commitments.avm_alu_u16_r13 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r13);
    commitments.avm_alu_u16_r14 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r14);
    commitments.avm_alu_u16_r2 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r2);
    commitments.avm_alu_u16_r3 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r3);
    commitments.avm_alu_u16_r4 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r4);
    commitments.avm_alu_u16_r5 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r5);
    commitments.avm_alu_u16_r6 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r6);
    commitments.avm_alu_u16_r7 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r7);
    commitments.avm_alu_u16_r8 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r8);
    commitments.avm_alu_u16_r9 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r9);
    commitments.avm_alu_u16_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_tag);
    commitments.avm_alu_u32_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u32_tag);
    commitments.avm_alu_u64_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u64_tag);
    commitments.avm_alu_u8_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u8_r0);
    commitments.avm_alu_u8_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u8_r1);
    commitments.avm_alu_u8_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u8_tag);
    commitments.avm_binary_acc_ia =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_acc_ia);
    commitments.avm_binary_acc_ib =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_acc_ib);
    commitments.avm_binary_acc_ic =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_acc_ic);
    commitments.avm_binary_bin_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_bin_sel);
    commitments.avm_binary_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_clk);
    commitments.avm_binary_ia_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_ia_bytes);
    commitments.avm_binary_ib_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_ib_bytes);
    commitments.avm_binary_ic_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_ic_bytes);
    commitments.avm_binary_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_in_tag);
    commitments.avm_binary_mem_tag_ctr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_mem_tag_ctr);
    commitments.avm_binary_mem_tag_ctr_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_mem_tag_ctr_inv);
    commitments.avm_binary_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_op_id);
    commitments.avm_binary_start =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_start);
    commitments.avm_byte_lookup_bin_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_bin_sel);
    commitments.avm_byte_lookup_table_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_byte_lengths);
    commitments.avm_byte_lookup_table_in_tags =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_in_tags);
    commitments.avm_byte_lookup_table_input_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_input_a);
    commitments.avm_byte_lookup_table_input_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_input_b);
    commitments.avm_byte_lookup_table_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_op_id);
    commitments.avm_byte_lookup_table_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_output);
    commitments.avm_conversion_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_conversion_clk);
    commitments.avm_conversion_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_conversion_input);
    commitments.avm_conversion_num_limbs =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_conversion_num_limbs);
    commitments.avm_conversion_radix =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_conversion_radix);
    commitments.avm_conversion_to_radix_le_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_conversion_to_radix_le_sel);
    commitments.avm_gas_da_gas_fixed_table =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_gas_da_gas_fixed_table);
    commitments.avm_gas_gas_cost_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_gas_gas_cost_sel);
    commitments.avm_gas_l2_gas_fixed_table =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_gas_l2_gas_fixed_table);
    commitments.avm_keccakf1600_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_keccakf1600_clk);
    commitments.avm_keccakf1600_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_keccakf1600_input);
    commitments.avm_keccakf1600_keccakf1600_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_keccakf1600_keccakf1600_sel);
    commitments.avm_keccakf1600_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_keccakf1600_output);
    commitments.avm_kernel_emit_l2_to_l1_msg_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_emit_l2_to_l1_msg_write_offset);
    commitments.avm_kernel_emit_note_hash_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_emit_note_hash_write_offset);
    commitments.avm_kernel_emit_nullifier_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_emit_nullifier_write_offset);
    commitments.avm_kernel_emit_unencrypted_log_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_emit_unencrypted_log_write_offset);
    commitments.avm_kernel_kernel_in_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_kernel_in_offset);
    commitments.avm_kernel_kernel_inputs__is_public =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_kernel_inputs__is_public);
    commitments.avm_kernel_kernel_metadata_out__is_public = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_kernel_metadata_out__is_public);
    commitments.avm_kernel_kernel_out_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_kernel_out_offset);
    commitments.avm_kernel_kernel_side_effect_out__is_public = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_kernel_side_effect_out__is_public);
    commitments.avm_kernel_kernel_value_out__is_public =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_kernel_value_out__is_public);
    commitments.avm_kernel_l1_to_l2_msg_exists_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_l1_to_l2_msg_exists_write_offset);
    commitments.avm_kernel_note_hash_exist_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_note_hash_exist_write_offset);
    commitments.avm_kernel_nullifier_exists_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_nullifier_exists_write_offset);
    commitments.avm_kernel_nullifier_non_exists_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_nullifier_non_exists_write_offset);
    commitments.avm_kernel_q_public_input_kernel_add_to_table = transcript->template receive_from_prover<Commitment>(
        commitment_labels.avm_kernel_q_public_input_kernel_add_to_table);
    commitments.avm_kernel_q_public_input_kernel_out_add_to_table =
        transcript->template receive_from_prover<Commitment>(
            commitment_labels.avm_kernel_q_public_input_kernel_out_add_to_table);
    commitments.avm_kernel_side_effect_counter =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_side_effect_counter);
    commitments.avm_kernel_sload_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_sload_write_offset);
    commitments.avm_kernel_sstore_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_kernel_sstore_write_offset);
    commitments.avm_main_alu_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_alu_in_tag);
    commitments.avm_main_alu_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_alu_sel);
    commitments.avm_main_bin_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_bin_op_id);
    commitments.avm_main_bin_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_bin_sel);
    commitments.avm_main_call_ptr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_call_ptr);
    commitments.avm_main_da_gas_op =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_da_gas_op);
    commitments.avm_main_da_gas_remaining =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_da_gas_remaining);
    commitments.avm_main_gas_cost_active =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_gas_cost_active);
    commitments.avm_main_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ia);
    commitments.avm_main_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ib);
    commitments.avm_main_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ic);
    commitments.avm_main_id = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_id);
    commitments.avm_main_id_zero =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_id_zero);
    commitments.avm_main_ind_a = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_a);
    commitments.avm_main_ind_b = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_b);
    commitments.avm_main_ind_c = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_c);
    commitments.avm_main_ind_d = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_d);
    commitments.avm_main_ind_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_a);
    commitments.avm_main_ind_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_b);
    commitments.avm_main_ind_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_c);
    commitments.avm_main_ind_op_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_d);
    commitments.avm_main_internal_return_ptr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_internal_return_ptr);
    commitments.avm_main_inv = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_inv);
    commitments.avm_main_l2_gas_op =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_l2_gas_op);
    commitments.avm_main_l2_gas_remaining =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_l2_gas_remaining);
    commitments.avm_main_last = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_last);
    commitments.avm_main_mem_idx_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_a);
    commitments.avm_main_mem_idx_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_b);
    commitments.avm_main_mem_idx_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_c);
    commitments.avm_main_mem_idx_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_d);
    commitments.avm_main_mem_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_a);
    commitments.avm_main_mem_op_activate_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_activate_gas);
    commitments.avm_main_mem_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_b);
    commitments.avm_main_mem_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_c);
    commitments.avm_main_mem_op_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_d);
    commitments.avm_main_op_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_op_err);
    commitments.avm_main_opcode_val =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_opcode_val);
    commitments.avm_main_pc = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_pc);
    commitments.avm_main_q_kernel_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_q_kernel_lookup);
    commitments.avm_main_q_kernel_output_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_q_kernel_output_lookup);
    commitments.avm_main_r_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_r_in_tag);
    commitments.avm_main_rwa = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwa);
    commitments.avm_main_rwb = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwb);
    commitments.avm_main_rwc = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwc);
    commitments.avm_main_rwd = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwd);
    commitments.avm_main_sel_cmov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_cmov);
    commitments.avm_main_sel_external_call =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_external_call);
    commitments.avm_main_sel_halt =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_halt);
    commitments.avm_main_sel_internal_call =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_internal_call);
    commitments.avm_main_sel_internal_return =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_internal_return);
    commitments.avm_main_sel_jump =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_jump);
    commitments.avm_main_sel_jumpi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_jumpi);
    commitments.avm_main_sel_mov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_mov);
    commitments.avm_main_sel_mov_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_mov_a);
    commitments.avm_main_sel_mov_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_mov_b);
    commitments.avm_main_sel_op_add =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_add);
    commitments.avm_main_sel_op_address =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_address);
    commitments.avm_main_sel_op_and =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_and);
    commitments.avm_main_sel_op_block_number =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_block_number);
    commitments.avm_main_sel_op_cast =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_cast);
    commitments.avm_main_sel_op_chain_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_chain_id);
    commitments.avm_main_sel_op_coinbase =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_coinbase);
    commitments.avm_main_sel_op_dagasleft =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_dagasleft);
    commitments.avm_main_sel_op_div =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_div);
    commitments.avm_main_sel_op_emit_l2_to_l1_msg =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_emit_l2_to_l1_msg);
    commitments.avm_main_sel_op_emit_note_hash =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_emit_note_hash);
    commitments.avm_main_sel_op_emit_nullifier =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_emit_nullifier);
    commitments.avm_main_sel_op_emit_unencrypted_log =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_emit_unencrypted_log);
    commitments.avm_main_sel_op_eq =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_eq);
    commitments.avm_main_sel_op_fdiv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_fdiv);
    commitments.avm_main_sel_op_fee_per_da_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_fee_per_da_gas);
    commitments.avm_main_sel_op_fee_per_l2_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_fee_per_l2_gas);
    commitments.avm_main_sel_op_get_contract_instance =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_get_contract_instance);
    commitments.avm_main_sel_op_keccak =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_keccak);
    commitments.avm_main_sel_op_l1_to_l2_msg_exists =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_l1_to_l2_msg_exists);
    commitments.avm_main_sel_op_l2gasleft =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_l2gasleft);
    commitments.avm_main_sel_op_lt =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_lt);
    commitments.avm_main_sel_op_lte =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_lte);
    commitments.avm_main_sel_op_mul =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_mul);
    commitments.avm_main_sel_op_not =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_not);
    commitments.avm_main_sel_op_note_hash_exists =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_note_hash_exists);
    commitments.avm_main_sel_op_nullifier_exists =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_nullifier_exists);
    commitments.avm_main_sel_op_or =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_or);
    commitments.avm_main_sel_op_pedersen =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_pedersen);
    commitments.avm_main_sel_op_poseidon2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_poseidon2);
    commitments.avm_main_sel_op_radix_le =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_radix_le);
    commitments.avm_main_sel_op_sender =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_sender);
    commitments.avm_main_sel_op_sha256 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_sha256);
    commitments.avm_main_sel_op_shl =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_shl);
    commitments.avm_main_sel_op_shr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_shr);
    commitments.avm_main_sel_op_sload =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_sload);
    commitments.avm_main_sel_op_sstore =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_sstore);
    commitments.avm_main_sel_op_storage_address =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_storage_address);
    commitments.avm_main_sel_op_sub =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_sub);
    commitments.avm_main_sel_op_timestamp =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_timestamp);
    commitments.avm_main_sel_op_transaction_fee =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_transaction_fee);
    commitments.avm_main_sel_op_version =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_version);
    commitments.avm_main_sel_op_xor =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_xor);
    commitments.avm_main_sel_rng_16 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_rng_16);
    commitments.avm_main_sel_rng_8 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_rng_8);
    commitments.avm_main_space_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_space_id);
    commitments.avm_main_table_pow_2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_table_pow_2);
    commitments.avm_main_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_tag_err);
    commitments.avm_main_w_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_w_in_tag);
    commitments.avm_mem_addr = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_addr);
    commitments.avm_mem_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_clk);
    commitments.avm_mem_diff_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_diff_hi);
    commitments.avm_mem_diff_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_diff_lo);
    commitments.avm_mem_diff_mid =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_diff_mid);
    commitments.avm_mem_glob_addr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_glob_addr);
    commitments.avm_mem_ind_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_a);
    commitments.avm_mem_ind_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_b);
    commitments.avm_mem_ind_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_c);
    commitments.avm_mem_ind_op_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_d);
    commitments.avm_mem_last = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_last);
    commitments.avm_mem_lastAccess =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_lastAccess);
    commitments.avm_mem_mem_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_mem_sel);
    commitments.avm_mem_one_min_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_one_min_inv);
    commitments.avm_mem_op_a = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_a);
    commitments.avm_mem_op_b = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_b);
    commitments.avm_mem_op_c = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_c);
    commitments.avm_mem_op_d = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_d);
    commitments.avm_mem_r_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_r_in_tag);
    commitments.avm_mem_rng_chk_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_rng_chk_sel);
    commitments.avm_mem_rw = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_rw);
    commitments.avm_mem_sel_cmov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_sel_cmov);
    commitments.avm_mem_sel_mov_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_sel_mov_a);
    commitments.avm_mem_sel_mov_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_sel_mov_b);
    commitments.avm_mem_skip_check_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_skip_check_tag);
    commitments.avm_mem_space_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_space_id);
    commitments.avm_mem_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_tag);
    commitments.avm_mem_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_tag_err);
    commitments.avm_mem_tsp = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_tsp);
    commitments.avm_mem_val = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_val);
    commitments.avm_mem_w_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_w_in_tag);
    commitments.avm_pedersen_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_pedersen_clk);
    commitments.avm_pedersen_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_pedersen_input);
    commitments.avm_pedersen_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_pedersen_output);
    commitments.avm_pedersen_pedersen_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_pedersen_pedersen_sel);
    commitments.avm_poseidon2_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_poseidon2_clk);
    commitments.avm_poseidon2_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_poseidon2_input);
    commitments.avm_poseidon2_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_poseidon2_output);
    commitments.avm_poseidon2_poseidon_perm_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_poseidon2_poseidon_perm_sel);
    commitments.avm_sha256_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_sha256_clk);
    commitments.avm_sha256_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_sha256_input);
    commitments.avm_sha256_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_sha256_output);
    commitments.avm_sha256_sha256_compression_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_sha256_sha256_compression_sel);
    commitments.avm_sha256_state =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_sha256_state);
    commitments.lookup_byte_lengths_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths_counts);
    commitments.lookup_byte_operations_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations_counts);
    commitments.lookup_opcode_gas_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_opcode_gas_counts);
    commitments.kernel_output_lookup_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_output_lookup_counts);
    commitments.lookup_into_kernel_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_into_kernel_counts);
    commitments.incl_main_tag_err_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_main_tag_err_counts);
    commitments.incl_mem_tag_err_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_mem_tag_err_counts);
    commitments.lookup_mem_rng_chk_lo_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_lo_counts);
    commitments.lookup_mem_rng_chk_mid_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_mid_counts);
    commitments.lookup_mem_rng_chk_hi_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_hi_counts);
    commitments.lookup_pow_2_0_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_pow_2_0_counts);
    commitments.lookup_pow_2_1_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_pow_2_1_counts);
    commitments.lookup_u8_0_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u8_0_counts);
    commitments.lookup_u8_1_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u8_1_counts);
    commitments.lookup_u16_0_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_0_counts);
    commitments.lookup_u16_1_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_1_counts);
    commitments.lookup_u16_2_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_2_counts);
    commitments.lookup_u16_3_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_3_counts);
    commitments.lookup_u16_4_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_4_counts);
    commitments.lookup_u16_5_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_5_counts);
    commitments.lookup_u16_6_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_6_counts);
    commitments.lookup_u16_7_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_7_counts);
    commitments.lookup_u16_8_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_8_counts);
    commitments.lookup_u16_9_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_9_counts);
    commitments.lookup_u16_10_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_10_counts);
    commitments.lookup_u16_11_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_11_counts);
    commitments.lookup_u16_12_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_12_counts);
    commitments.lookup_u16_13_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_13_counts);
    commitments.lookup_u16_14_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_14_counts);
    commitments.lookup_div_u16_0_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_0_counts);
    commitments.lookup_div_u16_1_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_1_counts);
    commitments.lookup_div_u16_2_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_2_counts);
    commitments.lookup_div_u16_3_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_3_counts);
    commitments.lookup_div_u16_4_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_4_counts);
    commitments.lookup_div_u16_5_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_5_counts);
    commitments.lookup_div_u16_6_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_6_counts);
    commitments.lookup_div_u16_7_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_7_counts);

    auto [beta, gamm] = transcript->template get_challenges<FF>("beta", "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamm;

    // Get commitments to inverses
    commitments.perm_main_alu = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_alu);
    commitments.perm_main_bin = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_bin);
    commitments.perm_main_conv = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_conv);
    commitments.perm_main_pos2_perm =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_pos2_perm);
    commitments.perm_main_pedersen =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_pedersen);
    commitments.perm_main_mem_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_a);
    commitments.perm_main_mem_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_b);
    commitments.perm_main_mem_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_c);
    commitments.perm_main_mem_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_d);
    commitments.perm_main_mem_ind_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_a);
    commitments.perm_main_mem_ind_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_b);
    commitments.perm_main_mem_ind_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_c);
    commitments.perm_main_mem_ind_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_d);
    commitments.lookup_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths);
    commitments.lookup_byte_operations =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations);
    commitments.lookup_opcode_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_opcode_gas);
    commitments.kernel_output_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_output_lookup);
    commitments.lookup_into_kernel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_into_kernel);
    commitments.incl_main_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_main_tag_err);
    commitments.incl_mem_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_mem_tag_err);
    commitments.lookup_mem_rng_chk_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_lo);
    commitments.lookup_mem_rng_chk_mid =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_mid);
    commitments.lookup_mem_rng_chk_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_hi);
    commitments.lookup_pow_2_0 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_pow_2_0);
    commitments.lookup_pow_2_1 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_pow_2_1);
    commitments.lookup_u8_0 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u8_0);
    commitments.lookup_u8_1 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u8_1);
    commitments.lookup_u16_0 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_0);
    commitments.lookup_u16_1 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_1);
    commitments.lookup_u16_2 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_2);
    commitments.lookup_u16_3 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_3);
    commitments.lookup_u16_4 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_4);
    commitments.lookup_u16_5 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_5);
    commitments.lookup_u16_6 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_6);
    commitments.lookup_u16_7 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_7);
    commitments.lookup_u16_8 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_8);
    commitments.lookup_u16_9 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_9);
    commitments.lookup_u16_10 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_10);
    commitments.lookup_u16_11 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_11);
    commitments.lookup_u16_12 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_12);
    commitments.lookup_u16_13 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_13);
    commitments.lookup_u16_14 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_14);
    commitments.lookup_div_u16_0 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_0);
    commitments.lookup_div_u16_1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_1);
    commitments.lookup_div_u16_2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_2);
    commitments.lookup_div_u16_3 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_3);
    commitments.lookup_div_u16_4 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_4);
    commitments.lookup_div_u16_5 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_5);
    commitments.lookup_div_u16_6 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_6);
    commitments.lookup_div_u16_7 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_7);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);

    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");

    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        info("failed sumcheck");
        return false;
    }

    // Public columns evaluation checks

    FF avm_kernel_kernel_inputs__is_public_evaluation =
        evaluate_public_input_column(public_inputs[0], circuit_size, multivariate_challenge);
    if (avm_kernel_kernel_inputs__is_public_evaluation != claimed_evaluations.avm_kernel_kernel_inputs__is_public) {
        info("failed kernel inputs public inputs");
        return false;
    }

    FF avm_kernel_kernel_value_out__is_public_evaluation =
        evaluate_public_input_column(public_inputs[1], circuit_size, multivariate_challenge);
    if (avm_kernel_kernel_value_out__is_public_evaluation !=
        claimed_evaluations.avm_kernel_kernel_value_out__is_public) {
        info("failed value out inputs");
        return false;
    }

    FF avm_kernel_kernel_side_effect_out__is_public_evaluation =
        evaluate_public_input_column(public_inputs[2], circuit_size, multivariate_challenge);
    if (avm_kernel_kernel_side_effect_out__is_public_evaluation !=
        claimed_evaluations.avm_kernel_kernel_side_effect_out__is_public) {
        info("failed side effect inputs");
        return false;
    }

    FF avm_kernel_kernel_metadata_out__is_public_evaluation =
        evaluate_public_input_column(public_inputs[3], circuit_size, multivariate_challenge);
    if (avm_kernel_kernel_metadata_out__is_public_evaluation !=
        claimed_evaluations.avm_kernel_kernel_metadata_out__is_public) {
        info("failed kernel metadata inputs");
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    // NOTE: temporarily disabled - facing integration issues
    // auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
    //                                         commitments.get_to_be_shifted(),
    //                                         claimed_evaluations.get_unshifted(),
    //                                         claimed_evaluations.get_shifted(),
    //                                         multivariate_challenge,
    //                                         transcript);

    // auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);
    // return sumcheck_verified.value() && verified;
    return sumcheck_verified.value();
}

} // namespace bb
