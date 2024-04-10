

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
    witness_commitments.avm_alu_a_hi = commitment_key->commit(key->avm_alu_a_hi);
    witness_commitments.avm_alu_a_lo = commitment_key->commit(key->avm_alu_a_lo);
    witness_commitments.avm_alu_alu_sel = commitment_key->commit(key->avm_alu_alu_sel);
    witness_commitments.avm_alu_b_hi = commitment_key->commit(key->avm_alu_b_hi);
    witness_commitments.avm_alu_b_lo = commitment_key->commit(key->avm_alu_b_lo);
    witness_commitments.avm_alu_borrow = commitment_key->commit(key->avm_alu_borrow);
    witness_commitments.avm_alu_cf = commitment_key->commit(key->avm_alu_cf);
    witness_commitments.avm_alu_clk = commitment_key->commit(key->avm_alu_clk);
    witness_commitments.avm_alu_cmp_rng_ctr = commitment_key->commit(key->avm_alu_cmp_rng_ctr);
    witness_commitments.avm_alu_cmp_sel = commitment_key->commit(key->avm_alu_cmp_sel);
    witness_commitments.avm_alu_ff_tag = commitment_key->commit(key->avm_alu_ff_tag);
    witness_commitments.avm_alu_ia = commitment_key->commit(key->avm_alu_ia);
    witness_commitments.avm_alu_ib = commitment_key->commit(key->avm_alu_ib);
    witness_commitments.avm_alu_ic = commitment_key->commit(key->avm_alu_ic);
    witness_commitments.avm_alu_in_tag = commitment_key->commit(key->avm_alu_in_tag);
    witness_commitments.avm_alu_op_add = commitment_key->commit(key->avm_alu_op_add);
    witness_commitments.avm_alu_op_div = commitment_key->commit(key->avm_alu_op_div);
    witness_commitments.avm_alu_op_eq = commitment_key->commit(key->avm_alu_op_eq);
    witness_commitments.avm_alu_op_eq_diff_inv = commitment_key->commit(key->avm_alu_op_eq_diff_inv);
    witness_commitments.avm_alu_op_lt = commitment_key->commit(key->avm_alu_op_lt);
    witness_commitments.avm_alu_op_lte = commitment_key->commit(key->avm_alu_op_lte);
    witness_commitments.avm_alu_op_mul = commitment_key->commit(key->avm_alu_op_mul);
    witness_commitments.avm_alu_op_not = commitment_key->commit(key->avm_alu_op_not);
    witness_commitments.avm_alu_op_sub = commitment_key->commit(key->avm_alu_op_sub);
    witness_commitments.avm_alu_p_a_borrow = commitment_key->commit(key->avm_alu_p_a_borrow);
    witness_commitments.avm_alu_p_b_borrow = commitment_key->commit(key->avm_alu_p_b_borrow);
    witness_commitments.avm_alu_p_sub_a_hi = commitment_key->commit(key->avm_alu_p_sub_a_hi);
    witness_commitments.avm_alu_p_sub_a_lo = commitment_key->commit(key->avm_alu_p_sub_a_lo);
    witness_commitments.avm_alu_p_sub_b_hi = commitment_key->commit(key->avm_alu_p_sub_b_hi);
    witness_commitments.avm_alu_p_sub_b_lo = commitment_key->commit(key->avm_alu_p_sub_b_lo);
    witness_commitments.avm_alu_res_hi = commitment_key->commit(key->avm_alu_res_hi);
    witness_commitments.avm_alu_res_lo = commitment_key->commit(key->avm_alu_res_lo);
    witness_commitments.avm_alu_rng_chk_lookup_selector = commitment_key->commit(key->avm_alu_rng_chk_lookup_selector);
    witness_commitments.avm_alu_rng_chk_sel = commitment_key->commit(key->avm_alu_rng_chk_sel);
    witness_commitments.avm_alu_u128_tag = commitment_key->commit(key->avm_alu_u128_tag);
    witness_commitments.avm_alu_u16_r0 = commitment_key->commit(key->avm_alu_u16_r0);
    witness_commitments.avm_alu_u16_r1 = commitment_key->commit(key->avm_alu_u16_r1);
    witness_commitments.avm_alu_u16_r10 = commitment_key->commit(key->avm_alu_u16_r10);
    witness_commitments.avm_alu_u16_r11 = commitment_key->commit(key->avm_alu_u16_r11);
    witness_commitments.avm_alu_u16_r12 = commitment_key->commit(key->avm_alu_u16_r12);
    witness_commitments.avm_alu_u16_r13 = commitment_key->commit(key->avm_alu_u16_r13);
    witness_commitments.avm_alu_u16_r14 = commitment_key->commit(key->avm_alu_u16_r14);
    witness_commitments.avm_alu_u16_r2 = commitment_key->commit(key->avm_alu_u16_r2);
    witness_commitments.avm_alu_u16_r3 = commitment_key->commit(key->avm_alu_u16_r3);
    witness_commitments.avm_alu_u16_r4 = commitment_key->commit(key->avm_alu_u16_r4);
    witness_commitments.avm_alu_u16_r5 = commitment_key->commit(key->avm_alu_u16_r5);
    witness_commitments.avm_alu_u16_r6 = commitment_key->commit(key->avm_alu_u16_r6);
    witness_commitments.avm_alu_u16_r7 = commitment_key->commit(key->avm_alu_u16_r7);
    witness_commitments.avm_alu_u16_r8 = commitment_key->commit(key->avm_alu_u16_r8);
    witness_commitments.avm_alu_u16_r9 = commitment_key->commit(key->avm_alu_u16_r9);
    witness_commitments.avm_alu_u16_tag = commitment_key->commit(key->avm_alu_u16_tag);
    witness_commitments.avm_alu_u32_tag = commitment_key->commit(key->avm_alu_u32_tag);
    witness_commitments.avm_alu_u64_r0 = commitment_key->commit(key->avm_alu_u64_r0);
    witness_commitments.avm_alu_u64_tag = commitment_key->commit(key->avm_alu_u64_tag);
    witness_commitments.avm_alu_u8_r0 = commitment_key->commit(key->avm_alu_u8_r0);
    witness_commitments.avm_alu_u8_r1 = commitment_key->commit(key->avm_alu_u8_r1);
    witness_commitments.avm_alu_u8_tag = commitment_key->commit(key->avm_alu_u8_tag);
    witness_commitments.avm_binary_acc_ia = commitment_key->commit(key->avm_binary_acc_ia);
    witness_commitments.avm_binary_acc_ib = commitment_key->commit(key->avm_binary_acc_ib);
    witness_commitments.avm_binary_acc_ic = commitment_key->commit(key->avm_binary_acc_ic);
    witness_commitments.avm_binary_bin_sel = commitment_key->commit(key->avm_binary_bin_sel);
    witness_commitments.avm_binary_clk = commitment_key->commit(key->avm_binary_clk);
    witness_commitments.avm_binary_ia_bytes = commitment_key->commit(key->avm_binary_ia_bytes);
    witness_commitments.avm_binary_ib_bytes = commitment_key->commit(key->avm_binary_ib_bytes);
    witness_commitments.avm_binary_ic_bytes = commitment_key->commit(key->avm_binary_ic_bytes);
    witness_commitments.avm_binary_in_tag = commitment_key->commit(key->avm_binary_in_tag);
    witness_commitments.avm_binary_mem_tag_ctr = commitment_key->commit(key->avm_binary_mem_tag_ctr);
    witness_commitments.avm_binary_mem_tag_ctr_inv = commitment_key->commit(key->avm_binary_mem_tag_ctr_inv);
    witness_commitments.avm_binary_op_id = commitment_key->commit(key->avm_binary_op_id);
    witness_commitments.avm_binary_start = commitment_key->commit(key->avm_binary_start);
    witness_commitments.avm_byte_lookup_bin_sel = commitment_key->commit(key->avm_byte_lookup_bin_sel);
    witness_commitments.avm_byte_lookup_table_byte_lengths =
        commitment_key->commit(key->avm_byte_lookup_table_byte_lengths);
    witness_commitments.avm_byte_lookup_table_in_tags = commitment_key->commit(key->avm_byte_lookup_table_in_tags);
    witness_commitments.avm_byte_lookup_table_input_a = commitment_key->commit(key->avm_byte_lookup_table_input_a);
    witness_commitments.avm_byte_lookup_table_input_b = commitment_key->commit(key->avm_byte_lookup_table_input_b);
    witness_commitments.avm_byte_lookup_table_op_id = commitment_key->commit(key->avm_byte_lookup_table_op_id);
    witness_commitments.avm_byte_lookup_table_output = commitment_key->commit(key->avm_byte_lookup_table_output);
    witness_commitments.avm_main_alu_sel = commitment_key->commit(key->avm_main_alu_sel);
    witness_commitments.avm_main_bin_op_id = commitment_key->commit(key->avm_main_bin_op_id);
    witness_commitments.avm_main_bin_sel = commitment_key->commit(key->avm_main_bin_sel);
    witness_commitments.avm_main_ia = commitment_key->commit(key->avm_main_ia);
    witness_commitments.avm_main_ib = commitment_key->commit(key->avm_main_ib);
    witness_commitments.avm_main_ic = commitment_key->commit(key->avm_main_ic);
    witness_commitments.avm_main_ind_a = commitment_key->commit(key->avm_main_ind_a);
    witness_commitments.avm_main_ind_b = commitment_key->commit(key->avm_main_ind_b);
    witness_commitments.avm_main_ind_c = commitment_key->commit(key->avm_main_ind_c);
    witness_commitments.avm_main_ind_op_a = commitment_key->commit(key->avm_main_ind_op_a);
    witness_commitments.avm_main_ind_op_b = commitment_key->commit(key->avm_main_ind_op_b);
    witness_commitments.avm_main_ind_op_c = commitment_key->commit(key->avm_main_ind_op_c);
    witness_commitments.avm_main_internal_return_ptr = commitment_key->commit(key->avm_main_internal_return_ptr);
    witness_commitments.avm_main_inv = commitment_key->commit(key->avm_main_inv);
    witness_commitments.avm_main_last = commitment_key->commit(key->avm_main_last);
    witness_commitments.avm_main_mem_idx_a = commitment_key->commit(key->avm_main_mem_idx_a);
    witness_commitments.avm_main_mem_idx_b = commitment_key->commit(key->avm_main_mem_idx_b);
    witness_commitments.avm_main_mem_idx_c = commitment_key->commit(key->avm_main_mem_idx_c);
    witness_commitments.avm_main_mem_op_a = commitment_key->commit(key->avm_main_mem_op_a);
    witness_commitments.avm_main_mem_op_b = commitment_key->commit(key->avm_main_mem_op_b);
    witness_commitments.avm_main_mem_op_c = commitment_key->commit(key->avm_main_mem_op_c);
    witness_commitments.avm_main_op_err = commitment_key->commit(key->avm_main_op_err);
    witness_commitments.avm_main_pc = commitment_key->commit(key->avm_main_pc);
    witness_commitments.avm_main_r_in_tag = commitment_key->commit(key->avm_main_r_in_tag);
    witness_commitments.avm_main_rwa = commitment_key->commit(key->avm_main_rwa);
    witness_commitments.avm_main_rwb = commitment_key->commit(key->avm_main_rwb);
    witness_commitments.avm_main_rwc = commitment_key->commit(key->avm_main_rwc);
    witness_commitments.avm_main_sel_halt = commitment_key->commit(key->avm_main_sel_halt);
    witness_commitments.avm_main_sel_internal_call = commitment_key->commit(key->avm_main_sel_internal_call);
    witness_commitments.avm_main_sel_internal_return = commitment_key->commit(key->avm_main_sel_internal_return);
    witness_commitments.avm_main_sel_jump = commitment_key->commit(key->avm_main_sel_jump);
    witness_commitments.avm_main_sel_mov = commitment_key->commit(key->avm_main_sel_mov);
    witness_commitments.avm_main_sel_op_add = commitment_key->commit(key->avm_main_sel_op_add);
    witness_commitments.avm_main_sel_op_and = commitment_key->commit(key->avm_main_sel_op_and);
    witness_commitments.avm_main_sel_op_div = commitment_key->commit(key->avm_main_sel_op_div);
    witness_commitments.avm_main_sel_op_eq = commitment_key->commit(key->avm_main_sel_op_eq);
    witness_commitments.avm_main_sel_op_lt = commitment_key->commit(key->avm_main_sel_op_lt);
    witness_commitments.avm_main_sel_op_lte = commitment_key->commit(key->avm_main_sel_op_lte);
    witness_commitments.avm_main_sel_op_mul = commitment_key->commit(key->avm_main_sel_op_mul);
    witness_commitments.avm_main_sel_op_not = commitment_key->commit(key->avm_main_sel_op_not);
    witness_commitments.avm_main_sel_op_or = commitment_key->commit(key->avm_main_sel_op_or);
    witness_commitments.avm_main_sel_op_sub = commitment_key->commit(key->avm_main_sel_op_sub);
    witness_commitments.avm_main_sel_op_xor = commitment_key->commit(key->avm_main_sel_op_xor);
    witness_commitments.avm_main_sel_rng_16 = commitment_key->commit(key->avm_main_sel_rng_16);
    witness_commitments.avm_main_sel_rng_8 = commitment_key->commit(key->avm_main_sel_rng_8);
    witness_commitments.avm_main_tag_err = commitment_key->commit(key->avm_main_tag_err);
    witness_commitments.avm_main_w_in_tag = commitment_key->commit(key->avm_main_w_in_tag);
    witness_commitments.avm_mem_addr = commitment_key->commit(key->avm_mem_addr);
    witness_commitments.avm_mem_clk = commitment_key->commit(key->avm_mem_clk);
    witness_commitments.avm_mem_ind_op_a = commitment_key->commit(key->avm_mem_ind_op_a);
    witness_commitments.avm_mem_ind_op_b = commitment_key->commit(key->avm_mem_ind_op_b);
    witness_commitments.avm_mem_ind_op_c = commitment_key->commit(key->avm_mem_ind_op_c);
    witness_commitments.avm_mem_last = commitment_key->commit(key->avm_mem_last);
    witness_commitments.avm_mem_lastAccess = commitment_key->commit(key->avm_mem_lastAccess);
    witness_commitments.avm_mem_one_min_inv = commitment_key->commit(key->avm_mem_one_min_inv);
    witness_commitments.avm_mem_op_a = commitment_key->commit(key->avm_mem_op_a);
    witness_commitments.avm_mem_op_b = commitment_key->commit(key->avm_mem_op_b);
    witness_commitments.avm_mem_op_c = commitment_key->commit(key->avm_mem_op_c);
    witness_commitments.avm_mem_r_in_tag = commitment_key->commit(key->avm_mem_r_in_tag);
    witness_commitments.avm_mem_rw = commitment_key->commit(key->avm_mem_rw);
    witness_commitments.avm_mem_sel_mov = commitment_key->commit(key->avm_mem_sel_mov);
    witness_commitments.avm_mem_sub_clk = commitment_key->commit(key->avm_mem_sub_clk);
    witness_commitments.avm_mem_tag = commitment_key->commit(key->avm_mem_tag);
    witness_commitments.avm_mem_tag_err = commitment_key->commit(key->avm_mem_tag_err);
    witness_commitments.avm_mem_val = commitment_key->commit(key->avm_mem_val);
    witness_commitments.avm_mem_w_in_tag = commitment_key->commit(key->avm_mem_w_in_tag);
    witness_commitments.lookup_byte_lengths_counts = commitment_key->commit(key->lookup_byte_lengths_counts);
    witness_commitments.lookup_byte_operations_counts = commitment_key->commit(key->lookup_byte_operations_counts);
    witness_commitments.incl_main_tag_err_counts = commitment_key->commit(key->incl_main_tag_err_counts);
    witness_commitments.incl_mem_tag_err_counts = commitment_key->commit(key->incl_mem_tag_err_counts);
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

    // Send all commitments to the verifier
    transcript->send_to_verifier(commitment_labels.avm_alu_a_hi, witness_commitments.avm_alu_a_hi);
    transcript->send_to_verifier(commitment_labels.avm_alu_a_lo, witness_commitments.avm_alu_a_lo);
    transcript->send_to_verifier(commitment_labels.avm_alu_alu_sel, witness_commitments.avm_alu_alu_sel);
    transcript->send_to_verifier(commitment_labels.avm_alu_b_hi, witness_commitments.avm_alu_b_hi);
    transcript->send_to_verifier(commitment_labels.avm_alu_b_lo, witness_commitments.avm_alu_b_lo);
    transcript->send_to_verifier(commitment_labels.avm_alu_borrow, witness_commitments.avm_alu_borrow);
    transcript->send_to_verifier(commitment_labels.avm_alu_cf, witness_commitments.avm_alu_cf);
    transcript->send_to_verifier(commitment_labels.avm_alu_clk, witness_commitments.avm_alu_clk);
    transcript->send_to_verifier(commitment_labels.avm_alu_cmp_rng_ctr, witness_commitments.avm_alu_cmp_rng_ctr);
    transcript->send_to_verifier(commitment_labels.avm_alu_cmp_sel, witness_commitments.avm_alu_cmp_sel);
    transcript->send_to_verifier(commitment_labels.avm_alu_ff_tag, witness_commitments.avm_alu_ff_tag);
    transcript->send_to_verifier(commitment_labels.avm_alu_ia, witness_commitments.avm_alu_ia);
    transcript->send_to_verifier(commitment_labels.avm_alu_ib, witness_commitments.avm_alu_ib);
    transcript->send_to_verifier(commitment_labels.avm_alu_ic, witness_commitments.avm_alu_ic);
    transcript->send_to_verifier(commitment_labels.avm_alu_in_tag, witness_commitments.avm_alu_in_tag);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_add, witness_commitments.avm_alu_op_add);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_div, witness_commitments.avm_alu_op_div);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_eq, witness_commitments.avm_alu_op_eq);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_eq_diff_inv, witness_commitments.avm_alu_op_eq_diff_inv);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_lt, witness_commitments.avm_alu_op_lt);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_lte, witness_commitments.avm_alu_op_lte);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_mul, witness_commitments.avm_alu_op_mul);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_not, witness_commitments.avm_alu_op_not);
    transcript->send_to_verifier(commitment_labels.avm_alu_op_sub, witness_commitments.avm_alu_op_sub);
    transcript->send_to_verifier(commitment_labels.avm_alu_p_a_borrow, witness_commitments.avm_alu_p_a_borrow);
    transcript->send_to_verifier(commitment_labels.avm_alu_p_b_borrow, witness_commitments.avm_alu_p_b_borrow);
    transcript->send_to_verifier(commitment_labels.avm_alu_p_sub_a_hi, witness_commitments.avm_alu_p_sub_a_hi);
    transcript->send_to_verifier(commitment_labels.avm_alu_p_sub_a_lo, witness_commitments.avm_alu_p_sub_a_lo);
    transcript->send_to_verifier(commitment_labels.avm_alu_p_sub_b_hi, witness_commitments.avm_alu_p_sub_b_hi);
    transcript->send_to_verifier(commitment_labels.avm_alu_p_sub_b_lo, witness_commitments.avm_alu_p_sub_b_lo);
    transcript->send_to_verifier(commitment_labels.avm_alu_res_hi, witness_commitments.avm_alu_res_hi);
    transcript->send_to_verifier(commitment_labels.avm_alu_res_lo, witness_commitments.avm_alu_res_lo);
    transcript->send_to_verifier(commitment_labels.avm_alu_rng_chk_lookup_selector,
                                 witness_commitments.avm_alu_rng_chk_lookup_selector);
    transcript->send_to_verifier(commitment_labels.avm_alu_rng_chk_sel, witness_commitments.avm_alu_rng_chk_sel);
    transcript->send_to_verifier(commitment_labels.avm_alu_u128_tag, witness_commitments.avm_alu_u128_tag);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r0, witness_commitments.avm_alu_u16_r0);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r1, witness_commitments.avm_alu_u16_r1);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r10, witness_commitments.avm_alu_u16_r10);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r11, witness_commitments.avm_alu_u16_r11);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r12, witness_commitments.avm_alu_u16_r12);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r13, witness_commitments.avm_alu_u16_r13);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r14, witness_commitments.avm_alu_u16_r14);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r2, witness_commitments.avm_alu_u16_r2);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r3, witness_commitments.avm_alu_u16_r3);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r4, witness_commitments.avm_alu_u16_r4);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r5, witness_commitments.avm_alu_u16_r5);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r6, witness_commitments.avm_alu_u16_r6);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r7, witness_commitments.avm_alu_u16_r7);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r8, witness_commitments.avm_alu_u16_r8);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_r9, witness_commitments.avm_alu_u16_r9);
    transcript->send_to_verifier(commitment_labels.avm_alu_u16_tag, witness_commitments.avm_alu_u16_tag);
    transcript->send_to_verifier(commitment_labels.avm_alu_u32_tag, witness_commitments.avm_alu_u32_tag);
    transcript->send_to_verifier(commitment_labels.avm_alu_u64_r0, witness_commitments.avm_alu_u64_r0);
    transcript->send_to_verifier(commitment_labels.avm_alu_u64_tag, witness_commitments.avm_alu_u64_tag);
    transcript->send_to_verifier(commitment_labels.avm_alu_u8_r0, witness_commitments.avm_alu_u8_r0);
    transcript->send_to_verifier(commitment_labels.avm_alu_u8_r1, witness_commitments.avm_alu_u8_r1);
    transcript->send_to_verifier(commitment_labels.avm_alu_u8_tag, witness_commitments.avm_alu_u8_tag);
    transcript->send_to_verifier(commitment_labels.avm_binary_acc_ia, witness_commitments.avm_binary_acc_ia);
    transcript->send_to_verifier(commitment_labels.avm_binary_acc_ib, witness_commitments.avm_binary_acc_ib);
    transcript->send_to_verifier(commitment_labels.avm_binary_acc_ic, witness_commitments.avm_binary_acc_ic);
    transcript->send_to_verifier(commitment_labels.avm_binary_bin_sel, witness_commitments.avm_binary_bin_sel);
    transcript->send_to_verifier(commitment_labels.avm_binary_clk, witness_commitments.avm_binary_clk);
    transcript->send_to_verifier(commitment_labels.avm_binary_ia_bytes, witness_commitments.avm_binary_ia_bytes);
    transcript->send_to_verifier(commitment_labels.avm_binary_ib_bytes, witness_commitments.avm_binary_ib_bytes);
    transcript->send_to_verifier(commitment_labels.avm_binary_ic_bytes, witness_commitments.avm_binary_ic_bytes);
    transcript->send_to_verifier(commitment_labels.avm_binary_in_tag, witness_commitments.avm_binary_in_tag);
    transcript->send_to_verifier(commitment_labels.avm_binary_mem_tag_ctr, witness_commitments.avm_binary_mem_tag_ctr);
    transcript->send_to_verifier(commitment_labels.avm_binary_mem_tag_ctr_inv,
                                 witness_commitments.avm_binary_mem_tag_ctr_inv);
    transcript->send_to_verifier(commitment_labels.avm_binary_op_id, witness_commitments.avm_binary_op_id);
    transcript->send_to_verifier(commitment_labels.avm_binary_start, witness_commitments.avm_binary_start);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_bin_sel,
                                 witness_commitments.avm_byte_lookup_bin_sel);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_table_byte_lengths,
                                 witness_commitments.avm_byte_lookup_table_byte_lengths);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_table_in_tags,
                                 witness_commitments.avm_byte_lookup_table_in_tags);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_table_input_a,
                                 witness_commitments.avm_byte_lookup_table_input_a);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_table_input_b,
                                 witness_commitments.avm_byte_lookup_table_input_b);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_table_op_id,
                                 witness_commitments.avm_byte_lookup_table_op_id);
    transcript->send_to_verifier(commitment_labels.avm_byte_lookup_table_output,
                                 witness_commitments.avm_byte_lookup_table_output);
    transcript->send_to_verifier(commitment_labels.avm_main_alu_sel, witness_commitments.avm_main_alu_sel);
    transcript->send_to_verifier(commitment_labels.avm_main_bin_op_id, witness_commitments.avm_main_bin_op_id);
    transcript->send_to_verifier(commitment_labels.avm_main_bin_sel, witness_commitments.avm_main_bin_sel);
    transcript->send_to_verifier(commitment_labels.avm_main_ia, witness_commitments.avm_main_ia);
    transcript->send_to_verifier(commitment_labels.avm_main_ib, witness_commitments.avm_main_ib);
    transcript->send_to_verifier(commitment_labels.avm_main_ic, witness_commitments.avm_main_ic);
    transcript->send_to_verifier(commitment_labels.avm_main_ind_a, witness_commitments.avm_main_ind_a);
    transcript->send_to_verifier(commitment_labels.avm_main_ind_b, witness_commitments.avm_main_ind_b);
    transcript->send_to_verifier(commitment_labels.avm_main_ind_c, witness_commitments.avm_main_ind_c);
    transcript->send_to_verifier(commitment_labels.avm_main_ind_op_a, witness_commitments.avm_main_ind_op_a);
    transcript->send_to_verifier(commitment_labels.avm_main_ind_op_b, witness_commitments.avm_main_ind_op_b);
    transcript->send_to_verifier(commitment_labels.avm_main_ind_op_c, witness_commitments.avm_main_ind_op_c);
    transcript->send_to_verifier(commitment_labels.avm_main_internal_return_ptr,
                                 witness_commitments.avm_main_internal_return_ptr);
    transcript->send_to_verifier(commitment_labels.avm_main_inv, witness_commitments.avm_main_inv);
    transcript->send_to_verifier(commitment_labels.avm_main_last, witness_commitments.avm_main_last);
    transcript->send_to_verifier(commitment_labels.avm_main_mem_idx_a, witness_commitments.avm_main_mem_idx_a);
    transcript->send_to_verifier(commitment_labels.avm_main_mem_idx_b, witness_commitments.avm_main_mem_idx_b);
    transcript->send_to_verifier(commitment_labels.avm_main_mem_idx_c, witness_commitments.avm_main_mem_idx_c);
    transcript->send_to_verifier(commitment_labels.avm_main_mem_op_a, witness_commitments.avm_main_mem_op_a);
    transcript->send_to_verifier(commitment_labels.avm_main_mem_op_b, witness_commitments.avm_main_mem_op_b);
    transcript->send_to_verifier(commitment_labels.avm_main_mem_op_c, witness_commitments.avm_main_mem_op_c);
    transcript->send_to_verifier(commitment_labels.avm_main_op_err, witness_commitments.avm_main_op_err);
    transcript->send_to_verifier(commitment_labels.avm_main_pc, witness_commitments.avm_main_pc);
    transcript->send_to_verifier(commitment_labels.avm_main_r_in_tag, witness_commitments.avm_main_r_in_tag);
    transcript->send_to_verifier(commitment_labels.avm_main_rwa, witness_commitments.avm_main_rwa);
    transcript->send_to_verifier(commitment_labels.avm_main_rwb, witness_commitments.avm_main_rwb);
    transcript->send_to_verifier(commitment_labels.avm_main_rwc, witness_commitments.avm_main_rwc);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_halt, witness_commitments.avm_main_sel_halt);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_internal_call,
                                 witness_commitments.avm_main_sel_internal_call);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_internal_return,
                                 witness_commitments.avm_main_sel_internal_return);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_jump, witness_commitments.avm_main_sel_jump);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_mov, witness_commitments.avm_main_sel_mov);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_add, witness_commitments.avm_main_sel_op_add);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_and, witness_commitments.avm_main_sel_op_and);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_div, witness_commitments.avm_main_sel_op_div);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_eq, witness_commitments.avm_main_sel_op_eq);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_lt, witness_commitments.avm_main_sel_op_lt);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_lte, witness_commitments.avm_main_sel_op_lte);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_mul, witness_commitments.avm_main_sel_op_mul);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_not, witness_commitments.avm_main_sel_op_not);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_or, witness_commitments.avm_main_sel_op_or);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_sub, witness_commitments.avm_main_sel_op_sub);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_op_xor, witness_commitments.avm_main_sel_op_xor);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_rng_16, witness_commitments.avm_main_sel_rng_16);
    transcript->send_to_verifier(commitment_labels.avm_main_sel_rng_8, witness_commitments.avm_main_sel_rng_8);
    transcript->send_to_verifier(commitment_labels.avm_main_tag_err, witness_commitments.avm_main_tag_err);
    transcript->send_to_verifier(commitment_labels.avm_main_w_in_tag, witness_commitments.avm_main_w_in_tag);
    transcript->send_to_verifier(commitment_labels.avm_mem_addr, witness_commitments.avm_mem_addr);
    transcript->send_to_verifier(commitment_labels.avm_mem_clk, witness_commitments.avm_mem_clk);
    transcript->send_to_verifier(commitment_labels.avm_mem_ind_op_a, witness_commitments.avm_mem_ind_op_a);
    transcript->send_to_verifier(commitment_labels.avm_mem_ind_op_b, witness_commitments.avm_mem_ind_op_b);
    transcript->send_to_verifier(commitment_labels.avm_mem_ind_op_c, witness_commitments.avm_mem_ind_op_c);
    transcript->send_to_verifier(commitment_labels.avm_mem_last, witness_commitments.avm_mem_last);
    transcript->send_to_verifier(commitment_labels.avm_mem_lastAccess, witness_commitments.avm_mem_lastAccess);
    transcript->send_to_verifier(commitment_labels.avm_mem_one_min_inv, witness_commitments.avm_mem_one_min_inv);
    transcript->send_to_verifier(commitment_labels.avm_mem_op_a, witness_commitments.avm_mem_op_a);
    transcript->send_to_verifier(commitment_labels.avm_mem_op_b, witness_commitments.avm_mem_op_b);
    transcript->send_to_verifier(commitment_labels.avm_mem_op_c, witness_commitments.avm_mem_op_c);
    transcript->send_to_verifier(commitment_labels.avm_mem_r_in_tag, witness_commitments.avm_mem_r_in_tag);
    transcript->send_to_verifier(commitment_labels.avm_mem_rw, witness_commitments.avm_mem_rw);
    transcript->send_to_verifier(commitment_labels.avm_mem_sel_mov, witness_commitments.avm_mem_sel_mov);
    transcript->send_to_verifier(commitment_labels.avm_mem_sub_clk, witness_commitments.avm_mem_sub_clk);
    transcript->send_to_verifier(commitment_labels.avm_mem_tag, witness_commitments.avm_mem_tag);
    transcript->send_to_verifier(commitment_labels.avm_mem_tag_err, witness_commitments.avm_mem_tag_err);
    transcript->send_to_verifier(commitment_labels.avm_mem_val, witness_commitments.avm_mem_val);
    transcript->send_to_verifier(commitment_labels.avm_mem_w_in_tag, witness_commitments.avm_mem_w_in_tag);
    transcript->send_to_verifier(commitment_labels.lookup_byte_lengths_counts,
                                 witness_commitments.lookup_byte_lengths_counts);
    transcript->send_to_verifier(commitment_labels.lookup_byte_operations_counts,
                                 witness_commitments.lookup_byte_operations_counts);
    transcript->send_to_verifier(commitment_labels.incl_main_tag_err_counts,
                                 witness_commitments.incl_main_tag_err_counts);
    transcript->send_to_verifier(commitment_labels.incl_mem_tag_err_counts,
                                 witness_commitments.incl_mem_tag_err_counts);
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
    witness_commitments.perm_main_mem_a = commitment_key->commit(key->perm_main_mem_a);
    witness_commitments.perm_main_mem_b = commitment_key->commit(key->perm_main_mem_b);
    witness_commitments.perm_main_mem_c = commitment_key->commit(key->perm_main_mem_c);
    witness_commitments.perm_main_mem_ind_a = commitment_key->commit(key->perm_main_mem_ind_a);
    witness_commitments.perm_main_mem_ind_b = commitment_key->commit(key->perm_main_mem_ind_b);
    witness_commitments.perm_main_mem_ind_c = commitment_key->commit(key->perm_main_mem_ind_c);
    witness_commitments.lookup_byte_lengths = commitment_key->commit(key->lookup_byte_lengths);
    witness_commitments.lookup_byte_operations = commitment_key->commit(key->lookup_byte_operations);
    witness_commitments.incl_main_tag_err = commitment_key->commit(key->incl_main_tag_err);
    witness_commitments.incl_mem_tag_err = commitment_key->commit(key->incl_mem_tag_err);
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

    // Send all commitments to the verifier
    transcript->send_to_verifier(commitment_labels.perm_main_alu, witness_commitments.perm_main_alu);
    transcript->send_to_verifier(commitment_labels.perm_main_bin, witness_commitments.perm_main_bin);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_a, witness_commitments.perm_main_mem_a);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_b, witness_commitments.perm_main_mem_b);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_c, witness_commitments.perm_main_mem_c);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_a, witness_commitments.perm_main_mem_ind_a);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_b, witness_commitments.perm_main_mem_ind_b);
    transcript->send_to_verifier(commitment_labels.perm_main_mem_ind_c, witness_commitments.perm_main_mem_ind_c);
    transcript->send_to_verifier(commitment_labels.lookup_byte_lengths, witness_commitments.lookup_byte_lengths);
    transcript->send_to_verifier(commitment_labels.lookup_byte_operations, witness_commitments.lookup_byte_operations);
    transcript->send_to_verifier(commitment_labels.incl_main_tag_err, witness_commitments.incl_main_tag_err);
    transcript->send_to_verifier(commitment_labels.incl_mem_tag_err, witness_commitments.incl_mem_tag_err);
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
void AvmProver::execute_zeromorph_rounds()
{
    ZeroMorph::prove(prover_polynomials.get_unshifted(),
                     prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript);
}

HonkProof& AvmProver::export_proof()
{
    proof = transcript->proof_data;
    return proof;
}

HonkProof& AvmProver::construct_proof()
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
    execute_zeromorph_rounds();

    return export_proof();
}

} // namespace bb
