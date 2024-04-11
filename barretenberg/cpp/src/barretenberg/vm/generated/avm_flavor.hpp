

#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/generated/avm/avm_alu.hpp"
#include "barretenberg/relations/generated/avm/avm_binary.hpp"
#include "barretenberg/relations/generated/avm/avm_main.hpp"
#include "barretenberg/relations/generated/avm/avm_mem.hpp"
#include "barretenberg/relations/generated/avm/incl_main_tag_err.hpp"
#include "barretenberg/relations/generated/avm/incl_mem_tag_err.hpp"
#include "barretenberg/relations/generated/avm/lookup_byte_lengths.hpp"
#include "barretenberg/relations/generated/avm/lookup_byte_operations.hpp"
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
#include "barretenberg/relations/generated/avm/perm_main_mem_a.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_b.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_c.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_d.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_a.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_b.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_c.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_d.hpp"
#include "barretenberg/transcript/transcript.hpp"

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
    static constexpr size_t NUM_WITNESS_ENTITIES = 210;
    static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
    // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for
    // the unshifted and one for the shifted
    static constexpr size_t NUM_ALL_ENTITIES = 241;

    using GrandProductRelations = std::tuple<perm_main_alu_relation<FF>,
                                             perm_main_bin_relation<FF>,
                                             perm_main_mem_a_relation<FF>,
                                             perm_main_mem_b_relation<FF>,
                                             perm_main_mem_c_relation<FF>,
                                             perm_main_mem_d_relation<FF>,
                                             perm_main_mem_ind_a_relation<FF>,
                                             perm_main_mem_ind_b_relation<FF>,
                                             perm_main_mem_ind_c_relation<FF>,
                                             perm_main_mem_ind_d_relation<FF>,
                                             lookup_byte_lengths_relation<FF>,
                                             lookup_byte_operations_relation<FF>,
                                             incl_main_tag_err_relation<FF>,
                                             incl_mem_tag_err_relation<FF>,
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
                                             lookup_u16_14_relation<FF>>;

    using Relations = std::tuple<Avm_vm::avm_alu<FF>,
                                 Avm_vm::avm_binary<FF>,
                                 Avm_vm::avm_main<FF>,
                                 Avm_vm::avm_mem<FF>,
                                 perm_main_alu_relation<FF>,
                                 perm_main_bin_relation<FF>,
                                 perm_main_mem_a_relation<FF>,
                                 perm_main_mem_b_relation<FF>,
                                 perm_main_mem_c_relation<FF>,
                                 perm_main_mem_d_relation<FF>,
                                 perm_main_mem_ind_a_relation<FF>,
                                 perm_main_mem_ind_b_relation<FF>,
                                 perm_main_mem_ind_c_relation<FF>,
                                 perm_main_mem_ind_d_relation<FF>,
                                 lookup_byte_lengths_relation<FF>,
                                 lookup_byte_operations_relation<FF>,
                                 incl_main_tag_err_relation<FF>,
                                 incl_mem_tag_err_relation<FF>,
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
                                 lookup_u16_14_relation<FF>>;

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

        DEFINE_FLAVOR_MEMBERS(DataType, avm_main_clk, avm_main_first)

        RefVector<DataType> get_selectors() { return { avm_main_clk, avm_main_first }; };
        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
        RefVector<DataType> get_table_polynomials() { return {}; };
    };

    template <typename DataType> class WitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              avm_alu_a_hi,
                              avm_alu_a_lo,
                              avm_alu_alu_sel,
                              avm_alu_b_hi,
                              avm_alu_b_lo,
                              avm_alu_borrow,
                              avm_alu_cf,
                              avm_alu_clk,
                              avm_alu_cmp_rng_ctr,
                              avm_alu_cmp_sel,
                              avm_alu_ff_tag,
                              avm_alu_ia,
                              avm_alu_ib,
                              avm_alu_ic,
                              avm_alu_in_tag,
                              avm_alu_op_add,
                              avm_alu_op_div,
                              avm_alu_op_eq,
                              avm_alu_op_eq_diff_inv,
                              avm_alu_op_lt,
                              avm_alu_op_lte,
                              avm_alu_op_mul,
                              avm_alu_op_not,
                              avm_alu_op_sub,
                              avm_alu_p_a_borrow,
                              avm_alu_p_b_borrow,
                              avm_alu_p_sub_a_hi,
                              avm_alu_p_sub_a_lo,
                              avm_alu_p_sub_b_hi,
                              avm_alu_p_sub_b_lo,
                              avm_alu_res_hi,
                              avm_alu_res_lo,
                              avm_alu_rng_chk_lookup_selector,
                              avm_alu_rng_chk_sel,
                              avm_alu_u128_tag,
                              avm_alu_u16_r0,
                              avm_alu_u16_r1,
                              avm_alu_u16_r10,
                              avm_alu_u16_r11,
                              avm_alu_u16_r12,
                              avm_alu_u16_r13,
                              avm_alu_u16_r14,
                              avm_alu_u16_r2,
                              avm_alu_u16_r3,
                              avm_alu_u16_r4,
                              avm_alu_u16_r5,
                              avm_alu_u16_r6,
                              avm_alu_u16_r7,
                              avm_alu_u16_r8,
                              avm_alu_u16_r9,
                              avm_alu_u16_tag,
                              avm_alu_u32_tag,
                              avm_alu_u64_r0,
                              avm_alu_u64_tag,
                              avm_alu_u8_r0,
                              avm_alu_u8_r1,
                              avm_alu_u8_tag,
                              avm_binary_acc_ia,
                              avm_binary_acc_ib,
                              avm_binary_acc_ic,
                              avm_binary_bin_sel,
                              avm_binary_clk,
                              avm_binary_ia_bytes,
                              avm_binary_ib_bytes,
                              avm_binary_ic_bytes,
                              avm_binary_in_tag,
                              avm_binary_mem_tag_ctr,
                              avm_binary_mem_tag_ctr_inv,
                              avm_binary_op_id,
                              avm_binary_start,
                              avm_byte_lookup_bin_sel,
                              avm_byte_lookup_table_byte_lengths,
                              avm_byte_lookup_table_in_tags,
                              avm_byte_lookup_table_input_a,
                              avm_byte_lookup_table_input_b,
                              avm_byte_lookup_table_op_id,
                              avm_byte_lookup_table_output,
                              avm_main_alu_sel,
                              avm_main_bin_op_id,
                              avm_main_bin_sel,
                              avm_main_ia,
                              avm_main_ib,
                              avm_main_ic,
                              avm_main_id,
                              avm_main_id_zero,
                              avm_main_ind_a,
                              avm_main_ind_b,
                              avm_main_ind_c,
                              avm_main_ind_d,
                              avm_main_ind_op_a,
                              avm_main_ind_op_b,
                              avm_main_ind_op_c,
                              avm_main_ind_op_d,
                              avm_main_internal_return_ptr,
                              avm_main_inv,
                              avm_main_last,
                              avm_main_mem_idx_a,
                              avm_main_mem_idx_b,
                              avm_main_mem_idx_c,
                              avm_main_mem_idx_d,
                              avm_main_mem_op_a,
                              avm_main_mem_op_b,
                              avm_main_mem_op_c,
                              avm_main_mem_op_d,
                              avm_main_op_err,
                              avm_main_pc,
                              avm_main_r_in_tag,
                              avm_main_rwa,
                              avm_main_rwb,
                              avm_main_rwc,
                              avm_main_rwd,
                              avm_main_sel_cmov,
                              avm_main_sel_halt,
                              avm_main_sel_internal_call,
                              avm_main_sel_internal_return,
                              avm_main_sel_jump,
                              avm_main_sel_mov,
                              avm_main_sel_mov_a,
                              avm_main_sel_mov_b,
                              avm_main_sel_op_add,
                              avm_main_sel_op_and,
                              avm_main_sel_op_div,
                              avm_main_sel_op_eq,
                              avm_main_sel_op_lt,
                              avm_main_sel_op_lte,
                              avm_main_sel_op_mul,
                              avm_main_sel_op_not,
                              avm_main_sel_op_or,
                              avm_main_sel_op_sub,
                              avm_main_sel_op_xor,
                              avm_main_sel_rng_16,
                              avm_main_sel_rng_8,
                              avm_main_tag_err,
                              avm_main_w_in_tag,
                              avm_mem_addr,
                              avm_mem_clk,
                              avm_mem_ind_op_a,
                              avm_mem_ind_op_b,
                              avm_mem_ind_op_c,
                              avm_mem_ind_op_d,
                              avm_mem_last,
                              avm_mem_lastAccess,
                              avm_mem_one_min_inv,
                              avm_mem_op_a,
                              avm_mem_op_b,
                              avm_mem_op_c,
                              avm_mem_op_d,
                              avm_mem_r_in_tag,
                              avm_mem_rw,
                              avm_mem_sel_cmov,
                              avm_mem_sel_mov_a,
                              avm_mem_sel_mov_b,
                              avm_mem_skip_check_tag,
                              avm_mem_sub_clk,
                              avm_mem_tag,
                              avm_mem_tag_err,
                              avm_mem_val,
                              avm_mem_w_in_tag,
                              perm_main_alu,
                              perm_main_bin,
                              perm_main_mem_a,
                              perm_main_mem_b,
                              perm_main_mem_c,
                              perm_main_mem_d,
                              perm_main_mem_ind_a,
                              perm_main_mem_ind_b,
                              perm_main_mem_ind_c,
                              perm_main_mem_ind_d,
                              lookup_byte_lengths,
                              lookup_byte_operations,
                              incl_main_tag_err,
                              incl_mem_tag_err,
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
                              lookup_byte_lengths_counts,
                              lookup_byte_operations_counts,
                              incl_main_tag_err_counts,
                              incl_mem_tag_err_counts,
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
                              lookup_u16_14_counts)

        RefVector<DataType> get_wires()
        {
            return { avm_alu_a_hi,
                     avm_alu_a_lo,
                     avm_alu_alu_sel,
                     avm_alu_b_hi,
                     avm_alu_b_lo,
                     avm_alu_borrow,
                     avm_alu_cf,
                     avm_alu_clk,
                     avm_alu_cmp_rng_ctr,
                     avm_alu_cmp_sel,
                     avm_alu_ff_tag,
                     avm_alu_ia,
                     avm_alu_ib,
                     avm_alu_ic,
                     avm_alu_in_tag,
                     avm_alu_op_add,
                     avm_alu_op_div,
                     avm_alu_op_eq,
                     avm_alu_op_eq_diff_inv,
                     avm_alu_op_lt,
                     avm_alu_op_lte,
                     avm_alu_op_mul,
                     avm_alu_op_not,
                     avm_alu_op_sub,
                     avm_alu_p_a_borrow,
                     avm_alu_p_b_borrow,
                     avm_alu_p_sub_a_hi,
                     avm_alu_p_sub_a_lo,
                     avm_alu_p_sub_b_hi,
                     avm_alu_p_sub_b_lo,
                     avm_alu_res_hi,
                     avm_alu_res_lo,
                     avm_alu_rng_chk_lookup_selector,
                     avm_alu_rng_chk_sel,
                     avm_alu_u128_tag,
                     avm_alu_u16_r0,
                     avm_alu_u16_r1,
                     avm_alu_u16_r10,
                     avm_alu_u16_r11,
                     avm_alu_u16_r12,
                     avm_alu_u16_r13,
                     avm_alu_u16_r14,
                     avm_alu_u16_r2,
                     avm_alu_u16_r3,
                     avm_alu_u16_r4,
                     avm_alu_u16_r5,
                     avm_alu_u16_r6,
                     avm_alu_u16_r7,
                     avm_alu_u16_r8,
                     avm_alu_u16_r9,
                     avm_alu_u16_tag,
                     avm_alu_u32_tag,
                     avm_alu_u64_r0,
                     avm_alu_u64_tag,
                     avm_alu_u8_r0,
                     avm_alu_u8_r1,
                     avm_alu_u8_tag,
                     avm_binary_acc_ia,
                     avm_binary_acc_ib,
                     avm_binary_acc_ic,
                     avm_binary_bin_sel,
                     avm_binary_clk,
                     avm_binary_ia_bytes,
                     avm_binary_ib_bytes,
                     avm_binary_ic_bytes,
                     avm_binary_in_tag,
                     avm_binary_mem_tag_ctr,
                     avm_binary_mem_tag_ctr_inv,
                     avm_binary_op_id,
                     avm_binary_start,
                     avm_byte_lookup_bin_sel,
                     avm_byte_lookup_table_byte_lengths,
                     avm_byte_lookup_table_in_tags,
                     avm_byte_lookup_table_input_a,
                     avm_byte_lookup_table_input_b,
                     avm_byte_lookup_table_op_id,
                     avm_byte_lookup_table_output,
                     avm_main_alu_sel,
                     avm_main_bin_op_id,
                     avm_main_bin_sel,
                     avm_main_ia,
                     avm_main_ib,
                     avm_main_ic,
                     avm_main_id,
                     avm_main_id_zero,
                     avm_main_ind_a,
                     avm_main_ind_b,
                     avm_main_ind_c,
                     avm_main_ind_d,
                     avm_main_ind_op_a,
                     avm_main_ind_op_b,
                     avm_main_ind_op_c,
                     avm_main_ind_op_d,
                     avm_main_internal_return_ptr,
                     avm_main_inv,
                     avm_main_last,
                     avm_main_mem_idx_a,
                     avm_main_mem_idx_b,
                     avm_main_mem_idx_c,
                     avm_main_mem_idx_d,
                     avm_main_mem_op_a,
                     avm_main_mem_op_b,
                     avm_main_mem_op_c,
                     avm_main_mem_op_d,
                     avm_main_op_err,
                     avm_main_pc,
                     avm_main_r_in_tag,
                     avm_main_rwa,
                     avm_main_rwb,
                     avm_main_rwc,
                     avm_main_rwd,
                     avm_main_sel_cmov,
                     avm_main_sel_halt,
                     avm_main_sel_internal_call,
                     avm_main_sel_internal_return,
                     avm_main_sel_jump,
                     avm_main_sel_mov,
                     avm_main_sel_mov_a,
                     avm_main_sel_mov_b,
                     avm_main_sel_op_add,
                     avm_main_sel_op_and,
                     avm_main_sel_op_div,
                     avm_main_sel_op_eq,
                     avm_main_sel_op_lt,
                     avm_main_sel_op_lte,
                     avm_main_sel_op_mul,
                     avm_main_sel_op_not,
                     avm_main_sel_op_or,
                     avm_main_sel_op_sub,
                     avm_main_sel_op_xor,
                     avm_main_sel_rng_16,
                     avm_main_sel_rng_8,
                     avm_main_tag_err,
                     avm_main_w_in_tag,
                     avm_mem_addr,
                     avm_mem_clk,
                     avm_mem_ind_op_a,
                     avm_mem_ind_op_b,
                     avm_mem_ind_op_c,
                     avm_mem_ind_op_d,
                     avm_mem_last,
                     avm_mem_lastAccess,
                     avm_mem_one_min_inv,
                     avm_mem_op_a,
                     avm_mem_op_b,
                     avm_mem_op_c,
                     avm_mem_op_d,
                     avm_mem_r_in_tag,
                     avm_mem_rw,
                     avm_mem_sel_cmov,
                     avm_mem_sel_mov_a,
                     avm_mem_sel_mov_b,
                     avm_mem_skip_check_tag,
                     avm_mem_sub_clk,
                     avm_mem_tag,
                     avm_mem_tag_err,
                     avm_mem_val,
                     avm_mem_w_in_tag,
                     perm_main_alu,
                     perm_main_bin,
                     perm_main_mem_a,
                     perm_main_mem_b,
                     perm_main_mem_c,
                     perm_main_mem_d,
                     perm_main_mem_ind_a,
                     perm_main_mem_ind_b,
                     perm_main_mem_ind_c,
                     perm_main_mem_ind_d,
                     lookup_byte_lengths,
                     lookup_byte_operations,
                     incl_main_tag_err,
                     incl_mem_tag_err,
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
                     lookup_byte_lengths_counts,
                     lookup_byte_operations_counts,
                     incl_main_tag_err_counts,
                     incl_mem_tag_err_counts,
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
                     lookup_u16_14_counts };
        };
    };

    template <typename DataType> class AllEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              avm_main_clk,
                              avm_main_first,
                              avm_alu_a_hi,
                              avm_alu_a_lo,
                              avm_alu_alu_sel,
                              avm_alu_b_hi,
                              avm_alu_b_lo,
                              avm_alu_borrow,
                              avm_alu_cf,
                              avm_alu_clk,
                              avm_alu_cmp_rng_ctr,
                              avm_alu_cmp_sel,
                              avm_alu_ff_tag,
                              avm_alu_ia,
                              avm_alu_ib,
                              avm_alu_ic,
                              avm_alu_in_tag,
                              avm_alu_op_add,
                              avm_alu_op_div,
                              avm_alu_op_eq,
                              avm_alu_op_eq_diff_inv,
                              avm_alu_op_lt,
                              avm_alu_op_lte,
                              avm_alu_op_mul,
                              avm_alu_op_not,
                              avm_alu_op_sub,
                              avm_alu_p_a_borrow,
                              avm_alu_p_b_borrow,
                              avm_alu_p_sub_a_hi,
                              avm_alu_p_sub_a_lo,
                              avm_alu_p_sub_b_hi,
                              avm_alu_p_sub_b_lo,
                              avm_alu_res_hi,
                              avm_alu_res_lo,
                              avm_alu_rng_chk_lookup_selector,
                              avm_alu_rng_chk_sel,
                              avm_alu_u128_tag,
                              avm_alu_u16_r0,
                              avm_alu_u16_r1,
                              avm_alu_u16_r10,
                              avm_alu_u16_r11,
                              avm_alu_u16_r12,
                              avm_alu_u16_r13,
                              avm_alu_u16_r14,
                              avm_alu_u16_r2,
                              avm_alu_u16_r3,
                              avm_alu_u16_r4,
                              avm_alu_u16_r5,
                              avm_alu_u16_r6,
                              avm_alu_u16_r7,
                              avm_alu_u16_r8,
                              avm_alu_u16_r9,
                              avm_alu_u16_tag,
                              avm_alu_u32_tag,
                              avm_alu_u64_r0,
                              avm_alu_u64_tag,
                              avm_alu_u8_r0,
                              avm_alu_u8_r1,
                              avm_alu_u8_tag,
                              avm_binary_acc_ia,
                              avm_binary_acc_ib,
                              avm_binary_acc_ic,
                              avm_binary_bin_sel,
                              avm_binary_clk,
                              avm_binary_ia_bytes,
                              avm_binary_ib_bytes,
                              avm_binary_ic_bytes,
                              avm_binary_in_tag,
                              avm_binary_mem_tag_ctr,
                              avm_binary_mem_tag_ctr_inv,
                              avm_binary_op_id,
                              avm_binary_start,
                              avm_byte_lookup_bin_sel,
                              avm_byte_lookup_table_byte_lengths,
                              avm_byte_lookup_table_in_tags,
                              avm_byte_lookup_table_input_a,
                              avm_byte_lookup_table_input_b,
                              avm_byte_lookup_table_op_id,
                              avm_byte_lookup_table_output,
                              avm_main_alu_sel,
                              avm_main_bin_op_id,
                              avm_main_bin_sel,
                              avm_main_ia,
                              avm_main_ib,
                              avm_main_ic,
                              avm_main_id,
                              avm_main_id_zero,
                              avm_main_ind_a,
                              avm_main_ind_b,
                              avm_main_ind_c,
                              avm_main_ind_d,
                              avm_main_ind_op_a,
                              avm_main_ind_op_b,
                              avm_main_ind_op_c,
                              avm_main_ind_op_d,
                              avm_main_internal_return_ptr,
                              avm_main_inv,
                              avm_main_last,
                              avm_main_mem_idx_a,
                              avm_main_mem_idx_b,
                              avm_main_mem_idx_c,
                              avm_main_mem_idx_d,
                              avm_main_mem_op_a,
                              avm_main_mem_op_b,
                              avm_main_mem_op_c,
                              avm_main_mem_op_d,
                              avm_main_op_err,
                              avm_main_pc,
                              avm_main_r_in_tag,
                              avm_main_rwa,
                              avm_main_rwb,
                              avm_main_rwc,
                              avm_main_rwd,
                              avm_main_sel_cmov,
                              avm_main_sel_halt,
                              avm_main_sel_internal_call,
                              avm_main_sel_internal_return,
                              avm_main_sel_jump,
                              avm_main_sel_mov,
                              avm_main_sel_mov_a,
                              avm_main_sel_mov_b,
                              avm_main_sel_op_add,
                              avm_main_sel_op_and,
                              avm_main_sel_op_div,
                              avm_main_sel_op_eq,
                              avm_main_sel_op_lt,
                              avm_main_sel_op_lte,
                              avm_main_sel_op_mul,
                              avm_main_sel_op_not,
                              avm_main_sel_op_or,
                              avm_main_sel_op_sub,
                              avm_main_sel_op_xor,
                              avm_main_sel_rng_16,
                              avm_main_sel_rng_8,
                              avm_main_tag_err,
                              avm_main_w_in_tag,
                              avm_mem_addr,
                              avm_mem_clk,
                              avm_mem_ind_op_a,
                              avm_mem_ind_op_b,
                              avm_mem_ind_op_c,
                              avm_mem_ind_op_d,
                              avm_mem_last,
                              avm_mem_lastAccess,
                              avm_mem_one_min_inv,
                              avm_mem_op_a,
                              avm_mem_op_b,
                              avm_mem_op_c,
                              avm_mem_op_d,
                              avm_mem_r_in_tag,
                              avm_mem_rw,
                              avm_mem_sel_cmov,
                              avm_mem_sel_mov_a,
                              avm_mem_sel_mov_b,
                              avm_mem_skip_check_tag,
                              avm_mem_sub_clk,
                              avm_mem_tag,
                              avm_mem_tag_err,
                              avm_mem_val,
                              avm_mem_w_in_tag,
                              perm_main_alu,
                              perm_main_bin,
                              perm_main_mem_a,
                              perm_main_mem_b,
                              perm_main_mem_c,
                              perm_main_mem_d,
                              perm_main_mem_ind_a,
                              perm_main_mem_ind_b,
                              perm_main_mem_ind_c,
                              perm_main_mem_ind_d,
                              lookup_byte_lengths,
                              lookup_byte_operations,
                              incl_main_tag_err,
                              incl_mem_tag_err,
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
                              lookup_byte_lengths_counts,
                              lookup_byte_operations_counts,
                              incl_main_tag_err_counts,
                              incl_mem_tag_err_counts,
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
                              avm_alu_a_hi_shift,
                              avm_alu_a_lo_shift,
                              avm_alu_b_hi_shift,
                              avm_alu_b_lo_shift,
                              avm_alu_cmp_rng_ctr_shift,
                              avm_alu_p_sub_a_hi_shift,
                              avm_alu_p_sub_a_lo_shift,
                              avm_alu_p_sub_b_hi_shift,
                              avm_alu_p_sub_b_lo_shift,
                              avm_alu_rng_chk_sel_shift,
                              avm_alu_u16_r0_shift,
                              avm_alu_u16_r1_shift,
                              avm_alu_u16_r2_shift,
                              avm_alu_u16_r3_shift,
                              avm_alu_u16_r4_shift,
                              avm_alu_u16_r5_shift,
                              avm_alu_u16_r6_shift,
                              avm_alu_u16_r7_shift,
                              avm_binary_acc_ia_shift,
                              avm_binary_acc_ib_shift,
                              avm_binary_acc_ic_shift,
                              avm_binary_mem_tag_ctr_shift,
                              avm_binary_op_id_shift,
                              avm_main_internal_return_ptr_shift,
                              avm_main_pc_shift,
                              avm_mem_addr_shift,
                              avm_mem_rw_shift,
                              avm_mem_tag_shift,
                              avm_mem_val_shift)

        RefVector<DataType> get_wires()
        {
            return { avm_main_clk,
                     avm_main_first,
                     avm_alu_a_hi,
                     avm_alu_a_lo,
                     avm_alu_alu_sel,
                     avm_alu_b_hi,
                     avm_alu_b_lo,
                     avm_alu_borrow,
                     avm_alu_cf,
                     avm_alu_clk,
                     avm_alu_cmp_rng_ctr,
                     avm_alu_cmp_sel,
                     avm_alu_ff_tag,
                     avm_alu_ia,
                     avm_alu_ib,
                     avm_alu_ic,
                     avm_alu_in_tag,
                     avm_alu_op_add,
                     avm_alu_op_div,
                     avm_alu_op_eq,
                     avm_alu_op_eq_diff_inv,
                     avm_alu_op_lt,
                     avm_alu_op_lte,
                     avm_alu_op_mul,
                     avm_alu_op_not,
                     avm_alu_op_sub,
                     avm_alu_p_a_borrow,
                     avm_alu_p_b_borrow,
                     avm_alu_p_sub_a_hi,
                     avm_alu_p_sub_a_lo,
                     avm_alu_p_sub_b_hi,
                     avm_alu_p_sub_b_lo,
                     avm_alu_res_hi,
                     avm_alu_res_lo,
                     avm_alu_rng_chk_lookup_selector,
                     avm_alu_rng_chk_sel,
                     avm_alu_u128_tag,
                     avm_alu_u16_r0,
                     avm_alu_u16_r1,
                     avm_alu_u16_r10,
                     avm_alu_u16_r11,
                     avm_alu_u16_r12,
                     avm_alu_u16_r13,
                     avm_alu_u16_r14,
                     avm_alu_u16_r2,
                     avm_alu_u16_r3,
                     avm_alu_u16_r4,
                     avm_alu_u16_r5,
                     avm_alu_u16_r6,
                     avm_alu_u16_r7,
                     avm_alu_u16_r8,
                     avm_alu_u16_r9,
                     avm_alu_u16_tag,
                     avm_alu_u32_tag,
                     avm_alu_u64_r0,
                     avm_alu_u64_tag,
                     avm_alu_u8_r0,
                     avm_alu_u8_r1,
                     avm_alu_u8_tag,
                     avm_binary_acc_ia,
                     avm_binary_acc_ib,
                     avm_binary_acc_ic,
                     avm_binary_bin_sel,
                     avm_binary_clk,
                     avm_binary_ia_bytes,
                     avm_binary_ib_bytes,
                     avm_binary_ic_bytes,
                     avm_binary_in_tag,
                     avm_binary_mem_tag_ctr,
                     avm_binary_mem_tag_ctr_inv,
                     avm_binary_op_id,
                     avm_binary_start,
                     avm_byte_lookup_bin_sel,
                     avm_byte_lookup_table_byte_lengths,
                     avm_byte_lookup_table_in_tags,
                     avm_byte_lookup_table_input_a,
                     avm_byte_lookup_table_input_b,
                     avm_byte_lookup_table_op_id,
                     avm_byte_lookup_table_output,
                     avm_main_alu_sel,
                     avm_main_bin_op_id,
                     avm_main_bin_sel,
                     avm_main_ia,
                     avm_main_ib,
                     avm_main_ic,
                     avm_main_id,
                     avm_main_id_zero,
                     avm_main_ind_a,
                     avm_main_ind_b,
                     avm_main_ind_c,
                     avm_main_ind_d,
                     avm_main_ind_op_a,
                     avm_main_ind_op_b,
                     avm_main_ind_op_c,
                     avm_main_ind_op_d,
                     avm_main_internal_return_ptr,
                     avm_main_inv,
                     avm_main_last,
                     avm_main_mem_idx_a,
                     avm_main_mem_idx_b,
                     avm_main_mem_idx_c,
                     avm_main_mem_idx_d,
                     avm_main_mem_op_a,
                     avm_main_mem_op_b,
                     avm_main_mem_op_c,
                     avm_main_mem_op_d,
                     avm_main_op_err,
                     avm_main_pc,
                     avm_main_r_in_tag,
                     avm_main_rwa,
                     avm_main_rwb,
                     avm_main_rwc,
                     avm_main_rwd,
                     avm_main_sel_cmov,
                     avm_main_sel_halt,
                     avm_main_sel_internal_call,
                     avm_main_sel_internal_return,
                     avm_main_sel_jump,
                     avm_main_sel_mov,
                     avm_main_sel_mov_a,
                     avm_main_sel_mov_b,
                     avm_main_sel_op_add,
                     avm_main_sel_op_and,
                     avm_main_sel_op_div,
                     avm_main_sel_op_eq,
                     avm_main_sel_op_lt,
                     avm_main_sel_op_lte,
                     avm_main_sel_op_mul,
                     avm_main_sel_op_not,
                     avm_main_sel_op_or,
                     avm_main_sel_op_sub,
                     avm_main_sel_op_xor,
                     avm_main_sel_rng_16,
                     avm_main_sel_rng_8,
                     avm_main_tag_err,
                     avm_main_w_in_tag,
                     avm_mem_addr,
                     avm_mem_clk,
                     avm_mem_ind_op_a,
                     avm_mem_ind_op_b,
                     avm_mem_ind_op_c,
                     avm_mem_ind_op_d,
                     avm_mem_last,
                     avm_mem_lastAccess,
                     avm_mem_one_min_inv,
                     avm_mem_op_a,
                     avm_mem_op_b,
                     avm_mem_op_c,
                     avm_mem_op_d,
                     avm_mem_r_in_tag,
                     avm_mem_rw,
                     avm_mem_sel_cmov,
                     avm_mem_sel_mov_a,
                     avm_mem_sel_mov_b,
                     avm_mem_skip_check_tag,
                     avm_mem_sub_clk,
                     avm_mem_tag,
                     avm_mem_tag_err,
                     avm_mem_val,
                     avm_mem_w_in_tag,
                     perm_main_alu,
                     perm_main_bin,
                     perm_main_mem_a,
                     perm_main_mem_b,
                     perm_main_mem_c,
                     perm_main_mem_d,
                     perm_main_mem_ind_a,
                     perm_main_mem_ind_b,
                     perm_main_mem_ind_c,
                     perm_main_mem_ind_d,
                     lookup_byte_lengths,
                     lookup_byte_operations,
                     incl_main_tag_err,
                     incl_mem_tag_err,
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
                     lookup_byte_lengths_counts,
                     lookup_byte_operations_counts,
                     incl_main_tag_err_counts,
                     incl_mem_tag_err_counts,
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
                     avm_alu_a_hi_shift,
                     avm_alu_a_lo_shift,
                     avm_alu_b_hi_shift,
                     avm_alu_b_lo_shift,
                     avm_alu_cmp_rng_ctr_shift,
                     avm_alu_p_sub_a_hi_shift,
                     avm_alu_p_sub_a_lo_shift,
                     avm_alu_p_sub_b_hi_shift,
                     avm_alu_p_sub_b_lo_shift,
                     avm_alu_rng_chk_sel_shift,
                     avm_alu_u16_r0_shift,
                     avm_alu_u16_r1_shift,
                     avm_alu_u16_r2_shift,
                     avm_alu_u16_r3_shift,
                     avm_alu_u16_r4_shift,
                     avm_alu_u16_r5_shift,
                     avm_alu_u16_r6_shift,
                     avm_alu_u16_r7_shift,
                     avm_binary_acc_ia_shift,
                     avm_binary_acc_ib_shift,
                     avm_binary_acc_ic_shift,
                     avm_binary_mem_tag_ctr_shift,
                     avm_binary_op_id_shift,
                     avm_main_internal_return_ptr_shift,
                     avm_main_pc_shift,
                     avm_mem_addr_shift,
                     avm_mem_rw_shift,
                     avm_mem_tag_shift,
                     avm_mem_val_shift };
        };
        RefVector<DataType> get_unshifted()
        {
            return { avm_main_clk,
                     avm_main_first,
                     avm_alu_a_hi,
                     avm_alu_a_lo,
                     avm_alu_alu_sel,
                     avm_alu_b_hi,
                     avm_alu_b_lo,
                     avm_alu_borrow,
                     avm_alu_cf,
                     avm_alu_clk,
                     avm_alu_cmp_rng_ctr,
                     avm_alu_cmp_sel,
                     avm_alu_ff_tag,
                     avm_alu_ia,
                     avm_alu_ib,
                     avm_alu_ic,
                     avm_alu_in_tag,
                     avm_alu_op_add,
                     avm_alu_op_div,
                     avm_alu_op_eq,
                     avm_alu_op_eq_diff_inv,
                     avm_alu_op_lt,
                     avm_alu_op_lte,
                     avm_alu_op_mul,
                     avm_alu_op_not,
                     avm_alu_op_sub,
                     avm_alu_p_a_borrow,
                     avm_alu_p_b_borrow,
                     avm_alu_p_sub_a_hi,
                     avm_alu_p_sub_a_lo,
                     avm_alu_p_sub_b_hi,
                     avm_alu_p_sub_b_lo,
                     avm_alu_res_hi,
                     avm_alu_res_lo,
                     avm_alu_rng_chk_lookup_selector,
                     avm_alu_rng_chk_sel,
                     avm_alu_u128_tag,
                     avm_alu_u16_r0,
                     avm_alu_u16_r1,
                     avm_alu_u16_r10,
                     avm_alu_u16_r11,
                     avm_alu_u16_r12,
                     avm_alu_u16_r13,
                     avm_alu_u16_r14,
                     avm_alu_u16_r2,
                     avm_alu_u16_r3,
                     avm_alu_u16_r4,
                     avm_alu_u16_r5,
                     avm_alu_u16_r6,
                     avm_alu_u16_r7,
                     avm_alu_u16_r8,
                     avm_alu_u16_r9,
                     avm_alu_u16_tag,
                     avm_alu_u32_tag,
                     avm_alu_u64_r0,
                     avm_alu_u64_tag,
                     avm_alu_u8_r0,
                     avm_alu_u8_r1,
                     avm_alu_u8_tag,
                     avm_binary_acc_ia,
                     avm_binary_acc_ib,
                     avm_binary_acc_ic,
                     avm_binary_bin_sel,
                     avm_binary_clk,
                     avm_binary_ia_bytes,
                     avm_binary_ib_bytes,
                     avm_binary_ic_bytes,
                     avm_binary_in_tag,
                     avm_binary_mem_tag_ctr,
                     avm_binary_mem_tag_ctr_inv,
                     avm_binary_op_id,
                     avm_binary_start,
                     avm_byte_lookup_bin_sel,
                     avm_byte_lookup_table_byte_lengths,
                     avm_byte_lookup_table_in_tags,
                     avm_byte_lookup_table_input_a,
                     avm_byte_lookup_table_input_b,
                     avm_byte_lookup_table_op_id,
                     avm_byte_lookup_table_output,
                     avm_main_alu_sel,
                     avm_main_bin_op_id,
                     avm_main_bin_sel,
                     avm_main_ia,
                     avm_main_ib,
                     avm_main_ic,
                     avm_main_id,
                     avm_main_id_zero,
                     avm_main_ind_a,
                     avm_main_ind_b,
                     avm_main_ind_c,
                     avm_main_ind_d,
                     avm_main_ind_op_a,
                     avm_main_ind_op_b,
                     avm_main_ind_op_c,
                     avm_main_ind_op_d,
                     avm_main_internal_return_ptr,
                     avm_main_inv,
                     avm_main_last,
                     avm_main_mem_idx_a,
                     avm_main_mem_idx_b,
                     avm_main_mem_idx_c,
                     avm_main_mem_idx_d,
                     avm_main_mem_op_a,
                     avm_main_mem_op_b,
                     avm_main_mem_op_c,
                     avm_main_mem_op_d,
                     avm_main_op_err,
                     avm_main_pc,
                     avm_main_r_in_tag,
                     avm_main_rwa,
                     avm_main_rwb,
                     avm_main_rwc,
                     avm_main_rwd,
                     avm_main_sel_cmov,
                     avm_main_sel_halt,
                     avm_main_sel_internal_call,
                     avm_main_sel_internal_return,
                     avm_main_sel_jump,
                     avm_main_sel_mov,
                     avm_main_sel_mov_a,
                     avm_main_sel_mov_b,
                     avm_main_sel_op_add,
                     avm_main_sel_op_and,
                     avm_main_sel_op_div,
                     avm_main_sel_op_eq,
                     avm_main_sel_op_lt,
                     avm_main_sel_op_lte,
                     avm_main_sel_op_mul,
                     avm_main_sel_op_not,
                     avm_main_sel_op_or,
                     avm_main_sel_op_sub,
                     avm_main_sel_op_xor,
                     avm_main_sel_rng_16,
                     avm_main_sel_rng_8,
                     avm_main_tag_err,
                     avm_main_w_in_tag,
                     avm_mem_addr,
                     avm_mem_clk,
                     avm_mem_ind_op_a,
                     avm_mem_ind_op_b,
                     avm_mem_ind_op_c,
                     avm_mem_ind_op_d,
                     avm_mem_last,
                     avm_mem_lastAccess,
                     avm_mem_one_min_inv,
                     avm_mem_op_a,
                     avm_mem_op_b,
                     avm_mem_op_c,
                     avm_mem_op_d,
                     avm_mem_r_in_tag,
                     avm_mem_rw,
                     avm_mem_sel_cmov,
                     avm_mem_sel_mov_a,
                     avm_mem_sel_mov_b,
                     avm_mem_skip_check_tag,
                     avm_mem_sub_clk,
                     avm_mem_tag,
                     avm_mem_tag_err,
                     avm_mem_val,
                     avm_mem_w_in_tag,
                     perm_main_alu,
                     perm_main_bin,
                     perm_main_mem_a,
                     perm_main_mem_b,
                     perm_main_mem_c,
                     perm_main_mem_d,
                     perm_main_mem_ind_a,
                     perm_main_mem_ind_b,
                     perm_main_mem_ind_c,
                     perm_main_mem_ind_d,
                     lookup_byte_lengths,
                     lookup_byte_operations,
                     incl_main_tag_err,
                     incl_mem_tag_err,
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
                     lookup_byte_lengths_counts,
                     lookup_byte_operations_counts,
                     incl_main_tag_err_counts,
                     incl_mem_tag_err_counts,
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
                     lookup_u16_14_counts };
        };
        RefVector<DataType> get_to_be_shifted()
        {
            return { avm_alu_a_hi,        avm_alu_a_lo,           avm_alu_b_hi,       avm_alu_b_lo,
                     avm_alu_cmp_rng_ctr, avm_alu_p_sub_a_hi,     avm_alu_p_sub_a_lo, avm_alu_p_sub_b_hi,
                     avm_alu_p_sub_b_lo,  avm_alu_rng_chk_sel,    avm_alu_u16_r0,     avm_alu_u16_r1,
                     avm_alu_u16_r2,      avm_alu_u16_r3,         avm_alu_u16_r4,     avm_alu_u16_r5,
                     avm_alu_u16_r6,      avm_alu_u16_r7,         avm_binary_acc_ia,  avm_binary_acc_ib,
                     avm_binary_acc_ic,   avm_binary_mem_tag_ctr, avm_binary_op_id,   avm_main_internal_return_ptr,
                     avm_main_pc,         avm_mem_addr,           avm_mem_rw,         avm_mem_tag,
                     avm_mem_val };
        };
        RefVector<DataType> get_shifted()
        {
            return { avm_alu_a_hi_shift,        avm_alu_a_lo_shift,
                     avm_alu_b_hi_shift,        avm_alu_b_lo_shift,
                     avm_alu_cmp_rng_ctr_shift, avm_alu_p_sub_a_hi_shift,
                     avm_alu_p_sub_a_lo_shift,  avm_alu_p_sub_b_hi_shift,
                     avm_alu_p_sub_b_lo_shift,  avm_alu_rng_chk_sel_shift,
                     avm_alu_u16_r0_shift,      avm_alu_u16_r1_shift,
                     avm_alu_u16_r2_shift,      avm_alu_u16_r3_shift,
                     avm_alu_u16_r4_shift,      avm_alu_u16_r5_shift,
                     avm_alu_u16_r6_shift,      avm_alu_u16_r7_shift,
                     avm_binary_acc_ia_shift,   avm_binary_acc_ib_shift,
                     avm_binary_acc_ic_shift,   avm_binary_mem_tag_ctr_shift,
                     avm_binary_op_id_shift,    avm_main_internal_return_ptr_shift,
                     avm_main_pc_shift,         avm_mem_addr_shift,
                     avm_mem_rw_shift,          avm_mem_tag_shift,
                     avm_mem_val_shift };
        };
    };

  public:
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey>;
        using Base::Base;

        RefVector<DataType> get_to_be_shifted()
        {
            return { avm_alu_a_hi,        avm_alu_a_lo,           avm_alu_b_hi,       avm_alu_b_lo,
                     avm_alu_cmp_rng_ctr, avm_alu_p_sub_a_hi,     avm_alu_p_sub_a_lo, avm_alu_p_sub_b_hi,
                     avm_alu_p_sub_b_lo,  avm_alu_rng_chk_sel,    avm_alu_u16_r0,     avm_alu_u16_r1,
                     avm_alu_u16_r2,      avm_alu_u16_r3,         avm_alu_u16_r4,     avm_alu_u16_r5,
                     avm_alu_u16_r6,      avm_alu_u16_r7,         avm_binary_acc_ia,  avm_binary_acc_ib,
                     avm_binary_acc_ic,   avm_binary_mem_tag_ctr, avm_binary_op_id,   avm_main_internal_return_ptr,
                     avm_main_pc,         avm_mem_addr,           avm_mem_rw,         avm_mem_tag,
                     avm_mem_val };
        };

        void compute_logderivative_inverses(const RelationParameters<FF>& relation_parameters)
        {
            ProverPolynomials prover_polynomials = ProverPolynomials(*this);

            bb::compute_logderivative_inverse<AvmFlavor, perm_main_alu_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_bin_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_a_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_b_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_c_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_d_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_a_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_b_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_c_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, perm_main_mem_ind_d_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_byte_lengths_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_byte_operations_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, incl_main_tag_err_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, incl_mem_tag_err_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u8_0_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u8_1_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_0_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_1_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_2_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_3_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_4_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_5_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_6_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_7_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_8_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_9_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_10_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_11_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_12_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_13_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
            bb::compute_logderivative_inverse<AvmFlavor, lookup_u16_14_relation<FF>>(
                prover_polynomials, relation_parameters, this->circuit_size);
        }
    };

    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>, VerifierCommitmentKey>;

    using FoldedPolynomials = AllEntities<std::vector<FF>>;

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

        [[nodiscard]] size_t get_polynomial_size() const { return avm_alu_a_hi.size(); }
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

    using RowPolynomials = AllEntities<FF>;

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
            : AllEntities<std::string>()
        {
            Base::avm_main_clk = "AVM_MAIN_CLK";
            Base::avm_main_first = "AVM_MAIN_FIRST";
            Base::avm_alu_a_hi = "AVM_ALU_A_HI";
            Base::avm_alu_a_lo = "AVM_ALU_A_LO";
            Base::avm_alu_alu_sel = "AVM_ALU_ALU_SEL";
            Base::avm_alu_b_hi = "AVM_ALU_B_HI";
            Base::avm_alu_b_lo = "AVM_ALU_B_LO";
            Base::avm_alu_borrow = "AVM_ALU_BORROW";
            Base::avm_alu_cf = "AVM_ALU_CF";
            Base::avm_alu_clk = "AVM_ALU_CLK";
            Base::avm_alu_cmp_rng_ctr = "AVM_ALU_CMP_RNG_CTR";
            Base::avm_alu_cmp_sel = "AVM_ALU_CMP_SEL";
            Base::avm_alu_ff_tag = "AVM_ALU_FF_TAG";
            Base::avm_alu_ia = "AVM_ALU_IA";
            Base::avm_alu_ib = "AVM_ALU_IB";
            Base::avm_alu_ic = "AVM_ALU_IC";
            Base::avm_alu_in_tag = "AVM_ALU_IN_TAG";
            Base::avm_alu_op_add = "AVM_ALU_OP_ADD";
            Base::avm_alu_op_div = "AVM_ALU_OP_DIV";
            Base::avm_alu_op_eq = "AVM_ALU_OP_EQ";
            Base::avm_alu_op_eq_diff_inv = "AVM_ALU_OP_EQ_DIFF_INV";
            Base::avm_alu_op_lt = "AVM_ALU_OP_LT";
            Base::avm_alu_op_lte = "AVM_ALU_OP_LTE";
            Base::avm_alu_op_mul = "AVM_ALU_OP_MUL";
            Base::avm_alu_op_not = "AVM_ALU_OP_NOT";
            Base::avm_alu_op_sub = "AVM_ALU_OP_SUB";
            Base::avm_alu_p_a_borrow = "AVM_ALU_P_A_BORROW";
            Base::avm_alu_p_b_borrow = "AVM_ALU_P_B_BORROW";
            Base::avm_alu_p_sub_a_hi = "AVM_ALU_P_SUB_A_HI";
            Base::avm_alu_p_sub_a_lo = "AVM_ALU_P_SUB_A_LO";
            Base::avm_alu_p_sub_b_hi = "AVM_ALU_P_SUB_B_HI";
            Base::avm_alu_p_sub_b_lo = "AVM_ALU_P_SUB_B_LO";
            Base::avm_alu_res_hi = "AVM_ALU_RES_HI";
            Base::avm_alu_res_lo = "AVM_ALU_RES_LO";
            Base::avm_alu_rng_chk_lookup_selector = "AVM_ALU_RNG_CHK_LOOKUP_SELECTOR";
            Base::avm_alu_rng_chk_sel = "AVM_ALU_RNG_CHK_SEL";
            Base::avm_alu_u128_tag = "AVM_ALU_U128_TAG";
            Base::avm_alu_u16_r0 = "AVM_ALU_U16_R0";
            Base::avm_alu_u16_r1 = "AVM_ALU_U16_R1";
            Base::avm_alu_u16_r10 = "AVM_ALU_U16_R10";
            Base::avm_alu_u16_r11 = "AVM_ALU_U16_R11";
            Base::avm_alu_u16_r12 = "AVM_ALU_U16_R12";
            Base::avm_alu_u16_r13 = "AVM_ALU_U16_R13";
            Base::avm_alu_u16_r14 = "AVM_ALU_U16_R14";
            Base::avm_alu_u16_r2 = "AVM_ALU_U16_R2";
            Base::avm_alu_u16_r3 = "AVM_ALU_U16_R3";
            Base::avm_alu_u16_r4 = "AVM_ALU_U16_R4";
            Base::avm_alu_u16_r5 = "AVM_ALU_U16_R5";
            Base::avm_alu_u16_r6 = "AVM_ALU_U16_R6";
            Base::avm_alu_u16_r7 = "AVM_ALU_U16_R7";
            Base::avm_alu_u16_r8 = "AVM_ALU_U16_R8";
            Base::avm_alu_u16_r9 = "AVM_ALU_U16_R9";
            Base::avm_alu_u16_tag = "AVM_ALU_U16_TAG";
            Base::avm_alu_u32_tag = "AVM_ALU_U32_TAG";
            Base::avm_alu_u64_r0 = "AVM_ALU_U64_R0";
            Base::avm_alu_u64_tag = "AVM_ALU_U64_TAG";
            Base::avm_alu_u8_r0 = "AVM_ALU_U8_R0";
            Base::avm_alu_u8_r1 = "AVM_ALU_U8_R1";
            Base::avm_alu_u8_tag = "AVM_ALU_U8_TAG";
            Base::avm_binary_acc_ia = "AVM_BINARY_ACC_IA";
            Base::avm_binary_acc_ib = "AVM_BINARY_ACC_IB";
            Base::avm_binary_acc_ic = "AVM_BINARY_ACC_IC";
            Base::avm_binary_bin_sel = "AVM_BINARY_BIN_SEL";
            Base::avm_binary_clk = "AVM_BINARY_CLK";
            Base::avm_binary_ia_bytes = "AVM_BINARY_IA_BYTES";
            Base::avm_binary_ib_bytes = "AVM_BINARY_IB_BYTES";
            Base::avm_binary_ic_bytes = "AVM_BINARY_IC_BYTES";
            Base::avm_binary_in_tag = "AVM_BINARY_IN_TAG";
            Base::avm_binary_mem_tag_ctr = "AVM_BINARY_MEM_TAG_CTR";
            Base::avm_binary_mem_tag_ctr_inv = "AVM_BINARY_MEM_TAG_CTR_INV";
            Base::avm_binary_op_id = "AVM_BINARY_OP_ID";
            Base::avm_binary_start = "AVM_BINARY_START";
            Base::avm_byte_lookup_bin_sel = "AVM_BYTE_LOOKUP_BIN_SEL";
            Base::avm_byte_lookup_table_byte_lengths = "AVM_BYTE_LOOKUP_TABLE_BYTE_LENGTHS";
            Base::avm_byte_lookup_table_in_tags = "AVM_BYTE_LOOKUP_TABLE_IN_TAGS";
            Base::avm_byte_lookup_table_input_a = "AVM_BYTE_LOOKUP_TABLE_INPUT_A";
            Base::avm_byte_lookup_table_input_b = "AVM_BYTE_LOOKUP_TABLE_INPUT_B";
            Base::avm_byte_lookup_table_op_id = "AVM_BYTE_LOOKUP_TABLE_OP_ID";
            Base::avm_byte_lookup_table_output = "AVM_BYTE_LOOKUP_TABLE_OUTPUT";
            Base::avm_main_alu_sel = "AVM_MAIN_ALU_SEL";
            Base::avm_main_bin_op_id = "AVM_MAIN_BIN_OP_ID";
            Base::avm_main_bin_sel = "AVM_MAIN_BIN_SEL";
            Base::avm_main_ia = "AVM_MAIN_IA";
            Base::avm_main_ib = "AVM_MAIN_IB";
            Base::avm_main_ic = "AVM_MAIN_IC";
            Base::avm_main_id = "AVM_MAIN_ID";
            Base::avm_main_id_zero = "AVM_MAIN_ID_ZERO";
            Base::avm_main_ind_a = "AVM_MAIN_IND_A";
            Base::avm_main_ind_b = "AVM_MAIN_IND_B";
            Base::avm_main_ind_c = "AVM_MAIN_IND_C";
            Base::avm_main_ind_d = "AVM_MAIN_IND_D";
            Base::avm_main_ind_op_a = "AVM_MAIN_IND_OP_A";
            Base::avm_main_ind_op_b = "AVM_MAIN_IND_OP_B";
            Base::avm_main_ind_op_c = "AVM_MAIN_IND_OP_C";
            Base::avm_main_ind_op_d = "AVM_MAIN_IND_OP_D";
            Base::avm_main_internal_return_ptr = "AVM_MAIN_INTERNAL_RETURN_PTR";
            Base::avm_main_inv = "AVM_MAIN_INV";
            Base::avm_main_last = "AVM_MAIN_LAST";
            Base::avm_main_mem_idx_a = "AVM_MAIN_MEM_IDX_A";
            Base::avm_main_mem_idx_b = "AVM_MAIN_MEM_IDX_B";
            Base::avm_main_mem_idx_c = "AVM_MAIN_MEM_IDX_C";
            Base::avm_main_mem_idx_d = "AVM_MAIN_MEM_IDX_D";
            Base::avm_main_mem_op_a = "AVM_MAIN_MEM_OP_A";
            Base::avm_main_mem_op_b = "AVM_MAIN_MEM_OP_B";
            Base::avm_main_mem_op_c = "AVM_MAIN_MEM_OP_C";
            Base::avm_main_mem_op_d = "AVM_MAIN_MEM_OP_D";
            Base::avm_main_op_err = "AVM_MAIN_OP_ERR";
            Base::avm_main_pc = "AVM_MAIN_PC";
            Base::avm_main_r_in_tag = "AVM_MAIN_R_IN_TAG";
            Base::avm_main_rwa = "AVM_MAIN_RWA";
            Base::avm_main_rwb = "AVM_MAIN_RWB";
            Base::avm_main_rwc = "AVM_MAIN_RWC";
            Base::avm_main_rwd = "AVM_MAIN_RWD";
            Base::avm_main_sel_cmov = "AVM_MAIN_SEL_CMOV";
            Base::avm_main_sel_halt = "AVM_MAIN_SEL_HALT";
            Base::avm_main_sel_internal_call = "AVM_MAIN_SEL_INTERNAL_CALL";
            Base::avm_main_sel_internal_return = "AVM_MAIN_SEL_INTERNAL_RETURN";
            Base::avm_main_sel_jump = "AVM_MAIN_SEL_JUMP";
            Base::avm_main_sel_mov = "AVM_MAIN_SEL_MOV";
            Base::avm_main_sel_mov_a = "AVM_MAIN_SEL_MOV_A";
            Base::avm_main_sel_mov_b = "AVM_MAIN_SEL_MOV_B";
            Base::avm_main_sel_op_add = "AVM_MAIN_SEL_OP_ADD";
            Base::avm_main_sel_op_and = "AVM_MAIN_SEL_OP_AND";
            Base::avm_main_sel_op_div = "AVM_MAIN_SEL_OP_DIV";
            Base::avm_main_sel_op_eq = "AVM_MAIN_SEL_OP_EQ";
            Base::avm_main_sel_op_lt = "AVM_MAIN_SEL_OP_LT";
            Base::avm_main_sel_op_lte = "AVM_MAIN_SEL_OP_LTE";
            Base::avm_main_sel_op_mul = "AVM_MAIN_SEL_OP_MUL";
            Base::avm_main_sel_op_not = "AVM_MAIN_SEL_OP_NOT";
            Base::avm_main_sel_op_or = "AVM_MAIN_SEL_OP_OR";
            Base::avm_main_sel_op_sub = "AVM_MAIN_SEL_OP_SUB";
            Base::avm_main_sel_op_xor = "AVM_MAIN_SEL_OP_XOR";
            Base::avm_main_sel_rng_16 = "AVM_MAIN_SEL_RNG_16";
            Base::avm_main_sel_rng_8 = "AVM_MAIN_SEL_RNG_8";
            Base::avm_main_tag_err = "AVM_MAIN_TAG_ERR";
            Base::avm_main_w_in_tag = "AVM_MAIN_W_IN_TAG";
            Base::avm_mem_addr = "AVM_MEM_ADDR";
            Base::avm_mem_clk = "AVM_MEM_CLK";
            Base::avm_mem_ind_op_a = "AVM_MEM_IND_OP_A";
            Base::avm_mem_ind_op_b = "AVM_MEM_IND_OP_B";
            Base::avm_mem_ind_op_c = "AVM_MEM_IND_OP_C";
            Base::avm_mem_ind_op_d = "AVM_MEM_IND_OP_D";
            Base::avm_mem_last = "AVM_MEM_LAST";
            Base::avm_mem_lastAccess = "AVM_MEM_LASTACCESS";
            Base::avm_mem_one_min_inv = "AVM_MEM_ONE_MIN_INV";
            Base::avm_mem_op_a = "AVM_MEM_OP_A";
            Base::avm_mem_op_b = "AVM_MEM_OP_B";
            Base::avm_mem_op_c = "AVM_MEM_OP_C";
            Base::avm_mem_op_d = "AVM_MEM_OP_D";
            Base::avm_mem_r_in_tag = "AVM_MEM_R_IN_TAG";
            Base::avm_mem_rw = "AVM_MEM_RW";
            Base::avm_mem_sel_cmov = "AVM_MEM_SEL_CMOV";
            Base::avm_mem_sel_mov_a = "AVM_MEM_SEL_MOV_A";
            Base::avm_mem_sel_mov_b = "AVM_MEM_SEL_MOV_B";
            Base::avm_mem_skip_check_tag = "AVM_MEM_SKIP_CHECK_TAG";
            Base::avm_mem_sub_clk = "AVM_MEM_SUB_CLK";
            Base::avm_mem_tag = "AVM_MEM_TAG";
            Base::avm_mem_tag_err = "AVM_MEM_TAG_ERR";
            Base::avm_mem_val = "AVM_MEM_VAL";
            Base::avm_mem_w_in_tag = "AVM_MEM_W_IN_TAG";
            Base::perm_main_alu = "PERM_MAIN_ALU";
            Base::perm_main_bin = "PERM_MAIN_BIN";
            Base::perm_main_mem_a = "PERM_MAIN_MEM_A";
            Base::perm_main_mem_b = "PERM_MAIN_MEM_B";
            Base::perm_main_mem_c = "PERM_MAIN_MEM_C";
            Base::perm_main_mem_d = "PERM_MAIN_MEM_D";
            Base::perm_main_mem_ind_a = "PERM_MAIN_MEM_IND_A";
            Base::perm_main_mem_ind_b = "PERM_MAIN_MEM_IND_B";
            Base::perm_main_mem_ind_c = "PERM_MAIN_MEM_IND_C";
            Base::perm_main_mem_ind_d = "PERM_MAIN_MEM_IND_D";
            Base::lookup_byte_lengths = "LOOKUP_BYTE_LENGTHS";
            Base::lookup_byte_operations = "LOOKUP_BYTE_OPERATIONS";
            Base::incl_main_tag_err = "INCL_MAIN_TAG_ERR";
            Base::incl_mem_tag_err = "INCL_MEM_TAG_ERR";
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
            Base::lookup_byte_lengths_counts = "LOOKUP_BYTE_LENGTHS_COUNTS";
            Base::lookup_byte_operations_counts = "LOOKUP_BYTE_OPERATIONS_COUNTS";
            Base::incl_main_tag_err_counts = "INCL_MAIN_TAG_ERR_COUNTS";
            Base::incl_mem_tag_err_counts = "INCL_MEM_TAG_ERR_COUNTS";
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
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      private:
        using Base = AllEntities<Commitment>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            avm_main_clk = verification_key->avm_main_clk;
            avm_main_first = verification_key->avm_main_first;
        }
    };

    class Transcript : public NativeTranscript {
      public:
        uint32_t circuit_size;

        Commitment avm_alu_a_hi;
        Commitment avm_alu_a_lo;
        Commitment avm_alu_alu_sel;
        Commitment avm_alu_b_hi;
        Commitment avm_alu_b_lo;
        Commitment avm_alu_borrow;
        Commitment avm_alu_cf;
        Commitment avm_alu_clk;
        Commitment avm_alu_cmp_rng_ctr;
        Commitment avm_alu_cmp_sel;
        Commitment avm_alu_ff_tag;
        Commitment avm_alu_ia;
        Commitment avm_alu_ib;
        Commitment avm_alu_ic;
        Commitment avm_alu_in_tag;
        Commitment avm_alu_op_add;
        Commitment avm_alu_op_div;
        Commitment avm_alu_op_eq;
        Commitment avm_alu_op_eq_diff_inv;
        Commitment avm_alu_op_lt;
        Commitment avm_alu_op_lte;
        Commitment avm_alu_op_mul;
        Commitment avm_alu_op_not;
        Commitment avm_alu_op_sub;
        Commitment avm_alu_p_a_borrow;
        Commitment avm_alu_p_b_borrow;
        Commitment avm_alu_p_sub_a_hi;
        Commitment avm_alu_p_sub_a_lo;
        Commitment avm_alu_p_sub_b_hi;
        Commitment avm_alu_p_sub_b_lo;
        Commitment avm_alu_res_hi;
        Commitment avm_alu_res_lo;
        Commitment avm_alu_rng_chk_lookup_selector;
        Commitment avm_alu_rng_chk_sel;
        Commitment avm_alu_u128_tag;
        Commitment avm_alu_u16_r0;
        Commitment avm_alu_u16_r1;
        Commitment avm_alu_u16_r10;
        Commitment avm_alu_u16_r11;
        Commitment avm_alu_u16_r12;
        Commitment avm_alu_u16_r13;
        Commitment avm_alu_u16_r14;
        Commitment avm_alu_u16_r2;
        Commitment avm_alu_u16_r3;
        Commitment avm_alu_u16_r4;
        Commitment avm_alu_u16_r5;
        Commitment avm_alu_u16_r6;
        Commitment avm_alu_u16_r7;
        Commitment avm_alu_u16_r8;
        Commitment avm_alu_u16_r9;
        Commitment avm_alu_u16_tag;
        Commitment avm_alu_u32_tag;
        Commitment avm_alu_u64_r0;
        Commitment avm_alu_u64_tag;
        Commitment avm_alu_u8_r0;
        Commitment avm_alu_u8_r1;
        Commitment avm_alu_u8_tag;
        Commitment avm_binary_acc_ia;
        Commitment avm_binary_acc_ib;
        Commitment avm_binary_acc_ic;
        Commitment avm_binary_bin_sel;
        Commitment avm_binary_clk;
        Commitment avm_binary_ia_bytes;
        Commitment avm_binary_ib_bytes;
        Commitment avm_binary_ic_bytes;
        Commitment avm_binary_in_tag;
        Commitment avm_binary_mem_tag_ctr;
        Commitment avm_binary_mem_tag_ctr_inv;
        Commitment avm_binary_op_id;
        Commitment avm_binary_start;
        Commitment avm_byte_lookup_bin_sel;
        Commitment avm_byte_lookup_table_byte_lengths;
        Commitment avm_byte_lookup_table_in_tags;
        Commitment avm_byte_lookup_table_input_a;
        Commitment avm_byte_lookup_table_input_b;
        Commitment avm_byte_lookup_table_op_id;
        Commitment avm_byte_lookup_table_output;
        Commitment avm_main_alu_sel;
        Commitment avm_main_bin_op_id;
        Commitment avm_main_bin_sel;
        Commitment avm_main_ia;
        Commitment avm_main_ib;
        Commitment avm_main_ic;
        Commitment avm_main_id;
        Commitment avm_main_id_zero;
        Commitment avm_main_ind_a;
        Commitment avm_main_ind_b;
        Commitment avm_main_ind_c;
        Commitment avm_main_ind_d;
        Commitment avm_main_ind_op_a;
        Commitment avm_main_ind_op_b;
        Commitment avm_main_ind_op_c;
        Commitment avm_main_ind_op_d;
        Commitment avm_main_internal_return_ptr;
        Commitment avm_main_inv;
        Commitment avm_main_last;
        Commitment avm_main_mem_idx_a;
        Commitment avm_main_mem_idx_b;
        Commitment avm_main_mem_idx_c;
        Commitment avm_main_mem_idx_d;
        Commitment avm_main_mem_op_a;
        Commitment avm_main_mem_op_b;
        Commitment avm_main_mem_op_c;
        Commitment avm_main_mem_op_d;
        Commitment avm_main_op_err;
        Commitment avm_main_pc;
        Commitment avm_main_r_in_tag;
        Commitment avm_main_rwa;
        Commitment avm_main_rwb;
        Commitment avm_main_rwc;
        Commitment avm_main_rwd;
        Commitment avm_main_sel_cmov;
        Commitment avm_main_sel_halt;
        Commitment avm_main_sel_internal_call;
        Commitment avm_main_sel_internal_return;
        Commitment avm_main_sel_jump;
        Commitment avm_main_sel_mov;
        Commitment avm_main_sel_mov_a;
        Commitment avm_main_sel_mov_b;
        Commitment avm_main_sel_op_add;
        Commitment avm_main_sel_op_and;
        Commitment avm_main_sel_op_div;
        Commitment avm_main_sel_op_eq;
        Commitment avm_main_sel_op_lt;
        Commitment avm_main_sel_op_lte;
        Commitment avm_main_sel_op_mul;
        Commitment avm_main_sel_op_not;
        Commitment avm_main_sel_op_or;
        Commitment avm_main_sel_op_sub;
        Commitment avm_main_sel_op_xor;
        Commitment avm_main_sel_rng_16;
        Commitment avm_main_sel_rng_8;
        Commitment avm_main_tag_err;
        Commitment avm_main_w_in_tag;
        Commitment avm_mem_addr;
        Commitment avm_mem_clk;
        Commitment avm_mem_ind_op_a;
        Commitment avm_mem_ind_op_b;
        Commitment avm_mem_ind_op_c;
        Commitment avm_mem_ind_op_d;
        Commitment avm_mem_last;
        Commitment avm_mem_lastAccess;
        Commitment avm_mem_one_min_inv;
        Commitment avm_mem_op_a;
        Commitment avm_mem_op_b;
        Commitment avm_mem_op_c;
        Commitment avm_mem_op_d;
        Commitment avm_mem_r_in_tag;
        Commitment avm_mem_rw;
        Commitment avm_mem_sel_cmov;
        Commitment avm_mem_sel_mov_a;
        Commitment avm_mem_sel_mov_b;
        Commitment avm_mem_skip_check_tag;
        Commitment avm_mem_sub_clk;
        Commitment avm_mem_tag;
        Commitment avm_mem_tag_err;
        Commitment avm_mem_val;
        Commitment avm_mem_w_in_tag;
        Commitment perm_main_alu;
        Commitment perm_main_bin;
        Commitment perm_main_mem_a;
        Commitment perm_main_mem_b;
        Commitment perm_main_mem_c;
        Commitment perm_main_mem_d;
        Commitment perm_main_mem_ind_a;
        Commitment perm_main_mem_ind_b;
        Commitment perm_main_mem_ind_c;
        Commitment perm_main_mem_ind_d;
        Commitment lookup_byte_lengths;
        Commitment lookup_byte_operations;
        Commitment incl_main_tag_err;
        Commitment incl_mem_tag_err;
        Commitment lookup_u8_0;
        Commitment lookup_u8_1;
        Commitment lookup_u16_0;
        Commitment lookup_u16_1;
        Commitment lookup_u16_2;
        Commitment lookup_u16_3;
        Commitment lookup_u16_4;
        Commitment lookup_u16_5;
        Commitment lookup_u16_6;
        Commitment lookup_u16_7;
        Commitment lookup_u16_8;
        Commitment lookup_u16_9;
        Commitment lookup_u16_10;
        Commitment lookup_u16_11;
        Commitment lookup_u16_12;
        Commitment lookup_u16_13;
        Commitment lookup_u16_14;
        Commitment lookup_byte_lengths_counts;
        Commitment lookup_byte_operations_counts;
        Commitment incl_main_tag_err_counts;
        Commitment incl_mem_tag_err_counts;
        Commitment lookup_u8_0_counts;
        Commitment lookup_u8_1_counts;
        Commitment lookup_u16_0_counts;
        Commitment lookup_u16_1_counts;
        Commitment lookup_u16_2_counts;
        Commitment lookup_u16_3_counts;
        Commitment lookup_u16_4_counts;
        Commitment lookup_u16_5_counts;
        Commitment lookup_u16_6_counts;
        Commitment lookup_u16_7_counts;
        Commitment lookup_u16_8_counts;
        Commitment lookup_u16_9_counts;
        Commitment lookup_u16_10_counts;
        Commitment lookup_u16_11_counts;
        Commitment lookup_u16_12_counts;
        Commitment lookup_u16_13_counts;
        Commitment lookup_u16_14_counts;

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

            avm_alu_a_hi = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_a_lo = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_b_hi = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_b_lo = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_borrow = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_cf = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_cmp_rng_ctr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_cmp_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_ff_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_add = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_div = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_eq = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_eq_diff_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_lt = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_lte = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_mul = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_not = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_op_sub = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_p_a_borrow = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_p_b_borrow = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_p_sub_a_hi = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_p_sub_a_lo = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_p_sub_b_hi = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_p_sub_b_lo = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_res_hi = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_res_lo = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_rng_chk_lookup_selector = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_rng_chk_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u128_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r10 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r11 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r12 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r13 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r14 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r2 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r3 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r4 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r5 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r6 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r7 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r8 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_r9 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u16_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u32_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u64_r0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u64_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u8_r0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u8_r1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_u8_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_acc_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_acc_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_acc_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_bin_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_ia_bytes = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_ib_bytes = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_ic_bytes = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_mem_tag_ctr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_mem_tag_ctr_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_op_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_start = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_bin_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_byte_lengths =
                deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_in_tags = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_input_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_input_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_op_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_output = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_alu_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_bin_op_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_bin_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_id_zero = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_internal_return_ptr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_last = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_op_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_pc = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_r_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwa = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwb = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwc = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwd = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_cmov = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_halt = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_internal_call = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_internal_return = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_jump = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_mov = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_mov_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_mov_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_add = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_and = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_div = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_eq = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_lt = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_lte = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_mul = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_not = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_or = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_sub = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_xor = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_rng_16 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_rng_8 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_w_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_addr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_ind_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_ind_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_ind_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_ind_op_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_last = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_lastAccess = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_one_min_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_op_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_r_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_rw = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_sel_cmov = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_sel_mov_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_sel_mov_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_skip_check_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_sub_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_val = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_w_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_alu = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_bin = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_d = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_lengths = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_operations = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_main_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_mem_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u8_0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u8_1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_2 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_3 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_4 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_5 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_6 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_7 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_8 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_9 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_10 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_11 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_12 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_13 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_14 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_lengths_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_operations_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_main_tag_err_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_mem_tag_err_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u8_0_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u8_1_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_0_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_1_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_2_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_3_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_4_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_5_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_6_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_7_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_8_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_9_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_10_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_11_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_12_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_13_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_u16_14_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);

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

            serialize_to_buffer<Commitment>(avm_alu_a_hi, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_a_lo, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_b_hi, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_b_lo, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_borrow, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_cf, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_cmp_rng_ctr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_cmp_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_ff_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_add, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_div, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_eq, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_eq_diff_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_lt, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_lte, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_mul, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_not, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_op_sub, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_p_a_borrow, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_p_b_borrow, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_p_sub_a_hi, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_p_sub_a_lo, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_p_sub_b_hi, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_p_sub_b_lo, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_res_hi, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_res_lo, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_rng_chk_lookup_selector, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_rng_chk_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u128_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r10, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r11, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r12, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r13, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r14, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r2, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r3, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r4, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r5, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r6, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r7, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r8, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_r9, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u16_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u32_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u64_r0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u64_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u8_r0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u8_r1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_u8_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_acc_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_acc_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_acc_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_bin_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_ia_bytes, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_ib_bytes, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_ic_bytes, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_mem_tag_ctr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_mem_tag_ctr_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_op_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_start, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_bin_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_byte_lengths, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_in_tags, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_input_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_input_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_op_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_output, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_alu_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_bin_op_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_bin_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_id_zero, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_internal_return_ptr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_last, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_op_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_pc, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_r_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwa, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwb, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwc, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwd, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_cmov, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_halt, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_internal_call, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_internal_return, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_jump, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_mov, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_mov_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_mov_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_add, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_and, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_div, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_eq, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_lt, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_lte, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_mul, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_not, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_or, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_sub, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_xor, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_rng_16, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_rng_8, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_w_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_addr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_ind_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_ind_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_ind_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_ind_op_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_last, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_lastAccess, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_one_min_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_op_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_r_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_rw, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_sel_cmov, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_sel_mov_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_sel_mov_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_skip_check_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_sub_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_val, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_w_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_alu, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_bin, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_d, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_lengths, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_operations, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_main_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_mem_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u8_0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u8_1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_2, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_3, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_4, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_5, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_6, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_7, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_8, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_9, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_10, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_11, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_12, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_13, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_14, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_lengths_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_operations_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_main_tag_err_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_mem_tag_err_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u8_0_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u8_1_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_0_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_1_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_2_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_3_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_4_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_5_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_6_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_7_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_8_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_9_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_10_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_11_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_12_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_13_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_u16_14_counts, Transcript::proof_data);

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
