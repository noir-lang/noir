
#include "barretenberg/vm/generated/avm_circuit_builder.hpp"

namespace bb {
namespace {

template <typename FF> std::string field_to_string(const FF& ff)
{
    std::ostringstream os;
    os << ff;
    std::string raw = os.str();
    auto first_not_zero = raw.find_first_not_of('0', 2);
    std::string result = "0x" + (first_not_zero != std::string::npos ? raw.substr(first_not_zero) : "0");
    return result;
}

} // namespace

template <typename FF> std::vector<std::string> AvmFullRow<FF>::names()
{
    return { "avm_main_clk",
             "avm_main_first",
             "avm_alu_a_hi",
             "avm_alu_a_lo",
             "avm_alu_alu_sel",
             "avm_alu_b_hi",
             "avm_alu_b_lo",
             "avm_alu_borrow",
             "avm_alu_cf",
             "avm_alu_clk",
             "avm_alu_cmp_rng_ctr",
             "avm_alu_cmp_sel",
             "avm_alu_div_rng_chk_selector",
             "avm_alu_div_u16_r0",
             "avm_alu_div_u16_r1",
             "avm_alu_div_u16_r2",
             "avm_alu_div_u16_r3",
             "avm_alu_div_u16_r4",
             "avm_alu_div_u16_r5",
             "avm_alu_div_u16_r6",
             "avm_alu_div_u16_r7",
             "avm_alu_divisor_hi",
             "avm_alu_divisor_lo",
             "avm_alu_ff_tag",
             "avm_alu_ia",
             "avm_alu_ib",
             "avm_alu_ic",
             "avm_alu_in_tag",
             "avm_alu_op_add",
             "avm_alu_op_cast",
             "avm_alu_op_cast_prev",
             "avm_alu_op_div",
             "avm_alu_op_div_a_lt_b",
             "avm_alu_op_div_std",
             "avm_alu_op_eq",
             "avm_alu_op_eq_diff_inv",
             "avm_alu_op_lt",
             "avm_alu_op_lte",
             "avm_alu_op_mul",
             "avm_alu_op_not",
             "avm_alu_op_shl",
             "avm_alu_op_shr",
             "avm_alu_op_sub",
             "avm_alu_p_a_borrow",
             "avm_alu_p_b_borrow",
             "avm_alu_p_sub_a_hi",
             "avm_alu_p_sub_a_lo",
             "avm_alu_p_sub_b_hi",
             "avm_alu_p_sub_b_lo",
             "avm_alu_partial_prod_hi",
             "avm_alu_partial_prod_lo",
             "avm_alu_quotient_hi",
             "avm_alu_quotient_lo",
             "avm_alu_remainder",
             "avm_alu_res_hi",
             "avm_alu_res_lo",
             "avm_alu_rng_chk_lookup_selector",
             "avm_alu_rng_chk_sel",
             "avm_alu_shift_lt_bit_len",
             "avm_alu_shift_sel",
             "avm_alu_t_sub_s_bits",
             "avm_alu_two_pow_s",
             "avm_alu_two_pow_t_sub_s",
             "avm_alu_u128_tag",
             "avm_alu_u16_r0",
             "avm_alu_u16_r1",
             "avm_alu_u16_r10",
             "avm_alu_u16_r11",
             "avm_alu_u16_r12",
             "avm_alu_u16_r13",
             "avm_alu_u16_r14",
             "avm_alu_u16_r2",
             "avm_alu_u16_r3",
             "avm_alu_u16_r4",
             "avm_alu_u16_r5",
             "avm_alu_u16_r6",
             "avm_alu_u16_r7",
             "avm_alu_u16_r8",
             "avm_alu_u16_r9",
             "avm_alu_u16_tag",
             "avm_alu_u32_tag",
             "avm_alu_u64_tag",
             "avm_alu_u8_r0",
             "avm_alu_u8_r1",
             "avm_alu_u8_tag",
             "avm_binary_acc_ia",
             "avm_binary_acc_ib",
             "avm_binary_acc_ic",
             "avm_binary_bin_sel",
             "avm_binary_clk",
             "avm_binary_ia_bytes",
             "avm_binary_ib_bytes",
             "avm_binary_ic_bytes",
             "avm_binary_in_tag",
             "avm_binary_mem_tag_ctr",
             "avm_binary_mem_tag_ctr_inv",
             "avm_binary_op_id",
             "avm_binary_start",
             "avm_byte_lookup_bin_sel",
             "avm_byte_lookup_table_byte_lengths",
             "avm_byte_lookup_table_in_tags",
             "avm_byte_lookup_table_input_a",
             "avm_byte_lookup_table_input_b",
             "avm_byte_lookup_table_op_id",
             "avm_byte_lookup_table_output",
             "avm_conversion_clk",
             "avm_conversion_input",
             "avm_conversion_num_limbs",
             "avm_conversion_radix",
             "avm_conversion_to_radix_le_sel",
             "avm_gas_da_gas_fixed_table",
             "avm_gas_gas_cost_sel",
             "avm_gas_l2_gas_fixed_table",
             "avm_keccakf1600_clk",
             "avm_keccakf1600_input",
             "avm_keccakf1600_keccakf1600_sel",
             "avm_keccakf1600_output",
             "avm_kernel_emit_l2_to_l1_msg_write_offset",
             "avm_kernel_emit_note_hash_write_offset",
             "avm_kernel_emit_nullifier_write_offset",
             "avm_kernel_emit_unencrypted_log_write_offset",
             "avm_kernel_kernel_in_offset",
             "avm_kernel_kernel_inputs__is_public",
             "avm_kernel_kernel_metadata_out__is_public",
             "avm_kernel_kernel_out_offset",
             "avm_kernel_kernel_side_effect_out__is_public",
             "avm_kernel_kernel_value_out__is_public",
             "avm_kernel_l1_to_l2_msg_exists_write_offset",
             "avm_kernel_note_hash_exist_write_offset",
             "avm_kernel_nullifier_exists_write_offset",
             "avm_kernel_nullifier_non_exists_write_offset",
             "avm_kernel_q_public_input_kernel_add_to_table",
             "avm_kernel_q_public_input_kernel_out_add_to_table",
             "avm_kernel_side_effect_counter",
             "avm_kernel_sload_write_offset",
             "avm_kernel_sstore_write_offset",
             "avm_main_alu_in_tag",
             "avm_main_alu_sel",
             "avm_main_bin_op_id",
             "avm_main_bin_sel",
             "avm_main_call_ptr",
             "avm_main_da_gas_op",
             "avm_main_da_gas_remaining",
             "avm_main_gas_cost_active",
             "avm_main_ia",
             "avm_main_ib",
             "avm_main_ic",
             "avm_main_id",
             "avm_main_id_zero",
             "avm_main_ind_a",
             "avm_main_ind_b",
             "avm_main_ind_c",
             "avm_main_ind_d",
             "avm_main_ind_op_a",
             "avm_main_ind_op_b",
             "avm_main_ind_op_c",
             "avm_main_ind_op_d",
             "avm_main_internal_return_ptr",
             "avm_main_inv",
             "avm_main_l2_gas_op",
             "avm_main_l2_gas_remaining",
             "avm_main_last",
             "avm_main_mem_idx_a",
             "avm_main_mem_idx_b",
             "avm_main_mem_idx_c",
             "avm_main_mem_idx_d",
             "avm_main_mem_op_a",
             "avm_main_mem_op_activate_gas",
             "avm_main_mem_op_b",
             "avm_main_mem_op_c",
             "avm_main_mem_op_d",
             "avm_main_op_err",
             "avm_main_opcode_val",
             "avm_main_pc",
             "avm_main_q_kernel_lookup",
             "avm_main_q_kernel_output_lookup",
             "avm_main_r_in_tag",
             "avm_main_rwa",
             "avm_main_rwb",
             "avm_main_rwc",
             "avm_main_rwd",
             "avm_main_sel_cmov",
             "avm_main_sel_external_call",
             "avm_main_sel_halt",
             "avm_main_sel_internal_call",
             "avm_main_sel_internal_return",
             "avm_main_sel_jump",
             "avm_main_sel_jumpi",
             "avm_main_sel_mov",
             "avm_main_sel_mov_a",
             "avm_main_sel_mov_b",
             "avm_main_sel_op_add",
             "avm_main_sel_op_address",
             "avm_main_sel_op_and",
             "avm_main_sel_op_block_number",
             "avm_main_sel_op_cast",
             "avm_main_sel_op_chain_id",
             "avm_main_sel_op_coinbase",
             "avm_main_sel_op_dagasleft",
             "avm_main_sel_op_div",
             "avm_main_sel_op_emit_l2_to_l1_msg",
             "avm_main_sel_op_emit_note_hash",
             "avm_main_sel_op_emit_nullifier",
             "avm_main_sel_op_emit_unencrypted_log",
             "avm_main_sel_op_eq",
             "avm_main_sel_op_fdiv",
             "avm_main_sel_op_fee_per_da_gas",
             "avm_main_sel_op_fee_per_l2_gas",
             "avm_main_sel_op_get_contract_instance",
             "avm_main_sel_op_keccak",
             "avm_main_sel_op_l1_to_l2_msg_exists",
             "avm_main_sel_op_l2gasleft",
             "avm_main_sel_op_lt",
             "avm_main_sel_op_lte",
             "avm_main_sel_op_mul",
             "avm_main_sel_op_not",
             "avm_main_sel_op_note_hash_exists",
             "avm_main_sel_op_nullifier_exists",
             "avm_main_sel_op_or",
             "avm_main_sel_op_pedersen",
             "avm_main_sel_op_poseidon2",
             "avm_main_sel_op_radix_le",
             "avm_main_sel_op_sender",
             "avm_main_sel_op_sha256",
             "avm_main_sel_op_shl",
             "avm_main_sel_op_shr",
             "avm_main_sel_op_sload",
             "avm_main_sel_op_sstore",
             "avm_main_sel_op_storage_address",
             "avm_main_sel_op_sub",
             "avm_main_sel_op_timestamp",
             "avm_main_sel_op_transaction_fee",
             "avm_main_sel_op_version",
             "avm_main_sel_op_xor",
             "avm_main_sel_rng_16",
             "avm_main_sel_rng_8",
             "avm_main_space_id",
             "avm_main_table_pow_2",
             "avm_main_tag_err",
             "avm_main_w_in_tag",
             "avm_mem_addr",
             "avm_mem_clk",
             "avm_mem_diff_hi",
             "avm_mem_diff_lo",
             "avm_mem_diff_mid",
             "avm_mem_glob_addr",
             "avm_mem_ind_op_a",
             "avm_mem_ind_op_b",
             "avm_mem_ind_op_c",
             "avm_mem_ind_op_d",
             "avm_mem_last",
             "avm_mem_lastAccess",
             "avm_mem_mem_sel",
             "avm_mem_one_min_inv",
             "avm_mem_op_a",
             "avm_mem_op_b",
             "avm_mem_op_c",
             "avm_mem_op_d",
             "avm_mem_r_in_tag",
             "avm_mem_rng_chk_sel",
             "avm_mem_rw",
             "avm_mem_sel_cmov",
             "avm_mem_sel_mov_a",
             "avm_mem_sel_mov_b",
             "avm_mem_skip_check_tag",
             "avm_mem_space_id",
             "avm_mem_tag",
             "avm_mem_tag_err",
             "avm_mem_tsp",
             "avm_mem_val",
             "avm_mem_w_in_tag",
             "avm_pedersen_clk",
             "avm_pedersen_input",
             "avm_pedersen_output",
             "avm_pedersen_pedersen_sel",
             "avm_poseidon2_clk",
             "avm_poseidon2_input",
             "avm_poseidon2_output",
             "avm_poseidon2_poseidon_perm_sel",
             "avm_sha256_clk",
             "avm_sha256_input",
             "avm_sha256_output",
             "avm_sha256_sha256_compression_sel",
             "avm_sha256_state",
             "perm_main_alu",
             "perm_main_bin",
             "perm_main_conv",
             "perm_main_pos2_perm",
             "perm_main_pedersen",
             "perm_main_mem_a",
             "perm_main_mem_b",
             "perm_main_mem_c",
             "perm_main_mem_d",
             "perm_main_mem_ind_a",
             "perm_main_mem_ind_b",
             "perm_main_mem_ind_c",
             "perm_main_mem_ind_d",
             "lookup_byte_lengths",
             "lookup_byte_operations",
             "lookup_opcode_gas",
             "kernel_output_lookup",
             "lookup_into_kernel",
             "incl_main_tag_err",
             "incl_mem_tag_err",
             "lookup_mem_rng_chk_lo",
             "lookup_mem_rng_chk_mid",
             "lookup_mem_rng_chk_hi",
             "lookup_pow_2_0",
             "lookup_pow_2_1",
             "lookup_u8_0",
             "lookup_u8_1",
             "lookup_u16_0",
             "lookup_u16_1",
             "lookup_u16_2",
             "lookup_u16_3",
             "lookup_u16_4",
             "lookup_u16_5",
             "lookup_u16_6",
             "lookup_u16_7",
             "lookup_u16_8",
             "lookup_u16_9",
             "lookup_u16_10",
             "lookup_u16_11",
             "lookup_u16_12",
             "lookup_u16_13",
             "lookup_u16_14",
             "lookup_div_u16_0",
             "lookup_div_u16_1",
             "lookup_div_u16_2",
             "lookup_div_u16_3",
             "lookup_div_u16_4",
             "lookup_div_u16_5",
             "lookup_div_u16_6",
             "lookup_div_u16_7",
             "lookup_byte_lengths_counts",
             "lookup_byte_operations_counts",
             "lookup_opcode_gas_counts",
             "kernel_output_lookup_counts",
             "lookup_into_kernel_counts",
             "incl_main_tag_err_counts",
             "incl_mem_tag_err_counts",
             "lookup_mem_rng_chk_lo_counts",
             "lookup_mem_rng_chk_mid_counts",
             "lookup_mem_rng_chk_hi_counts",
             "lookup_pow_2_0_counts",
             "lookup_pow_2_1_counts",
             "lookup_u8_0_counts",
             "lookup_u8_1_counts",
             "lookup_u16_0_counts",
             "lookup_u16_1_counts",
             "lookup_u16_2_counts",
             "lookup_u16_3_counts",
             "lookup_u16_4_counts",
             "lookup_u16_5_counts",
             "lookup_u16_6_counts",
             "lookup_u16_7_counts",
             "lookup_u16_8_counts",
             "lookup_u16_9_counts",
             "lookup_u16_10_counts",
             "lookup_u16_11_counts",
             "lookup_u16_12_counts",
             "lookup_u16_13_counts",
             "lookup_u16_14_counts",
             "lookup_div_u16_0_counts",
             "lookup_div_u16_1_counts",
             "lookup_div_u16_2_counts",
             "lookup_div_u16_3_counts",
             "lookup_div_u16_4_counts",
             "lookup_div_u16_5_counts",
             "lookup_div_u16_6_counts",
             "lookup_div_u16_7_counts",
             "" };
}

template <typename FF> std::ostream& operator<<(std::ostream& os, AvmFullRow<FF> const& row)
{
    return os
           << field_to_string(row.avm_main_clk) << "," << field_to_string(row.avm_main_first) << ","
           << field_to_string(row.avm_alu_a_hi) << "," << field_to_string(row.avm_alu_a_lo) << ","
           << field_to_string(row.avm_alu_alu_sel) << "," << field_to_string(row.avm_alu_b_hi) << ","
           << field_to_string(row.avm_alu_b_lo) << "," << field_to_string(row.avm_alu_borrow) << ","
           << field_to_string(row.avm_alu_cf) << "," << field_to_string(row.avm_alu_clk) << ","
           << field_to_string(row.avm_alu_cmp_rng_ctr) << "," << field_to_string(row.avm_alu_cmp_sel) << ","
           << field_to_string(row.avm_alu_div_rng_chk_selector) << "," << field_to_string(row.avm_alu_div_u16_r0) << ","
           << field_to_string(row.avm_alu_div_u16_r1) << "," << field_to_string(row.avm_alu_div_u16_r2) << ","
           << field_to_string(row.avm_alu_div_u16_r3) << "," << field_to_string(row.avm_alu_div_u16_r4) << ","
           << field_to_string(row.avm_alu_div_u16_r5) << "," << field_to_string(row.avm_alu_div_u16_r6) << ","
           << field_to_string(row.avm_alu_div_u16_r7) << "," << field_to_string(row.avm_alu_divisor_hi) << ","
           << field_to_string(row.avm_alu_divisor_lo) << "," << field_to_string(row.avm_alu_ff_tag) << ","
           << field_to_string(row.avm_alu_ia) << "," << field_to_string(row.avm_alu_ib) << ","
           << field_to_string(row.avm_alu_ic) << "," << field_to_string(row.avm_alu_in_tag) << ","
           << field_to_string(row.avm_alu_op_add) << "," << field_to_string(row.avm_alu_op_cast) << ","
           << field_to_string(row.avm_alu_op_cast_prev) << "," << field_to_string(row.avm_alu_op_div) << ","
           << field_to_string(row.avm_alu_op_div_a_lt_b) << "," << field_to_string(row.avm_alu_op_div_std) << ","
           << field_to_string(row.avm_alu_op_eq) << "," << field_to_string(row.avm_alu_op_eq_diff_inv) << ","
           << field_to_string(row.avm_alu_op_lt) << "," << field_to_string(row.avm_alu_op_lte) << ","
           << field_to_string(row.avm_alu_op_mul) << "," << field_to_string(row.avm_alu_op_not) << ","
           << field_to_string(row.avm_alu_op_shl) << "," << field_to_string(row.avm_alu_op_shr) << ","
           << field_to_string(row.avm_alu_op_sub) << "," << field_to_string(row.avm_alu_p_a_borrow) << ","
           << field_to_string(row.avm_alu_p_b_borrow) << "," << field_to_string(row.avm_alu_p_sub_a_hi) << ","
           << field_to_string(row.avm_alu_p_sub_a_lo) << "," << field_to_string(row.avm_alu_p_sub_b_hi) << ","
           << field_to_string(row.avm_alu_p_sub_b_lo) << "," << field_to_string(row.avm_alu_partial_prod_hi) << ","
           << field_to_string(row.avm_alu_partial_prod_lo) << "," << field_to_string(row.avm_alu_quotient_hi) << ","
           << field_to_string(row.avm_alu_quotient_lo) << "," << field_to_string(row.avm_alu_remainder) << ","
           << field_to_string(row.avm_alu_res_hi) << "," << field_to_string(row.avm_alu_res_lo) << ","
           << field_to_string(row.avm_alu_rng_chk_lookup_selector) << "," << field_to_string(row.avm_alu_rng_chk_sel)
           << "," << field_to_string(row.avm_alu_shift_lt_bit_len) << "," << field_to_string(row.avm_alu_shift_sel)
           << "," << field_to_string(row.avm_alu_t_sub_s_bits) << "," << field_to_string(row.avm_alu_two_pow_s) << ","
           << field_to_string(row.avm_alu_two_pow_t_sub_s) << "," << field_to_string(row.avm_alu_u128_tag) << ","
           << field_to_string(row.avm_alu_u16_r0) << "," << field_to_string(row.avm_alu_u16_r1) << ","
           << field_to_string(row.avm_alu_u16_r10) << "," << field_to_string(row.avm_alu_u16_r11) << ","
           << field_to_string(row.avm_alu_u16_r12) << "," << field_to_string(row.avm_alu_u16_r13) << ","
           << field_to_string(row.avm_alu_u16_r14) << "," << field_to_string(row.avm_alu_u16_r2) << ","
           << field_to_string(row.avm_alu_u16_r3) << "," << field_to_string(row.avm_alu_u16_r4) << ","
           << field_to_string(row.avm_alu_u16_r5) << "," << field_to_string(row.avm_alu_u16_r6) << ","
           << field_to_string(row.avm_alu_u16_r7) << "," << field_to_string(row.avm_alu_u16_r8) << ","
           << field_to_string(row.avm_alu_u16_r9) << "," << field_to_string(row.avm_alu_u16_tag) << ","
           << field_to_string(row.avm_alu_u32_tag) << "," << field_to_string(row.avm_alu_u64_tag) << ","
           << field_to_string(row.avm_alu_u8_r0) << "," << field_to_string(row.avm_alu_u8_r1) << ","
           << field_to_string(row.avm_alu_u8_tag) << "," << field_to_string(row.avm_binary_acc_ia) << ","
           << field_to_string(row.avm_binary_acc_ib) << "," << field_to_string(row.avm_binary_acc_ic) << ","
           << field_to_string(row.avm_binary_bin_sel) << "," << field_to_string(row.avm_binary_clk) << ","
           << field_to_string(row.avm_binary_ia_bytes) << "," << field_to_string(row.avm_binary_ib_bytes) << ","
           << field_to_string(row.avm_binary_ic_bytes) << "," << field_to_string(row.avm_binary_in_tag) << ","
           << field_to_string(row.avm_binary_mem_tag_ctr) << "," << field_to_string(row.avm_binary_mem_tag_ctr_inv)
           << "," << field_to_string(row.avm_binary_op_id) << "," << field_to_string(row.avm_binary_start) << ","
           << field_to_string(row.avm_byte_lookup_bin_sel) << ","
           << field_to_string(row.avm_byte_lookup_table_byte_lengths) << ","
           << field_to_string(row.avm_byte_lookup_table_in_tags) << ","
           << field_to_string(row.avm_byte_lookup_table_input_a) << ","
           << field_to_string(row.avm_byte_lookup_table_input_b) << ","
           << field_to_string(row.avm_byte_lookup_table_op_id) << ","
           << field_to_string(row.avm_byte_lookup_table_output) << "," << field_to_string(row.avm_conversion_clk) << ","
           << field_to_string(row.avm_conversion_input) << "," << field_to_string(row.avm_conversion_num_limbs) << ","
           << field_to_string(row.avm_conversion_radix) << "," << field_to_string(row.avm_conversion_to_radix_le_sel)
           << "," << field_to_string(row.avm_gas_da_gas_fixed_table) << "," << field_to_string(row.avm_gas_gas_cost_sel)
           << "," << field_to_string(row.avm_gas_l2_gas_fixed_table) << "," << field_to_string(row.avm_keccakf1600_clk)
           << "," << field_to_string(row.avm_keccakf1600_input) << ","
           << field_to_string(row.avm_keccakf1600_keccakf1600_sel) << "," << field_to_string(row.avm_keccakf1600_output)
           << "," << field_to_string(row.avm_kernel_emit_l2_to_l1_msg_write_offset) << ","
           << field_to_string(row.avm_kernel_emit_note_hash_write_offset) << ","
           << field_to_string(row.avm_kernel_emit_nullifier_write_offset) << ","
           << field_to_string(row.avm_kernel_emit_unencrypted_log_write_offset) << ","
           << field_to_string(row.avm_kernel_kernel_in_offset) << ","
           << field_to_string(row.avm_kernel_kernel_inputs__is_public) << ","
           << field_to_string(row.avm_kernel_kernel_metadata_out__is_public) << ","
           << field_to_string(row.avm_kernel_kernel_out_offset) << ","
           << field_to_string(row.avm_kernel_kernel_side_effect_out__is_public) << ","
           << field_to_string(row.avm_kernel_kernel_value_out__is_public) << ","
           << field_to_string(row.avm_kernel_l1_to_l2_msg_exists_write_offset) << ","
           << field_to_string(row.avm_kernel_note_hash_exist_write_offset) << ","
           << field_to_string(row.avm_kernel_nullifier_exists_write_offset) << ","
           << field_to_string(row.avm_kernel_nullifier_non_exists_write_offset) << ","
           << field_to_string(row.avm_kernel_q_public_input_kernel_add_to_table) << ","
           << field_to_string(row.avm_kernel_q_public_input_kernel_out_add_to_table) << ","
           << field_to_string(row.avm_kernel_side_effect_counter) << ","
           << field_to_string(row.avm_kernel_sload_write_offset) << ","
           << field_to_string(row.avm_kernel_sstore_write_offset) << "," << field_to_string(row.avm_main_alu_in_tag)
           << "," << field_to_string(row.avm_main_alu_sel) << "," << field_to_string(row.avm_main_bin_op_id) << ","
           << field_to_string(row.avm_main_bin_sel) << "," << field_to_string(row.avm_main_call_ptr) << ","
           << field_to_string(row.avm_main_da_gas_op) << "," << field_to_string(row.avm_main_da_gas_remaining) << ","
           << field_to_string(row.avm_main_gas_cost_active) << "," << field_to_string(row.avm_main_ia) << ","
           << field_to_string(row.avm_main_ib) << "," << field_to_string(row.avm_main_ic) << ","
           << field_to_string(row.avm_main_id) << "," << field_to_string(row.avm_main_id_zero) << ","
           << field_to_string(row.avm_main_ind_a) << "," << field_to_string(row.avm_main_ind_b) << ","
           << field_to_string(row.avm_main_ind_c) << "," << field_to_string(row.avm_main_ind_d) << ","
           << field_to_string(row.avm_main_ind_op_a) << "," << field_to_string(row.avm_main_ind_op_b) << ","
           << field_to_string(row.avm_main_ind_op_c) << "," << field_to_string(row.avm_main_ind_op_d) << ","
           << field_to_string(row.avm_main_internal_return_ptr) << "," << field_to_string(row.avm_main_inv) << ","
           << field_to_string(row.avm_main_l2_gas_op) << "," << field_to_string(row.avm_main_l2_gas_remaining) << ","
           << field_to_string(row.avm_main_last) << "," << field_to_string(row.avm_main_mem_idx_a) << ","
           << field_to_string(row.avm_main_mem_idx_b) << "," << field_to_string(row.avm_main_mem_idx_c) << ","
           << field_to_string(row.avm_main_mem_idx_d) << "," << field_to_string(row.avm_main_mem_op_a) << ","
           << field_to_string(row.avm_main_mem_op_activate_gas) << "," << field_to_string(row.avm_main_mem_op_b) << ","
           << field_to_string(row.avm_main_mem_op_c) << "," << field_to_string(row.avm_main_mem_op_d) << ","
           << field_to_string(row.avm_main_op_err) << "," << field_to_string(row.avm_main_opcode_val) << ","
           << field_to_string(row.avm_main_pc) << "," << field_to_string(row.avm_main_q_kernel_lookup) << ","
           << field_to_string(row.avm_main_q_kernel_output_lookup) << "," << field_to_string(row.avm_main_r_in_tag)
           << "," << field_to_string(row.avm_main_rwa) << "," << field_to_string(row.avm_main_rwb) << ","
           << field_to_string(row.avm_main_rwc) << "," << field_to_string(row.avm_main_rwd) << ","
           << field_to_string(row.avm_main_sel_cmov) << "," << field_to_string(row.avm_main_sel_external_call) << ","
           << field_to_string(row.avm_main_sel_halt) << "," << field_to_string(row.avm_main_sel_internal_call) << ","
           << field_to_string(row.avm_main_sel_internal_return) << "," << field_to_string(row.avm_main_sel_jump) << ","
           << field_to_string(row.avm_main_sel_jumpi) << "," << field_to_string(row.avm_main_sel_mov) << ","
           << field_to_string(row.avm_main_sel_mov_a) << "," << field_to_string(row.avm_main_sel_mov_b) << ","
           << field_to_string(row.avm_main_sel_op_add) << "," << field_to_string(row.avm_main_sel_op_address) << ","
           << field_to_string(row.avm_main_sel_op_and) << "," << field_to_string(row.avm_main_sel_op_block_number)
           << "," << field_to_string(row.avm_main_sel_op_cast) << "," << field_to_string(row.avm_main_sel_op_chain_id)
           << "," << field_to_string(row.avm_main_sel_op_coinbase) << ","
           << field_to_string(row.avm_main_sel_op_dagasleft) << "," << field_to_string(row.avm_main_sel_op_div) << ","
           << field_to_string(row.avm_main_sel_op_emit_l2_to_l1_msg) << ","
           << field_to_string(row.avm_main_sel_op_emit_note_hash) << ","
           << field_to_string(row.avm_main_sel_op_emit_nullifier) << ","
           << field_to_string(row.avm_main_sel_op_emit_unencrypted_log) << ","
           << field_to_string(row.avm_main_sel_op_eq) << "," << field_to_string(row.avm_main_sel_op_fdiv) << ","
           << field_to_string(row.avm_main_sel_op_fee_per_da_gas) << ","
           << field_to_string(row.avm_main_sel_op_fee_per_l2_gas) << ","
           << field_to_string(row.avm_main_sel_op_get_contract_instance) << ","
           << field_to_string(row.avm_main_sel_op_keccak) << ","
           << field_to_string(row.avm_main_sel_op_l1_to_l2_msg_exists) << ","
           << field_to_string(row.avm_main_sel_op_l2gasleft) << "," << field_to_string(row.avm_main_sel_op_lt) << ","
           << field_to_string(row.avm_main_sel_op_lte) << "," << field_to_string(row.avm_main_sel_op_mul) << ","
           << field_to_string(row.avm_main_sel_op_not) << "," << field_to_string(row.avm_main_sel_op_note_hash_exists)
           << "," << field_to_string(row.avm_main_sel_op_nullifier_exists) << ","
           << field_to_string(row.avm_main_sel_op_or) << "," << field_to_string(row.avm_main_sel_op_pedersen) << ","
           << field_to_string(row.avm_main_sel_op_poseidon2) << "," << field_to_string(row.avm_main_sel_op_radix_le)
           << "," << field_to_string(row.avm_main_sel_op_sender) << "," << field_to_string(row.avm_main_sel_op_sha256)
           << "," << field_to_string(row.avm_main_sel_op_shl) << "," << field_to_string(row.avm_main_sel_op_shr) << ","
           << field_to_string(row.avm_main_sel_op_sload) << "," << field_to_string(row.avm_main_sel_op_sstore) << ","
           << field_to_string(row.avm_main_sel_op_storage_address) << "," << field_to_string(row.avm_main_sel_op_sub)
           << "," << field_to_string(row.avm_main_sel_op_timestamp) << ","
           << field_to_string(row.avm_main_sel_op_transaction_fee) << ","
           << field_to_string(row.avm_main_sel_op_version) << "," << field_to_string(row.avm_main_sel_op_xor) << ","
           << field_to_string(row.avm_main_sel_rng_16) << "," << field_to_string(row.avm_main_sel_rng_8) << ","
           << field_to_string(row.avm_main_space_id) << "," << field_to_string(row.avm_main_table_pow_2) << ","
           << field_to_string(row.avm_main_tag_err) << "," << field_to_string(row.avm_main_w_in_tag) << ","
           << field_to_string(row.avm_mem_addr) << "," << field_to_string(row.avm_mem_clk) << ","
           << field_to_string(row.avm_mem_diff_hi) << "," << field_to_string(row.avm_mem_diff_lo) << ","
           << field_to_string(row.avm_mem_diff_mid) << "," << field_to_string(row.avm_mem_glob_addr) << ","
           << field_to_string(row.avm_mem_ind_op_a) << "," << field_to_string(row.avm_mem_ind_op_b) << ","
           << field_to_string(row.avm_mem_ind_op_c) << "," << field_to_string(row.avm_mem_ind_op_d) << ","
           << field_to_string(row.avm_mem_last) << "," << field_to_string(row.avm_mem_lastAccess) << ","
           << field_to_string(row.avm_mem_mem_sel) << "," << field_to_string(row.avm_mem_one_min_inv) << ","
           << field_to_string(row.avm_mem_op_a) << "," << field_to_string(row.avm_mem_op_b) << ","
           << field_to_string(row.avm_mem_op_c) << "," << field_to_string(row.avm_mem_op_d) << ","
           << field_to_string(row.avm_mem_r_in_tag) << "," << field_to_string(row.avm_mem_rng_chk_sel) << ","
           << field_to_string(row.avm_mem_rw) << "," << field_to_string(row.avm_mem_sel_cmov) << ","
           << field_to_string(row.avm_mem_sel_mov_a) << "," << field_to_string(row.avm_mem_sel_mov_b) << ","
           << field_to_string(row.avm_mem_skip_check_tag) << "," << field_to_string(row.avm_mem_space_id) << ","
           << field_to_string(row.avm_mem_tag) << "," << field_to_string(row.avm_mem_tag_err) << ","
           << field_to_string(row.avm_mem_tsp) << "," << field_to_string(row.avm_mem_val) << ","
           << field_to_string(row.avm_mem_w_in_tag) << "," << field_to_string(row.avm_pedersen_clk) << ","
           << field_to_string(row.avm_pedersen_input) << "," << field_to_string(row.avm_pedersen_output) << ","
           << field_to_string(row.avm_pedersen_pedersen_sel) << "," << field_to_string(row.avm_poseidon2_clk) << ","
           << field_to_string(row.avm_poseidon2_input) << "," << field_to_string(row.avm_poseidon2_output) << ","
           << field_to_string(row.avm_poseidon2_poseidon_perm_sel) << "," << field_to_string(row.avm_sha256_clk) << ","
           << field_to_string(row.avm_sha256_input) << "," << field_to_string(row.avm_sha256_output) << ","
           << field_to_string(row.avm_sha256_sha256_compression_sel) << "," << field_to_string(row.avm_sha256_state)
           << "," << field_to_string(row.perm_main_alu) << "," << field_to_string(row.perm_main_bin) << ","
           << field_to_string(row.perm_main_conv) << "," << field_to_string(row.perm_main_pos2_perm) << ","
           << field_to_string(row.perm_main_pedersen) << "," << field_to_string(row.perm_main_mem_a) << ","
           << field_to_string(row.perm_main_mem_b) << "," << field_to_string(row.perm_main_mem_c) << ","
           << field_to_string(row.perm_main_mem_d) << "," << field_to_string(row.perm_main_mem_ind_a) << ","
           << field_to_string(row.perm_main_mem_ind_b) << "," << field_to_string(row.perm_main_mem_ind_c) << ","
           << field_to_string(row.perm_main_mem_ind_d) << "," << field_to_string(row.lookup_byte_lengths) << ","
           << field_to_string(row.lookup_byte_operations) << "," << field_to_string(row.lookup_opcode_gas) << ","
           << field_to_string(row.kernel_output_lookup) << "," << field_to_string(row.lookup_into_kernel) << ","
           << field_to_string(row.incl_main_tag_err) << "," << field_to_string(row.incl_mem_tag_err) << ","
           << field_to_string(row.lookup_mem_rng_chk_lo) << "," << field_to_string(row.lookup_mem_rng_chk_mid) << ","
           << field_to_string(row.lookup_mem_rng_chk_hi) << "," << field_to_string(row.lookup_pow_2_0) << ","
           << field_to_string(row.lookup_pow_2_1) << "," << field_to_string(row.lookup_u8_0) << ","
           << field_to_string(row.lookup_u8_1) << "," << field_to_string(row.lookup_u16_0) << ","
           << field_to_string(row.lookup_u16_1) << "," << field_to_string(row.lookup_u16_2) << ","
           << field_to_string(row.lookup_u16_3) << "," << field_to_string(row.lookup_u16_4) << ","
           << field_to_string(row.lookup_u16_5) << "," << field_to_string(row.lookup_u16_6) << ","
           << field_to_string(row.lookup_u16_7) << "," << field_to_string(row.lookup_u16_8) << ","
           << field_to_string(row.lookup_u16_9) << "," << field_to_string(row.lookup_u16_10) << ","
           << field_to_string(row.lookup_u16_11) << "," << field_to_string(row.lookup_u16_12) << ","
           << field_to_string(row.lookup_u16_13) << "," << field_to_string(row.lookup_u16_14) << ","
           << field_to_string(row.lookup_div_u16_0) << "," << field_to_string(row.lookup_div_u16_1) << ","
           << field_to_string(row.lookup_div_u16_2) << "," << field_to_string(row.lookup_div_u16_3) << ","
           << field_to_string(row.lookup_div_u16_4) << "," << field_to_string(row.lookup_div_u16_5) << ","
           << field_to_string(row.lookup_div_u16_6) << "," << field_to_string(row.lookup_div_u16_7) << ","
           << field_to_string(row.lookup_byte_lengths_counts) << ","
           << field_to_string(row.lookup_byte_operations_counts) << "," << field_to_string(row.lookup_opcode_gas_counts)
           << "," << field_to_string(row.kernel_output_lookup_counts) << ","
           << field_to_string(row.lookup_into_kernel_counts) << "," << field_to_string(row.incl_main_tag_err_counts)
           << "," << field_to_string(row.incl_mem_tag_err_counts) << ","
           << field_to_string(row.lookup_mem_rng_chk_lo_counts) << ","
           << field_to_string(row.lookup_mem_rng_chk_mid_counts) << ","
           << field_to_string(row.lookup_mem_rng_chk_hi_counts) << "," << field_to_string(row.lookup_pow_2_0_counts)
           << "," << field_to_string(row.lookup_pow_2_1_counts) << "," << field_to_string(row.lookup_u8_0_counts) << ","
           << field_to_string(row.lookup_u8_1_counts) << "," << field_to_string(row.lookup_u16_0_counts) << ","
           << field_to_string(row.lookup_u16_1_counts) << "," << field_to_string(row.lookup_u16_2_counts) << ","
           << field_to_string(row.lookup_u16_3_counts) << "," << field_to_string(row.lookup_u16_4_counts) << ","
           << field_to_string(row.lookup_u16_5_counts) << "," << field_to_string(row.lookup_u16_6_counts) << ","
           << field_to_string(row.lookup_u16_7_counts) << "," << field_to_string(row.lookup_u16_8_counts) << ","
           << field_to_string(row.lookup_u16_9_counts) << "," << field_to_string(row.lookup_u16_10_counts) << ","
           << field_to_string(row.lookup_u16_11_counts) << "," << field_to_string(row.lookup_u16_12_counts) << ","
           << field_to_string(row.lookup_u16_13_counts) << "," << field_to_string(row.lookup_u16_14_counts) << ","
           << field_to_string(row.lookup_div_u16_0_counts) << "," << field_to_string(row.lookup_div_u16_1_counts) << ","
           << field_to_string(row.lookup_div_u16_2_counts) << "," << field_to_string(row.lookup_div_u16_3_counts) << ","
           << field_to_string(row.lookup_div_u16_4_counts) << "," << field_to_string(row.lookup_div_u16_5_counts) << ","
           << field_to_string(row.lookup_div_u16_6_counts) << "," << field_to_string(row.lookup_div_u16_7_counts)
           << ","
              "";
}

// Explicit template instantiation.
template std::ostream& operator<<(std::ostream& os, AvmFullRow<bb::AvmFlavor::FF> const& row);
template std::vector<std::string> AvmFullRow<bb::AvmFlavor::FF>::names();

} // namespace bb