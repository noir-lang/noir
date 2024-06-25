

#include "avm_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
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
    witness_commitments.kernel_kernel_inputs = commitment_key->commit(key->kernel_kernel_inputs);
    witness_commitments.kernel_kernel_value_out = commitment_key->commit(key->kernel_kernel_value_out);
    witness_commitments.kernel_kernel_side_effect_out = commitment_key->commit(key->kernel_kernel_side_effect_out);
    witness_commitments.kernel_kernel_metadata_out = commitment_key->commit(key->kernel_kernel_metadata_out);
    witness_commitments.alu_a_hi = commitment_key->commit(key->alu_a_hi);
    witness_commitments.alu_a_lo = commitment_key->commit(key->alu_a_lo);
    witness_commitments.alu_b_hi = commitment_key->commit(key->alu_b_hi);
    witness_commitments.alu_b_lo = commitment_key->commit(key->alu_b_lo);
    witness_commitments.alu_borrow = commitment_key->commit(key->alu_borrow);
    witness_commitments.alu_cf = commitment_key->commit(key->alu_cf);
    witness_commitments.alu_clk = commitment_key->commit(key->alu_clk);
    witness_commitments.alu_cmp_rng_ctr = commitment_key->commit(key->alu_cmp_rng_ctr);
    witness_commitments.alu_div_u16_r0 = commitment_key->commit(key->alu_div_u16_r0);
    witness_commitments.alu_div_u16_r1 = commitment_key->commit(key->alu_div_u16_r1);
    witness_commitments.alu_div_u16_r2 = commitment_key->commit(key->alu_div_u16_r2);
    witness_commitments.alu_div_u16_r3 = commitment_key->commit(key->alu_div_u16_r3);
    witness_commitments.alu_div_u16_r4 = commitment_key->commit(key->alu_div_u16_r4);
    witness_commitments.alu_div_u16_r5 = commitment_key->commit(key->alu_div_u16_r5);
    witness_commitments.alu_div_u16_r6 = commitment_key->commit(key->alu_div_u16_r6);
    witness_commitments.alu_div_u16_r7 = commitment_key->commit(key->alu_div_u16_r7);
    witness_commitments.alu_divisor_hi = commitment_key->commit(key->alu_divisor_hi);
    witness_commitments.alu_divisor_lo = commitment_key->commit(key->alu_divisor_lo);
    witness_commitments.alu_ff_tag = commitment_key->commit(key->alu_ff_tag);
    witness_commitments.alu_ia = commitment_key->commit(key->alu_ia);
    witness_commitments.alu_ib = commitment_key->commit(key->alu_ib);
    witness_commitments.alu_ic = commitment_key->commit(key->alu_ic);
    witness_commitments.alu_in_tag = commitment_key->commit(key->alu_in_tag);
    witness_commitments.alu_op_add = commitment_key->commit(key->alu_op_add);
    witness_commitments.alu_op_cast = commitment_key->commit(key->alu_op_cast);
    witness_commitments.alu_op_cast_prev = commitment_key->commit(key->alu_op_cast_prev);
    witness_commitments.alu_op_div = commitment_key->commit(key->alu_op_div);
    witness_commitments.alu_op_div_a_lt_b = commitment_key->commit(key->alu_op_div_a_lt_b);
    witness_commitments.alu_op_div_std = commitment_key->commit(key->alu_op_div_std);
    witness_commitments.alu_op_eq = commitment_key->commit(key->alu_op_eq);
    witness_commitments.alu_op_eq_diff_inv = commitment_key->commit(key->alu_op_eq_diff_inv);
    witness_commitments.alu_op_lt = commitment_key->commit(key->alu_op_lt);
    witness_commitments.alu_op_lte = commitment_key->commit(key->alu_op_lte);
    witness_commitments.alu_op_mul = commitment_key->commit(key->alu_op_mul);
    witness_commitments.alu_op_not = commitment_key->commit(key->alu_op_not);
    witness_commitments.alu_op_shl = commitment_key->commit(key->alu_op_shl);
    witness_commitments.alu_op_shr = commitment_key->commit(key->alu_op_shr);
    witness_commitments.alu_op_sub = commitment_key->commit(key->alu_op_sub);
    witness_commitments.alu_p_a_borrow = commitment_key->commit(key->alu_p_a_borrow);
    witness_commitments.alu_p_b_borrow = commitment_key->commit(key->alu_p_b_borrow);
    witness_commitments.alu_p_sub_a_hi = commitment_key->commit(key->alu_p_sub_a_hi);
    witness_commitments.alu_p_sub_a_lo = commitment_key->commit(key->alu_p_sub_a_lo);
    witness_commitments.alu_p_sub_b_hi = commitment_key->commit(key->alu_p_sub_b_hi);
    witness_commitments.alu_p_sub_b_lo = commitment_key->commit(key->alu_p_sub_b_lo);
    witness_commitments.alu_partial_prod_hi = commitment_key->commit(key->alu_partial_prod_hi);
    witness_commitments.alu_partial_prod_lo = commitment_key->commit(key->alu_partial_prod_lo);
    witness_commitments.alu_quotient_hi = commitment_key->commit(key->alu_quotient_hi);
    witness_commitments.alu_quotient_lo = commitment_key->commit(key->alu_quotient_lo);
    witness_commitments.alu_remainder = commitment_key->commit(key->alu_remainder);
    witness_commitments.alu_res_hi = commitment_key->commit(key->alu_res_hi);
    witness_commitments.alu_res_lo = commitment_key->commit(key->alu_res_lo);
    witness_commitments.alu_sel_alu = commitment_key->commit(key->alu_sel_alu);
    witness_commitments.alu_sel_cmp = commitment_key->commit(key->alu_sel_cmp);
    witness_commitments.alu_sel_div_rng_chk = commitment_key->commit(key->alu_sel_div_rng_chk);
    witness_commitments.alu_sel_rng_chk = commitment_key->commit(key->alu_sel_rng_chk);
    witness_commitments.alu_sel_rng_chk_lookup = commitment_key->commit(key->alu_sel_rng_chk_lookup);
    witness_commitments.alu_sel_shift_which = commitment_key->commit(key->alu_sel_shift_which);
    witness_commitments.alu_shift_lt_bit_len = commitment_key->commit(key->alu_shift_lt_bit_len);
    witness_commitments.alu_t_sub_s_bits = commitment_key->commit(key->alu_t_sub_s_bits);
    witness_commitments.alu_two_pow_s = commitment_key->commit(key->alu_two_pow_s);
    witness_commitments.alu_two_pow_t_sub_s = commitment_key->commit(key->alu_two_pow_t_sub_s);
    witness_commitments.alu_u128_tag = commitment_key->commit(key->alu_u128_tag);
    witness_commitments.alu_u16_r0 = commitment_key->commit(key->alu_u16_r0);
    witness_commitments.alu_u16_r1 = commitment_key->commit(key->alu_u16_r1);
    witness_commitments.alu_u16_r10 = commitment_key->commit(key->alu_u16_r10);
    witness_commitments.alu_u16_r11 = commitment_key->commit(key->alu_u16_r11);
    witness_commitments.alu_u16_r12 = commitment_key->commit(key->alu_u16_r12);
    witness_commitments.alu_u16_r13 = commitment_key->commit(key->alu_u16_r13);
    witness_commitments.alu_u16_r14 = commitment_key->commit(key->alu_u16_r14);
    witness_commitments.alu_u16_r2 = commitment_key->commit(key->alu_u16_r2);
    witness_commitments.alu_u16_r3 = commitment_key->commit(key->alu_u16_r3);
    witness_commitments.alu_u16_r4 = commitment_key->commit(key->alu_u16_r4);
    witness_commitments.alu_u16_r5 = commitment_key->commit(key->alu_u16_r5);
    witness_commitments.alu_u16_r6 = commitment_key->commit(key->alu_u16_r6);
    witness_commitments.alu_u16_r7 = commitment_key->commit(key->alu_u16_r7);
    witness_commitments.alu_u16_r8 = commitment_key->commit(key->alu_u16_r8);
    witness_commitments.alu_u16_r9 = commitment_key->commit(key->alu_u16_r9);
    witness_commitments.alu_u16_tag = commitment_key->commit(key->alu_u16_tag);
    witness_commitments.alu_u32_tag = commitment_key->commit(key->alu_u32_tag);
    witness_commitments.alu_u64_tag = commitment_key->commit(key->alu_u64_tag);
    witness_commitments.alu_u8_r0 = commitment_key->commit(key->alu_u8_r0);
    witness_commitments.alu_u8_r1 = commitment_key->commit(key->alu_u8_r1);
    witness_commitments.alu_u8_tag = commitment_key->commit(key->alu_u8_tag);
    witness_commitments.binary_acc_ia = commitment_key->commit(key->binary_acc_ia);
    witness_commitments.binary_acc_ib = commitment_key->commit(key->binary_acc_ib);
    witness_commitments.binary_acc_ic = commitment_key->commit(key->binary_acc_ic);
    witness_commitments.binary_clk = commitment_key->commit(key->binary_clk);
    witness_commitments.binary_ia_bytes = commitment_key->commit(key->binary_ia_bytes);
    witness_commitments.binary_ib_bytes = commitment_key->commit(key->binary_ib_bytes);
    witness_commitments.binary_ic_bytes = commitment_key->commit(key->binary_ic_bytes);
    witness_commitments.binary_in_tag = commitment_key->commit(key->binary_in_tag);
    witness_commitments.binary_mem_tag_ctr = commitment_key->commit(key->binary_mem_tag_ctr);
    witness_commitments.binary_mem_tag_ctr_inv = commitment_key->commit(key->binary_mem_tag_ctr_inv);
    witness_commitments.binary_op_id = commitment_key->commit(key->binary_op_id);
    witness_commitments.binary_sel_bin = commitment_key->commit(key->binary_sel_bin);
    witness_commitments.binary_start = commitment_key->commit(key->binary_start);
    witness_commitments.byte_lookup_sel_bin = commitment_key->commit(key->byte_lookup_sel_bin);
    witness_commitments.byte_lookup_table_byte_lengths = commitment_key->commit(key->byte_lookup_table_byte_lengths);
    witness_commitments.byte_lookup_table_in_tags = commitment_key->commit(key->byte_lookup_table_in_tags);
    witness_commitments.byte_lookup_table_input_a = commitment_key->commit(key->byte_lookup_table_input_a);
    witness_commitments.byte_lookup_table_input_b = commitment_key->commit(key->byte_lookup_table_input_b);
    witness_commitments.byte_lookup_table_op_id = commitment_key->commit(key->byte_lookup_table_op_id);
    witness_commitments.byte_lookup_table_output = commitment_key->commit(key->byte_lookup_table_output);
    witness_commitments.conversion_clk = commitment_key->commit(key->conversion_clk);
    witness_commitments.conversion_input = commitment_key->commit(key->conversion_input);
    witness_commitments.conversion_num_limbs = commitment_key->commit(key->conversion_num_limbs);
    witness_commitments.conversion_radix = commitment_key->commit(key->conversion_radix);
    witness_commitments.conversion_sel_to_radix_le = commitment_key->commit(key->conversion_sel_to_radix_le);
    witness_commitments.gas_da_gas_fixed_table = commitment_key->commit(key->gas_da_gas_fixed_table);
    witness_commitments.gas_l2_gas_fixed_table = commitment_key->commit(key->gas_l2_gas_fixed_table);
    witness_commitments.gas_sel_gas_cost = commitment_key->commit(key->gas_sel_gas_cost);
    witness_commitments.keccakf1600_clk = commitment_key->commit(key->keccakf1600_clk);
    witness_commitments.keccakf1600_input = commitment_key->commit(key->keccakf1600_input);
    witness_commitments.keccakf1600_output = commitment_key->commit(key->keccakf1600_output);
    witness_commitments.keccakf1600_sel_keccakf1600 = commitment_key->commit(key->keccakf1600_sel_keccakf1600);
    witness_commitments.kernel_emit_l2_to_l1_msg_write_offset =
        commitment_key->commit(key->kernel_emit_l2_to_l1_msg_write_offset);
    witness_commitments.kernel_emit_note_hash_write_offset =
        commitment_key->commit(key->kernel_emit_note_hash_write_offset);
    witness_commitments.kernel_emit_nullifier_write_offset =
        commitment_key->commit(key->kernel_emit_nullifier_write_offset);
    witness_commitments.kernel_emit_unencrypted_log_write_offset =
        commitment_key->commit(key->kernel_emit_unencrypted_log_write_offset);
    witness_commitments.kernel_kernel_in_offset = commitment_key->commit(key->kernel_kernel_in_offset);
    witness_commitments.kernel_kernel_out_offset = commitment_key->commit(key->kernel_kernel_out_offset);
    witness_commitments.kernel_l1_to_l2_msg_exists_write_offset =
        commitment_key->commit(key->kernel_l1_to_l2_msg_exists_write_offset);
    witness_commitments.kernel_note_hash_exist_write_offset =
        commitment_key->commit(key->kernel_note_hash_exist_write_offset);
    witness_commitments.kernel_nullifier_exists_write_offset =
        commitment_key->commit(key->kernel_nullifier_exists_write_offset);
    witness_commitments.kernel_nullifier_non_exists_write_offset =
        commitment_key->commit(key->kernel_nullifier_non_exists_write_offset);
    witness_commitments.kernel_q_public_input_kernel_add_to_table =
        commitment_key->commit(key->kernel_q_public_input_kernel_add_to_table);
    witness_commitments.kernel_q_public_input_kernel_out_add_to_table =
        commitment_key->commit(key->kernel_q_public_input_kernel_out_add_to_table);
    witness_commitments.kernel_side_effect_counter = commitment_key->commit(key->kernel_side_effect_counter);
    witness_commitments.kernel_sload_write_offset = commitment_key->commit(key->kernel_sload_write_offset);
    witness_commitments.kernel_sstore_write_offset = commitment_key->commit(key->kernel_sstore_write_offset);
    witness_commitments.main_abs_da_rem_gas_hi = commitment_key->commit(key->main_abs_da_rem_gas_hi);
    witness_commitments.main_abs_da_rem_gas_lo = commitment_key->commit(key->main_abs_da_rem_gas_lo);
    witness_commitments.main_abs_l2_rem_gas_hi = commitment_key->commit(key->main_abs_l2_rem_gas_hi);
    witness_commitments.main_abs_l2_rem_gas_lo = commitment_key->commit(key->main_abs_l2_rem_gas_lo);
    witness_commitments.main_alu_in_tag = commitment_key->commit(key->main_alu_in_tag);
    witness_commitments.main_bin_op_id = commitment_key->commit(key->main_bin_op_id);
    witness_commitments.main_call_ptr = commitment_key->commit(key->main_call_ptr);
    witness_commitments.main_da_gas_op_cost = commitment_key->commit(key->main_da_gas_op_cost);
    witness_commitments.main_da_gas_remaining = commitment_key->commit(key->main_da_gas_remaining);
    witness_commitments.main_da_out_of_gas = commitment_key->commit(key->main_da_out_of_gas);
    witness_commitments.main_ia = commitment_key->commit(key->main_ia);
    witness_commitments.main_ib = commitment_key->commit(key->main_ib);
    witness_commitments.main_ic = commitment_key->commit(key->main_ic);
    witness_commitments.main_id = commitment_key->commit(key->main_id);
    witness_commitments.main_id_zero = commitment_key->commit(key->main_id_zero);
    witness_commitments.main_ind_addr_a = commitment_key->commit(key->main_ind_addr_a);
    witness_commitments.main_ind_addr_b = commitment_key->commit(key->main_ind_addr_b);
    witness_commitments.main_ind_addr_c = commitment_key->commit(key->main_ind_addr_c);
    witness_commitments.main_ind_addr_d = commitment_key->commit(key->main_ind_addr_d);
    witness_commitments.main_internal_return_ptr = commitment_key->commit(key->main_internal_return_ptr);
    witness_commitments.main_inv = commitment_key->commit(key->main_inv);
    witness_commitments.main_l2_gas_op_cost = commitment_key->commit(key->main_l2_gas_op_cost);
    witness_commitments.main_l2_gas_remaining = commitment_key->commit(key->main_l2_gas_remaining);
    witness_commitments.main_l2_out_of_gas = commitment_key->commit(key->main_l2_out_of_gas);
    witness_commitments.main_mem_addr_a = commitment_key->commit(key->main_mem_addr_a);
    witness_commitments.main_mem_addr_b = commitment_key->commit(key->main_mem_addr_b);
    witness_commitments.main_mem_addr_c = commitment_key->commit(key->main_mem_addr_c);
    witness_commitments.main_mem_addr_d = commitment_key->commit(key->main_mem_addr_d);
    witness_commitments.main_op_err = commitment_key->commit(key->main_op_err);
    witness_commitments.main_opcode_val = commitment_key->commit(key->main_opcode_val);
    witness_commitments.main_pc = commitment_key->commit(key->main_pc);
    witness_commitments.main_r_in_tag = commitment_key->commit(key->main_r_in_tag);
    witness_commitments.main_rwa = commitment_key->commit(key->main_rwa);
    witness_commitments.main_rwb = commitment_key->commit(key->main_rwb);
    witness_commitments.main_rwc = commitment_key->commit(key->main_rwc);
    witness_commitments.main_rwd = commitment_key->commit(key->main_rwd);
    witness_commitments.main_sel_alu = commitment_key->commit(key->main_sel_alu);
    witness_commitments.main_sel_bin = commitment_key->commit(key->main_sel_bin);
    witness_commitments.main_sel_gas_accounting_active = commitment_key->commit(key->main_sel_gas_accounting_active);
    witness_commitments.main_sel_last = commitment_key->commit(key->main_sel_last);
    witness_commitments.main_sel_mem_op_a = commitment_key->commit(key->main_sel_mem_op_a);
    witness_commitments.main_sel_mem_op_activate_gas = commitment_key->commit(key->main_sel_mem_op_activate_gas);
    witness_commitments.main_sel_mem_op_b = commitment_key->commit(key->main_sel_mem_op_b);
    witness_commitments.main_sel_mem_op_c = commitment_key->commit(key->main_sel_mem_op_c);
    witness_commitments.main_sel_mem_op_d = commitment_key->commit(key->main_sel_mem_op_d);
    witness_commitments.main_sel_mov_ia_to_ic = commitment_key->commit(key->main_sel_mov_ia_to_ic);
    witness_commitments.main_sel_mov_ib_to_ic = commitment_key->commit(key->main_sel_mov_ib_to_ic);
    witness_commitments.main_sel_op_add = commitment_key->commit(key->main_sel_op_add);
    witness_commitments.main_sel_op_address = commitment_key->commit(key->main_sel_op_address);
    witness_commitments.main_sel_op_and = commitment_key->commit(key->main_sel_op_and);
    witness_commitments.main_sel_op_block_number = commitment_key->commit(key->main_sel_op_block_number);
    witness_commitments.main_sel_op_cast = commitment_key->commit(key->main_sel_op_cast);
    witness_commitments.main_sel_op_chain_id = commitment_key->commit(key->main_sel_op_chain_id);
    witness_commitments.main_sel_op_cmov = commitment_key->commit(key->main_sel_op_cmov);
    witness_commitments.main_sel_op_coinbase = commitment_key->commit(key->main_sel_op_coinbase);
    witness_commitments.main_sel_op_dagasleft = commitment_key->commit(key->main_sel_op_dagasleft);
    witness_commitments.main_sel_op_div = commitment_key->commit(key->main_sel_op_div);
    witness_commitments.main_sel_op_emit_l2_to_l1_msg = commitment_key->commit(key->main_sel_op_emit_l2_to_l1_msg);
    witness_commitments.main_sel_op_emit_note_hash = commitment_key->commit(key->main_sel_op_emit_note_hash);
    witness_commitments.main_sel_op_emit_nullifier = commitment_key->commit(key->main_sel_op_emit_nullifier);
    witness_commitments.main_sel_op_emit_unencrypted_log =
        commitment_key->commit(key->main_sel_op_emit_unencrypted_log);
    witness_commitments.main_sel_op_eq = commitment_key->commit(key->main_sel_op_eq);
    witness_commitments.main_sel_op_external_call = commitment_key->commit(key->main_sel_op_external_call);
    witness_commitments.main_sel_op_fdiv = commitment_key->commit(key->main_sel_op_fdiv);
    witness_commitments.main_sel_op_fee_per_da_gas = commitment_key->commit(key->main_sel_op_fee_per_da_gas);
    witness_commitments.main_sel_op_fee_per_l2_gas = commitment_key->commit(key->main_sel_op_fee_per_l2_gas);
    witness_commitments.main_sel_op_get_contract_instance =
        commitment_key->commit(key->main_sel_op_get_contract_instance);
    witness_commitments.main_sel_op_halt = commitment_key->commit(key->main_sel_op_halt);
    witness_commitments.main_sel_op_internal_call = commitment_key->commit(key->main_sel_op_internal_call);
    witness_commitments.main_sel_op_internal_return = commitment_key->commit(key->main_sel_op_internal_return);
    witness_commitments.main_sel_op_jump = commitment_key->commit(key->main_sel_op_jump);
    witness_commitments.main_sel_op_jumpi = commitment_key->commit(key->main_sel_op_jumpi);
    witness_commitments.main_sel_op_keccak = commitment_key->commit(key->main_sel_op_keccak);
    witness_commitments.main_sel_op_l1_to_l2_msg_exists = commitment_key->commit(key->main_sel_op_l1_to_l2_msg_exists);
    witness_commitments.main_sel_op_l2gasleft = commitment_key->commit(key->main_sel_op_l2gasleft);
    witness_commitments.main_sel_op_lt = commitment_key->commit(key->main_sel_op_lt);
    witness_commitments.main_sel_op_lte = commitment_key->commit(key->main_sel_op_lte);
    witness_commitments.main_sel_op_mov = commitment_key->commit(key->main_sel_op_mov);
    witness_commitments.main_sel_op_mul = commitment_key->commit(key->main_sel_op_mul);
    witness_commitments.main_sel_op_not = commitment_key->commit(key->main_sel_op_not);
    witness_commitments.main_sel_op_note_hash_exists = commitment_key->commit(key->main_sel_op_note_hash_exists);
    witness_commitments.main_sel_op_nullifier_exists = commitment_key->commit(key->main_sel_op_nullifier_exists);
    witness_commitments.main_sel_op_or = commitment_key->commit(key->main_sel_op_or);
    witness_commitments.main_sel_op_pedersen = commitment_key->commit(key->main_sel_op_pedersen);
    witness_commitments.main_sel_op_poseidon2 = commitment_key->commit(key->main_sel_op_poseidon2);
    witness_commitments.main_sel_op_radix_le = commitment_key->commit(key->main_sel_op_radix_le);
    witness_commitments.main_sel_op_sender = commitment_key->commit(key->main_sel_op_sender);
    witness_commitments.main_sel_op_sha256 = commitment_key->commit(key->main_sel_op_sha256);
    witness_commitments.main_sel_op_shl = commitment_key->commit(key->main_sel_op_shl);
    witness_commitments.main_sel_op_shr = commitment_key->commit(key->main_sel_op_shr);
    witness_commitments.main_sel_op_sload = commitment_key->commit(key->main_sel_op_sload);
    witness_commitments.main_sel_op_sstore = commitment_key->commit(key->main_sel_op_sstore);
    witness_commitments.main_sel_op_storage_address = commitment_key->commit(key->main_sel_op_storage_address);
    witness_commitments.main_sel_op_sub = commitment_key->commit(key->main_sel_op_sub);
    witness_commitments.main_sel_op_timestamp = commitment_key->commit(key->main_sel_op_timestamp);
    witness_commitments.main_sel_op_transaction_fee = commitment_key->commit(key->main_sel_op_transaction_fee);
    witness_commitments.main_sel_op_version = commitment_key->commit(key->main_sel_op_version);
    witness_commitments.main_sel_op_xor = commitment_key->commit(key->main_sel_op_xor);
    witness_commitments.main_sel_q_kernel_lookup = commitment_key->commit(key->main_sel_q_kernel_lookup);
    witness_commitments.main_sel_q_kernel_output_lookup = commitment_key->commit(key->main_sel_q_kernel_output_lookup);
    witness_commitments.main_sel_resolve_ind_addr_a = commitment_key->commit(key->main_sel_resolve_ind_addr_a);
    witness_commitments.main_sel_resolve_ind_addr_b = commitment_key->commit(key->main_sel_resolve_ind_addr_b);
    witness_commitments.main_sel_resolve_ind_addr_c = commitment_key->commit(key->main_sel_resolve_ind_addr_c);
    witness_commitments.main_sel_resolve_ind_addr_d = commitment_key->commit(key->main_sel_resolve_ind_addr_d);
    witness_commitments.main_sel_rng_16 = commitment_key->commit(key->main_sel_rng_16);
    witness_commitments.main_sel_rng_8 = commitment_key->commit(key->main_sel_rng_8);
    witness_commitments.main_space_id = commitment_key->commit(key->main_space_id);
    witness_commitments.main_tag_err = commitment_key->commit(key->main_tag_err);
    witness_commitments.main_w_in_tag = commitment_key->commit(key->main_w_in_tag);
    witness_commitments.mem_addr = commitment_key->commit(key->mem_addr);
    witness_commitments.mem_clk = commitment_key->commit(key->mem_clk);
    witness_commitments.mem_diff_hi = commitment_key->commit(key->mem_diff_hi);
    witness_commitments.mem_diff_lo = commitment_key->commit(key->mem_diff_lo);
    witness_commitments.mem_diff_mid = commitment_key->commit(key->mem_diff_mid);
    witness_commitments.mem_glob_addr = commitment_key->commit(key->mem_glob_addr);
    witness_commitments.mem_last = commitment_key->commit(key->mem_last);
    witness_commitments.mem_lastAccess = commitment_key->commit(key->mem_lastAccess);
    witness_commitments.mem_one_min_inv = commitment_key->commit(key->mem_one_min_inv);
    witness_commitments.mem_r_in_tag = commitment_key->commit(key->mem_r_in_tag);
    witness_commitments.mem_rw = commitment_key->commit(key->mem_rw);
    witness_commitments.mem_sel_mem = commitment_key->commit(key->mem_sel_mem);
    witness_commitments.mem_sel_mov_ia_to_ic = commitment_key->commit(key->mem_sel_mov_ia_to_ic);
    witness_commitments.mem_sel_mov_ib_to_ic = commitment_key->commit(key->mem_sel_mov_ib_to_ic);
    witness_commitments.mem_sel_op_a = commitment_key->commit(key->mem_sel_op_a);
    witness_commitments.mem_sel_op_b = commitment_key->commit(key->mem_sel_op_b);
    witness_commitments.mem_sel_op_c = commitment_key->commit(key->mem_sel_op_c);
    witness_commitments.mem_sel_op_cmov = commitment_key->commit(key->mem_sel_op_cmov);
    witness_commitments.mem_sel_op_d = commitment_key->commit(key->mem_sel_op_d);
    witness_commitments.mem_sel_resolve_ind_addr_a = commitment_key->commit(key->mem_sel_resolve_ind_addr_a);
    witness_commitments.mem_sel_resolve_ind_addr_b = commitment_key->commit(key->mem_sel_resolve_ind_addr_b);
    witness_commitments.mem_sel_resolve_ind_addr_c = commitment_key->commit(key->mem_sel_resolve_ind_addr_c);
    witness_commitments.mem_sel_resolve_ind_addr_d = commitment_key->commit(key->mem_sel_resolve_ind_addr_d);
    witness_commitments.mem_sel_rng_chk = commitment_key->commit(key->mem_sel_rng_chk);
    witness_commitments.mem_skip_check_tag = commitment_key->commit(key->mem_skip_check_tag);
    witness_commitments.mem_space_id = commitment_key->commit(key->mem_space_id);
    witness_commitments.mem_tag = commitment_key->commit(key->mem_tag);
    witness_commitments.mem_tag_err = commitment_key->commit(key->mem_tag_err);
    witness_commitments.mem_tsp = commitment_key->commit(key->mem_tsp);
    witness_commitments.mem_val = commitment_key->commit(key->mem_val);
    witness_commitments.mem_w_in_tag = commitment_key->commit(key->mem_w_in_tag);
    witness_commitments.pedersen_clk = commitment_key->commit(key->pedersen_clk);
    witness_commitments.pedersen_input = commitment_key->commit(key->pedersen_input);
    witness_commitments.pedersen_output = commitment_key->commit(key->pedersen_output);
    witness_commitments.pedersen_sel_pedersen = commitment_key->commit(key->pedersen_sel_pedersen);
    witness_commitments.poseidon2_clk = commitment_key->commit(key->poseidon2_clk);
    witness_commitments.poseidon2_input = commitment_key->commit(key->poseidon2_input);
    witness_commitments.poseidon2_output = commitment_key->commit(key->poseidon2_output);
    witness_commitments.poseidon2_sel_poseidon_perm = commitment_key->commit(key->poseidon2_sel_poseidon_perm);
    witness_commitments.powers_power_of_2 = commitment_key->commit(key->powers_power_of_2);
    witness_commitments.sha256_clk = commitment_key->commit(key->sha256_clk);
    witness_commitments.sha256_input = commitment_key->commit(key->sha256_input);
    witness_commitments.sha256_output = commitment_key->commit(key->sha256_output);
    witness_commitments.sha256_sel_sha256_compression = commitment_key->commit(key->sha256_sel_sha256_compression);
    witness_commitments.sha256_state = commitment_key->commit(key->sha256_state);
    witness_commitments.lookup_byte_lengths_counts = commitment_key->commit(key->lookup_byte_lengths_counts);
    witness_commitments.lookup_byte_operations_counts = commitment_key->commit(key->lookup_byte_operations_counts);
    witness_commitments.lookup_opcode_gas_counts = commitment_key->commit(key->lookup_opcode_gas_counts);
    witness_commitments.range_check_l2_gas_hi_counts = commitment_key->commit(key->range_check_l2_gas_hi_counts);
    witness_commitments.range_check_l2_gas_lo_counts = commitment_key->commit(key->range_check_l2_gas_lo_counts);
    witness_commitments.range_check_da_gas_hi_counts = commitment_key->commit(key->range_check_da_gas_hi_counts);
    witness_commitments.range_check_da_gas_lo_counts = commitment_key->commit(key->range_check_da_gas_lo_counts);
    witness_commitments.kernel_output_lookup_counts = commitment_key->commit(key->kernel_output_lookup_counts);
    witness_commitments.lookup_into_kernel_counts = commitment_key->commit(key->lookup_into_kernel_counts);
    witness_commitments.incl_main_tag_err_counts = commitment_key->commit(key->incl_main_tag_err_counts);
    witness_commitments.incl_mem_tag_err_counts = commitment_key->commit(key->incl_mem_tag_err_counts);
    witness_commitments.lookup_mem_rng_chk_lo_counts = commitment_key->commit(key->lookup_mem_rng_chk_lo_counts);
    witness_commitments.lookup_mem_rng_chk_mid_counts = commitment_key->commit(key->lookup_mem_rng_chk_mid_counts);
    witness_commitments.lookup_mem_rng_chk_hi_counts = commitment_key->commit(key->lookup_mem_rng_chk_hi_counts);
    witness_commitments.lookup_pow_2_0_counts = commitment_key->commit(key->lookup_pow_2_0_counts);
    witness_commitments.lookup_pow_2_1_counts = commitment_key->commit(key->lookup_pow_2_1_counts);
    witness_commitments.lookup_u8_0_counts = commitment_key->commit(key->lookup_u8_0_counts);
    witness_commitments.lookup_u8_1_counts = commitment_key->commit(key->lookup_u8_1_counts);
    witness_commitments.lookup_u16_0_counts = commitment_key->commit(key->lookup_u16_0_counts);
    witness_commitments.lookup_u16_1_counts = commitment_key->commit(key->lookup_u16_1_counts);
    witness_commitments.lookup_u16_2_counts = commitment_key->commit(key->lookup_u16_2_counts);
    witness_commitments.lookup_u16_3_counts = commitment_key->commit(key->lookup_u16_3_counts);
    witness_commitments.lookup_u16_4_counts = commitment_key->commit(key->lookup_u16_4_counts);
    witness_commitments.lookup_u16_5_counts = commitment_key->commit(key->lookup_u16_5_counts);
    witness_commitments.lookup_u16_6_counts = commitment_key->commit(key->lookup_u16_6_counts);
    witness_commitments.lookup_u16_7_counts = commitment_key->commit(key->lookup_u16_7_counts);
    witness_commitments.lookup_u16_8_counts = commitment_key->commit(key->lookup_u16_8_counts);
    witness_commitments.lookup_u16_9_counts = commitment_key->commit(key->lookup_u16_9_counts);
    witness_commitments.lookup_u16_10_counts = commitment_key->commit(key->lookup_u16_10_counts);
    witness_commitments.lookup_u16_11_counts = commitment_key->commit(key->lookup_u16_11_counts);
    witness_commitments.lookup_u16_12_counts = commitment_key->commit(key->lookup_u16_12_counts);
    witness_commitments.lookup_u16_13_counts = commitment_key->commit(key->lookup_u16_13_counts);
    witness_commitments.lookup_u16_14_counts = commitment_key->commit(key->lookup_u16_14_counts);
    witness_commitments.lookup_div_u16_0_counts = commitment_key->commit(key->lookup_div_u16_0_counts);
    witness_commitments.lookup_div_u16_1_counts = commitment_key->commit(key->lookup_div_u16_1_counts);
    witness_commitments.lookup_div_u16_2_counts = commitment_key->commit(key->lookup_div_u16_2_counts);
    witness_commitments.lookup_div_u16_3_counts = commitment_key->commit(key->lookup_div_u16_3_counts);
    witness_commitments.lookup_div_u16_4_counts = commitment_key->commit(key->lookup_div_u16_4_counts);
    witness_commitments.lookup_div_u16_5_counts = commitment_key->commit(key->lookup_div_u16_5_counts);
    witness_commitments.lookup_div_u16_6_counts = commitment_key->commit(key->lookup_div_u16_6_counts);
    witness_commitments.lookup_div_u16_7_counts = commitment_key->commit(key->lookup_div_u16_7_counts);

    // Send all commitments to the verifier
    transcript->send_to_verifier(commitment_labels.kernel_kernel_inputs, witness_commitments.kernel_kernel_inputs);
    transcript->send_to_verifier(commitment_labels.kernel_kernel_value_out,
                                 witness_commitments.kernel_kernel_value_out);
    transcript->send_to_verifier(commitment_labels.kernel_kernel_side_effect_out,
                                 witness_commitments.kernel_kernel_side_effect_out);
    transcript->send_to_verifier(commitment_labels.kernel_kernel_metadata_out,
                                 witness_commitments.kernel_kernel_metadata_out);
    transcript->send_to_verifier(commitment_labels.alu_a_hi, witness_commitments.alu_a_hi);
    transcript->send_to_verifier(commitment_labels.alu_a_lo, witness_commitments.alu_a_lo);
    transcript->send_to_verifier(commitment_labels.alu_b_hi, witness_commitments.alu_b_hi);
    transcript->send_to_verifier(commitment_labels.alu_b_lo, witness_commitments.alu_b_lo);
    transcript->send_to_verifier(commitment_labels.alu_borrow, witness_commitments.alu_borrow);
    transcript->send_to_verifier(commitment_labels.alu_cf, witness_commitments.alu_cf);
    transcript->send_to_verifier(commitment_labels.alu_clk, witness_commitments.alu_clk);
    transcript->send_to_verifier(commitment_labels.alu_cmp_rng_ctr, witness_commitments.alu_cmp_rng_ctr);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r0, witness_commitments.alu_div_u16_r0);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r1, witness_commitments.alu_div_u16_r1);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r2, witness_commitments.alu_div_u16_r2);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r3, witness_commitments.alu_div_u16_r3);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r4, witness_commitments.alu_div_u16_r4);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r5, witness_commitments.alu_div_u16_r5);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r6, witness_commitments.alu_div_u16_r6);
    transcript->send_to_verifier(commitment_labels.alu_div_u16_r7, witness_commitments.alu_div_u16_r7);
    transcript->send_to_verifier(commitment_labels.alu_divisor_hi, witness_commitments.alu_divisor_hi);
    transcript->send_to_verifier(commitment_labels.alu_divisor_lo, witness_commitments.alu_divisor_lo);
    transcript->send_to_verifier(commitment_labels.alu_ff_tag, witness_commitments.alu_ff_tag);
    transcript->send_to_verifier(commitment_labels.alu_ia, witness_commitments.alu_ia);
    transcript->send_to_verifier(commitment_labels.alu_ib, witness_commitments.alu_ib);
    transcript->send_to_verifier(commitment_labels.alu_ic, witness_commitments.alu_ic);
    transcript->send_to_verifier(commitment_labels.alu_in_tag, witness_commitments.alu_in_tag);
    transcript->send_to_verifier(commitment_labels.alu_op_add, witness_commitments.alu_op_add);
    transcript->send_to_verifier(commitment_labels.alu_op_cast, witness_commitments.alu_op_cast);
    transcript->send_to_verifier(commitment_labels.alu_op_cast_prev, witness_commitments.alu_op_cast_prev);
    transcript->send_to_verifier(commitment_labels.alu_op_div, witness_commitments.alu_op_div);
    transcript->send_to_verifier(commitment_labels.alu_op_div_a_lt_b, witness_commitments.alu_op_div_a_lt_b);
    transcript->send_to_verifier(commitment_labels.alu_op_div_std, witness_commitments.alu_op_div_std);
    transcript->send_to_verifier(commitment_labels.alu_op_eq, witness_commitments.alu_op_eq);
    transcript->send_to_verifier(commitment_labels.alu_op_eq_diff_inv, witness_commitments.alu_op_eq_diff_inv);
    transcript->send_to_verifier(commitment_labels.alu_op_lt, witness_commitments.alu_op_lt);
    transcript->send_to_verifier(commitment_labels.alu_op_lte, witness_commitments.alu_op_lte);
    transcript->send_to_verifier(commitment_labels.alu_op_mul, witness_commitments.alu_op_mul);
    transcript->send_to_verifier(commitment_labels.alu_op_not, witness_commitments.alu_op_not);
    transcript->send_to_verifier(commitment_labels.alu_op_shl, witness_commitments.alu_op_shl);
    transcript->send_to_verifier(commitment_labels.alu_op_shr, witness_commitments.alu_op_shr);
    transcript->send_to_verifier(commitment_labels.alu_op_sub, witness_commitments.alu_op_sub);
    transcript->send_to_verifier(commitment_labels.alu_p_a_borrow, witness_commitments.alu_p_a_borrow);
    transcript->send_to_verifier(commitment_labels.alu_p_b_borrow, witness_commitments.alu_p_b_borrow);
    transcript->send_to_verifier(commitment_labels.alu_p_sub_a_hi, witness_commitments.alu_p_sub_a_hi);
    transcript->send_to_verifier(commitment_labels.alu_p_sub_a_lo, witness_commitments.alu_p_sub_a_lo);
    transcript->send_to_verifier(commitment_labels.alu_p_sub_b_hi, witness_commitments.alu_p_sub_b_hi);
    transcript->send_to_verifier(commitment_labels.alu_p_sub_b_lo, witness_commitments.alu_p_sub_b_lo);
    transcript->send_to_verifier(commitment_labels.alu_partial_prod_hi, witness_commitments.alu_partial_prod_hi);
    transcript->send_to_verifier(commitment_labels.alu_partial_prod_lo, witness_commitments.alu_partial_prod_lo);
    transcript->send_to_verifier(commitment_labels.alu_quotient_hi, witness_commitments.alu_quotient_hi);
    transcript->send_to_verifier(commitment_labels.alu_quotient_lo, witness_commitments.alu_quotient_lo);
    transcript->send_to_verifier(commitment_labels.alu_remainder, witness_commitments.alu_remainder);
    transcript->send_to_verifier(commitment_labels.alu_res_hi, witness_commitments.alu_res_hi);
    transcript->send_to_verifier(commitment_labels.alu_res_lo, witness_commitments.alu_res_lo);
    transcript->send_to_verifier(commitment_labels.alu_sel_alu, witness_commitments.alu_sel_alu);
    transcript->send_to_verifier(commitment_labels.alu_sel_cmp, witness_commitments.alu_sel_cmp);
    transcript->send_to_verifier(commitment_labels.alu_sel_div_rng_chk, witness_commitments.alu_sel_div_rng_chk);
    transcript->send_to_verifier(commitment_labels.alu_sel_rng_chk, witness_commitments.alu_sel_rng_chk);
    transcript->send_to_verifier(commitment_labels.alu_sel_rng_chk_lookup, witness_commitments.alu_sel_rng_chk_lookup);
    transcript->send_to_verifier(commitment_labels.alu_sel_shift_which, witness_commitments.alu_sel_shift_which);
    transcript->send_to_verifier(commitment_labels.alu_shift_lt_bit_len, witness_commitments.alu_shift_lt_bit_len);
    transcript->send_to_verifier(commitment_labels.alu_t_sub_s_bits, witness_commitments.alu_t_sub_s_bits);
    transcript->send_to_verifier(commitment_labels.alu_two_pow_s, witness_commitments.alu_two_pow_s);
    transcript->send_to_verifier(commitment_labels.alu_two_pow_t_sub_s, witness_commitments.alu_two_pow_t_sub_s);
    transcript->send_to_verifier(commitment_labels.alu_u128_tag, witness_commitments.alu_u128_tag);
    transcript->send_to_verifier(commitment_labels.alu_u16_r0, witness_commitments.alu_u16_r0);
    transcript->send_to_verifier(commitment_labels.alu_u16_r1, witness_commitments.alu_u16_r1);
    transcript->send_to_verifier(commitment_labels.alu_u16_r10, witness_commitments.alu_u16_r10);
    transcript->send_to_verifier(commitment_labels.alu_u16_r11, witness_commitments.alu_u16_r11);
    transcript->send_to_verifier(commitment_labels.alu_u16_r12, witness_commitments.alu_u16_r12);
    transcript->send_to_verifier(commitment_labels.alu_u16_r13, witness_commitments.alu_u16_r13);
    transcript->send_to_verifier(commitment_labels.alu_u16_r14, witness_commitments.alu_u16_r14);
    transcript->send_to_verifier(commitment_labels.alu_u16_r2, witness_commitments.alu_u16_r2);
    transcript->send_to_verifier(commitment_labels.alu_u16_r3, witness_commitments.alu_u16_r3);
    transcript->send_to_verifier(commitment_labels.alu_u16_r4, witness_commitments.alu_u16_r4);
    transcript->send_to_verifier(commitment_labels.alu_u16_r5, witness_commitments.alu_u16_r5);
    transcript->send_to_verifier(commitment_labels.alu_u16_r6, witness_commitments.alu_u16_r6);
    transcript->send_to_verifier(commitment_labels.alu_u16_r7, witness_commitments.alu_u16_r7);
    transcript->send_to_verifier(commitment_labels.alu_u16_r8, witness_commitments.alu_u16_r8);
    transcript->send_to_verifier(commitment_labels.alu_u16_r9, witness_commitments.alu_u16_r9);
    transcript->send_to_verifier(commitment_labels.alu_u16_tag, witness_commitments.alu_u16_tag);
    transcript->send_to_verifier(commitment_labels.alu_u32_tag, witness_commitments.alu_u32_tag);
    transcript->send_to_verifier(commitment_labels.alu_u64_tag, witness_commitments.alu_u64_tag);
    transcript->send_to_verifier(commitment_labels.alu_u8_r0, witness_commitments.alu_u8_r0);
    transcript->send_to_verifier(commitment_labels.alu_u8_r1, witness_commitments.alu_u8_r1);
    transcript->send_to_verifier(commitment_labels.alu_u8_tag, witness_commitments.alu_u8_tag);
    transcript->send_to_verifier(commitment_labels.binary_acc_ia, witness_commitments.binary_acc_ia);
    transcript->send_to_verifier(commitment_labels.binary_acc_ib, witness_commitments.binary_acc_ib);
    transcript->send_to_verifier(commitment_labels.binary_acc_ic, witness_commitments.binary_acc_ic);
    transcript->send_to_verifier(commitment_labels.binary_clk, witness_commitments.binary_clk);
    transcript->send_to_verifier(commitment_labels.binary_ia_bytes, witness_commitments.binary_ia_bytes);
    transcript->send_to_verifier(commitment_labels.binary_ib_bytes, witness_commitments.binary_ib_bytes);
    transcript->send_to_verifier(commitment_labels.binary_ic_bytes, witness_commitments.binary_ic_bytes);
    transcript->send_to_verifier(commitment_labels.binary_in_tag, witness_commitments.binary_in_tag);
    transcript->send_to_verifier(commitment_labels.binary_mem_tag_ctr, witness_commitments.binary_mem_tag_ctr);
    transcript->send_to_verifier(commitment_labels.binary_mem_tag_ctr_inv, witness_commitments.binary_mem_tag_ctr_inv);
    transcript->send_to_verifier(commitment_labels.binary_op_id, witness_commitments.binary_op_id);
    transcript->send_to_verifier(commitment_labels.binary_sel_bin, witness_commitments.binary_sel_bin);
    transcript->send_to_verifier(commitment_labels.binary_start, witness_commitments.binary_start);
    transcript->send_to_verifier(commitment_labels.byte_lookup_sel_bin, witness_commitments.byte_lookup_sel_bin);
    transcript->send_to_verifier(commitment_labels.byte_lookup_table_byte_lengths,
                                 witness_commitments.byte_lookup_table_byte_lengths);
    transcript->send_to_verifier(commitment_labels.byte_lookup_table_in_tags,
                                 witness_commitments.byte_lookup_table_in_tags);
    transcript->send_to_verifier(commitment_labels.byte_lookup_table_input_a,
                                 witness_commitments.byte_lookup_table_input_a);
    transcript->send_to_verifier(commitment_labels.byte_lookup_table_input_b,
                                 witness_commitments.byte_lookup_table_input_b);
    transcript->send_to_verifier(commitment_labels.byte_lookup_table_op_id,
                                 witness_commitments.byte_lookup_table_op_id);
    transcript->send_to_verifier(commitment_labels.byte_lookup_table_output,
                                 witness_commitments.byte_lookup_table_output);
    transcript->send_to_verifier(commitment_labels.conversion_clk, witness_commitments.conversion_clk);
    transcript->send_to_verifier(commitment_labels.conversion_input, witness_commitments.conversion_input);
    transcript->send_to_verifier(commitment_labels.conversion_num_limbs, witness_commitments.conversion_num_limbs);
    transcript->send_to_verifier(commitment_labels.conversion_radix, witness_commitments.conversion_radix);
    transcript->send_to_verifier(commitment_labels.conversion_sel_to_radix_le,
                                 witness_commitments.conversion_sel_to_radix_le);
    transcript->send_to_verifier(commitment_labels.gas_da_gas_fixed_table, witness_commitments.gas_da_gas_fixed_table);
    transcript->send_to_verifier(commitment_labels.gas_l2_gas_fixed_table, witness_commitments.gas_l2_gas_fixed_table);
    transcript->send_to_verifier(commitment_labels.gas_sel_gas_cost, witness_commitments.gas_sel_gas_cost);
    transcript->send_to_verifier(commitment_labels.keccakf1600_clk, witness_commitments.keccakf1600_clk);
    transcript->send_to_verifier(commitment_labels.keccakf1600_input, witness_commitments.keccakf1600_input);
    transcript->send_to_verifier(commitment_labels.keccakf1600_output, witness_commitments.keccakf1600_output);
    transcript->send_to_verifier(commitment_labels.keccakf1600_sel_keccakf1600,
                                 witness_commitments.keccakf1600_sel_keccakf1600);
    transcript->send_to_verifier(commitment_labels.kernel_emit_l2_to_l1_msg_write_offset,
                                 witness_commitments.kernel_emit_l2_to_l1_msg_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_emit_note_hash_write_offset,
                                 witness_commitments.kernel_emit_note_hash_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_emit_nullifier_write_offset,
                                 witness_commitments.kernel_emit_nullifier_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_emit_unencrypted_log_write_offset,
                                 witness_commitments.kernel_emit_unencrypted_log_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_kernel_in_offset,
                                 witness_commitments.kernel_kernel_in_offset);
    transcript->send_to_verifier(commitment_labels.kernel_kernel_out_offset,
                                 witness_commitments.kernel_kernel_out_offset);
    transcript->send_to_verifier(commitment_labels.kernel_l1_to_l2_msg_exists_write_offset,
                                 witness_commitments.kernel_l1_to_l2_msg_exists_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_note_hash_exist_write_offset,
                                 witness_commitments.kernel_note_hash_exist_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_nullifier_exists_write_offset,
                                 witness_commitments.kernel_nullifier_exists_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_nullifier_non_exists_write_offset,
                                 witness_commitments.kernel_nullifier_non_exists_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_q_public_input_kernel_add_to_table,
                                 witness_commitments.kernel_q_public_input_kernel_add_to_table);
    transcript->send_to_verifier(commitment_labels.kernel_q_public_input_kernel_out_add_to_table,
                                 witness_commitments.kernel_q_public_input_kernel_out_add_to_table);
    transcript->send_to_verifier(commitment_labels.kernel_side_effect_counter,
                                 witness_commitments.kernel_side_effect_counter);
    transcript->send_to_verifier(commitment_labels.kernel_sload_write_offset,
                                 witness_commitments.kernel_sload_write_offset);
    transcript->send_to_verifier(commitment_labels.kernel_sstore_write_offset,
                                 witness_commitments.kernel_sstore_write_offset);
    transcript->send_to_verifier(commitment_labels.main_abs_da_rem_gas_hi, witness_commitments.main_abs_da_rem_gas_hi);
    transcript->send_to_verifier(commitment_labels.main_abs_da_rem_gas_lo, witness_commitments.main_abs_da_rem_gas_lo);
    transcript->send_to_verifier(commitment_labels.main_abs_l2_rem_gas_hi, witness_commitments.main_abs_l2_rem_gas_hi);
    transcript->send_to_verifier(commitment_labels.main_abs_l2_rem_gas_lo, witness_commitments.main_abs_l2_rem_gas_lo);
    transcript->send_to_verifier(commitment_labels.main_alu_in_tag, witness_commitments.main_alu_in_tag);
    transcript->send_to_verifier(commitment_labels.main_bin_op_id, witness_commitments.main_bin_op_id);
    transcript->send_to_verifier(commitment_labels.main_call_ptr, witness_commitments.main_call_ptr);
    transcript->send_to_verifier(commitment_labels.main_da_gas_op_cost, witness_commitments.main_da_gas_op_cost);
    transcript->send_to_verifier(commitment_labels.main_da_gas_remaining, witness_commitments.main_da_gas_remaining);
    transcript->send_to_verifier(commitment_labels.main_da_out_of_gas, witness_commitments.main_da_out_of_gas);
    transcript->send_to_verifier(commitment_labels.main_ia, witness_commitments.main_ia);
    transcript->send_to_verifier(commitment_labels.main_ib, witness_commitments.main_ib);
    transcript->send_to_verifier(commitment_labels.main_ic, witness_commitments.main_ic);
    transcript->send_to_verifier(commitment_labels.main_id, witness_commitments.main_id);
    transcript->send_to_verifier(commitment_labels.main_id_zero, witness_commitments.main_id_zero);
    transcript->send_to_verifier(commitment_labels.main_ind_addr_a, witness_commitments.main_ind_addr_a);
    transcript->send_to_verifier(commitment_labels.main_ind_addr_b, witness_commitments.main_ind_addr_b);
    transcript->send_to_verifier(commitment_labels.main_ind_addr_c, witness_commitments.main_ind_addr_c);
    transcript->send_to_verifier(commitment_labels.main_ind_addr_d, witness_commitments.main_ind_addr_d);
    transcript->send_to_verifier(commitment_labels.main_internal_return_ptr,
                                 witness_commitments.main_internal_return_ptr);
    transcript->send_to_verifier(commitment_labels.main_inv, witness_commitments.main_inv);
    transcript->send_to_verifier(commitment_labels.main_l2_gas_op_cost, witness_commitments.main_l2_gas_op_cost);
    transcript->send_to_verifier(commitment_labels.main_l2_gas_remaining, witness_commitments.main_l2_gas_remaining);
    transcript->send_to_verifier(commitment_labels.main_l2_out_of_gas, witness_commitments.main_l2_out_of_gas);
    transcript->send_to_verifier(commitment_labels.main_mem_addr_a, witness_commitments.main_mem_addr_a);
    transcript->send_to_verifier(commitment_labels.main_mem_addr_b, witness_commitments.main_mem_addr_b);
    transcript->send_to_verifier(commitment_labels.main_mem_addr_c, witness_commitments.main_mem_addr_c);
    transcript->send_to_verifier(commitment_labels.main_mem_addr_d, witness_commitments.main_mem_addr_d);
    transcript->send_to_verifier(commitment_labels.main_op_err, witness_commitments.main_op_err);
    transcript->send_to_verifier(commitment_labels.main_opcode_val, witness_commitments.main_opcode_val);
    transcript->send_to_verifier(commitment_labels.main_pc, witness_commitments.main_pc);
    transcript->send_to_verifier(commitment_labels.main_r_in_tag, witness_commitments.main_r_in_tag);
    transcript->send_to_verifier(commitment_labels.main_rwa, witness_commitments.main_rwa);
    transcript->send_to_verifier(commitment_labels.main_rwb, witness_commitments.main_rwb);
    transcript->send_to_verifier(commitment_labels.main_rwc, witness_commitments.main_rwc);
    transcript->send_to_verifier(commitment_labels.main_rwd, witness_commitments.main_rwd);
    transcript->send_to_verifier(commitment_labels.main_sel_alu, witness_commitments.main_sel_alu);
    transcript->send_to_verifier(commitment_labels.main_sel_bin, witness_commitments.main_sel_bin);
    transcript->send_to_verifier(commitment_labels.main_sel_gas_accounting_active,
                                 witness_commitments.main_sel_gas_accounting_active);
    transcript->send_to_verifier(commitment_labels.main_sel_last, witness_commitments.main_sel_last);
    transcript->send_to_verifier(commitment_labels.main_sel_mem_op_a, witness_commitments.main_sel_mem_op_a);
    transcript->send_to_verifier(commitment_labels.main_sel_mem_op_activate_gas,
                                 witness_commitments.main_sel_mem_op_activate_gas);
    transcript->send_to_verifier(commitment_labels.main_sel_mem_op_b, witness_commitments.main_sel_mem_op_b);
    transcript->send_to_verifier(commitment_labels.main_sel_mem_op_c, witness_commitments.main_sel_mem_op_c);
    transcript->send_to_verifier(commitment_labels.main_sel_mem_op_d, witness_commitments.main_sel_mem_op_d);
    transcript->send_to_verifier(commitment_labels.main_sel_mov_ia_to_ic, witness_commitments.main_sel_mov_ia_to_ic);
    transcript->send_to_verifier(commitment_labels.main_sel_mov_ib_to_ic, witness_commitments.main_sel_mov_ib_to_ic);
    transcript->send_to_verifier(commitment_labels.main_sel_op_add, witness_commitments.main_sel_op_add);
    transcript->send_to_verifier(commitment_labels.main_sel_op_address, witness_commitments.main_sel_op_address);
    transcript->send_to_verifier(commitment_labels.main_sel_op_and, witness_commitments.main_sel_op_and);
    transcript->send_to_verifier(commitment_labels.main_sel_op_block_number,
                                 witness_commitments.main_sel_op_block_number);
    transcript->send_to_verifier(commitment_labels.main_sel_op_cast, witness_commitments.main_sel_op_cast);
    transcript->send_to_verifier(commitment_labels.main_sel_op_chain_id, witness_commitments.main_sel_op_chain_id);
    transcript->send_to_verifier(commitment_labels.main_sel_op_cmov, witness_commitments.main_sel_op_cmov);
    transcript->send_to_verifier(commitment_labels.main_sel_op_coinbase, witness_commitments.main_sel_op_coinbase);
    transcript->send_to_verifier(commitment_labels.main_sel_op_dagasleft, witness_commitments.main_sel_op_dagasleft);
    transcript->send_to_verifier(commitment_labels.main_sel_op_div, witness_commitments.main_sel_op_div);
    transcript->send_to_verifier(commitment_labels.main_sel_op_emit_l2_to_l1_msg,
                                 witness_commitments.main_sel_op_emit_l2_to_l1_msg);
    transcript->send_to_verifier(commitment_labels.main_sel_op_emit_note_hash,
                                 witness_commitments.main_sel_op_emit_note_hash);
    transcript->send_to_verifier(commitment_labels.main_sel_op_emit_nullifier,
                                 witness_commitments.main_sel_op_emit_nullifier);
    transcript->send_to_verifier(commitment_labels.main_sel_op_emit_unencrypted_log,
                                 witness_commitments.main_sel_op_emit_unencrypted_log);
    transcript->send_to_verifier(commitment_labels.main_sel_op_eq, witness_commitments.main_sel_op_eq);
    transcript->send_to_verifier(commitment_labels.main_sel_op_external_call,
                                 witness_commitments.main_sel_op_external_call);
    transcript->send_to_verifier(commitment_labels.main_sel_op_fdiv, witness_commitments.main_sel_op_fdiv);
    transcript->send_to_verifier(commitment_labels.main_sel_op_fee_per_da_gas,
                                 witness_commitments.main_sel_op_fee_per_da_gas);
    transcript->send_to_verifier(commitment_labels.main_sel_op_fee_per_l2_gas,
                                 witness_commitments.main_sel_op_fee_per_l2_gas);
    transcript->send_to_verifier(commitment_labels.main_sel_op_get_contract_instance,
                                 witness_commitments.main_sel_op_get_contract_instance);
    transcript->send_to_verifier(commitment_labels.main_sel_op_halt, witness_commitments.main_sel_op_halt);
    transcript->send_to_verifier(commitment_labels.main_sel_op_internal_call,
                                 witness_commitments.main_sel_op_internal_call);
    transcript->send_to_verifier(commitment_labels.main_sel_op_internal_return,
                                 witness_commitments.main_sel_op_internal_return);
    transcript->send_to_verifier(commitment_labels.main_sel_op_jump, witness_commitments.main_sel_op_jump);
    transcript->send_to_verifier(commitment_labels.main_sel_op_jumpi, witness_commitments.main_sel_op_jumpi);
    transcript->send_to_verifier(commitment_labels.main_sel_op_keccak, witness_commitments.main_sel_op_keccak);
    transcript->send_to_verifier(commitment_labels.main_sel_op_l1_to_l2_msg_exists,
                                 witness_commitments.main_sel_op_l1_to_l2_msg_exists);
    transcript->send_to_verifier(commitment_labels.main_sel_op_l2gasleft, witness_commitments.main_sel_op_l2gasleft);
    transcript->send_to_verifier(commitment_labels.main_sel_op_lt, witness_commitments.main_sel_op_lt);
    transcript->send_to_verifier(commitment_labels.main_sel_op_lte, witness_commitments.main_sel_op_lte);
    transcript->send_to_verifier(commitment_labels.main_sel_op_mov, witness_commitments.main_sel_op_mov);
    transcript->send_to_verifier(commitment_labels.main_sel_op_mul, witness_commitments.main_sel_op_mul);
    transcript->send_to_verifier(commitment_labels.main_sel_op_not, witness_commitments.main_sel_op_not);
    transcript->send_to_verifier(commitment_labels.main_sel_op_note_hash_exists,
                                 witness_commitments.main_sel_op_note_hash_exists);
    transcript->send_to_verifier(commitment_labels.main_sel_op_nullifier_exists,
                                 witness_commitments.main_sel_op_nullifier_exists);
    transcript->send_to_verifier(commitment_labels.main_sel_op_or, witness_commitments.main_sel_op_or);
    transcript->send_to_verifier(commitment_labels.main_sel_op_pedersen, witness_commitments.main_sel_op_pedersen);
    transcript->send_to_verifier(commitment_labels.main_sel_op_poseidon2, witness_commitments.main_sel_op_poseidon2);
    transcript->send_to_verifier(commitment_labels.main_sel_op_radix_le, witness_commitments.main_sel_op_radix_le);
    transcript->send_to_verifier(commitment_labels.main_sel_op_sender, witness_commitments.main_sel_op_sender);
    transcript->send_to_verifier(commitment_labels.main_sel_op_sha256, witness_commitments.main_sel_op_sha256);
    transcript->send_to_verifier(commitment_labels.main_sel_op_shl, witness_commitments.main_sel_op_shl);
    transcript->send_to_verifier(commitment_labels.main_sel_op_shr, witness_commitments.main_sel_op_shr);
    transcript->send_to_verifier(commitment_labels.main_sel_op_sload, witness_commitments.main_sel_op_sload);
    transcript->send_to_verifier(commitment_labels.main_sel_op_sstore, witness_commitments.main_sel_op_sstore);
    transcript->send_to_verifier(commitment_labels.main_sel_op_storage_address,
                                 witness_commitments.main_sel_op_storage_address);
    transcript->send_to_verifier(commitment_labels.main_sel_op_sub, witness_commitments.main_sel_op_sub);
    transcript->send_to_verifier(commitment_labels.main_sel_op_timestamp, witness_commitments.main_sel_op_timestamp);
    transcript->send_to_verifier(commitment_labels.main_sel_op_transaction_fee,
                                 witness_commitments.main_sel_op_transaction_fee);
    transcript->send_to_verifier(commitment_labels.main_sel_op_version, witness_commitments.main_sel_op_version);
    transcript->send_to_verifier(commitment_labels.main_sel_op_xor, witness_commitments.main_sel_op_xor);
    transcript->send_to_verifier(commitment_labels.main_sel_q_kernel_lookup,
                                 witness_commitments.main_sel_q_kernel_lookup);
    transcript->send_to_verifier(commitment_labels.main_sel_q_kernel_output_lookup,
                                 witness_commitments.main_sel_q_kernel_output_lookup);
    transcript->send_to_verifier(commitment_labels.main_sel_resolve_ind_addr_a,
                                 witness_commitments.main_sel_resolve_ind_addr_a);
    transcript->send_to_verifier(commitment_labels.main_sel_resolve_ind_addr_b,
                                 witness_commitments.main_sel_resolve_ind_addr_b);
    transcript->send_to_verifier(commitment_labels.main_sel_resolve_ind_addr_c,
                                 witness_commitments.main_sel_resolve_ind_addr_c);
    transcript->send_to_verifier(commitment_labels.main_sel_resolve_ind_addr_d,
                                 witness_commitments.main_sel_resolve_ind_addr_d);
    transcript->send_to_verifier(commitment_labels.main_sel_rng_16, witness_commitments.main_sel_rng_16);
    transcript->send_to_verifier(commitment_labels.main_sel_rng_8, witness_commitments.main_sel_rng_8);
    transcript->send_to_verifier(commitment_labels.main_space_id, witness_commitments.main_space_id);
    transcript->send_to_verifier(commitment_labels.main_tag_err, witness_commitments.main_tag_err);
    transcript->send_to_verifier(commitment_labels.main_w_in_tag, witness_commitments.main_w_in_tag);
    transcript->send_to_verifier(commitment_labels.mem_addr, witness_commitments.mem_addr);
    transcript->send_to_verifier(commitment_labels.mem_clk, witness_commitments.mem_clk);
    transcript->send_to_verifier(commitment_labels.mem_diff_hi, witness_commitments.mem_diff_hi);
    transcript->send_to_verifier(commitment_labels.mem_diff_lo, witness_commitments.mem_diff_lo);
    transcript->send_to_verifier(commitment_labels.mem_diff_mid, witness_commitments.mem_diff_mid);
    transcript->send_to_verifier(commitment_labels.mem_glob_addr, witness_commitments.mem_glob_addr);
    transcript->send_to_verifier(commitment_labels.mem_last, witness_commitments.mem_last);
    transcript->send_to_verifier(commitment_labels.mem_lastAccess, witness_commitments.mem_lastAccess);
    transcript->send_to_verifier(commitment_labels.mem_one_min_inv, witness_commitments.mem_one_min_inv);
    transcript->send_to_verifier(commitment_labels.mem_r_in_tag, witness_commitments.mem_r_in_tag);
    transcript->send_to_verifier(commitment_labels.mem_rw, witness_commitments.mem_rw);
    transcript->send_to_verifier(commitment_labels.mem_sel_mem, witness_commitments.mem_sel_mem);
    transcript->send_to_verifier(commitment_labels.mem_sel_mov_ia_to_ic, witness_commitments.mem_sel_mov_ia_to_ic);
    transcript->send_to_verifier(commitment_labels.mem_sel_mov_ib_to_ic, witness_commitments.mem_sel_mov_ib_to_ic);
    transcript->send_to_verifier(commitment_labels.mem_sel_op_a, witness_commitments.mem_sel_op_a);
    transcript->send_to_verifier(commitment_labels.mem_sel_op_b, witness_commitments.mem_sel_op_b);
    transcript->send_to_verifier(commitment_labels.mem_sel_op_c, witness_commitments.mem_sel_op_c);
    transcript->send_to_verifier(commitment_labels.mem_sel_op_cmov, witness_commitments.mem_sel_op_cmov);
    transcript->send_to_verifier(commitment_labels.mem_sel_op_d, witness_commitments.mem_sel_op_d);
    transcript->send_to_verifier(commitment_labels.mem_sel_resolve_ind_addr_a,
                                 witness_commitments.mem_sel_resolve_ind_addr_a);
    transcript->send_to_verifier(commitment_labels.mem_sel_resolve_ind_addr_b,
                                 witness_commitments.mem_sel_resolve_ind_addr_b);
    transcript->send_to_verifier(commitment_labels.mem_sel_resolve_ind_addr_c,
                                 witness_commitments.mem_sel_resolve_ind_addr_c);
    transcript->send_to_verifier(commitment_labels.mem_sel_resolve_ind_addr_d,
                                 witness_commitments.mem_sel_resolve_ind_addr_d);
    transcript->send_to_verifier(commitment_labels.mem_sel_rng_chk, witness_commitments.mem_sel_rng_chk);
    transcript->send_to_verifier(commitment_labels.mem_skip_check_tag, witness_commitments.mem_skip_check_tag);
    transcript->send_to_verifier(commitment_labels.mem_space_id, witness_commitments.mem_space_id);
    transcript->send_to_verifier(commitment_labels.mem_tag, witness_commitments.mem_tag);
    transcript->send_to_verifier(commitment_labels.mem_tag_err, witness_commitments.mem_tag_err);
    transcript->send_to_verifier(commitment_labels.mem_tsp, witness_commitments.mem_tsp);
    transcript->send_to_verifier(commitment_labels.mem_val, witness_commitments.mem_val);
    transcript->send_to_verifier(commitment_labels.mem_w_in_tag, witness_commitments.mem_w_in_tag);
    transcript->send_to_verifier(commitment_labels.pedersen_clk, witness_commitments.pedersen_clk);
    transcript->send_to_verifier(commitment_labels.pedersen_input, witness_commitments.pedersen_input);
    transcript->send_to_verifier(commitment_labels.pedersen_output, witness_commitments.pedersen_output);
    transcript->send_to_verifier(commitment_labels.pedersen_sel_pedersen, witness_commitments.pedersen_sel_pedersen);
    transcript->send_to_verifier(commitment_labels.poseidon2_clk, witness_commitments.poseidon2_clk);
    transcript->send_to_verifier(commitment_labels.poseidon2_input, witness_commitments.poseidon2_input);
    transcript->send_to_verifier(commitment_labels.poseidon2_output, witness_commitments.poseidon2_output);
    transcript->send_to_verifier(commitment_labels.poseidon2_sel_poseidon_perm,
                                 witness_commitments.poseidon2_sel_poseidon_perm);
    transcript->send_to_verifier(commitment_labels.powers_power_of_2, witness_commitments.powers_power_of_2);
    transcript->send_to_verifier(commitment_labels.sha256_clk, witness_commitments.sha256_clk);
    transcript->send_to_verifier(commitment_labels.sha256_input, witness_commitments.sha256_input);
    transcript->send_to_verifier(commitment_labels.sha256_output, witness_commitments.sha256_output);
    transcript->send_to_verifier(commitment_labels.sha256_sel_sha256_compression,
                                 witness_commitments.sha256_sel_sha256_compression);
    transcript->send_to_verifier(commitment_labels.sha256_state, witness_commitments.sha256_state);
    transcript->send_to_verifier(commitment_labels.lookup_byte_lengths_counts,
                                 witness_commitments.lookup_byte_lengths_counts);
    transcript->send_to_verifier(commitment_labels.lookup_byte_operations_counts,
                                 witness_commitments.lookup_byte_operations_counts);
    transcript->send_to_verifier(commitment_labels.lookup_opcode_gas_counts,
                                 witness_commitments.lookup_opcode_gas_counts);
    transcript->send_to_verifier(commitment_labels.range_check_l2_gas_hi_counts,
                                 witness_commitments.range_check_l2_gas_hi_counts);
    transcript->send_to_verifier(commitment_labels.range_check_l2_gas_lo_counts,
                                 witness_commitments.range_check_l2_gas_lo_counts);
    transcript->send_to_verifier(commitment_labels.range_check_da_gas_hi_counts,
                                 witness_commitments.range_check_da_gas_hi_counts);
    transcript->send_to_verifier(commitment_labels.range_check_da_gas_lo_counts,
                                 witness_commitments.range_check_da_gas_lo_counts);
    transcript->send_to_verifier(commitment_labels.kernel_output_lookup_counts,
                                 witness_commitments.kernel_output_lookup_counts);
    transcript->send_to_verifier(commitment_labels.lookup_into_kernel_counts,
                                 witness_commitments.lookup_into_kernel_counts);
    transcript->send_to_verifier(commitment_labels.incl_main_tag_err_counts,
                                 witness_commitments.incl_main_tag_err_counts);
    transcript->send_to_verifier(commitment_labels.incl_mem_tag_err_counts,
                                 witness_commitments.incl_mem_tag_err_counts);
    transcript->send_to_verifier(commitment_labels.lookup_mem_rng_chk_lo_counts,
                                 witness_commitments.lookup_mem_rng_chk_lo_counts);
    transcript->send_to_verifier(commitment_labels.lookup_mem_rng_chk_mid_counts,
                                 witness_commitments.lookup_mem_rng_chk_mid_counts);
    transcript->send_to_verifier(commitment_labels.lookup_mem_rng_chk_hi_counts,
                                 witness_commitments.lookup_mem_rng_chk_hi_counts);
    transcript->send_to_verifier(commitment_labels.lookup_pow_2_0_counts, witness_commitments.lookup_pow_2_0_counts);
    transcript->send_to_verifier(commitment_labels.lookup_pow_2_1_counts, witness_commitments.lookup_pow_2_1_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u8_0_counts, witness_commitments.lookup_u8_0_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u8_1_counts, witness_commitments.lookup_u8_1_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_0_counts, witness_commitments.lookup_u16_0_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_1_counts, witness_commitments.lookup_u16_1_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_2_counts, witness_commitments.lookup_u16_2_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_3_counts, witness_commitments.lookup_u16_3_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_4_counts, witness_commitments.lookup_u16_4_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_5_counts, witness_commitments.lookup_u16_5_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_6_counts, witness_commitments.lookup_u16_6_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_7_counts, witness_commitments.lookup_u16_7_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_8_counts, witness_commitments.lookup_u16_8_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_9_counts, witness_commitments.lookup_u16_9_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_10_counts, witness_commitments.lookup_u16_10_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_11_counts, witness_commitments.lookup_u16_11_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_12_counts, witness_commitments.lookup_u16_12_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_13_counts, witness_commitments.lookup_u16_13_counts);
    transcript->send_to_verifier(commitment_labels.lookup_u16_14_counts, witness_commitments.lookup_u16_14_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_0_counts,
                                 witness_commitments.lookup_div_u16_0_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_1_counts,
                                 witness_commitments.lookup_div_u16_1_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_2_counts,
                                 witness_commitments.lookup_div_u16_2_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_3_counts,
                                 witness_commitments.lookup_div_u16_3_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_4_counts,
                                 witness_commitments.lookup_div_u16_4_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_5_counts,
                                 witness_commitments.lookup_div_u16_5_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_6_counts,
                                 witness_commitments.lookup_div_u16_6_counts);
    transcript->send_to_verifier(commitment_labels.lookup_div_u16_7_counts,
                                 witness_commitments.lookup_div_u16_7_counts);
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
    using Curve = typename Flavor::Curve;
    using ZeroMorph = ZeroMorphProver_<Curve>;

    auto prover_opening_claim = ZeroMorph::prove(prover_polynomials.get_unshifted(),
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
