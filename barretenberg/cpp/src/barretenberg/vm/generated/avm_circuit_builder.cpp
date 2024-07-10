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
    return { "main_clk",
             "main_sel_first",
             "kernel_kernel_inputs",
             "kernel_kernel_value_out",
             "kernel_kernel_side_effect_out",
             "kernel_kernel_metadata_out",
             "main_calldata",
             "main_returndata",
             "alu_a_hi",
             "alu_a_lo",
             "alu_b_hi",
             "alu_b_lo",
             "alu_borrow",
             "alu_cf",
             "alu_clk",
             "alu_cmp_rng_ctr",
             "alu_div_u16_r0",
             "alu_div_u16_r1",
             "alu_div_u16_r2",
             "alu_div_u16_r3",
             "alu_div_u16_r4",
             "alu_div_u16_r5",
             "alu_div_u16_r6",
             "alu_div_u16_r7",
             "alu_divisor_hi",
             "alu_divisor_lo",
             "alu_ff_tag",
             "alu_ia",
             "alu_ib",
             "alu_ic",
             "alu_in_tag",
             "alu_op_add",
             "alu_op_cast",
             "alu_op_cast_prev",
             "alu_op_div",
             "alu_op_div_a_lt_b",
             "alu_op_div_std",
             "alu_op_eq",
             "alu_op_eq_diff_inv",
             "alu_op_lt",
             "alu_op_lte",
             "alu_op_mul",
             "alu_op_not",
             "alu_op_shl",
             "alu_op_shr",
             "alu_op_sub",
             "alu_p_a_borrow",
             "alu_p_b_borrow",
             "alu_p_sub_a_hi",
             "alu_p_sub_a_lo",
             "alu_p_sub_b_hi",
             "alu_p_sub_b_lo",
             "alu_partial_prod_hi",
             "alu_partial_prod_lo",
             "alu_quotient_hi",
             "alu_quotient_lo",
             "alu_remainder",
             "alu_res_hi",
             "alu_res_lo",
             "alu_sel_alu",
             "alu_sel_cmp",
             "alu_sel_div_rng_chk",
             "alu_sel_rng_chk",
             "alu_sel_rng_chk_lookup",
             "alu_sel_shift_which",
             "alu_shift_lt_bit_len",
             "alu_t_sub_s_bits",
             "alu_two_pow_s",
             "alu_two_pow_t_sub_s",
             "alu_u128_tag",
             "alu_u16_r0",
             "alu_u16_r1",
             "alu_u16_r10",
             "alu_u16_r11",
             "alu_u16_r12",
             "alu_u16_r13",
             "alu_u16_r14",
             "alu_u16_r2",
             "alu_u16_r3",
             "alu_u16_r4",
             "alu_u16_r5",
             "alu_u16_r6",
             "alu_u16_r7",
             "alu_u16_r8",
             "alu_u16_r9",
             "alu_u16_tag",
             "alu_u32_tag",
             "alu_u64_tag",
             "alu_u8_r0",
             "alu_u8_r1",
             "alu_u8_tag",
             "binary_acc_ia",
             "binary_acc_ib",
             "binary_acc_ic",
             "binary_clk",
             "binary_ia_bytes",
             "binary_ib_bytes",
             "binary_ic_bytes",
             "binary_in_tag",
             "binary_mem_tag_ctr",
             "binary_mem_tag_ctr_inv",
             "binary_op_id",
             "binary_sel_bin",
             "binary_start",
             "byte_lookup_sel_bin",
             "byte_lookup_table_byte_lengths",
             "byte_lookup_table_in_tags",
             "byte_lookup_table_input_a",
             "byte_lookup_table_input_b",
             "byte_lookup_table_op_id",
             "byte_lookup_table_output",
             "conversion_clk",
             "conversion_input",
             "conversion_num_limbs",
             "conversion_radix",
             "conversion_sel_to_radix_le",
             "gas_da_gas_fixed_table",
             "gas_l2_gas_fixed_table",
             "gas_sel_gas_cost",
             "keccakf1600_clk",
             "keccakf1600_input",
             "keccakf1600_output",
             "keccakf1600_sel_keccakf1600",
             "kernel_emit_l2_to_l1_msg_write_offset",
             "kernel_emit_note_hash_write_offset",
             "kernel_emit_nullifier_write_offset",
             "kernel_emit_unencrypted_log_write_offset",
             "kernel_kernel_in_offset",
             "kernel_kernel_out_offset",
             "kernel_l1_to_l2_msg_exists_write_offset",
             "kernel_note_hash_exist_write_offset",
             "kernel_nullifier_exists_write_offset",
             "kernel_nullifier_non_exists_write_offset",
             "kernel_q_public_input_kernel_add_to_table",
             "kernel_q_public_input_kernel_out_add_to_table",
             "kernel_side_effect_counter",
             "kernel_sload_write_offset",
             "kernel_sstore_write_offset",
             "main_abs_da_rem_gas_hi",
             "main_abs_da_rem_gas_lo",
             "main_abs_l2_rem_gas_hi",
             "main_abs_l2_rem_gas_lo",
             "main_alu_in_tag",
             "main_bin_op_id",
             "main_call_ptr",
             "main_da_gas_op_cost",
             "main_da_gas_remaining",
             "main_da_out_of_gas",
             "main_ia",
             "main_ib",
             "main_ic",
             "main_id",
             "main_id_zero",
             "main_ind_addr_a",
             "main_ind_addr_b",
             "main_ind_addr_c",
             "main_ind_addr_d",
             "main_internal_return_ptr",
             "main_inv",
             "main_l2_gas_op_cost",
             "main_l2_gas_remaining",
             "main_l2_out_of_gas",
             "main_mem_addr_a",
             "main_mem_addr_b",
             "main_mem_addr_c",
             "main_mem_addr_d",
             "main_op_err",
             "main_opcode_val",
             "main_pc",
             "main_r_in_tag",
             "main_rwa",
             "main_rwb",
             "main_rwc",
             "main_rwd",
             "main_sel_alu",
             "main_sel_bin",
             "main_sel_calldata",
             "main_sel_gas_accounting_active",
             "main_sel_last",
             "main_sel_mem_op_a",
             "main_sel_mem_op_activate_gas",
             "main_sel_mem_op_b",
             "main_sel_mem_op_c",
             "main_sel_mem_op_d",
             "main_sel_mov_ia_to_ic",
             "main_sel_mov_ib_to_ic",
             "main_sel_op_add",
             "main_sel_op_address",
             "main_sel_op_and",
             "main_sel_op_block_number",
             "main_sel_op_calldata_copy",
             "main_sel_op_cast",
             "main_sel_op_chain_id",
             "main_sel_op_cmov",
             "main_sel_op_coinbase",
             "main_sel_op_dagasleft",
             "main_sel_op_div",
             "main_sel_op_emit_l2_to_l1_msg",
             "main_sel_op_emit_note_hash",
             "main_sel_op_emit_nullifier",
             "main_sel_op_emit_unencrypted_log",
             "main_sel_op_eq",
             "main_sel_op_external_call",
             "main_sel_op_external_return",
             "main_sel_op_fdiv",
             "main_sel_op_fee_per_da_gas",
             "main_sel_op_fee_per_l2_gas",
             "main_sel_op_function_selector",
             "main_sel_op_get_contract_instance",
             "main_sel_op_halt",
             "main_sel_op_internal_call",
             "main_sel_op_internal_return",
             "main_sel_op_jump",
             "main_sel_op_jumpi",
             "main_sel_op_keccak",
             "main_sel_op_l1_to_l2_msg_exists",
             "main_sel_op_l2gasleft",
             "main_sel_op_lt",
             "main_sel_op_lte",
             "main_sel_op_mov",
             "main_sel_op_mul",
             "main_sel_op_not",
             "main_sel_op_note_hash_exists",
             "main_sel_op_nullifier_exists",
             "main_sel_op_or",
             "main_sel_op_pedersen",
             "main_sel_op_poseidon2",
             "main_sel_op_radix_le",
             "main_sel_op_sender",
             "main_sel_op_sha256",
             "main_sel_op_shl",
             "main_sel_op_shr",
             "main_sel_op_sload",
             "main_sel_op_sstore",
             "main_sel_op_storage_address",
             "main_sel_op_sub",
             "main_sel_op_timestamp",
             "main_sel_op_transaction_fee",
             "main_sel_op_version",
             "main_sel_op_xor",
             "main_sel_q_kernel_lookup",
             "main_sel_q_kernel_output_lookup",
             "main_sel_resolve_ind_addr_a",
             "main_sel_resolve_ind_addr_b",
             "main_sel_resolve_ind_addr_c",
             "main_sel_resolve_ind_addr_d",
             "main_sel_returndata",
             "main_sel_rng_16",
             "main_sel_rng_8",
             "main_sel_slice_gadget",
             "main_space_id",
             "main_tag_err",
             "main_w_in_tag",
             "mem_addr",
             "mem_clk",
             "mem_diff_hi",
             "mem_diff_lo",
             "mem_diff_mid",
             "mem_glob_addr",
             "mem_last",
             "mem_lastAccess",
             "mem_one_min_inv",
             "mem_r_in_tag",
             "mem_rw",
             "mem_sel_mem",
             "mem_sel_mov_ia_to_ic",
             "mem_sel_mov_ib_to_ic",
             "mem_sel_op_a",
             "mem_sel_op_b",
             "mem_sel_op_c",
             "mem_sel_op_cmov",
             "mem_sel_op_d",
             "mem_sel_op_slice",
             "mem_sel_resolve_ind_addr_a",
             "mem_sel_resolve_ind_addr_b",
             "mem_sel_resolve_ind_addr_c",
             "mem_sel_resolve_ind_addr_d",
             "mem_sel_rng_chk",
             "mem_skip_check_tag",
             "mem_space_id",
             "mem_tag",
             "mem_tag_err",
             "mem_tsp",
             "mem_val",
             "mem_w_in_tag",
             "pedersen_clk",
             "pedersen_input",
             "pedersen_output",
             "pedersen_sel_pedersen",
             "poseidon2_clk",
             "poseidon2_input",
             "poseidon2_output",
             "poseidon2_sel_poseidon_perm",
             "powers_power_of_2",
             "sha256_clk",
             "sha256_input",
             "sha256_output",
             "sha256_sel_sha256_compression",
             "sha256_state",
             "slice_addr",
             "slice_clk",
             "slice_cnt",
             "slice_col_offset",
             "slice_one_min_inv",
             "slice_sel_cd_cpy",
             "slice_sel_mem_active",
             "slice_sel_return",
             "slice_sel_start",
             "slice_space_id",
             "slice_val",
             "perm_slice_mem",
             "perm_main_alu",
             "perm_main_bin",
             "perm_main_conv",
             "perm_main_pos2_perm",
             "perm_main_pedersen",
             "perm_main_slice",
             "perm_main_mem_a",
             "perm_main_mem_b",
             "perm_main_mem_c",
             "perm_main_mem_d",
             "perm_main_mem_ind_addr_a",
             "perm_main_mem_ind_addr_b",
             "perm_main_mem_ind_addr_c",
             "perm_main_mem_ind_addr_d",
             "lookup_byte_lengths",
             "lookup_byte_operations",
             "lookup_cd_value",
             "lookup_ret_value",
             "lookup_opcode_gas",
             "range_check_l2_gas_hi",
             "range_check_l2_gas_lo",
             "range_check_da_gas_hi",
             "range_check_da_gas_lo",
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
             "lookup_cd_value_counts",
             "lookup_ret_value_counts",
             "lookup_opcode_gas_counts",
             "range_check_l2_gas_hi_counts",
             "range_check_l2_gas_lo_counts",
             "range_check_da_gas_hi_counts",
             "range_check_da_gas_lo_counts",
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
             "lookup_div_u16_7_counts" };
}

template <typename FF> std::ostream& operator<<(std::ostream& os, AvmFullRow<FF> const& row)
{
    return os << field_to_string(row.main_clk)                                             //
              << "," << field_to_string(row.main_sel_first)                                //
              << "," << field_to_string(row.kernel_kernel_inputs)                          //
              << "," << field_to_string(row.kernel_kernel_value_out)                       //
              << "," << field_to_string(row.kernel_kernel_side_effect_out)                 //
              << "," << field_to_string(row.kernel_kernel_metadata_out)                    //
              << "," << field_to_string(row.main_calldata)                                 //
              << "," << field_to_string(row.main_returndata)                               //
              << "," << field_to_string(row.alu_a_hi)                                      //
              << "," << field_to_string(row.alu_a_lo)                                      //
              << "," << field_to_string(row.alu_b_hi)                                      //
              << "," << field_to_string(row.alu_b_lo)                                      //
              << "," << field_to_string(row.alu_borrow)                                    //
              << "," << field_to_string(row.alu_cf)                                        //
              << "," << field_to_string(row.alu_clk)                                       //
              << "," << field_to_string(row.alu_cmp_rng_ctr)                               //
              << "," << field_to_string(row.alu_div_u16_r0)                                //
              << "," << field_to_string(row.alu_div_u16_r1)                                //
              << "," << field_to_string(row.alu_div_u16_r2)                                //
              << "," << field_to_string(row.alu_div_u16_r3)                                //
              << "," << field_to_string(row.alu_div_u16_r4)                                //
              << "," << field_to_string(row.alu_div_u16_r5)                                //
              << "," << field_to_string(row.alu_div_u16_r6)                                //
              << "," << field_to_string(row.alu_div_u16_r7)                                //
              << "," << field_to_string(row.alu_divisor_hi)                                //
              << "," << field_to_string(row.alu_divisor_lo)                                //
              << "," << field_to_string(row.alu_ff_tag)                                    //
              << "," << field_to_string(row.alu_ia)                                        //
              << "," << field_to_string(row.alu_ib)                                        //
              << "," << field_to_string(row.alu_ic)                                        //
              << "," << field_to_string(row.alu_in_tag)                                    //
              << "," << field_to_string(row.alu_op_add)                                    //
              << "," << field_to_string(row.alu_op_cast)                                   //
              << "," << field_to_string(row.alu_op_cast_prev)                              //
              << "," << field_to_string(row.alu_op_div)                                    //
              << "," << field_to_string(row.alu_op_div_a_lt_b)                             //
              << "," << field_to_string(row.alu_op_div_std)                                //
              << "," << field_to_string(row.alu_op_eq)                                     //
              << "," << field_to_string(row.alu_op_eq_diff_inv)                            //
              << "," << field_to_string(row.alu_op_lt)                                     //
              << "," << field_to_string(row.alu_op_lte)                                    //
              << "," << field_to_string(row.alu_op_mul)                                    //
              << "," << field_to_string(row.alu_op_not)                                    //
              << "," << field_to_string(row.alu_op_shl)                                    //
              << "," << field_to_string(row.alu_op_shr)                                    //
              << "," << field_to_string(row.alu_op_sub)                                    //
              << "," << field_to_string(row.alu_p_a_borrow)                                //
              << "," << field_to_string(row.alu_p_b_borrow)                                //
              << "," << field_to_string(row.alu_p_sub_a_hi)                                //
              << "," << field_to_string(row.alu_p_sub_a_lo)                                //
              << "," << field_to_string(row.alu_p_sub_b_hi)                                //
              << "," << field_to_string(row.alu_p_sub_b_lo)                                //
              << "," << field_to_string(row.alu_partial_prod_hi)                           //
              << "," << field_to_string(row.alu_partial_prod_lo)                           //
              << "," << field_to_string(row.alu_quotient_hi)                               //
              << "," << field_to_string(row.alu_quotient_lo)                               //
              << "," << field_to_string(row.alu_remainder)                                 //
              << "," << field_to_string(row.alu_res_hi)                                    //
              << "," << field_to_string(row.alu_res_lo)                                    //
              << "," << field_to_string(row.alu_sel_alu)                                   //
              << "," << field_to_string(row.alu_sel_cmp)                                   //
              << "," << field_to_string(row.alu_sel_div_rng_chk)                           //
              << "," << field_to_string(row.alu_sel_rng_chk)                               //
              << "," << field_to_string(row.alu_sel_rng_chk_lookup)                        //
              << "," << field_to_string(row.alu_sel_shift_which)                           //
              << "," << field_to_string(row.alu_shift_lt_bit_len)                          //
              << "," << field_to_string(row.alu_t_sub_s_bits)                              //
              << "," << field_to_string(row.alu_two_pow_s)                                 //
              << "," << field_to_string(row.alu_two_pow_t_sub_s)                           //
              << "," << field_to_string(row.alu_u128_tag)                                  //
              << "," << field_to_string(row.alu_u16_r0)                                    //
              << "," << field_to_string(row.alu_u16_r1)                                    //
              << "," << field_to_string(row.alu_u16_r10)                                   //
              << "," << field_to_string(row.alu_u16_r11)                                   //
              << "," << field_to_string(row.alu_u16_r12)                                   //
              << "," << field_to_string(row.alu_u16_r13)                                   //
              << "," << field_to_string(row.alu_u16_r14)                                   //
              << "," << field_to_string(row.alu_u16_r2)                                    //
              << "," << field_to_string(row.alu_u16_r3)                                    //
              << "," << field_to_string(row.alu_u16_r4)                                    //
              << "," << field_to_string(row.alu_u16_r5)                                    //
              << "," << field_to_string(row.alu_u16_r6)                                    //
              << "," << field_to_string(row.alu_u16_r7)                                    //
              << "," << field_to_string(row.alu_u16_r8)                                    //
              << "," << field_to_string(row.alu_u16_r9)                                    //
              << "," << field_to_string(row.alu_u16_tag)                                   //
              << "," << field_to_string(row.alu_u32_tag)                                   //
              << "," << field_to_string(row.alu_u64_tag)                                   //
              << "," << field_to_string(row.alu_u8_r0)                                     //
              << "," << field_to_string(row.alu_u8_r1)                                     //
              << "," << field_to_string(row.alu_u8_tag)                                    //
              << "," << field_to_string(row.binary_acc_ia)                                 //
              << "," << field_to_string(row.binary_acc_ib)                                 //
              << "," << field_to_string(row.binary_acc_ic)                                 //
              << "," << field_to_string(row.binary_clk)                                    //
              << "," << field_to_string(row.binary_ia_bytes)                               //
              << "," << field_to_string(row.binary_ib_bytes)                               //
              << "," << field_to_string(row.binary_ic_bytes)                               //
              << "," << field_to_string(row.binary_in_tag)                                 //
              << "," << field_to_string(row.binary_mem_tag_ctr)                            //
              << "," << field_to_string(row.binary_mem_tag_ctr_inv)                        //
              << "," << field_to_string(row.binary_op_id)                                  //
              << "," << field_to_string(row.binary_sel_bin)                                //
              << "," << field_to_string(row.binary_start)                                  //
              << "," << field_to_string(row.byte_lookup_sel_bin)                           //
              << "," << field_to_string(row.byte_lookup_table_byte_lengths)                //
              << "," << field_to_string(row.byte_lookup_table_in_tags)                     //
              << "," << field_to_string(row.byte_lookup_table_input_a)                     //
              << "," << field_to_string(row.byte_lookup_table_input_b)                     //
              << "," << field_to_string(row.byte_lookup_table_op_id)                       //
              << "," << field_to_string(row.byte_lookup_table_output)                      //
              << "," << field_to_string(row.conversion_clk)                                //
              << "," << field_to_string(row.conversion_input)                              //
              << "," << field_to_string(row.conversion_num_limbs)                          //
              << "," << field_to_string(row.conversion_radix)                              //
              << "," << field_to_string(row.conversion_sel_to_radix_le)                    //
              << "," << field_to_string(row.gas_da_gas_fixed_table)                        //
              << "," << field_to_string(row.gas_l2_gas_fixed_table)                        //
              << "," << field_to_string(row.gas_sel_gas_cost)                              //
              << "," << field_to_string(row.keccakf1600_clk)                               //
              << "," << field_to_string(row.keccakf1600_input)                             //
              << "," << field_to_string(row.keccakf1600_output)                            //
              << "," << field_to_string(row.keccakf1600_sel_keccakf1600)                   //
              << "," << field_to_string(row.kernel_emit_l2_to_l1_msg_write_offset)         //
              << "," << field_to_string(row.kernel_emit_note_hash_write_offset)            //
              << "," << field_to_string(row.kernel_emit_nullifier_write_offset)            //
              << "," << field_to_string(row.kernel_emit_unencrypted_log_write_offset)      //
              << "," << field_to_string(row.kernel_kernel_in_offset)                       //
              << "," << field_to_string(row.kernel_kernel_out_offset)                      //
              << "," << field_to_string(row.kernel_l1_to_l2_msg_exists_write_offset)       //
              << "," << field_to_string(row.kernel_note_hash_exist_write_offset)           //
              << "," << field_to_string(row.kernel_nullifier_exists_write_offset)          //
              << "," << field_to_string(row.kernel_nullifier_non_exists_write_offset)      //
              << "," << field_to_string(row.kernel_q_public_input_kernel_add_to_table)     //
              << "," << field_to_string(row.kernel_q_public_input_kernel_out_add_to_table) //
              << "," << field_to_string(row.kernel_side_effect_counter)                    //
              << "," << field_to_string(row.kernel_sload_write_offset)                     //
              << "," << field_to_string(row.kernel_sstore_write_offset)                    //
              << "," << field_to_string(row.main_abs_da_rem_gas_hi)                        //
              << "," << field_to_string(row.main_abs_da_rem_gas_lo)                        //
              << "," << field_to_string(row.main_abs_l2_rem_gas_hi)                        //
              << "," << field_to_string(row.main_abs_l2_rem_gas_lo)                        //
              << "," << field_to_string(row.main_alu_in_tag)                               //
              << "," << field_to_string(row.main_bin_op_id)                                //
              << "," << field_to_string(row.main_call_ptr)                                 //
              << "," << field_to_string(row.main_da_gas_op_cost)                           //
              << "," << field_to_string(row.main_da_gas_remaining)                         //
              << "," << field_to_string(row.main_da_out_of_gas)                            //
              << "," << field_to_string(row.main_ia)                                       //
              << "," << field_to_string(row.main_ib)                                       //
              << "," << field_to_string(row.main_ic)                                       //
              << "," << field_to_string(row.main_id)                                       //
              << "," << field_to_string(row.main_id_zero)                                  //
              << "," << field_to_string(row.main_ind_addr_a)                               //
              << "," << field_to_string(row.main_ind_addr_b)                               //
              << "," << field_to_string(row.main_ind_addr_c)                               //
              << "," << field_to_string(row.main_ind_addr_d)                               //
              << "," << field_to_string(row.main_internal_return_ptr)                      //
              << "," << field_to_string(row.main_inv)                                      //
              << "," << field_to_string(row.main_l2_gas_op_cost)                           //
              << "," << field_to_string(row.main_l2_gas_remaining)                         //
              << "," << field_to_string(row.main_l2_out_of_gas)                            //
              << "," << field_to_string(row.main_mem_addr_a)                               //
              << "," << field_to_string(row.main_mem_addr_b)                               //
              << "," << field_to_string(row.main_mem_addr_c)                               //
              << "," << field_to_string(row.main_mem_addr_d)                               //
              << "," << field_to_string(row.main_op_err)                                   //
              << "," << field_to_string(row.main_opcode_val)                               //
              << "," << field_to_string(row.main_pc)                                       //
              << "," << field_to_string(row.main_r_in_tag)                                 //
              << "," << field_to_string(row.main_rwa)                                      //
              << "," << field_to_string(row.main_rwb)                                      //
              << "," << field_to_string(row.main_rwc)                                      //
              << "," << field_to_string(row.main_rwd)                                      //
              << "," << field_to_string(row.main_sel_alu)                                  //
              << "," << field_to_string(row.main_sel_bin)                                  //
              << "," << field_to_string(row.main_sel_calldata)                             //
              << "," << field_to_string(row.main_sel_gas_accounting_active)                //
              << "," << field_to_string(row.main_sel_last)                                 //
              << "," << field_to_string(row.main_sel_mem_op_a)                             //
              << "," << field_to_string(row.main_sel_mem_op_activate_gas)                  //
              << "," << field_to_string(row.main_sel_mem_op_b)                             //
              << "," << field_to_string(row.main_sel_mem_op_c)                             //
              << "," << field_to_string(row.main_sel_mem_op_d)                             //
              << "," << field_to_string(row.main_sel_mov_ia_to_ic)                         //
              << "," << field_to_string(row.main_sel_mov_ib_to_ic)                         //
              << "," << field_to_string(row.main_sel_op_add)                               //
              << "," << field_to_string(row.main_sel_op_address)                           //
              << "," << field_to_string(row.main_sel_op_and)                               //
              << "," << field_to_string(row.main_sel_op_block_number)                      //
              << "," << field_to_string(row.main_sel_op_calldata_copy)                     //
              << "," << field_to_string(row.main_sel_op_cast)                              //
              << "," << field_to_string(row.main_sel_op_chain_id)                          //
              << "," << field_to_string(row.main_sel_op_cmov)                              //
              << "," << field_to_string(row.main_sel_op_coinbase)                          //
              << "," << field_to_string(row.main_sel_op_dagasleft)                         //
              << "," << field_to_string(row.main_sel_op_div)                               //
              << "," << field_to_string(row.main_sel_op_emit_l2_to_l1_msg)                 //
              << "," << field_to_string(row.main_sel_op_emit_note_hash)                    //
              << "," << field_to_string(row.main_sel_op_emit_nullifier)                    //
              << "," << field_to_string(row.main_sel_op_emit_unencrypted_log)              //
              << "," << field_to_string(row.main_sel_op_eq)                                //
              << "," << field_to_string(row.main_sel_op_external_call)                     //
              << "," << field_to_string(row.main_sel_op_external_return)                   //
              << "," << field_to_string(row.main_sel_op_fdiv)                              //
              << "," << field_to_string(row.main_sel_op_fee_per_da_gas)                    //
              << "," << field_to_string(row.main_sel_op_fee_per_l2_gas)                    //
              << "," << field_to_string(row.main_sel_op_function_selector)                 //
              << "," << field_to_string(row.main_sel_op_get_contract_instance)             //
              << "," << field_to_string(row.main_sel_op_halt)                              //
              << "," << field_to_string(row.main_sel_op_internal_call)                     //
              << "," << field_to_string(row.main_sel_op_internal_return)                   //
              << "," << field_to_string(row.main_sel_op_jump)                              //
              << "," << field_to_string(row.main_sel_op_jumpi)                             //
              << "," << field_to_string(row.main_sel_op_keccak)                            //
              << "," << field_to_string(row.main_sel_op_l1_to_l2_msg_exists)               //
              << "," << field_to_string(row.main_sel_op_l2gasleft)                         //
              << "," << field_to_string(row.main_sel_op_lt)                                //
              << "," << field_to_string(row.main_sel_op_lte)                               //
              << "," << field_to_string(row.main_sel_op_mov)                               //
              << "," << field_to_string(row.main_sel_op_mul)                               //
              << "," << field_to_string(row.main_sel_op_not)                               //
              << "," << field_to_string(row.main_sel_op_note_hash_exists)                  //
              << "," << field_to_string(row.main_sel_op_nullifier_exists)                  //
              << "," << field_to_string(row.main_sel_op_or)                                //
              << "," << field_to_string(row.main_sel_op_pedersen)                          //
              << "," << field_to_string(row.main_sel_op_poseidon2)                         //
              << "," << field_to_string(row.main_sel_op_radix_le)                          //
              << "," << field_to_string(row.main_sel_op_sender)                            //
              << "," << field_to_string(row.main_sel_op_sha256)                            //
              << "," << field_to_string(row.main_sel_op_shl)                               //
              << "," << field_to_string(row.main_sel_op_shr)                               //
              << "," << field_to_string(row.main_sel_op_sload)                             //
              << "," << field_to_string(row.main_sel_op_sstore)                            //
              << "," << field_to_string(row.main_sel_op_storage_address)                   //
              << "," << field_to_string(row.main_sel_op_sub)                               //
              << "," << field_to_string(row.main_sel_op_timestamp)                         //
              << "," << field_to_string(row.main_sel_op_transaction_fee)                   //
              << "," << field_to_string(row.main_sel_op_version)                           //
              << "," << field_to_string(row.main_sel_op_xor)                               //
              << "," << field_to_string(row.main_sel_q_kernel_lookup)                      //
              << "," << field_to_string(row.main_sel_q_kernel_output_lookup)               //
              << "," << field_to_string(row.main_sel_resolve_ind_addr_a)                   //
              << "," << field_to_string(row.main_sel_resolve_ind_addr_b)                   //
              << "," << field_to_string(row.main_sel_resolve_ind_addr_c)                   //
              << "," << field_to_string(row.main_sel_resolve_ind_addr_d)                   //
              << "," << field_to_string(row.main_sel_returndata)                           //
              << "," << field_to_string(row.main_sel_rng_16)                               //
              << "," << field_to_string(row.main_sel_rng_8)                                //
              << "," << field_to_string(row.main_sel_slice_gadget)                         //
              << "," << field_to_string(row.main_space_id)                                 //
              << "," << field_to_string(row.main_tag_err)                                  //
              << "," << field_to_string(row.main_w_in_tag)                                 //
              << "," << field_to_string(row.mem_addr)                                      //
              << "," << field_to_string(row.mem_clk)                                       //
              << "," << field_to_string(row.mem_diff_hi)                                   //
              << "," << field_to_string(row.mem_diff_lo)                                   //
              << "," << field_to_string(row.mem_diff_mid)                                  //
              << "," << field_to_string(row.mem_glob_addr)                                 //
              << "," << field_to_string(row.mem_last)                                      //
              << "," << field_to_string(row.mem_lastAccess)                                //
              << "," << field_to_string(row.mem_one_min_inv)                               //
              << "," << field_to_string(row.mem_r_in_tag)                                  //
              << "," << field_to_string(row.mem_rw)                                        //
              << "," << field_to_string(row.mem_sel_mem)                                   //
              << "," << field_to_string(row.mem_sel_mov_ia_to_ic)                          //
              << "," << field_to_string(row.mem_sel_mov_ib_to_ic)                          //
              << "," << field_to_string(row.mem_sel_op_a)                                  //
              << "," << field_to_string(row.mem_sel_op_b)                                  //
              << "," << field_to_string(row.mem_sel_op_c)                                  //
              << "," << field_to_string(row.mem_sel_op_cmov)                               //
              << "," << field_to_string(row.mem_sel_op_d)                                  //
              << "," << field_to_string(row.mem_sel_op_slice)                              //
              << "," << field_to_string(row.mem_sel_resolve_ind_addr_a)                    //
              << "," << field_to_string(row.mem_sel_resolve_ind_addr_b)                    //
              << "," << field_to_string(row.mem_sel_resolve_ind_addr_c)                    //
              << "," << field_to_string(row.mem_sel_resolve_ind_addr_d)                    //
              << "," << field_to_string(row.mem_sel_rng_chk)                               //
              << "," << field_to_string(row.mem_skip_check_tag)                            //
              << "," << field_to_string(row.mem_space_id)                                  //
              << "," << field_to_string(row.mem_tag)                                       //
              << "," << field_to_string(row.mem_tag_err)                                   //
              << "," << field_to_string(row.mem_tsp)                                       //
              << "," << field_to_string(row.mem_val)                                       //
              << "," << field_to_string(row.mem_w_in_tag)                                  //
              << "," << field_to_string(row.pedersen_clk)                                  //
              << "," << field_to_string(row.pedersen_input)                                //
              << "," << field_to_string(row.pedersen_output)                               //
              << "," << field_to_string(row.pedersen_sel_pedersen)                         //
              << "," << field_to_string(row.poseidon2_clk)                                 //
              << "," << field_to_string(row.poseidon2_input)                               //
              << "," << field_to_string(row.poseidon2_output)                              //
              << "," << field_to_string(row.poseidon2_sel_poseidon_perm)                   //
              << "," << field_to_string(row.powers_power_of_2)                             //
              << "," << field_to_string(row.sha256_clk)                                    //
              << "," << field_to_string(row.sha256_input)                                  //
              << "," << field_to_string(row.sha256_output)                                 //
              << "," << field_to_string(row.sha256_sel_sha256_compression)                 //
              << "," << field_to_string(row.sha256_state)                                  //
              << "," << field_to_string(row.slice_addr)                                    //
              << "," << field_to_string(row.slice_clk)                                     //
              << "," << field_to_string(row.slice_cnt)                                     //
              << "," << field_to_string(row.slice_col_offset)                              //
              << "," << field_to_string(row.slice_one_min_inv)                             //
              << "," << field_to_string(row.slice_sel_cd_cpy)                              //
              << "," << field_to_string(row.slice_sel_mem_active)                          //
              << "," << field_to_string(row.slice_sel_return)                              //
              << "," << field_to_string(row.slice_sel_start)                               //
              << "," << field_to_string(row.slice_space_id)                                //
              << "," << field_to_string(row.slice_val)                                     //
              << "," << field_to_string(row.perm_slice_mem)                                //
              << "," << field_to_string(row.perm_main_alu)                                 //
              << "," << field_to_string(row.perm_main_bin)                                 //
              << "," << field_to_string(row.perm_main_conv)                                //
              << "," << field_to_string(row.perm_main_pos2_perm)                           //
              << "," << field_to_string(row.perm_main_pedersen)                            //
              << "," << field_to_string(row.perm_main_slice)                               //
              << "," << field_to_string(row.perm_main_mem_a)                               //
              << "," << field_to_string(row.perm_main_mem_b)                               //
              << "," << field_to_string(row.perm_main_mem_c)                               //
              << "," << field_to_string(row.perm_main_mem_d)                               //
              << "," << field_to_string(row.perm_main_mem_ind_addr_a)                      //
              << "," << field_to_string(row.perm_main_mem_ind_addr_b)                      //
              << "," << field_to_string(row.perm_main_mem_ind_addr_c)                      //
              << "," << field_to_string(row.perm_main_mem_ind_addr_d)                      //
              << "," << field_to_string(row.lookup_byte_lengths)                           //
              << "," << field_to_string(row.lookup_byte_operations)                        //
              << "," << field_to_string(row.lookup_cd_value)                               //
              << "," << field_to_string(row.lookup_ret_value)                              //
              << "," << field_to_string(row.lookup_opcode_gas)                             //
              << "," << field_to_string(row.range_check_l2_gas_hi)                         //
              << "," << field_to_string(row.range_check_l2_gas_lo)                         //
              << "," << field_to_string(row.range_check_da_gas_hi)                         //
              << "," << field_to_string(row.range_check_da_gas_lo)                         //
              << "," << field_to_string(row.kernel_output_lookup)                          //
              << "," << field_to_string(row.lookup_into_kernel)                            //
              << "," << field_to_string(row.incl_main_tag_err)                             //
              << "," << field_to_string(row.incl_mem_tag_err)                              //
              << "," << field_to_string(row.lookup_mem_rng_chk_lo)                         //
              << "," << field_to_string(row.lookup_mem_rng_chk_mid)                        //
              << "," << field_to_string(row.lookup_mem_rng_chk_hi)                         //
              << "," << field_to_string(row.lookup_pow_2_0)                                //
              << "," << field_to_string(row.lookup_pow_2_1)                                //
              << "," << field_to_string(row.lookup_u8_0)                                   //
              << "," << field_to_string(row.lookup_u8_1)                                   //
              << "," << field_to_string(row.lookup_u16_0)                                  //
              << "," << field_to_string(row.lookup_u16_1)                                  //
              << "," << field_to_string(row.lookup_u16_2)                                  //
              << "," << field_to_string(row.lookup_u16_3)                                  //
              << "," << field_to_string(row.lookup_u16_4)                                  //
              << "," << field_to_string(row.lookup_u16_5)                                  //
              << "," << field_to_string(row.lookup_u16_6)                                  //
              << "," << field_to_string(row.lookup_u16_7)                                  //
              << "," << field_to_string(row.lookup_u16_8)                                  //
              << "," << field_to_string(row.lookup_u16_9)                                  //
              << "," << field_to_string(row.lookup_u16_10)                                 //
              << "," << field_to_string(row.lookup_u16_11)                                 //
              << "," << field_to_string(row.lookup_u16_12)                                 //
              << "," << field_to_string(row.lookup_u16_13)                                 //
              << "," << field_to_string(row.lookup_u16_14)                                 //
              << "," << field_to_string(row.lookup_div_u16_0)                              //
              << "," << field_to_string(row.lookup_div_u16_1)                              //
              << "," << field_to_string(row.lookup_div_u16_2)                              //
              << "," << field_to_string(row.lookup_div_u16_3)                              //
              << "," << field_to_string(row.lookup_div_u16_4)                              //
              << "," << field_to_string(row.lookup_div_u16_5)                              //
              << "," << field_to_string(row.lookup_div_u16_6)                              //
              << "," << field_to_string(row.lookup_div_u16_7)                              //
              << "," << field_to_string(row.lookup_byte_lengths_counts)                    //
              << "," << field_to_string(row.lookup_byte_operations_counts)                 //
              << "," << field_to_string(row.lookup_cd_value_counts)                        //
              << "," << field_to_string(row.lookup_ret_value_counts)                       //
              << "," << field_to_string(row.lookup_opcode_gas_counts)                      //
              << "," << field_to_string(row.range_check_l2_gas_hi_counts)                  //
              << "," << field_to_string(row.range_check_l2_gas_lo_counts)                  //
              << "," << field_to_string(row.range_check_da_gas_hi_counts)                  //
              << "," << field_to_string(row.range_check_da_gas_lo_counts)                  //
              << "," << field_to_string(row.kernel_output_lookup_counts)                   //
              << "," << field_to_string(row.lookup_into_kernel_counts)                     //
              << "," << field_to_string(row.incl_main_tag_err_counts)                      //
              << "," << field_to_string(row.incl_mem_tag_err_counts)                       //
              << "," << field_to_string(row.lookup_mem_rng_chk_lo_counts)                  //
              << "," << field_to_string(row.lookup_mem_rng_chk_mid_counts)                 //
              << "," << field_to_string(row.lookup_mem_rng_chk_hi_counts)                  //
              << "," << field_to_string(row.lookup_pow_2_0_counts)                         //
              << "," << field_to_string(row.lookup_pow_2_1_counts)                         //
              << "," << field_to_string(row.lookup_u8_0_counts)                            //
              << "," << field_to_string(row.lookup_u8_1_counts)                            //
              << "," << field_to_string(row.lookup_u16_0_counts)                           //
              << "," << field_to_string(row.lookup_u16_1_counts)                           //
              << "," << field_to_string(row.lookup_u16_2_counts)                           //
              << "," << field_to_string(row.lookup_u16_3_counts)                           //
              << "," << field_to_string(row.lookup_u16_4_counts)                           //
              << "," << field_to_string(row.lookup_u16_5_counts)                           //
              << "," << field_to_string(row.lookup_u16_6_counts)                           //
              << "," << field_to_string(row.lookup_u16_7_counts)                           //
              << "," << field_to_string(row.lookup_u16_8_counts)                           //
              << "," << field_to_string(row.lookup_u16_9_counts)                           //
              << "," << field_to_string(row.lookup_u16_10_counts)                          //
              << "," << field_to_string(row.lookup_u16_11_counts)                          //
              << "," << field_to_string(row.lookup_u16_12_counts)                          //
              << "," << field_to_string(row.lookup_u16_13_counts)                          //
              << "," << field_to_string(row.lookup_u16_14_counts)                          //
              << "," << field_to_string(row.lookup_div_u16_0_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_1_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_2_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_3_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_4_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_5_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_6_counts)                       //
              << "," << field_to_string(row.lookup_div_u16_7_counts)                       //
        ;
}

// Explicit template instantiation.
template std::ostream& operator<<(std::ostream& os, AvmFullRow<bb::AvmFlavor::FF> const& row);
template std::vector<std::string> AvmFullRow<bb::AvmFlavor::FF>::names();

} // namespace bb
