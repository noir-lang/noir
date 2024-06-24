

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
    commitments.kernel_kernel_inputs =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_kernel_inputs);
    commitments.kernel_kernel_value_out =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_kernel_value_out);
    commitments.kernel_kernel_side_effect_out =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_kernel_side_effect_out);
    commitments.kernel_kernel_metadata_out =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_kernel_metadata_out);
    commitments.alu_a_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_a_hi);
    commitments.alu_a_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_a_lo);
    commitments.alu_b_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_b_hi);
    commitments.alu_b_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_b_lo);
    commitments.alu_borrow = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_borrow);
    commitments.alu_cf = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_cf);
    commitments.alu_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_clk);
    commitments.alu_cmp_rng_ctr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_cmp_rng_ctr);
    commitments.alu_div_u16_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r0);
    commitments.alu_div_u16_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r1);
    commitments.alu_div_u16_r2 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r2);
    commitments.alu_div_u16_r3 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r3);
    commitments.alu_div_u16_r4 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r4);
    commitments.alu_div_u16_r5 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r5);
    commitments.alu_div_u16_r6 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r6);
    commitments.alu_div_u16_r7 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_div_u16_r7);
    commitments.alu_divisor_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_divisor_hi);
    commitments.alu_divisor_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_divisor_lo);
    commitments.alu_ff_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_ff_tag);
    commitments.alu_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_ia);
    commitments.alu_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_ib);
    commitments.alu_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_ic);
    commitments.alu_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_in_tag);
    commitments.alu_op_add = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_add);
    commitments.alu_op_cast = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_cast);
    commitments.alu_op_cast_prev =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_cast_prev);
    commitments.alu_op_div = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_div);
    commitments.alu_op_div_a_lt_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_div_a_lt_b);
    commitments.alu_op_div_std = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_div_std);
    commitments.alu_op_eq = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_eq);
    commitments.alu_op_eq_diff_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_eq_diff_inv);
    commitments.alu_op_lt = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_lt);
    commitments.alu_op_lte = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_lte);
    commitments.alu_op_mul = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_mul);
    commitments.alu_op_not = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_not);
    commitments.alu_op_shl = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_shl);
    commitments.alu_op_shr = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_shr);
    commitments.alu_op_sub = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_op_sub);
    commitments.alu_p_a_borrow = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_p_a_borrow);
    commitments.alu_p_b_borrow = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_p_b_borrow);
    commitments.alu_p_sub_a_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_p_sub_a_hi);
    commitments.alu_p_sub_a_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_p_sub_a_lo);
    commitments.alu_p_sub_b_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_p_sub_b_hi);
    commitments.alu_p_sub_b_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_p_sub_b_lo);
    commitments.alu_partial_prod_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_partial_prod_hi);
    commitments.alu_partial_prod_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_partial_prod_lo);
    commitments.alu_quotient_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_quotient_hi);
    commitments.alu_quotient_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_quotient_lo);
    commitments.alu_remainder = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_remainder);
    commitments.alu_res_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_res_hi);
    commitments.alu_res_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_res_lo);
    commitments.alu_sel_alu = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_sel_alu);
    commitments.alu_sel_cmp = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_sel_cmp);
    commitments.alu_sel_div_rng_chk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_sel_div_rng_chk);
    commitments.alu_sel_rng_chk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_sel_rng_chk);
    commitments.alu_sel_rng_chk_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_sel_rng_chk_lookup);
    commitments.alu_sel_shift_which =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_sel_shift_which);
    commitments.alu_shift_lt_bit_len =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_shift_lt_bit_len);
    commitments.alu_t_sub_s_bits =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_t_sub_s_bits);
    commitments.alu_two_pow_s = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_two_pow_s);
    commitments.alu_two_pow_t_sub_s =
        transcript->template receive_from_prover<Commitment>(commitment_labels.alu_two_pow_t_sub_s);
    commitments.alu_u128_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u128_tag);
    commitments.alu_u16_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r0);
    commitments.alu_u16_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r1);
    commitments.alu_u16_r10 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r10);
    commitments.alu_u16_r11 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r11);
    commitments.alu_u16_r12 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r12);
    commitments.alu_u16_r13 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r13);
    commitments.alu_u16_r14 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r14);
    commitments.alu_u16_r2 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r2);
    commitments.alu_u16_r3 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r3);
    commitments.alu_u16_r4 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r4);
    commitments.alu_u16_r5 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r5);
    commitments.alu_u16_r6 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r6);
    commitments.alu_u16_r7 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r7);
    commitments.alu_u16_r8 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r8);
    commitments.alu_u16_r9 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_r9);
    commitments.alu_u16_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u16_tag);
    commitments.alu_u32_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u32_tag);
    commitments.alu_u64_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u64_tag);
    commitments.alu_u8_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u8_r0);
    commitments.alu_u8_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u8_r1);
    commitments.alu_u8_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.alu_u8_tag);
    commitments.binary_acc_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_acc_ia);
    commitments.binary_acc_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_acc_ib);
    commitments.binary_acc_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_acc_ic);
    commitments.binary_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_clk);
    commitments.binary_ia_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.binary_ia_bytes);
    commitments.binary_ib_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.binary_ib_bytes);
    commitments.binary_ic_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.binary_ic_bytes);
    commitments.binary_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_in_tag);
    commitments.binary_mem_tag_ctr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.binary_mem_tag_ctr);
    commitments.binary_mem_tag_ctr_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.binary_mem_tag_ctr_inv);
    commitments.binary_op_id = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_op_id);
    commitments.binary_sel_bin = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_sel_bin);
    commitments.binary_start = transcript->template receive_from_prover<Commitment>(commitment_labels.binary_start);
    commitments.byte_lookup_sel_bin =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_sel_bin);
    commitments.byte_lookup_table_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_table_byte_lengths);
    commitments.byte_lookup_table_in_tags =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_table_in_tags);
    commitments.byte_lookup_table_input_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_table_input_a);
    commitments.byte_lookup_table_input_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_table_input_b);
    commitments.byte_lookup_table_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_table_op_id);
    commitments.byte_lookup_table_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.byte_lookup_table_output);
    commitments.conversion_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.conversion_clk);
    commitments.conversion_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.conversion_input);
    commitments.conversion_num_limbs =
        transcript->template receive_from_prover<Commitment>(commitment_labels.conversion_num_limbs);
    commitments.conversion_radix =
        transcript->template receive_from_prover<Commitment>(commitment_labels.conversion_radix);
    commitments.conversion_sel_to_radix_le =
        transcript->template receive_from_prover<Commitment>(commitment_labels.conversion_sel_to_radix_le);
    commitments.gas_da_gas_fixed_table =
        transcript->template receive_from_prover<Commitment>(commitment_labels.gas_da_gas_fixed_table);
    commitments.gas_l2_gas_fixed_table =
        transcript->template receive_from_prover<Commitment>(commitment_labels.gas_l2_gas_fixed_table);
    commitments.gas_sel_gas_cost =
        transcript->template receive_from_prover<Commitment>(commitment_labels.gas_sel_gas_cost);
    commitments.keccakf1600_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.keccakf1600_clk);
    commitments.keccakf1600_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.keccakf1600_input);
    commitments.keccakf1600_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.keccakf1600_output);
    commitments.keccakf1600_sel_keccakf1600 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.keccakf1600_sel_keccakf1600);
    commitments.kernel_emit_l2_to_l1_msg_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_emit_l2_to_l1_msg_write_offset);
    commitments.kernel_emit_note_hash_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_emit_note_hash_write_offset);
    commitments.kernel_emit_nullifier_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_emit_nullifier_write_offset);
    commitments.kernel_emit_unencrypted_log_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.kernel_emit_unencrypted_log_write_offset);
    commitments.kernel_kernel_in_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_kernel_in_offset);
    commitments.kernel_kernel_out_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_kernel_out_offset);
    commitments.kernel_l1_to_l2_msg_exists_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_l1_to_l2_msg_exists_write_offset);
    commitments.kernel_note_hash_exist_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_note_hash_exist_write_offset);
    commitments.kernel_nullifier_exists_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_nullifier_exists_write_offset);
    commitments.kernel_nullifier_non_exists_write_offset = transcript->template receive_from_prover<Commitment>(
        commitment_labels.kernel_nullifier_non_exists_write_offset);
    commitments.kernel_q_public_input_kernel_add_to_table = transcript->template receive_from_prover<Commitment>(
        commitment_labels.kernel_q_public_input_kernel_add_to_table);
    commitments.kernel_q_public_input_kernel_out_add_to_table = transcript->template receive_from_prover<Commitment>(
        commitment_labels.kernel_q_public_input_kernel_out_add_to_table);
    commitments.kernel_side_effect_counter =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_side_effect_counter);
    commitments.kernel_sload_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_sload_write_offset);
    commitments.kernel_sstore_write_offset =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_sstore_write_offset);
    commitments.main_abs_da_rem_gas_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_abs_da_rem_gas_hi);
    commitments.main_abs_da_rem_gas_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_abs_da_rem_gas_lo);
    commitments.main_abs_l2_rem_gas_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_abs_l2_rem_gas_hi);
    commitments.main_abs_l2_rem_gas_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_abs_l2_rem_gas_lo);
    commitments.main_alu_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_alu_in_tag);
    commitments.main_bin_op_id = transcript->template receive_from_prover<Commitment>(commitment_labels.main_bin_op_id);
    commitments.main_call_ptr = transcript->template receive_from_prover<Commitment>(commitment_labels.main_call_ptr);
    commitments.main_da_gas_op_cost =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_da_gas_op_cost);
    commitments.main_da_gas_remaining =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_da_gas_remaining);
    commitments.main_da_out_of_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_da_out_of_gas);
    commitments.main_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.main_ia);
    commitments.main_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.main_ib);
    commitments.main_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.main_ic);
    commitments.main_id = transcript->template receive_from_prover<Commitment>(commitment_labels.main_id);
    commitments.main_id_zero = transcript->template receive_from_prover<Commitment>(commitment_labels.main_id_zero);
    commitments.main_ind_addr_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_ind_addr_a);
    commitments.main_ind_addr_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_ind_addr_b);
    commitments.main_ind_addr_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_ind_addr_c);
    commitments.main_ind_addr_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_ind_addr_d);
    commitments.main_internal_return_ptr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_internal_return_ptr);
    commitments.main_inv = transcript->template receive_from_prover<Commitment>(commitment_labels.main_inv);
    commitments.main_l2_gas_op_cost =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_l2_gas_op_cost);
    commitments.main_l2_gas_remaining =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_l2_gas_remaining);
    commitments.main_l2_out_of_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_l2_out_of_gas);
    commitments.main_mem_addr_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_mem_addr_a);
    commitments.main_mem_addr_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_mem_addr_b);
    commitments.main_mem_addr_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_mem_addr_c);
    commitments.main_mem_addr_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_mem_addr_d);
    commitments.main_op_err = transcript->template receive_from_prover<Commitment>(commitment_labels.main_op_err);
    commitments.main_opcode_val =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_opcode_val);
    commitments.main_pc = transcript->template receive_from_prover<Commitment>(commitment_labels.main_pc);
    commitments.main_r_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.main_r_in_tag);
    commitments.main_rwa = transcript->template receive_from_prover<Commitment>(commitment_labels.main_rwa);
    commitments.main_rwb = transcript->template receive_from_prover<Commitment>(commitment_labels.main_rwb);
    commitments.main_rwc = transcript->template receive_from_prover<Commitment>(commitment_labels.main_rwc);
    commitments.main_rwd = transcript->template receive_from_prover<Commitment>(commitment_labels.main_rwd);
    commitments.main_sel_alu = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_alu);
    commitments.main_sel_bin = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_bin);
    commitments.main_sel_gas_accounting_active =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_gas_accounting_active);
    commitments.main_sel_last = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_last);
    commitments.main_sel_mem_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mem_op_a);
    commitments.main_sel_mem_op_activate_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mem_op_activate_gas);
    commitments.main_sel_mem_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mem_op_b);
    commitments.main_sel_mem_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mem_op_c);
    commitments.main_sel_mem_op_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mem_op_d);
    commitments.main_sel_mov_ia_to_ic =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mov_ia_to_ic);
    commitments.main_sel_mov_ib_to_ic =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_mov_ib_to_ic);
    commitments.main_sel_op_add =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_add);
    commitments.main_sel_op_address =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_address);
    commitments.main_sel_op_and =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_and);
    commitments.main_sel_op_block_number =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_block_number);
    commitments.main_sel_op_cast =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_cast);
    commitments.main_sel_op_chain_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_chain_id);
    commitments.main_sel_op_cmov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_cmov);
    commitments.main_sel_op_coinbase =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_coinbase);
    commitments.main_sel_op_dagasleft =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_dagasleft);
    commitments.main_sel_op_div =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_div);
    commitments.main_sel_op_emit_l2_to_l1_msg =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_emit_l2_to_l1_msg);
    commitments.main_sel_op_emit_note_hash =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_emit_note_hash);
    commitments.main_sel_op_emit_nullifier =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_emit_nullifier);
    commitments.main_sel_op_emit_unencrypted_log =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_emit_unencrypted_log);
    commitments.main_sel_op_eq = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_eq);
    commitments.main_sel_op_external_call =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_external_call);
    commitments.main_sel_op_fdiv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_fdiv);
    commitments.main_sel_op_fee_per_da_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_fee_per_da_gas);
    commitments.main_sel_op_fee_per_l2_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_fee_per_l2_gas);
    commitments.main_sel_op_get_contract_instance =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_get_contract_instance);
    commitments.main_sel_op_halt =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_halt);
    commitments.main_sel_op_internal_call =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_internal_call);
    commitments.main_sel_op_internal_return =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_internal_return);
    commitments.main_sel_op_jump =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_jump);
    commitments.main_sel_op_jumpi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_jumpi);
    commitments.main_sel_op_keccak =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_keccak);
    commitments.main_sel_op_l1_to_l2_msg_exists =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_l1_to_l2_msg_exists);
    commitments.main_sel_op_l2gasleft =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_l2gasleft);
    commitments.main_sel_op_lt = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_lt);
    commitments.main_sel_op_lte =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_lte);
    commitments.main_sel_op_mov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_mov);
    commitments.main_sel_op_mul =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_mul);
    commitments.main_sel_op_not =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_not);
    commitments.main_sel_op_note_hash_exists =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_note_hash_exists);
    commitments.main_sel_op_nullifier_exists =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_nullifier_exists);
    commitments.main_sel_op_or = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_or);
    commitments.main_sel_op_pedersen =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_pedersen);
    commitments.main_sel_op_poseidon2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_poseidon2);
    commitments.main_sel_op_radix_le =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_radix_le);
    commitments.main_sel_op_sender =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_sender);
    commitments.main_sel_op_sha256 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_sha256);
    commitments.main_sel_op_shl =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_shl);
    commitments.main_sel_op_shr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_shr);
    commitments.main_sel_op_sload =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_sload);
    commitments.main_sel_op_sstore =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_sstore);
    commitments.main_sel_op_storage_address =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_storage_address);
    commitments.main_sel_op_sub =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_sub);
    commitments.main_sel_op_timestamp =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_timestamp);
    commitments.main_sel_op_transaction_fee =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_transaction_fee);
    commitments.main_sel_op_version =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_version);
    commitments.main_sel_op_xor =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_op_xor);
    commitments.main_sel_q_kernel_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_q_kernel_lookup);
    commitments.main_sel_q_kernel_output_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_q_kernel_output_lookup);
    commitments.main_sel_resolve_ind_addr_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_resolve_ind_addr_a);
    commitments.main_sel_resolve_ind_addr_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_resolve_ind_addr_b);
    commitments.main_sel_resolve_ind_addr_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_resolve_ind_addr_c);
    commitments.main_sel_resolve_ind_addr_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_resolve_ind_addr_d);
    commitments.main_sel_rng_16 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_rng_16);
    commitments.main_sel_rng_8 = transcript->template receive_from_prover<Commitment>(commitment_labels.main_sel_rng_8);
    commitments.main_space_id = transcript->template receive_from_prover<Commitment>(commitment_labels.main_space_id);
    commitments.main_tag_err = transcript->template receive_from_prover<Commitment>(commitment_labels.main_tag_err);
    commitments.main_w_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.main_w_in_tag);
    commitments.mem_addr = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_addr);
    commitments.mem_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_clk);
    commitments.mem_diff_hi = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_diff_hi);
    commitments.mem_diff_lo = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_diff_lo);
    commitments.mem_diff_mid = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_diff_mid);
    commitments.mem_glob_addr = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_glob_addr);
    commitments.mem_last = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_last);
    commitments.mem_lastAccess = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_lastAccess);
    commitments.mem_one_min_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_one_min_inv);
    commitments.mem_r_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_r_in_tag);
    commitments.mem_rw = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_rw);
    commitments.mem_sel_mem = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_mem);
    commitments.mem_sel_mov_ia_to_ic =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_mov_ia_to_ic);
    commitments.mem_sel_mov_ib_to_ic =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_mov_ib_to_ic);
    commitments.mem_sel_op_a = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_op_a);
    commitments.mem_sel_op_b = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_op_b);
    commitments.mem_sel_op_c = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_op_c);
    commitments.mem_sel_op_cmov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_op_cmov);
    commitments.mem_sel_op_d = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_op_d);
    commitments.mem_sel_resolve_ind_addr_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_resolve_ind_addr_a);
    commitments.mem_sel_resolve_ind_addr_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_resolve_ind_addr_b);
    commitments.mem_sel_resolve_ind_addr_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_resolve_ind_addr_c);
    commitments.mem_sel_resolve_ind_addr_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_resolve_ind_addr_d);
    commitments.mem_sel_rng_chk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_sel_rng_chk);
    commitments.mem_skip_check_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.mem_skip_check_tag);
    commitments.mem_space_id = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_space_id);
    commitments.mem_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_tag);
    commitments.mem_tag_err = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_tag_err);
    commitments.mem_tsp = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_tsp);
    commitments.mem_val = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_val);
    commitments.mem_w_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.mem_w_in_tag);
    commitments.pedersen_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.pedersen_clk);
    commitments.pedersen_input = transcript->template receive_from_prover<Commitment>(commitment_labels.pedersen_input);
    commitments.pedersen_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.pedersen_output);
    commitments.pedersen_sel_pedersen =
        transcript->template receive_from_prover<Commitment>(commitment_labels.pedersen_sel_pedersen);
    commitments.poseidon2_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.poseidon2_clk);
    commitments.poseidon2_input =
        transcript->template receive_from_prover<Commitment>(commitment_labels.poseidon2_input);
    commitments.poseidon2_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.poseidon2_output);
    commitments.poseidon2_sel_poseidon_perm =
        transcript->template receive_from_prover<Commitment>(commitment_labels.poseidon2_sel_poseidon_perm);
    commitments.powers_power_of_2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.powers_power_of_2);
    commitments.sha256_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.sha256_clk);
    commitments.sha256_input = transcript->template receive_from_prover<Commitment>(commitment_labels.sha256_input);
    commitments.sha256_output = transcript->template receive_from_prover<Commitment>(commitment_labels.sha256_output);
    commitments.sha256_sel_sha256_compression =
        transcript->template receive_from_prover<Commitment>(commitment_labels.sha256_sel_sha256_compression);
    commitments.sha256_state = transcript->template receive_from_prover<Commitment>(commitment_labels.sha256_state);
    commitments.lookup_byte_lengths_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths_counts);
    commitments.lookup_byte_operations_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations_counts);
    commitments.lookup_opcode_gas_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_opcode_gas_counts);
    commitments.range_check_l2_gas_hi_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_l2_gas_hi_counts);
    commitments.range_check_l2_gas_lo_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_l2_gas_lo_counts);
    commitments.range_check_da_gas_hi_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_da_gas_hi_counts);
    commitments.range_check_da_gas_lo_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_da_gas_lo_counts);
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
    commitments.perm_main_mem_ind_addr_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_a);
    commitments.perm_main_mem_ind_addr_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_b);
    commitments.perm_main_mem_ind_addr_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_c);
    commitments.perm_main_mem_ind_addr_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_d);
    commitments.lookup_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths);
    commitments.lookup_byte_operations =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations);
    commitments.lookup_opcode_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_opcode_gas);
    commitments.range_check_l2_gas_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_l2_gas_hi);
    commitments.range_check_l2_gas_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_l2_gas_lo);
    commitments.range_check_da_gas_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_da_gas_hi);
    commitments.range_check_da_gas_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_da_gas_lo);
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
        return false;
    }

    // Public columns evaluation checks

    FF kernel_kernel_inputs_evaluation =
        evaluate_public_input_column(public_inputs[0], circuit_size, multivariate_challenge);
    if (kernel_kernel_inputs_evaluation != claimed_evaluations.kernel_kernel_inputs) {
        return false;
    }

    FF kernel_kernel_value_out_evaluation =
        evaluate_public_input_column(public_inputs[1], circuit_size, multivariate_challenge);
    if (kernel_kernel_value_out_evaluation != claimed_evaluations.kernel_kernel_value_out) {
        return false;
    }

    FF kernel_kernel_side_effect_out_evaluation =
        evaluate_public_input_column(public_inputs[2], circuit_size, multivariate_challenge);
    if (kernel_kernel_side_effect_out_evaluation != claimed_evaluations.kernel_kernel_side_effect_out) {
        return false;
    }

    FF kernel_kernel_metadata_out_evaluation =
        evaluate_public_input_column(public_inputs[3], circuit_size, multivariate_challenge);
    if (kernel_kernel_metadata_out_evaluation != claimed_evaluations.kernel_kernel_metadata_out) {
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
