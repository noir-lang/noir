#pragma once

#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript.hpp"

#include "barretenberg/vm/avm_trace/stats.hpp"

// Relations
#include "barretenberg/relations/generated/avm/alu.hpp"
#include "barretenberg/relations/generated/avm/binary.hpp"
#include "barretenberg/relations/generated/avm/conversion.hpp"
#include "barretenberg/relations/generated/avm/gas.hpp"
#include "barretenberg/relations/generated/avm/keccakf1600.hpp"
#include "barretenberg/relations/generated/avm/kernel.hpp"
#include "barretenberg/relations/generated/avm/main.hpp"
#include "barretenberg/relations/generated/avm/mem.hpp"
#include "barretenberg/relations/generated/avm/pedersen.hpp"
#include "barretenberg/relations/generated/avm/poseidon2.hpp"
#include "barretenberg/relations/generated/avm/powers.hpp"
#include "barretenberg/relations/generated/avm/sha256.hpp"

// Lookup relations
#include "barretenberg/relations/generated/avm/incl_main_tag_err.hpp"
#include "barretenberg/relations/generated/avm/incl_mem_tag_err.hpp"
#include "barretenberg/relations/generated/avm/kernel_output_lookup.hpp"
#include "barretenberg/relations/generated/avm/lookup_byte_lengths.hpp"
#include "barretenberg/relations/generated/avm/lookup_byte_operations.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_0.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_1.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_2.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_3.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_4.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_5.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_6.hpp"
#include "barretenberg/relations/generated/avm/lookup_div_u16_7.hpp"
#include "barretenberg/relations/generated/avm/lookup_into_kernel.hpp"
#include "barretenberg/relations/generated/avm/lookup_mem_rng_chk_hi.hpp"
#include "barretenberg/relations/generated/avm/lookup_mem_rng_chk_lo.hpp"
#include "barretenberg/relations/generated/avm/lookup_mem_rng_chk_mid.hpp"
#include "barretenberg/relations/generated/avm/lookup_opcode_gas.hpp"
#include "barretenberg/relations/generated/avm/lookup_pow_2_0.hpp"
#include "barretenberg/relations/generated/avm/lookup_pow_2_1.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_0.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_1.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_10.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_11.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_12.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_13.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_14.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_2.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_3.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_4.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_5.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_6.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_7.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_8.hpp"
#include "barretenberg/relations/generated/avm/lookup_u16_9.hpp"
#include "barretenberg/relations/generated/avm/lookup_u8_0.hpp"
#include "barretenberg/relations/generated/avm/lookup_u8_1.hpp"
#include "barretenberg/relations/generated/avm/perm_main_alu.hpp"
#include "barretenberg/relations/generated/avm/perm_main_bin.hpp"
#include "barretenberg/relations/generated/avm/perm_main_conv.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_a.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_b.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_c.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_d.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_addr_a.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_addr_b.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_addr_c.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_addr_d.hpp"
#include "barretenberg/relations/generated/avm/perm_main_pedersen.hpp"
#include "barretenberg/relations/generated/avm/perm_main_pos2_perm.hpp"
#include "barretenberg/relations/generated/avm/range_check_da_gas_hi.hpp"
#include "barretenberg/relations/generated/avm/range_check_da_gas_lo.hpp"
#include "barretenberg/relations/generated/avm/range_check_l2_gas_hi.hpp"
#include "barretenberg/relations/generated/avm/range_check_l2_gas_lo.hpp"
#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

namespace bb {

class AvmFlavor {
  public:
    using Curve = curve::BN254;
    using G1 = Curve::Group;
    using PCS = KZG<Curve>;

    using FF = G1::subgroup_field;
    using Polynomial = bb::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = G1::element;
    using Commitment = G1::affine_element;
    using CommitmentHandle = G1::affine_element;
    using CommitmentKey = bb::CommitmentKey<Curve>;
    using VerifierCommitmentKey = bb::VerifierCommitmentKey<Curve>;
    using RelationSeparator = FF;

    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 2;
    static constexpr size_t NUM_WITNESS_ENTITIES = 385;
    static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
    // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for
    // the unshifted and one for the shifted
    static constexpr size_t NUM_ALL_ENTITIES = 452;

    using Relations = std::tuple<
        // Relations
        Avm_vm::alu<FF>,
        Avm_vm::binary<FF>,
        Avm_vm::conversion<FF>,
        Avm_vm::gas<FF>,
        Avm_vm::keccakf1600<FF>,
        Avm_vm::kernel<FF>,
        Avm_vm::main<FF>,
        Avm_vm::mem<FF>,
        Avm_vm::pedersen<FF>,
        Avm_vm::poseidon2<FF>,
        Avm_vm::powers<FF>,
        Avm_vm::sha256<FF>,
        // Lookups
        perm_main_alu_relation<FF>,
        perm_main_bin_relation<FF>,
        perm_main_conv_relation<FF>,
        perm_main_pos2_perm_relation<FF>,
        perm_main_pedersen_relation<FF>,
        perm_main_mem_a_relation<FF>,
        perm_main_mem_b_relation<FF>,
        perm_main_mem_c_relation<FF>,
        perm_main_mem_d_relation<FF>,
        perm_main_mem_ind_addr_a_relation<FF>,
        perm_main_mem_ind_addr_b_relation<FF>,
        perm_main_mem_ind_addr_c_relation<FF>,
        perm_main_mem_ind_addr_d_relation<FF>,
        lookup_byte_lengths_relation<FF>,
        lookup_byte_operations_relation<FF>,
        lookup_opcode_gas_relation<FF>,
        range_check_l2_gas_hi_relation<FF>,
        range_check_l2_gas_lo_relation<FF>,
        range_check_da_gas_hi_relation<FF>,
        range_check_da_gas_lo_relation<FF>,
        kernel_output_lookup_relation<FF>,
        lookup_into_kernel_relation<FF>,
        incl_main_tag_err_relation<FF>,
        incl_mem_tag_err_relation<FF>,
        lookup_mem_rng_chk_lo_relation<FF>,
        lookup_mem_rng_chk_mid_relation<FF>,
        lookup_mem_rng_chk_hi_relation<FF>,
        lookup_pow_2_0_relation<FF>,
        lookup_pow_2_1_relation<FF>,
        lookup_u8_0_relation<FF>,
        lookup_u8_1_relation<FF>,
        lookup_u16_0_relation<FF>,
        lookup_u16_1_relation<FF>,
        lookup_u16_2_relation<FF>,
        lookup_u16_3_relation<FF>,
        lookup_u16_4_relation<FF>,
        lookup_u16_5_relation<FF>,
        lookup_u16_6_relation<FF>,
        lookup_u16_7_relation<FF>,
        lookup_u16_8_relation<FF>,
        lookup_u16_9_relation<FF>,
        lookup_u16_10_relation<FF>,
        lookup_u16_11_relation<FF>,
        lookup_u16_12_relation<FF>,
        lookup_u16_13_relation<FF>,
        lookup_u16_14_relation<FF>,
        lookup_div_u16_0_relation<FF>,
        lookup_div_u16_1_relation<FF>,
        lookup_div_u16_2_relation<FF>,
        lookup_div_u16_3_relation<FF>,
        lookup_div_u16_4_relation<FF>,
        lookup_div_u16_5_relation<FF>,
        lookup_div_u16_6_relation<FF>,
        lookup_div_u16_7_relation<FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size_v<Relations>;

    template <size_t NUM_INSTANCES>
    using ProtogalaxyTupleOfTuplesOfUnivariates =
        decltype(create_protogalaxy_tuple_of_tuples_of_univariates<Relations, NUM_INSTANCES>());
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

    static constexpr bool has_zero_row = true;

  private:
    template <typename DataType_> class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        using DataType = DataType_;

        DEFINE_FLAVOR_MEMBERS(DataType, main_clk, main_sel_first)

        RefVector<DataType> get_selectors() { return { main_clk, main_sel_first }; }
        RefVector<DataType> get_sigma_polynomials() { return {}; }
        RefVector<DataType> get_id_polynomials() { return {}; }
        RefVector<DataType> get_table_polynomials() { return {}; }
    };

    template <typename DataType> class WireEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              kernel_kernel_inputs,
                              kernel_kernel_value_out,
                              kernel_kernel_side_effect_out,
                              kernel_kernel_metadata_out,
                              main_calldata,
                              alu_a_hi,
                              alu_a_lo,
                              alu_b_hi,
                              alu_b_lo,
                              alu_borrow,
                              alu_cf,
                              alu_clk,
                              alu_cmp_rng_ctr,
                              alu_div_u16_r0,
                              alu_div_u16_r1,
                              alu_div_u16_r2,
                              alu_div_u16_r3,
                              alu_div_u16_r4,
                              alu_div_u16_r5,
                              alu_div_u16_r6,
                              alu_div_u16_r7,
                              alu_divisor_hi,
                              alu_divisor_lo,
                              alu_ff_tag,
                              alu_ia,
                              alu_ib,
                              alu_ic,
                              alu_in_tag,
                              alu_op_add,
                              alu_op_cast,
                              alu_op_cast_prev,
                              alu_op_div,
                              alu_op_div_a_lt_b,
                              alu_op_div_std,
                              alu_op_eq,
                              alu_op_eq_diff_inv,
                              alu_op_lt,
                              alu_op_lte,
                              alu_op_mul,
                              alu_op_not,
                              alu_op_shl,
                              alu_op_shr,
                              alu_op_sub,
                              alu_p_a_borrow,
                              alu_p_b_borrow,
                              alu_p_sub_a_hi,
                              alu_p_sub_a_lo,
                              alu_p_sub_b_hi,
                              alu_p_sub_b_lo,
                              alu_partial_prod_hi,
                              alu_partial_prod_lo,
                              alu_quotient_hi,
                              alu_quotient_lo,
                              alu_remainder,
                              alu_res_hi,
                              alu_res_lo,
                              alu_sel_alu,
                              alu_sel_cmp,
                              alu_sel_div_rng_chk,
                              alu_sel_rng_chk,
                              alu_sel_rng_chk_lookup,
                              alu_sel_shift_which,
                              alu_shift_lt_bit_len,
                              alu_t_sub_s_bits,
                              alu_two_pow_s,
                              alu_two_pow_t_sub_s,
                              alu_u128_tag,
                              alu_u16_r0,
                              alu_u16_r1,
                              alu_u16_r10,
                              alu_u16_r11,
                              alu_u16_r12,
                              alu_u16_r13,
                              alu_u16_r14,
                              alu_u16_r2,
                              alu_u16_r3,
                              alu_u16_r4,
                              alu_u16_r5,
                              alu_u16_r6,
                              alu_u16_r7,
                              alu_u16_r8,
                              alu_u16_r9,
                              alu_u16_tag,
                              alu_u32_tag,
                              alu_u64_tag,
                              alu_u8_r0,
                              alu_u8_r1,
                              alu_u8_tag,
                              binary_acc_ia,
                              binary_acc_ib,
                              binary_acc_ic,
                              binary_clk,
                              binary_ia_bytes,
                              binary_ib_bytes,
                              binary_ic_bytes,
                              binary_in_tag,
                              binary_mem_tag_ctr,
                              binary_mem_tag_ctr_inv,
                              binary_op_id,
                              binary_sel_bin,
                              binary_start,
                              byte_lookup_sel_bin,
                              byte_lookup_table_byte_lengths,
                              byte_lookup_table_in_tags,
                              byte_lookup_table_input_a,
                              byte_lookup_table_input_b,
                              byte_lookup_table_op_id,
                              byte_lookup_table_output,
                              conversion_clk,
                              conversion_input,
                              conversion_num_limbs,
                              conversion_radix,
                              conversion_sel_to_radix_le,
                              gas_da_gas_fixed_table,
                              gas_l2_gas_fixed_table,
                              gas_sel_gas_cost,
                              keccakf1600_clk,
                              keccakf1600_input,
                              keccakf1600_output,
                              keccakf1600_sel_keccakf1600,
                              kernel_emit_l2_to_l1_msg_write_offset,
                              kernel_emit_note_hash_write_offset,
                              kernel_emit_nullifier_write_offset,
                              kernel_emit_unencrypted_log_write_offset,
                              kernel_kernel_in_offset,
                              kernel_kernel_out_offset,
                              kernel_l1_to_l2_msg_exists_write_offset,
                              kernel_note_hash_exist_write_offset,
                              kernel_nullifier_exists_write_offset,
                              kernel_nullifier_non_exists_write_offset,
                              kernel_q_public_input_kernel_add_to_table,
                              kernel_q_public_input_kernel_out_add_to_table,
                              kernel_side_effect_counter,
                              kernel_sload_write_offset,
                              kernel_sstore_write_offset,
                              main_abs_da_rem_gas_hi,
                              main_abs_da_rem_gas_lo,
                              main_abs_l2_rem_gas_hi,
                              main_abs_l2_rem_gas_lo,
                              main_alu_in_tag,
                              main_bin_op_id,
                              main_call_ptr,
                              main_da_gas_op_cost,
                              main_da_gas_remaining,
                              main_da_out_of_gas,
                              main_ia,
                              main_ib,
                              main_ic,
                              main_id,
                              main_id_zero,
                              main_ind_addr_a,
                              main_ind_addr_b,
                              main_ind_addr_c,
                              main_ind_addr_d,
                              main_internal_return_ptr,
                              main_inv,
                              main_l2_gas_op_cost,
                              main_l2_gas_remaining,
                              main_l2_out_of_gas,
                              main_mem_addr_a,
                              main_mem_addr_b,
                              main_mem_addr_c,
                              main_mem_addr_d,
                              main_op_err,
                              main_opcode_val,
                              main_pc,
                              main_r_in_tag,
                              main_rwa,
                              main_rwb,
                              main_rwc,
                              main_rwd,
                              main_sel_alu,
                              main_sel_bin,
                              main_sel_gas_accounting_active,
                              main_sel_last,
                              main_sel_mem_op_a,
                              main_sel_mem_op_activate_gas,
                              main_sel_mem_op_b,
                              main_sel_mem_op_c,
                              main_sel_mem_op_d,
                              main_sel_mov_ia_to_ic,
                              main_sel_mov_ib_to_ic,
                              main_sel_op_add,
                              main_sel_op_address,
                              main_sel_op_and,
                              main_sel_op_block_number,
                              main_sel_op_cast,
                              main_sel_op_chain_id,
                              main_sel_op_cmov,
                              main_sel_op_coinbase,
                              main_sel_op_dagasleft,
                              main_sel_op_div,
                              main_sel_op_emit_l2_to_l1_msg,
                              main_sel_op_emit_note_hash,
                              main_sel_op_emit_nullifier,
                              main_sel_op_emit_unencrypted_log,
                              main_sel_op_eq,
                              main_sel_op_external_call,
                              main_sel_op_fdiv,
                              main_sel_op_fee_per_da_gas,
                              main_sel_op_fee_per_l2_gas,
                              main_sel_op_function_selector,
                              main_sel_op_get_contract_instance,
                              main_sel_op_halt,
                              main_sel_op_internal_call,
                              main_sel_op_internal_return,
                              main_sel_op_jump,
                              main_sel_op_jumpi,
                              main_sel_op_keccak,
                              main_sel_op_l1_to_l2_msg_exists,
                              main_sel_op_l2gasleft,
                              main_sel_op_lt,
                              main_sel_op_lte,
                              main_sel_op_mov,
                              main_sel_op_mul,
                              main_sel_op_not,
                              main_sel_op_note_hash_exists,
                              main_sel_op_nullifier_exists,
                              main_sel_op_or,
                              main_sel_op_pedersen,
                              main_sel_op_poseidon2,
                              main_sel_op_radix_le,
                              main_sel_op_sender,
                              main_sel_op_sha256,
                              main_sel_op_shl,
                              main_sel_op_shr,
                              main_sel_op_sload,
                              main_sel_op_sstore,
                              main_sel_op_storage_address,
                              main_sel_op_sub,
                              main_sel_op_timestamp,
                              main_sel_op_transaction_fee,
                              main_sel_op_version,
                              main_sel_op_xor,
                              main_sel_q_kernel_lookup,
                              main_sel_q_kernel_output_lookup,
                              main_sel_resolve_ind_addr_a,
                              main_sel_resolve_ind_addr_b,
                              main_sel_resolve_ind_addr_c,
                              main_sel_resolve_ind_addr_d,
                              main_sel_rng_16,
                              main_sel_rng_8,
                              main_space_id,
                              main_tag_err,
                              main_w_in_tag,
                              mem_addr,
                              mem_clk,
                              mem_diff_hi,
                              mem_diff_lo,
                              mem_diff_mid,
                              mem_glob_addr,
                              mem_last,
                              mem_lastAccess,
                              mem_one_min_inv,
                              mem_r_in_tag,
                              mem_rw,
                              mem_sel_mem,
                              mem_sel_mov_ia_to_ic,
                              mem_sel_mov_ib_to_ic,
                              mem_sel_op_a,
                              mem_sel_op_b,
                              mem_sel_op_c,
                              mem_sel_op_cmov,
                              mem_sel_op_d,
                              mem_sel_resolve_ind_addr_a,
                              mem_sel_resolve_ind_addr_b,
                              mem_sel_resolve_ind_addr_c,
                              mem_sel_resolve_ind_addr_d,
                              mem_sel_rng_chk,
                              mem_skip_check_tag,
                              mem_space_id,
                              mem_tag,
                              mem_tag_err,
                              mem_tsp,
                              mem_val,
                              mem_w_in_tag,
                              pedersen_clk,
                              pedersen_input,
                              pedersen_output,
                              pedersen_sel_pedersen,
                              poseidon2_clk,
                              poseidon2_input,
                              poseidon2_output,
                              poseidon2_sel_poseidon_perm,
                              powers_power_of_2,
                              sha256_clk,
                              sha256_input,
                              sha256_output,
                              sha256_sel_sha256_compression,
                              sha256_state,
                              lookup_byte_lengths_counts,
                              lookup_byte_operations_counts,
                              lookup_opcode_gas_counts,
                              range_check_l2_gas_hi_counts,
                              range_check_l2_gas_lo_counts,
                              range_check_da_gas_hi_counts,
                              range_check_da_gas_lo_counts,
                              kernel_output_lookup_counts,
                              lookup_into_kernel_counts,
                              incl_main_tag_err_counts,
                              incl_mem_tag_err_counts,
                              lookup_mem_rng_chk_lo_counts,
                              lookup_mem_rng_chk_mid_counts,
                              lookup_mem_rng_chk_hi_counts,
                              lookup_pow_2_0_counts,
                              lookup_pow_2_1_counts,
                              lookup_u8_0_counts,
                              lookup_u8_1_counts,
                              lookup_u16_0_counts,
                              lookup_u16_1_counts,
                              lookup_u16_2_counts,
                              lookup_u16_3_counts,
                              lookup_u16_4_counts,
                              lookup_u16_5_counts,
                              lookup_u16_6_counts,
                              lookup_u16_7_counts,
                              lookup_u16_8_counts,
                              lookup_u16_9_counts,
                              lookup_u16_10_counts,
                              lookup_u16_11_counts,
                              lookup_u16_12_counts,
                              lookup_u16_13_counts,
                              lookup_u16_14_counts,
                              lookup_div_u16_0_counts,
                              lookup_div_u16_1_counts,
                              lookup_div_u16_2_counts,
                              lookup_div_u16_3_counts,
                              lookup_div_u16_4_counts,
                              lookup_div_u16_5_counts,
                              lookup_div_u16_6_counts,
                              lookup_div_u16_7_counts)
    };

    template <typename DataType> class DerivedWitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              perm_main_alu,
                              perm_main_bin,
                              perm_main_conv,
                              perm_main_pos2_perm,
                              perm_main_pedersen,
                              perm_main_mem_a,
                              perm_main_mem_b,
                              perm_main_mem_c,
                              perm_main_mem_d,
                              perm_main_mem_ind_addr_a,
                              perm_main_mem_ind_addr_b,
                              perm_main_mem_ind_addr_c,
                              perm_main_mem_ind_addr_d,
                              lookup_byte_lengths,
                              lookup_byte_operations,
                              lookup_opcode_gas,
                              range_check_l2_gas_hi,
                              range_check_l2_gas_lo,
                              range_check_da_gas_hi,
                              range_check_da_gas_lo,
                              kernel_output_lookup,
                              lookup_into_kernel,
                              incl_main_tag_err,
                              incl_mem_tag_err,
                              lookup_mem_rng_chk_lo,
                              lookup_mem_rng_chk_mid,
                              lookup_mem_rng_chk_hi,
                              lookup_pow_2_0,
                              lookup_pow_2_1,
                              lookup_u8_0,
                              lookup_u8_1,
                              lookup_u16_0,
                              lookup_u16_1,
                              lookup_u16_2,
                              lookup_u16_3,
                              lookup_u16_4,
                              lookup_u16_5,
                              lookup_u16_6,
                              lookup_u16_7,
                              lookup_u16_8,
                              lookup_u16_9,
                              lookup_u16_10,
                              lookup_u16_11,
                              lookup_u16_12,
                              lookup_u16_13,
                              lookup_u16_14,
                              lookup_div_u16_0,
                              lookup_div_u16_1,
                              lookup_div_u16_2,
                              lookup_div_u16_3,
                              lookup_div_u16_4,
                              lookup_div_u16_5,
                              lookup_div_u16_6,
                              lookup_div_u16_7)
    };

    template <typename DataType> class ShiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              alu_a_hi_shift,
                              alu_a_lo_shift,
                              alu_b_hi_shift,
                              alu_b_lo_shift,
                              alu_cmp_rng_ctr_shift,
                              alu_div_u16_r0_shift,
                              alu_div_u16_r1_shift,
                              alu_div_u16_r2_shift,
                              alu_div_u16_r3_shift,
                              alu_div_u16_r4_shift,
                              alu_div_u16_r5_shift,
                              alu_div_u16_r6_shift,
                              alu_div_u16_r7_shift,
                              alu_op_add_shift,
                              alu_op_cast_prev_shift,
                              alu_op_cast_shift,
                              alu_op_div_shift,
                              alu_op_mul_shift,
                              alu_op_shl_shift,
                              alu_op_shr_shift,
                              alu_op_sub_shift,
                              alu_p_sub_a_hi_shift,
                              alu_p_sub_a_lo_shift,
                              alu_p_sub_b_hi_shift,
                              alu_p_sub_b_lo_shift,
                              alu_sel_alu_shift,
                              alu_sel_cmp_shift,
                              alu_sel_div_rng_chk_shift,
                              alu_sel_rng_chk_lookup_shift,
                              alu_sel_rng_chk_shift,
                              alu_u16_r0_shift,
                              alu_u16_r1_shift,
                              alu_u16_r2_shift,
                              alu_u16_r3_shift,
                              alu_u16_r4_shift,
                              alu_u16_r5_shift,
                              alu_u16_r6_shift,
                              alu_u8_r0_shift,
                              alu_u8_r1_shift,
                              binary_acc_ia_shift,
                              binary_acc_ib_shift,
                              binary_acc_ic_shift,
                              binary_mem_tag_ctr_shift,
                              binary_op_id_shift,
                              kernel_emit_l2_to_l1_msg_write_offset_shift,
                              kernel_emit_note_hash_write_offset_shift,
                              kernel_emit_nullifier_write_offset_shift,
                              kernel_emit_unencrypted_log_write_offset_shift,
                              kernel_l1_to_l2_msg_exists_write_offset_shift,
                              kernel_note_hash_exist_write_offset_shift,
                              kernel_nullifier_exists_write_offset_shift,
                              kernel_nullifier_non_exists_write_offset_shift,
                              kernel_side_effect_counter_shift,
                              kernel_sload_write_offset_shift,
                              kernel_sstore_write_offset_shift,
                              main_da_gas_remaining_shift,
                              main_internal_return_ptr_shift,
                              main_l2_gas_remaining_shift,
                              main_pc_shift,
                              mem_glob_addr_shift,
                              mem_rw_shift,
                              mem_sel_mem_shift,
                              mem_tag_shift,
                              mem_tsp_shift,
                              mem_val_shift)
    };

    template <typename DataType, typename PrecomputedAndWitnessEntitiesSuperset>
    static auto get_to_be_shifted(PrecomputedAndWitnessEntitiesSuperset& entities)
    {
        return RefArray{ entities.alu_a_hi,
                         entities.alu_a_lo,
                         entities.alu_b_hi,
                         entities.alu_b_lo,
                         entities.alu_cmp_rng_ctr,
                         entities.alu_div_u16_r0,
                         entities.alu_div_u16_r1,
                         entities.alu_div_u16_r2,
                         entities.alu_div_u16_r3,
                         entities.alu_div_u16_r4,
                         entities.alu_div_u16_r5,
                         entities.alu_div_u16_r6,
                         entities.alu_div_u16_r7,
                         entities.alu_op_add,
                         entities.alu_op_cast_prev,
                         entities.alu_op_cast,
                         entities.alu_op_div,
                         entities.alu_op_mul,
                         entities.alu_op_shl,
                         entities.alu_op_shr,
                         entities.alu_op_sub,
                         entities.alu_p_sub_a_hi,
                         entities.alu_p_sub_a_lo,
                         entities.alu_p_sub_b_hi,
                         entities.alu_p_sub_b_lo,
                         entities.alu_sel_alu,
                         entities.alu_sel_cmp,
                         entities.alu_sel_div_rng_chk,
                         entities.alu_sel_rng_chk_lookup,
                         entities.alu_sel_rng_chk,
                         entities.alu_u16_r0,
                         entities.alu_u16_r1,
                         entities.alu_u16_r2,
                         entities.alu_u16_r3,
                         entities.alu_u16_r4,
                         entities.alu_u16_r5,
                         entities.alu_u16_r6,
                         entities.alu_u8_r0,
                         entities.alu_u8_r1,
                         entities.binary_acc_ia,
                         entities.binary_acc_ib,
                         entities.binary_acc_ic,
                         entities.binary_mem_tag_ctr,
                         entities.binary_op_id,
                         entities.kernel_emit_l2_to_l1_msg_write_offset,
                         entities.kernel_emit_note_hash_write_offset,
                         entities.kernel_emit_nullifier_write_offset,
                         entities.kernel_emit_unencrypted_log_write_offset,
                         entities.kernel_l1_to_l2_msg_exists_write_offset,
                         entities.kernel_note_hash_exist_write_offset,
                         entities.kernel_nullifier_exists_write_offset,
                         entities.kernel_nullifier_non_exists_write_offset,
                         entities.kernel_side_effect_counter,
                         entities.kernel_sload_write_offset,
                         entities.kernel_sstore_write_offset,
                         entities.main_da_gas_remaining,
                         entities.main_internal_return_ptr,
                         entities.main_l2_gas_remaining,
                         entities.main_pc,
                         entities.mem_glob_addr,
                         entities.mem_rw,
                         entities.mem_sel_mem,
                         entities.mem_tag,
                         entities.mem_tsp,
                         entities.mem_val };
    }

    template <typename DataType>
    class WitnessEntities : public WireEntities<DataType>, public DerivedWitnessEntities<DataType> {
      public:
        DEFINE_COMPOUND_GET_ALL(WireEntities<DataType>, DerivedWitnessEntities<DataType>)
        auto get_wires() { return WireEntities<DataType>::get_all(); };
    };

    template <typename DataType>
    class AllEntities : public PrecomputedEntities<DataType>,
                        public WitnessEntities<DataType>,
                        public ShiftedEntities<DataType> {
      public:
        AllEntities()
            : PrecomputedEntities<DataType>{}
            , WitnessEntities<DataType>{}
            , ShiftedEntities<DataType>{}
        {}

        DEFINE_COMPOUND_GET_ALL(PrecomputedEntities<DataType>, WitnessEntities<DataType>, ShiftedEntities<DataType>)

        auto get_unshifted()
        {
            return concatenate(PrecomputedEntities<DataType>::get_all(), WitnessEntities<DataType>::get_all());
        }
        auto get_to_be_shifted() { return AvmFlavor::get_to_be_shifted<DataType>(*this); }
        auto get_shifted() { return ShiftedEntities<DataType>::get_all(); }
        auto get_precomputed() { return PrecomputedEntities<DataType>::get_all(); }
    };

  public:
    class ProvingKey
        : public ProvingKeyAvm_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKeyAvm_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey>;
        using Base::Base;

        RefVector<DataType> get_to_be_shifted()
        {
            return { alu_a_hi,
                     alu_a_lo,
                     alu_b_hi,
                     alu_b_lo,
                     alu_cmp_rng_ctr,
                     alu_div_u16_r0,
                     alu_div_u16_r1,
                     alu_div_u16_r2,
                     alu_div_u16_r3,
                     alu_div_u16_r4,
                     alu_div_u16_r5,
                     alu_div_u16_r6,
                     alu_div_u16_r7,
                     alu_op_add,
                     alu_op_cast_prev,
                     alu_op_cast,
                     alu_op_div,
                     alu_op_mul,
                     alu_op_shl,
                     alu_op_shr,
                     alu_op_sub,
                     alu_p_sub_a_hi,
                     alu_p_sub_a_lo,
                     alu_p_sub_b_hi,
                     alu_p_sub_b_lo,
                     alu_sel_alu,
                     alu_sel_cmp,
                     alu_sel_div_rng_chk,
                     alu_sel_rng_chk_lookup,
                     alu_sel_rng_chk,
                     alu_u16_r0,
                     alu_u16_r1,
                     alu_u16_r2,
                     alu_u16_r3,
                     alu_u16_r4,
                     alu_u16_r5,
                     alu_u16_r6,
                     alu_u8_r0,
                     alu_u8_r1,
                     binary_acc_ia,
                     binary_acc_ib,
                     binary_acc_ic,
                     binary_mem_tag_ctr,
                     binary_op_id,
                     kernel_emit_l2_to_l1_msg_write_offset,
                     kernel_emit_note_hash_write_offset,
                     kernel_emit_nullifier_write_offset,
                     kernel_emit_unencrypted_log_write_offset,
                     kernel_l1_to_l2_msg_exists_write_offset,
                     kernel_note_hash_exist_write_offset,
                     kernel_nullifier_exists_write_offset,
                     kernel_nullifier_non_exists_write_offset,
                     kernel_side_effect_counter,
                     kernel_sload_write_offset,
                     kernel_sstore_write_offset,
                     main_da_gas_remaining,
                     main_internal_return_ptr,
                     main_l2_gas_remaining,
                     main_pc,
                     mem_glob_addr,
                     mem_rw,
                     mem_sel_mem,
                     mem_tag,
                     mem_tsp,
                     mem_val };
        }

        void compute_logderivative_inverses(const RelationParameters<FF>& relation_parameters)
        {
            ProverPolynomials prover_polynomials = ProverPolynomials(*this);

            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_alu_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_alu_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_bin_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_bin_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_conv_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_conv_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_pos2_perm_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_pos2_perm_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_pedersen_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_pedersen_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_a_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_a_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_b_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_b_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_c_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_c_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_d_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_d_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_ind_addr_a_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_addr_a_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_ind_addr_b_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_addr_b_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_ind_addr_c_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_addr_c_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/perm_main_mem_ind_addr_d_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_addr_d_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_byte_lengths_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_byte_lengths_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_byte_operations_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_byte_operations_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_opcode_gas_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_opcode_gas_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/range_check_l2_gas_hi_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, range_check_l2_gas_hi_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/range_check_l2_gas_lo_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, range_check_l2_gas_lo_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/range_check_da_gas_hi_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, range_check_da_gas_hi_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/range_check_da_gas_lo_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, range_check_da_gas_lo_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/kernel_output_lookup_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, kernel_output_lookup_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_into_kernel_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_into_kernel_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/incl_main_tag_err_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, incl_main_tag_err_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/incl_mem_tag_err_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, incl_mem_tag_err_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_mem_rng_chk_lo_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_mem_rng_chk_lo_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_mem_rng_chk_mid_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_mem_rng_chk_mid_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_mem_rng_chk_hi_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_mem_rng_chk_hi_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_pow_2_0_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_pow_2_0_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_pow_2_1_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_pow_2_1_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u8_0_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u8_0_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u8_1_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u8_1_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_0_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_0_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_1_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_1_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_2_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_2_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_3_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_3_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_4_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_4_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_5_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_5_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_6_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_6_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_7_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_7_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_8_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_8_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_9_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_9_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_10_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_10_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_11_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_11_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_12_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_12_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_13_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_13_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_u16_14_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_14_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_0_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_0_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_1_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_1_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_2_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_2_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_3_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_3_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_4_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_4_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_5_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_5_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_6_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_6_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
            AVM_TRACK_TIME("compute_logderivative_inverse/lookup_div_u16_7_ms",
                           (bb::compute_logderivative_inverse<AvmFlavor, lookup_div_u16_7_relation<FF>>(
                               prover_polynomials, relation_parameters, this->circuit_size)));
        }
    };

    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>, VerifierCommitmentKey>;

    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
    };

    /**
     * @brief A container for the prover polynomials handles.
     */
    class ProverPolynomials : public AllEntities<Polynomial> {
      public:
        // Define all operations as default, except copy construction/assignment
        ProverPolynomials() = default;
        ProverPolynomials& operator=(const ProverPolynomials&) = delete;
        ProverPolynomials(const ProverPolynomials& o) = delete;
        ProverPolynomials(ProverPolynomials&& o) noexcept = default;
        ProverPolynomials& operator=(ProverPolynomials&& o) noexcept = default;
        ~ProverPolynomials() = default;

        ProverPolynomials(ProvingKey& proving_key)
        {
            for (auto [prover_poly, key_poly] : zip_view(this->get_unshifted(), proving_key.get_all())) {
                ASSERT(flavor_get_label(*this, prover_poly) == flavor_get_label(proving_key, key_poly));
                prover_poly = key_poly.share();
            }
            for (auto [prover_poly, key_poly] : zip_view(this->get_shifted(), proving_key.get_to_be_shifted())) {
                ASSERT(flavor_get_label(*this, prover_poly) == (flavor_get_label(proving_key, key_poly) + "_shift"));
                prover_poly = key_poly.shifted();
            }
        }

        [[nodiscard]] size_t get_polynomial_size() const { return kernel_kernel_inputs.size(); }
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        [[nodiscard]] AllValues get_row(size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }
    };

    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial> {
      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto& poly : get_all()) {
                poly = Polynomial(circuit_size / 2);
            }
        }
    };

    /**
     * @brief A container for univariates used during Protogalaxy folding and sumcheck.
     * @details During folding and sumcheck, the prover evaluates the relations on these univariates.
     */
    template <size_t LENGTH> using ProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates used during Protogalaxy folding and sumcheck with some of the computation
     * optimistically ignored
     * @details During folding and sumcheck, the prover evaluates the relations on these univariates.
     */
    template <size_t LENGTH, size_t SKIP_COUNT>
    using OptimisedProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH, 0, SKIP_COUNT>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A container for the witness commitments.
     *
     */
    using WitnessCommitments = WitnessEntities<Commitment>;

    class CommitmentLabels : public AllEntities<std::string> {
      private:
        using Base = AllEntities<std::string>;

      public:
        CommitmentLabels()
        {
            Base::main_clk = "MAIN_CLK";
            Base::main_sel_first = "MAIN_SEL_FIRST";
            Base::kernel_kernel_inputs = "KERNEL_KERNEL_INPUTS";
            Base::kernel_kernel_value_out = "KERNEL_KERNEL_VALUE_OUT";
            Base::kernel_kernel_side_effect_out = "KERNEL_KERNEL_SIDE_EFFECT_OUT";
            Base::kernel_kernel_metadata_out = "KERNEL_KERNEL_METADATA_OUT";
            Base::main_calldata = "MAIN_CALLDATA";
            Base::alu_a_hi = "ALU_A_HI";
            Base::alu_a_lo = "ALU_A_LO";
            Base::alu_b_hi = "ALU_B_HI";
            Base::alu_b_lo = "ALU_B_LO";
            Base::alu_borrow = "ALU_BORROW";
            Base::alu_cf = "ALU_CF";
            Base::alu_clk = "ALU_CLK";
            Base::alu_cmp_rng_ctr = "ALU_CMP_RNG_CTR";
            Base::alu_div_u16_r0 = "ALU_DIV_U16_R0";
            Base::alu_div_u16_r1 = "ALU_DIV_U16_R1";
            Base::alu_div_u16_r2 = "ALU_DIV_U16_R2";
            Base::alu_div_u16_r3 = "ALU_DIV_U16_R3";
            Base::alu_div_u16_r4 = "ALU_DIV_U16_R4";
            Base::alu_div_u16_r5 = "ALU_DIV_U16_R5";
            Base::alu_div_u16_r6 = "ALU_DIV_U16_R6";
            Base::alu_div_u16_r7 = "ALU_DIV_U16_R7";
            Base::alu_divisor_hi = "ALU_DIVISOR_HI";
            Base::alu_divisor_lo = "ALU_DIVISOR_LO";
            Base::alu_ff_tag = "ALU_FF_TAG";
            Base::alu_ia = "ALU_IA";
            Base::alu_ib = "ALU_IB";
            Base::alu_ic = "ALU_IC";
            Base::alu_in_tag = "ALU_IN_TAG";
            Base::alu_op_add = "ALU_OP_ADD";
            Base::alu_op_cast = "ALU_OP_CAST";
            Base::alu_op_cast_prev = "ALU_OP_CAST_PREV";
            Base::alu_op_div = "ALU_OP_DIV";
            Base::alu_op_div_a_lt_b = "ALU_OP_DIV_A_LT_B";
            Base::alu_op_div_std = "ALU_OP_DIV_STD";
            Base::alu_op_eq = "ALU_OP_EQ";
            Base::alu_op_eq_diff_inv = "ALU_OP_EQ_DIFF_INV";
            Base::alu_op_lt = "ALU_OP_LT";
            Base::alu_op_lte = "ALU_OP_LTE";
            Base::alu_op_mul = "ALU_OP_MUL";
            Base::alu_op_not = "ALU_OP_NOT";
            Base::alu_op_shl = "ALU_OP_SHL";
            Base::alu_op_shr = "ALU_OP_SHR";
            Base::alu_op_sub = "ALU_OP_SUB";
            Base::alu_p_a_borrow = "ALU_P_A_BORROW";
            Base::alu_p_b_borrow = "ALU_P_B_BORROW";
            Base::alu_p_sub_a_hi = "ALU_P_SUB_A_HI";
            Base::alu_p_sub_a_lo = "ALU_P_SUB_A_LO";
            Base::alu_p_sub_b_hi = "ALU_P_SUB_B_HI";
            Base::alu_p_sub_b_lo = "ALU_P_SUB_B_LO";
            Base::alu_partial_prod_hi = "ALU_PARTIAL_PROD_HI";
            Base::alu_partial_prod_lo = "ALU_PARTIAL_PROD_LO";
            Base::alu_quotient_hi = "ALU_QUOTIENT_HI";
            Base::alu_quotient_lo = "ALU_QUOTIENT_LO";
            Base::alu_remainder = "ALU_REMAINDER";
            Base::alu_res_hi = "ALU_RES_HI";
            Base::alu_res_lo = "ALU_RES_LO";
            Base::alu_sel_alu = "ALU_SEL_ALU";
            Base::alu_sel_cmp = "ALU_SEL_CMP";
            Base::alu_sel_div_rng_chk = "ALU_SEL_DIV_RNG_CHK";
            Base::alu_sel_rng_chk = "ALU_SEL_RNG_CHK";
            Base::alu_sel_rng_chk_lookup = "ALU_SEL_RNG_CHK_LOOKUP";
            Base::alu_sel_shift_which = "ALU_SEL_SHIFT_WHICH";
            Base::alu_shift_lt_bit_len = "ALU_SHIFT_LT_BIT_LEN";
            Base::alu_t_sub_s_bits = "ALU_T_SUB_S_BITS";
            Base::alu_two_pow_s = "ALU_TWO_POW_S";
            Base::alu_two_pow_t_sub_s = "ALU_TWO_POW_T_SUB_S";
            Base::alu_u128_tag = "ALU_U128_TAG";
            Base::alu_u16_r0 = "ALU_U16_R0";
            Base::alu_u16_r1 = "ALU_U16_R1";
            Base::alu_u16_r10 = "ALU_U16_R10";
            Base::alu_u16_r11 = "ALU_U16_R11";
            Base::alu_u16_r12 = "ALU_U16_R12";
            Base::alu_u16_r13 = "ALU_U16_R13";
            Base::alu_u16_r14 = "ALU_U16_R14";
            Base::alu_u16_r2 = "ALU_U16_R2";
            Base::alu_u16_r3 = "ALU_U16_R3";
            Base::alu_u16_r4 = "ALU_U16_R4";
            Base::alu_u16_r5 = "ALU_U16_R5";
            Base::alu_u16_r6 = "ALU_U16_R6";
            Base::alu_u16_r7 = "ALU_U16_R7";
            Base::alu_u16_r8 = "ALU_U16_R8";
            Base::alu_u16_r9 = "ALU_U16_R9";
            Base::alu_u16_tag = "ALU_U16_TAG";
            Base::alu_u32_tag = "ALU_U32_TAG";
            Base::alu_u64_tag = "ALU_U64_TAG";
            Base::alu_u8_r0 = "ALU_U8_R0";
            Base::alu_u8_r1 = "ALU_U8_R1";
            Base::alu_u8_tag = "ALU_U8_TAG";
            Base::binary_acc_ia = "BINARY_ACC_IA";
            Base::binary_acc_ib = "BINARY_ACC_IB";
            Base::binary_acc_ic = "BINARY_ACC_IC";
            Base::binary_clk = "BINARY_CLK";
            Base::binary_ia_bytes = "BINARY_IA_BYTES";
            Base::binary_ib_bytes = "BINARY_IB_BYTES";
            Base::binary_ic_bytes = "BINARY_IC_BYTES";
            Base::binary_in_tag = "BINARY_IN_TAG";
            Base::binary_mem_tag_ctr = "BINARY_MEM_TAG_CTR";
            Base::binary_mem_tag_ctr_inv = "BINARY_MEM_TAG_CTR_INV";
            Base::binary_op_id = "BINARY_OP_ID";
            Base::binary_sel_bin = "BINARY_SEL_BIN";
            Base::binary_start = "BINARY_START";
            Base::byte_lookup_sel_bin = "BYTE_LOOKUP_SEL_BIN";
            Base::byte_lookup_table_byte_lengths = "BYTE_LOOKUP_TABLE_BYTE_LENGTHS";
            Base::byte_lookup_table_in_tags = "BYTE_LOOKUP_TABLE_IN_TAGS";
            Base::byte_lookup_table_input_a = "BYTE_LOOKUP_TABLE_INPUT_A";
            Base::byte_lookup_table_input_b = "BYTE_LOOKUP_TABLE_INPUT_B";
            Base::byte_lookup_table_op_id = "BYTE_LOOKUP_TABLE_OP_ID";
            Base::byte_lookup_table_output = "BYTE_LOOKUP_TABLE_OUTPUT";
            Base::conversion_clk = "CONVERSION_CLK";
            Base::conversion_input = "CONVERSION_INPUT";
            Base::conversion_num_limbs = "CONVERSION_NUM_LIMBS";
            Base::conversion_radix = "CONVERSION_RADIX";
            Base::conversion_sel_to_radix_le = "CONVERSION_SEL_TO_RADIX_LE";
            Base::gas_da_gas_fixed_table = "GAS_DA_GAS_FIXED_TABLE";
            Base::gas_l2_gas_fixed_table = "GAS_L2_GAS_FIXED_TABLE";
            Base::gas_sel_gas_cost = "GAS_SEL_GAS_COST";
            Base::keccakf1600_clk = "KECCAKF1600_CLK";
            Base::keccakf1600_input = "KECCAKF1600_INPUT";
            Base::keccakf1600_output = "KECCAKF1600_OUTPUT";
            Base::keccakf1600_sel_keccakf1600 = "KECCAKF1600_SEL_KECCAKF1600";
            Base::kernel_emit_l2_to_l1_msg_write_offset = "KERNEL_EMIT_L2_TO_L1_MSG_WRITE_OFFSET";
            Base::kernel_emit_note_hash_write_offset = "KERNEL_EMIT_NOTE_HASH_WRITE_OFFSET";
            Base::kernel_emit_nullifier_write_offset = "KERNEL_EMIT_NULLIFIER_WRITE_OFFSET";
            Base::kernel_emit_unencrypted_log_write_offset = "KERNEL_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET";
            Base::kernel_kernel_in_offset = "KERNEL_KERNEL_IN_OFFSET";
            Base::kernel_kernel_out_offset = "KERNEL_KERNEL_OUT_OFFSET";
            Base::kernel_l1_to_l2_msg_exists_write_offset = "KERNEL_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET";
            Base::kernel_note_hash_exist_write_offset = "KERNEL_NOTE_HASH_EXIST_WRITE_OFFSET";
            Base::kernel_nullifier_exists_write_offset = "KERNEL_NULLIFIER_EXISTS_WRITE_OFFSET";
            Base::kernel_nullifier_non_exists_write_offset = "KERNEL_NULLIFIER_NON_EXISTS_WRITE_OFFSET";
            Base::kernel_q_public_input_kernel_add_to_table = "KERNEL_Q_PUBLIC_INPUT_KERNEL_ADD_TO_TABLE";
            Base::kernel_q_public_input_kernel_out_add_to_table = "KERNEL_Q_PUBLIC_INPUT_KERNEL_OUT_ADD_TO_TABLE";
            Base::kernel_side_effect_counter = "KERNEL_SIDE_EFFECT_COUNTER";
            Base::kernel_sload_write_offset = "KERNEL_SLOAD_WRITE_OFFSET";
            Base::kernel_sstore_write_offset = "KERNEL_SSTORE_WRITE_OFFSET";
            Base::main_abs_da_rem_gas_hi = "MAIN_ABS_DA_REM_GAS_HI";
            Base::main_abs_da_rem_gas_lo = "MAIN_ABS_DA_REM_GAS_LO";
            Base::main_abs_l2_rem_gas_hi = "MAIN_ABS_L2_REM_GAS_HI";
            Base::main_abs_l2_rem_gas_lo = "MAIN_ABS_L2_REM_GAS_LO";
            Base::main_alu_in_tag = "MAIN_ALU_IN_TAG";
            Base::main_bin_op_id = "MAIN_BIN_OP_ID";
            Base::main_call_ptr = "MAIN_CALL_PTR";
            Base::main_da_gas_op_cost = "MAIN_DA_GAS_OP_COST";
            Base::main_da_gas_remaining = "MAIN_DA_GAS_REMAINING";
            Base::main_da_out_of_gas = "MAIN_DA_OUT_OF_GAS";
            Base::main_ia = "MAIN_IA";
            Base::main_ib = "MAIN_IB";
            Base::main_ic = "MAIN_IC";
            Base::main_id = "MAIN_ID";
            Base::main_id_zero = "MAIN_ID_ZERO";
            Base::main_ind_addr_a = "MAIN_IND_ADDR_A";
            Base::main_ind_addr_b = "MAIN_IND_ADDR_B";
            Base::main_ind_addr_c = "MAIN_IND_ADDR_C";
            Base::main_ind_addr_d = "MAIN_IND_ADDR_D";
            Base::main_internal_return_ptr = "MAIN_INTERNAL_RETURN_PTR";
            Base::main_inv = "MAIN_INV";
            Base::main_l2_gas_op_cost = "MAIN_L2_GAS_OP_COST";
            Base::main_l2_gas_remaining = "MAIN_L2_GAS_REMAINING";
            Base::main_l2_out_of_gas = "MAIN_L2_OUT_OF_GAS";
            Base::main_mem_addr_a = "MAIN_MEM_ADDR_A";
            Base::main_mem_addr_b = "MAIN_MEM_ADDR_B";
            Base::main_mem_addr_c = "MAIN_MEM_ADDR_C";
            Base::main_mem_addr_d = "MAIN_MEM_ADDR_D";
            Base::main_op_err = "MAIN_OP_ERR";
            Base::main_opcode_val = "MAIN_OPCODE_VAL";
            Base::main_pc = "MAIN_PC";
            Base::main_r_in_tag = "MAIN_R_IN_TAG";
            Base::main_rwa = "MAIN_RWA";
            Base::main_rwb = "MAIN_RWB";
            Base::main_rwc = "MAIN_RWC";
            Base::main_rwd = "MAIN_RWD";
            Base::main_sel_alu = "MAIN_SEL_ALU";
            Base::main_sel_bin = "MAIN_SEL_BIN";
            Base::main_sel_gas_accounting_active = "MAIN_SEL_GAS_ACCOUNTING_ACTIVE";
            Base::main_sel_last = "MAIN_SEL_LAST";
            Base::main_sel_mem_op_a = "MAIN_SEL_MEM_OP_A";
            Base::main_sel_mem_op_activate_gas = "MAIN_SEL_MEM_OP_ACTIVATE_GAS";
            Base::main_sel_mem_op_b = "MAIN_SEL_MEM_OP_B";
            Base::main_sel_mem_op_c = "MAIN_SEL_MEM_OP_C";
            Base::main_sel_mem_op_d = "MAIN_SEL_MEM_OP_D";
            Base::main_sel_mov_ia_to_ic = "MAIN_SEL_MOV_IA_TO_IC";
            Base::main_sel_mov_ib_to_ic = "MAIN_SEL_MOV_IB_TO_IC";
            Base::main_sel_op_add = "MAIN_SEL_OP_ADD";
            Base::main_sel_op_address = "MAIN_SEL_OP_ADDRESS";
            Base::main_sel_op_and = "MAIN_SEL_OP_AND";
            Base::main_sel_op_block_number = "MAIN_SEL_OP_BLOCK_NUMBER";
            Base::main_sel_op_cast = "MAIN_SEL_OP_CAST";
            Base::main_sel_op_chain_id = "MAIN_SEL_OP_CHAIN_ID";
            Base::main_sel_op_cmov = "MAIN_SEL_OP_CMOV";
            Base::main_sel_op_coinbase = "MAIN_SEL_OP_COINBASE";
            Base::main_sel_op_dagasleft = "MAIN_SEL_OP_DAGASLEFT";
            Base::main_sel_op_div = "MAIN_SEL_OP_DIV";
            Base::main_sel_op_emit_l2_to_l1_msg = "MAIN_SEL_OP_EMIT_L2_TO_L1_MSG";
            Base::main_sel_op_emit_note_hash = "MAIN_SEL_OP_EMIT_NOTE_HASH";
            Base::main_sel_op_emit_nullifier = "MAIN_SEL_OP_EMIT_NULLIFIER";
            Base::main_sel_op_emit_unencrypted_log = "MAIN_SEL_OP_EMIT_UNENCRYPTED_LOG";
            Base::main_sel_op_eq = "MAIN_SEL_OP_EQ";
            Base::main_sel_op_external_call = "MAIN_SEL_OP_EXTERNAL_CALL";
            Base::main_sel_op_fdiv = "MAIN_SEL_OP_FDIV";
            Base::main_sel_op_fee_per_da_gas = "MAIN_SEL_OP_FEE_PER_DA_GAS";
            Base::main_sel_op_fee_per_l2_gas = "MAIN_SEL_OP_FEE_PER_L2_GAS";
            Base::main_sel_op_function_selector = "MAIN_SEL_OP_FUNCTION_SELECTOR";
            Base::main_sel_op_get_contract_instance = "MAIN_SEL_OP_GET_CONTRACT_INSTANCE";
            Base::main_sel_op_halt = "MAIN_SEL_OP_HALT";
            Base::main_sel_op_internal_call = "MAIN_SEL_OP_INTERNAL_CALL";
            Base::main_sel_op_internal_return = "MAIN_SEL_OP_INTERNAL_RETURN";
            Base::main_sel_op_jump = "MAIN_SEL_OP_JUMP";
            Base::main_sel_op_jumpi = "MAIN_SEL_OP_JUMPI";
            Base::main_sel_op_keccak = "MAIN_SEL_OP_KECCAK";
            Base::main_sel_op_l1_to_l2_msg_exists = "MAIN_SEL_OP_L1_TO_L2_MSG_EXISTS";
            Base::main_sel_op_l2gasleft = "MAIN_SEL_OP_L2GASLEFT";
            Base::main_sel_op_lt = "MAIN_SEL_OP_LT";
            Base::main_sel_op_lte = "MAIN_SEL_OP_LTE";
            Base::main_sel_op_mov = "MAIN_SEL_OP_MOV";
            Base::main_sel_op_mul = "MAIN_SEL_OP_MUL";
            Base::main_sel_op_not = "MAIN_SEL_OP_NOT";
            Base::main_sel_op_note_hash_exists = "MAIN_SEL_OP_NOTE_HASH_EXISTS";
            Base::main_sel_op_nullifier_exists = "MAIN_SEL_OP_NULLIFIER_EXISTS";
            Base::main_sel_op_or = "MAIN_SEL_OP_OR";
            Base::main_sel_op_pedersen = "MAIN_SEL_OP_PEDERSEN";
            Base::main_sel_op_poseidon2 = "MAIN_SEL_OP_POSEIDON2";
            Base::main_sel_op_radix_le = "MAIN_SEL_OP_RADIX_LE";
            Base::main_sel_op_sender = "MAIN_SEL_OP_SENDER";
            Base::main_sel_op_sha256 = "MAIN_SEL_OP_SHA256";
            Base::main_sel_op_shl = "MAIN_SEL_OP_SHL";
            Base::main_sel_op_shr = "MAIN_SEL_OP_SHR";
            Base::main_sel_op_sload = "MAIN_SEL_OP_SLOAD";
            Base::main_sel_op_sstore = "MAIN_SEL_OP_SSTORE";
            Base::main_sel_op_storage_address = "MAIN_SEL_OP_STORAGE_ADDRESS";
            Base::main_sel_op_sub = "MAIN_SEL_OP_SUB";
            Base::main_sel_op_timestamp = "MAIN_SEL_OP_TIMESTAMP";
            Base::main_sel_op_transaction_fee = "MAIN_SEL_OP_TRANSACTION_FEE";
            Base::main_sel_op_version = "MAIN_SEL_OP_VERSION";
            Base::main_sel_op_xor = "MAIN_SEL_OP_XOR";
            Base::main_sel_q_kernel_lookup = "MAIN_SEL_Q_KERNEL_LOOKUP";
            Base::main_sel_q_kernel_output_lookup = "MAIN_SEL_Q_KERNEL_OUTPUT_LOOKUP";
            Base::main_sel_resolve_ind_addr_a = "MAIN_SEL_RESOLVE_IND_ADDR_A";
            Base::main_sel_resolve_ind_addr_b = "MAIN_SEL_RESOLVE_IND_ADDR_B";
            Base::main_sel_resolve_ind_addr_c = "MAIN_SEL_RESOLVE_IND_ADDR_C";
            Base::main_sel_resolve_ind_addr_d = "MAIN_SEL_RESOLVE_IND_ADDR_D";
            Base::main_sel_rng_16 = "MAIN_SEL_RNG_16";
            Base::main_sel_rng_8 = "MAIN_SEL_RNG_8";
            Base::main_space_id = "MAIN_SPACE_ID";
            Base::main_tag_err = "MAIN_TAG_ERR";
            Base::main_w_in_tag = "MAIN_W_IN_TAG";
            Base::mem_addr = "MEM_ADDR";
            Base::mem_clk = "MEM_CLK";
            Base::mem_diff_hi = "MEM_DIFF_HI";
            Base::mem_diff_lo = "MEM_DIFF_LO";
            Base::mem_diff_mid = "MEM_DIFF_MID";
            Base::mem_glob_addr = "MEM_GLOB_ADDR";
            Base::mem_last = "MEM_LAST";
            Base::mem_lastAccess = "MEM_LAST_ACCESS";
            Base::mem_one_min_inv = "MEM_ONE_MIN_INV";
            Base::mem_r_in_tag = "MEM_R_IN_TAG";
            Base::mem_rw = "MEM_RW";
            Base::mem_sel_mem = "MEM_SEL_MEM";
            Base::mem_sel_mov_ia_to_ic = "MEM_SEL_MOV_IA_TO_IC";
            Base::mem_sel_mov_ib_to_ic = "MEM_SEL_MOV_IB_TO_IC";
            Base::mem_sel_op_a = "MEM_SEL_OP_A";
            Base::mem_sel_op_b = "MEM_SEL_OP_B";
            Base::mem_sel_op_c = "MEM_SEL_OP_C";
            Base::mem_sel_op_cmov = "MEM_SEL_OP_CMOV";
            Base::mem_sel_op_d = "MEM_SEL_OP_D";
            Base::mem_sel_resolve_ind_addr_a = "MEM_SEL_RESOLVE_IND_ADDR_A";
            Base::mem_sel_resolve_ind_addr_b = "MEM_SEL_RESOLVE_IND_ADDR_B";
            Base::mem_sel_resolve_ind_addr_c = "MEM_SEL_RESOLVE_IND_ADDR_C";
            Base::mem_sel_resolve_ind_addr_d = "MEM_SEL_RESOLVE_IND_ADDR_D";
            Base::mem_sel_rng_chk = "MEM_SEL_RNG_CHK";
            Base::mem_skip_check_tag = "MEM_SKIP_CHECK_TAG";
            Base::mem_space_id = "MEM_SPACE_ID";
            Base::mem_tag = "MEM_TAG";
            Base::mem_tag_err = "MEM_TAG_ERR";
            Base::mem_tsp = "MEM_TSP";
            Base::mem_val = "MEM_VAL";
            Base::mem_w_in_tag = "MEM_W_IN_TAG";
            Base::pedersen_clk = "PEDERSEN_CLK";
            Base::pedersen_input = "PEDERSEN_INPUT";
            Base::pedersen_output = "PEDERSEN_OUTPUT";
            Base::pedersen_sel_pedersen = "PEDERSEN_SEL_PEDERSEN";
            Base::poseidon2_clk = "POSEIDON2_CLK";
            Base::poseidon2_input = "POSEIDON2_INPUT";
            Base::poseidon2_output = "POSEIDON2_OUTPUT";
            Base::poseidon2_sel_poseidon_perm = "POSEIDON2_SEL_POSEIDON_PERM";
            Base::powers_power_of_2 = "POWERS_POWER_OF_2";
            Base::sha256_clk = "SHA256_CLK";
            Base::sha256_input = "SHA256_INPUT";
            Base::sha256_output = "SHA256_OUTPUT";
            Base::sha256_sel_sha256_compression = "SHA256_SEL_SHA256_COMPRESSION";
            Base::sha256_state = "SHA256_STATE";
            Base::perm_main_alu = "PERM_MAIN_ALU";
            Base::perm_main_bin = "PERM_MAIN_BIN";
            Base::perm_main_conv = "PERM_MAIN_CONV";
            Base::perm_main_pos2_perm = "PERM_MAIN_POS2_PERM";
            Base::perm_main_pedersen = "PERM_MAIN_PEDERSEN";
            Base::perm_main_mem_a = "PERM_MAIN_MEM_A";
            Base::perm_main_mem_b = "PERM_MAIN_MEM_B";
            Base::perm_main_mem_c = "PERM_MAIN_MEM_C";
            Base::perm_main_mem_d = "PERM_MAIN_MEM_D";
            Base::perm_main_mem_ind_addr_a = "PERM_MAIN_MEM_IND_ADDR_A";
            Base::perm_main_mem_ind_addr_b = "PERM_MAIN_MEM_IND_ADDR_B";
            Base::perm_main_mem_ind_addr_c = "PERM_MAIN_MEM_IND_ADDR_C";
            Base::perm_main_mem_ind_addr_d = "PERM_MAIN_MEM_IND_ADDR_D";
            Base::lookup_byte_lengths = "LOOKUP_BYTE_LENGTHS";
            Base::lookup_byte_operations = "LOOKUP_BYTE_OPERATIONS";
            Base::lookup_opcode_gas = "LOOKUP_OPCODE_GAS";
            Base::range_check_l2_gas_hi = "RANGE_CHECK_L2_GAS_HI";
            Base::range_check_l2_gas_lo = "RANGE_CHECK_L2_GAS_LO";
            Base::range_check_da_gas_hi = "RANGE_CHECK_DA_GAS_HI";
            Base::range_check_da_gas_lo = "RANGE_CHECK_DA_GAS_LO";
            Base::kernel_output_lookup = "KERNEL_OUTPUT_LOOKUP";
            Base::lookup_into_kernel = "LOOKUP_INTO_KERNEL";
            Base::incl_main_tag_err = "INCL_MAIN_TAG_ERR";
            Base::incl_mem_tag_err = "INCL_MEM_TAG_ERR";
            Base::lookup_mem_rng_chk_lo = "LOOKUP_MEM_RNG_CHK_LO";
            Base::lookup_mem_rng_chk_mid = "LOOKUP_MEM_RNG_CHK_MID";
            Base::lookup_mem_rng_chk_hi = "LOOKUP_MEM_RNG_CHK_HI";
            Base::lookup_pow_2_0 = "LOOKUP_POW_2_0";
            Base::lookup_pow_2_1 = "LOOKUP_POW_2_1";
            Base::lookup_u8_0 = "LOOKUP_U8_0";
            Base::lookup_u8_1 = "LOOKUP_U8_1";
            Base::lookup_u16_0 = "LOOKUP_U16_0";
            Base::lookup_u16_1 = "LOOKUP_U16_1";
            Base::lookup_u16_2 = "LOOKUP_U16_2";
            Base::lookup_u16_3 = "LOOKUP_U16_3";
            Base::lookup_u16_4 = "LOOKUP_U16_4";
            Base::lookup_u16_5 = "LOOKUP_U16_5";
            Base::lookup_u16_6 = "LOOKUP_U16_6";
            Base::lookup_u16_7 = "LOOKUP_U16_7";
            Base::lookup_u16_8 = "LOOKUP_U16_8";
            Base::lookup_u16_9 = "LOOKUP_U16_9";
            Base::lookup_u16_10 = "LOOKUP_U16_10";
            Base::lookup_u16_11 = "LOOKUP_U16_11";
            Base::lookup_u16_12 = "LOOKUP_U16_12";
            Base::lookup_u16_13 = "LOOKUP_U16_13";
            Base::lookup_u16_14 = "LOOKUP_U16_14";
            Base::lookup_div_u16_0 = "LOOKUP_DIV_U16_0";
            Base::lookup_div_u16_1 = "LOOKUP_DIV_U16_1";
            Base::lookup_div_u16_2 = "LOOKUP_DIV_U16_2";
            Base::lookup_div_u16_3 = "LOOKUP_DIV_U16_3";
            Base::lookup_div_u16_4 = "LOOKUP_DIV_U16_4";
            Base::lookup_div_u16_5 = "LOOKUP_DIV_U16_5";
            Base::lookup_div_u16_6 = "LOOKUP_DIV_U16_6";
            Base::lookup_div_u16_7 = "LOOKUP_DIV_U16_7";
            Base::lookup_byte_lengths_counts = "LOOKUP_BYTE_LENGTHS_COUNTS";
            Base::lookup_byte_operations_counts = "LOOKUP_BYTE_OPERATIONS_COUNTS";
            Base::lookup_opcode_gas_counts = "LOOKUP_OPCODE_GAS_COUNTS";
            Base::range_check_l2_gas_hi_counts = "RANGE_CHECK_L2_GAS_HI_COUNTS";
            Base::range_check_l2_gas_lo_counts = "RANGE_CHECK_L2_GAS_LO_COUNTS";
            Base::range_check_da_gas_hi_counts = "RANGE_CHECK_DA_GAS_HI_COUNTS";
            Base::range_check_da_gas_lo_counts = "RANGE_CHECK_DA_GAS_LO_COUNTS";
            Base::kernel_output_lookup_counts = "KERNEL_OUTPUT_LOOKUP_COUNTS";
            Base::lookup_into_kernel_counts = "LOOKUP_INTO_KERNEL_COUNTS";
            Base::incl_main_tag_err_counts = "INCL_MAIN_TAG_ERR_COUNTS";
            Base::incl_mem_tag_err_counts = "INCL_MEM_TAG_ERR_COUNTS";
            Base::lookup_mem_rng_chk_lo_counts = "LOOKUP_MEM_RNG_CHK_LO_COUNTS";
            Base::lookup_mem_rng_chk_mid_counts = "LOOKUP_MEM_RNG_CHK_MID_COUNTS";
            Base::lookup_mem_rng_chk_hi_counts = "LOOKUP_MEM_RNG_CHK_HI_COUNTS";
            Base::lookup_pow_2_0_counts = "LOOKUP_POW_2_0_COUNTS";
            Base::lookup_pow_2_1_counts = "LOOKUP_POW_2_1_COUNTS";
            Base::lookup_u8_0_counts = "LOOKUP_U8_0_COUNTS";
            Base::lookup_u8_1_counts = "LOOKUP_U8_1_COUNTS";
            Base::lookup_u16_0_counts = "LOOKUP_U16_0_COUNTS";
            Base::lookup_u16_1_counts = "LOOKUP_U16_1_COUNTS";
            Base::lookup_u16_2_counts = "LOOKUP_U16_2_COUNTS";
            Base::lookup_u16_3_counts = "LOOKUP_U16_3_COUNTS";
            Base::lookup_u16_4_counts = "LOOKUP_U16_4_COUNTS";
            Base::lookup_u16_5_counts = "LOOKUP_U16_5_COUNTS";
            Base::lookup_u16_6_counts = "LOOKUP_U16_6_COUNTS";
            Base::lookup_u16_7_counts = "LOOKUP_U16_7_COUNTS";
            Base::lookup_u16_8_counts = "LOOKUP_U16_8_COUNTS";
            Base::lookup_u16_9_counts = "LOOKUP_U16_9_COUNTS";
            Base::lookup_u16_10_counts = "LOOKUP_U16_10_COUNTS";
            Base::lookup_u16_11_counts = "LOOKUP_U16_11_COUNTS";
            Base::lookup_u16_12_counts = "LOOKUP_U16_12_COUNTS";
            Base::lookup_u16_13_counts = "LOOKUP_U16_13_COUNTS";
            Base::lookup_u16_14_counts = "LOOKUP_U16_14_COUNTS";
            Base::lookup_div_u16_0_counts = "LOOKUP_DIV_U16_0_COUNTS";
            Base::lookup_div_u16_1_counts = "LOOKUP_DIV_U16_1_COUNTS";
            Base::lookup_div_u16_2_counts = "LOOKUP_DIV_U16_2_COUNTS";
            Base::lookup_div_u16_3_counts = "LOOKUP_DIV_U16_3_COUNTS";
            Base::lookup_div_u16_4_counts = "LOOKUP_DIV_U16_4_COUNTS";
            Base::lookup_div_u16_5_counts = "LOOKUP_DIV_U16_5_COUNTS";
            Base::lookup_div_u16_6_counts = "LOOKUP_DIV_U16_6_COUNTS";
            Base::lookup_div_u16_7_counts = "LOOKUP_DIV_U16_7_COUNTS";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      private:
        using Base = AllEntities<Commitment>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            main_clk = verification_key->main_clk;
            main_sel_first = verification_key->main_sel_first;
        }
    };

    class Transcript : public NativeTranscript {
      public:
        uint32_t circuit_size;

        std::array<Commitment, NUM_WITNESS_ENTITIES> commitments;

        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript() = default;

        Transcript(const std::vector<FF>& proof)
            : NativeTranscript(proof)
        {}

        void deserialize_full_transcript()
        {
            size_t num_frs_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(proof_data, num_frs_read);
            size_t log_n = numeric::get_msb(circuit_size);

            for (auto& commitment : commitments) {
                commitment = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            }
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(
                    deserialize_from_buffer<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(Transcript::proof_data,
                                                                                                 num_frs_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(Transcript::proof_data, num_frs_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_frs_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
        }

        void serialize_full_transcript()
        {
            size_t old_proof_length = proof_data.size();
            Transcript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            serialize_to_buffer(circuit_size, Transcript::proof_data);

            for (const auto& commitment : commitments) {
                serialize_to_buffer(commitment, Transcript::proof_data);
            }
            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(sumcheck_univariates[i], Transcript::proof_data);
            }
            serialize_to_buffer(sumcheck_evaluations, Transcript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(zm_cq_comms[i], proof_data);
            }
            serialize_to_buffer(zm_cq_comm, proof_data);
            serialize_to_buffer(zm_pi_comm, proof_data);

            // sanity check to make sure we generate the same length of proof as before.
            ASSERT(proof_data.size() == old_proof_length);
        }
    };
};

} // namespace bb