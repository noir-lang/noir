

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
#include "barretenberg/relations/generated/avm/perm_main_alu.hpp"
#include "barretenberg/relations/generated/avm/perm_main_bin.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_a.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_b.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_c.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_a.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_b.hpp"
#include "barretenberg/relations/generated/avm/perm_main_mem_ind_c.hpp"
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
    static constexpr size_t NUM_WITNESS_ENTITIES = 131;
    static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
    // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for
    // the unshifted and one for the shifted
    static constexpr size_t NUM_ALL_ENTITIES = 152;

    using Relations = std::tuple<Avm_vm::avm_mem<FF>,
                                 Avm_vm::avm_binary<FF>,
                                 Avm_vm::avm_main<FF>,
                                 Avm_vm::avm_alu<FF>,
                                 perm_main_alu_relation<FF>,
                                 perm_main_bin_relation<FF>,
                                 perm_main_mem_a_relation<FF>,
                                 perm_main_mem_b_relation<FF>,
                                 perm_main_mem_c_relation<FF>,
                                 perm_main_mem_ind_a_relation<FF>,
                                 perm_main_mem_ind_b_relation<FF>,
                                 perm_main_mem_ind_c_relation<FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

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
                              avm_mem_m_clk,
                              avm_mem_m_sub_clk,
                              avm_mem_m_addr,
                              avm_mem_m_tag,
                              avm_mem_m_val,
                              avm_mem_m_lastAccess,
                              avm_mem_m_last,
                              avm_mem_m_rw,
                              avm_mem_r_in_tag,
                              avm_mem_w_in_tag,
                              avm_mem_m_op_a,
                              avm_mem_m_op_b,
                              avm_mem_m_op_c,
                              avm_mem_m_ind_op_a,
                              avm_mem_m_ind_op_b,
                              avm_mem_m_ind_op_c,
                              avm_mem_m_sel_mov,
                              avm_mem_m_tag_err,
                              avm_mem_m_one_min_inv,
                              avm_alu_alu_clk,
                              avm_alu_alu_ia,
                              avm_alu_alu_ib,
                              avm_alu_alu_ic,
                              avm_alu_alu_op_add,
                              avm_alu_alu_op_sub,
                              avm_alu_alu_op_mul,
                              avm_alu_alu_op_div,
                              avm_alu_alu_op_not,
                              avm_alu_alu_op_eq,
                              avm_alu_alu_sel,
                              avm_alu_alu_in_tag,
                              avm_alu_alu_ff_tag,
                              avm_alu_alu_u8_tag,
                              avm_alu_alu_u16_tag,
                              avm_alu_alu_u32_tag,
                              avm_alu_alu_u64_tag,
                              avm_alu_alu_u128_tag,
                              avm_alu_alu_u8_r0,
                              avm_alu_alu_u8_r1,
                              avm_alu_alu_u16_r0,
                              avm_alu_alu_u16_r1,
                              avm_alu_alu_u16_r2,
                              avm_alu_alu_u16_r3,
                              avm_alu_alu_u16_r4,
                              avm_alu_alu_u16_r5,
                              avm_alu_alu_u16_r6,
                              avm_alu_alu_u16_r7,
                              avm_alu_alu_u64_r0,
                              avm_alu_alu_cf,
                              avm_alu_alu_op_eq_diff_inv,
                              avm_byte_lookup_table_op_id,
                              avm_byte_lookup_table_input_a,
                              avm_byte_lookup_table_input_b,
                              avm_byte_lookup_table_output,
                              avm_byte_lookup_bin_sel,
                              avm_byte_lookup_table_in_tags,
                              avm_byte_lookup_table_byte_lengths,
                              avm_binary_bin_clk,
                              avm_binary_bin_sel,
                              avm_binary_acc_ia,
                              avm_binary_acc_ib,
                              avm_binary_acc_ic,
                              avm_binary_in_tag,
                              avm_binary_op_id,
                              avm_binary_bin_ia_bytes,
                              avm_binary_bin_ib_bytes,
                              avm_binary_bin_ic_bytes,
                              avm_binary_start,
                              avm_binary_mem_tag_ctr,
                              avm_binary_mem_tag_ctr_inv,
                              avm_main_sel_rng_8,
                              avm_main_sel_rng_16,
                              avm_main_pc,
                              avm_main_internal_return_ptr,
                              avm_main_sel_internal_call,
                              avm_main_sel_internal_return,
                              avm_main_sel_jump,
                              avm_main_sel_halt,
                              avm_main_sel_mov,
                              avm_main_sel_op_add,
                              avm_main_sel_op_sub,
                              avm_main_sel_op_mul,
                              avm_main_sel_op_div,
                              avm_main_sel_op_not,
                              avm_main_sel_op_eq,
                              avm_main_sel_op_and,
                              avm_main_sel_op_or,
                              avm_main_sel_op_xor,
                              avm_main_alu_sel,
                              avm_main_bin_sel,
                              avm_main_r_in_tag,
                              avm_main_w_in_tag,
                              avm_main_op_err,
                              avm_main_tag_err,
                              avm_main_inv,
                              avm_main_ia,
                              avm_main_ib,
                              avm_main_ic,
                              avm_main_mem_op_a,
                              avm_main_mem_op_b,
                              avm_main_mem_op_c,
                              avm_main_rwa,
                              avm_main_rwb,
                              avm_main_rwc,
                              avm_main_ind_a,
                              avm_main_ind_b,
                              avm_main_ind_c,
                              avm_main_ind_op_a,
                              avm_main_ind_op_b,
                              avm_main_ind_op_c,
                              avm_main_mem_idx_a,
                              avm_main_mem_idx_b,
                              avm_main_mem_idx_c,
                              avm_main_last,
                              avm_main_bin_op_id,
                              perm_main_alu,
                              perm_main_bin,
                              perm_main_mem_a,
                              perm_main_mem_b,
                              perm_main_mem_c,
                              perm_main_mem_ind_a,
                              perm_main_mem_ind_b,
                              perm_main_mem_ind_c,
                              lookup_byte_lengths,
                              lookup_byte_operations,
                              incl_main_tag_err,
                              incl_mem_tag_err,
                              lookup_byte_lengths_counts,
                              lookup_byte_operations_counts,
                              incl_main_tag_err_counts,
                              incl_mem_tag_err_counts)

        RefVector<DataType> get_wires()
        {
            return { avm_mem_m_clk,
                     avm_mem_m_sub_clk,
                     avm_mem_m_addr,
                     avm_mem_m_tag,
                     avm_mem_m_val,
                     avm_mem_m_lastAccess,
                     avm_mem_m_last,
                     avm_mem_m_rw,
                     avm_mem_r_in_tag,
                     avm_mem_w_in_tag,
                     avm_mem_m_op_a,
                     avm_mem_m_op_b,
                     avm_mem_m_op_c,
                     avm_mem_m_ind_op_a,
                     avm_mem_m_ind_op_b,
                     avm_mem_m_ind_op_c,
                     avm_mem_m_sel_mov,
                     avm_mem_m_tag_err,
                     avm_mem_m_one_min_inv,
                     avm_alu_alu_clk,
                     avm_alu_alu_ia,
                     avm_alu_alu_ib,
                     avm_alu_alu_ic,
                     avm_alu_alu_op_add,
                     avm_alu_alu_op_sub,
                     avm_alu_alu_op_mul,
                     avm_alu_alu_op_div,
                     avm_alu_alu_op_not,
                     avm_alu_alu_op_eq,
                     avm_alu_alu_sel,
                     avm_alu_alu_in_tag,
                     avm_alu_alu_ff_tag,
                     avm_alu_alu_u8_tag,
                     avm_alu_alu_u16_tag,
                     avm_alu_alu_u32_tag,
                     avm_alu_alu_u64_tag,
                     avm_alu_alu_u128_tag,
                     avm_alu_alu_u8_r0,
                     avm_alu_alu_u8_r1,
                     avm_alu_alu_u16_r0,
                     avm_alu_alu_u16_r1,
                     avm_alu_alu_u16_r2,
                     avm_alu_alu_u16_r3,
                     avm_alu_alu_u16_r4,
                     avm_alu_alu_u16_r5,
                     avm_alu_alu_u16_r6,
                     avm_alu_alu_u16_r7,
                     avm_alu_alu_u64_r0,
                     avm_alu_alu_cf,
                     avm_alu_alu_op_eq_diff_inv,
                     avm_byte_lookup_table_op_id,
                     avm_byte_lookup_table_input_a,
                     avm_byte_lookup_table_input_b,
                     avm_byte_lookup_table_output,
                     avm_byte_lookup_bin_sel,
                     avm_byte_lookup_table_in_tags,
                     avm_byte_lookup_table_byte_lengths,
                     avm_binary_bin_clk,
                     avm_binary_bin_sel,
                     avm_binary_acc_ia,
                     avm_binary_acc_ib,
                     avm_binary_acc_ic,
                     avm_binary_in_tag,
                     avm_binary_op_id,
                     avm_binary_bin_ia_bytes,
                     avm_binary_bin_ib_bytes,
                     avm_binary_bin_ic_bytes,
                     avm_binary_start,
                     avm_binary_mem_tag_ctr,
                     avm_binary_mem_tag_ctr_inv,
                     avm_main_sel_rng_8,
                     avm_main_sel_rng_16,
                     avm_main_pc,
                     avm_main_internal_return_ptr,
                     avm_main_sel_internal_call,
                     avm_main_sel_internal_return,
                     avm_main_sel_jump,
                     avm_main_sel_halt,
                     avm_main_sel_mov,
                     avm_main_sel_op_add,
                     avm_main_sel_op_sub,
                     avm_main_sel_op_mul,
                     avm_main_sel_op_div,
                     avm_main_sel_op_not,
                     avm_main_sel_op_eq,
                     avm_main_sel_op_and,
                     avm_main_sel_op_or,
                     avm_main_sel_op_xor,
                     avm_main_alu_sel,
                     avm_main_bin_sel,
                     avm_main_r_in_tag,
                     avm_main_w_in_tag,
                     avm_main_op_err,
                     avm_main_tag_err,
                     avm_main_inv,
                     avm_main_ia,
                     avm_main_ib,
                     avm_main_ic,
                     avm_main_mem_op_a,
                     avm_main_mem_op_b,
                     avm_main_mem_op_c,
                     avm_main_rwa,
                     avm_main_rwb,
                     avm_main_rwc,
                     avm_main_ind_a,
                     avm_main_ind_b,
                     avm_main_ind_c,
                     avm_main_ind_op_a,
                     avm_main_ind_op_b,
                     avm_main_ind_op_c,
                     avm_main_mem_idx_a,
                     avm_main_mem_idx_b,
                     avm_main_mem_idx_c,
                     avm_main_last,
                     avm_main_bin_op_id,
                     perm_main_alu,
                     perm_main_bin,
                     perm_main_mem_a,
                     perm_main_mem_b,
                     perm_main_mem_c,
                     perm_main_mem_ind_a,
                     perm_main_mem_ind_b,
                     perm_main_mem_ind_c,
                     lookup_byte_lengths,
                     lookup_byte_operations,
                     incl_main_tag_err,
                     incl_mem_tag_err,
                     lookup_byte_lengths_counts,
                     lookup_byte_operations_counts,
                     incl_main_tag_err_counts,
                     incl_mem_tag_err_counts };
        };
        RefVector<DataType> get_sorted_polynomials() { return {}; };
    };

    template <typename DataType> class AllEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              avm_main_clk,
                              avm_main_first,
                              avm_mem_m_clk,
                              avm_mem_m_sub_clk,
                              avm_mem_m_addr,
                              avm_mem_m_tag,
                              avm_mem_m_val,
                              avm_mem_m_lastAccess,
                              avm_mem_m_last,
                              avm_mem_m_rw,
                              avm_mem_r_in_tag,
                              avm_mem_w_in_tag,
                              avm_mem_m_op_a,
                              avm_mem_m_op_b,
                              avm_mem_m_op_c,
                              avm_mem_m_ind_op_a,
                              avm_mem_m_ind_op_b,
                              avm_mem_m_ind_op_c,
                              avm_mem_m_sel_mov,
                              avm_mem_m_tag_err,
                              avm_mem_m_one_min_inv,
                              avm_alu_alu_clk,
                              avm_alu_alu_ia,
                              avm_alu_alu_ib,
                              avm_alu_alu_ic,
                              avm_alu_alu_op_add,
                              avm_alu_alu_op_sub,
                              avm_alu_alu_op_mul,
                              avm_alu_alu_op_div,
                              avm_alu_alu_op_not,
                              avm_alu_alu_op_eq,
                              avm_alu_alu_sel,
                              avm_alu_alu_in_tag,
                              avm_alu_alu_ff_tag,
                              avm_alu_alu_u8_tag,
                              avm_alu_alu_u16_tag,
                              avm_alu_alu_u32_tag,
                              avm_alu_alu_u64_tag,
                              avm_alu_alu_u128_tag,
                              avm_alu_alu_u8_r0,
                              avm_alu_alu_u8_r1,
                              avm_alu_alu_u16_r0,
                              avm_alu_alu_u16_r1,
                              avm_alu_alu_u16_r2,
                              avm_alu_alu_u16_r3,
                              avm_alu_alu_u16_r4,
                              avm_alu_alu_u16_r5,
                              avm_alu_alu_u16_r6,
                              avm_alu_alu_u16_r7,
                              avm_alu_alu_u64_r0,
                              avm_alu_alu_cf,
                              avm_alu_alu_op_eq_diff_inv,
                              avm_byte_lookup_table_op_id,
                              avm_byte_lookup_table_input_a,
                              avm_byte_lookup_table_input_b,
                              avm_byte_lookup_table_output,
                              avm_byte_lookup_bin_sel,
                              avm_byte_lookup_table_in_tags,
                              avm_byte_lookup_table_byte_lengths,
                              avm_binary_bin_clk,
                              avm_binary_bin_sel,
                              avm_binary_acc_ia,
                              avm_binary_acc_ib,
                              avm_binary_acc_ic,
                              avm_binary_in_tag,
                              avm_binary_op_id,
                              avm_binary_bin_ia_bytes,
                              avm_binary_bin_ib_bytes,
                              avm_binary_bin_ic_bytes,
                              avm_binary_start,
                              avm_binary_mem_tag_ctr,
                              avm_binary_mem_tag_ctr_inv,
                              avm_main_sel_rng_8,
                              avm_main_sel_rng_16,
                              avm_main_pc,
                              avm_main_internal_return_ptr,
                              avm_main_sel_internal_call,
                              avm_main_sel_internal_return,
                              avm_main_sel_jump,
                              avm_main_sel_halt,
                              avm_main_sel_mov,
                              avm_main_sel_op_add,
                              avm_main_sel_op_sub,
                              avm_main_sel_op_mul,
                              avm_main_sel_op_div,
                              avm_main_sel_op_not,
                              avm_main_sel_op_eq,
                              avm_main_sel_op_and,
                              avm_main_sel_op_or,
                              avm_main_sel_op_xor,
                              avm_main_alu_sel,
                              avm_main_bin_sel,
                              avm_main_r_in_tag,
                              avm_main_w_in_tag,
                              avm_main_op_err,
                              avm_main_tag_err,
                              avm_main_inv,
                              avm_main_ia,
                              avm_main_ib,
                              avm_main_ic,
                              avm_main_mem_op_a,
                              avm_main_mem_op_b,
                              avm_main_mem_op_c,
                              avm_main_rwa,
                              avm_main_rwb,
                              avm_main_rwc,
                              avm_main_ind_a,
                              avm_main_ind_b,
                              avm_main_ind_c,
                              avm_main_ind_op_a,
                              avm_main_ind_op_b,
                              avm_main_ind_op_c,
                              avm_main_mem_idx_a,
                              avm_main_mem_idx_b,
                              avm_main_mem_idx_c,
                              avm_main_last,
                              avm_main_bin_op_id,
                              perm_main_alu,
                              perm_main_bin,
                              perm_main_mem_a,
                              perm_main_mem_b,
                              perm_main_mem_c,
                              perm_main_mem_ind_a,
                              perm_main_mem_ind_b,
                              perm_main_mem_ind_c,
                              lookup_byte_lengths,
                              lookup_byte_operations,
                              incl_main_tag_err,
                              incl_mem_tag_err,
                              lookup_byte_lengths_counts,
                              lookup_byte_operations_counts,
                              incl_main_tag_err_counts,
                              incl_mem_tag_err_counts,
                              avm_mem_m_tag_shift,
                              avm_mem_m_rw_shift,
                              avm_mem_m_addr_shift,
                              avm_mem_m_val_shift,
                              avm_binary_acc_ic_shift,
                              avm_binary_op_id_shift,
                              avm_binary_acc_ib_shift,
                              avm_binary_mem_tag_ctr_shift,
                              avm_binary_acc_ia_shift,
                              avm_main_internal_return_ptr_shift,
                              avm_main_pc_shift,
                              avm_alu_alu_u16_r1_shift,
                              avm_alu_alu_u16_r3_shift,
                              avm_alu_alu_u16_r2_shift,
                              avm_alu_alu_u16_r5_shift,
                              avm_alu_alu_u16_r4_shift,
                              avm_alu_alu_u16_r6_shift,
                              avm_alu_alu_u16_r0_shift,
                              avm_alu_alu_u16_r7_shift)

        RefVector<DataType> get_wires()
        {
            return { avm_main_clk,
                     avm_main_first,
                     avm_mem_m_clk,
                     avm_mem_m_sub_clk,
                     avm_mem_m_addr,
                     avm_mem_m_tag,
                     avm_mem_m_val,
                     avm_mem_m_lastAccess,
                     avm_mem_m_last,
                     avm_mem_m_rw,
                     avm_mem_r_in_tag,
                     avm_mem_w_in_tag,
                     avm_mem_m_op_a,
                     avm_mem_m_op_b,
                     avm_mem_m_op_c,
                     avm_mem_m_ind_op_a,
                     avm_mem_m_ind_op_b,
                     avm_mem_m_ind_op_c,
                     avm_mem_m_sel_mov,
                     avm_mem_m_tag_err,
                     avm_mem_m_one_min_inv,
                     avm_alu_alu_clk,
                     avm_alu_alu_ia,
                     avm_alu_alu_ib,
                     avm_alu_alu_ic,
                     avm_alu_alu_op_add,
                     avm_alu_alu_op_sub,
                     avm_alu_alu_op_mul,
                     avm_alu_alu_op_div,
                     avm_alu_alu_op_not,
                     avm_alu_alu_op_eq,
                     avm_alu_alu_sel,
                     avm_alu_alu_in_tag,
                     avm_alu_alu_ff_tag,
                     avm_alu_alu_u8_tag,
                     avm_alu_alu_u16_tag,
                     avm_alu_alu_u32_tag,
                     avm_alu_alu_u64_tag,
                     avm_alu_alu_u128_tag,
                     avm_alu_alu_u8_r0,
                     avm_alu_alu_u8_r1,
                     avm_alu_alu_u16_r0,
                     avm_alu_alu_u16_r1,
                     avm_alu_alu_u16_r2,
                     avm_alu_alu_u16_r3,
                     avm_alu_alu_u16_r4,
                     avm_alu_alu_u16_r5,
                     avm_alu_alu_u16_r6,
                     avm_alu_alu_u16_r7,
                     avm_alu_alu_u64_r0,
                     avm_alu_alu_cf,
                     avm_alu_alu_op_eq_diff_inv,
                     avm_byte_lookup_table_op_id,
                     avm_byte_lookup_table_input_a,
                     avm_byte_lookup_table_input_b,
                     avm_byte_lookup_table_output,
                     avm_byte_lookup_bin_sel,
                     avm_byte_lookup_table_in_tags,
                     avm_byte_lookup_table_byte_lengths,
                     avm_binary_bin_clk,
                     avm_binary_bin_sel,
                     avm_binary_acc_ia,
                     avm_binary_acc_ib,
                     avm_binary_acc_ic,
                     avm_binary_in_tag,
                     avm_binary_op_id,
                     avm_binary_bin_ia_bytes,
                     avm_binary_bin_ib_bytes,
                     avm_binary_bin_ic_bytes,
                     avm_binary_start,
                     avm_binary_mem_tag_ctr,
                     avm_binary_mem_tag_ctr_inv,
                     avm_main_sel_rng_8,
                     avm_main_sel_rng_16,
                     avm_main_pc,
                     avm_main_internal_return_ptr,
                     avm_main_sel_internal_call,
                     avm_main_sel_internal_return,
                     avm_main_sel_jump,
                     avm_main_sel_halt,
                     avm_main_sel_mov,
                     avm_main_sel_op_add,
                     avm_main_sel_op_sub,
                     avm_main_sel_op_mul,
                     avm_main_sel_op_div,
                     avm_main_sel_op_not,
                     avm_main_sel_op_eq,
                     avm_main_sel_op_and,
                     avm_main_sel_op_or,
                     avm_main_sel_op_xor,
                     avm_main_alu_sel,
                     avm_main_bin_sel,
                     avm_main_r_in_tag,
                     avm_main_w_in_tag,
                     avm_main_op_err,
                     avm_main_tag_err,
                     avm_main_inv,
                     avm_main_ia,
                     avm_main_ib,
                     avm_main_ic,
                     avm_main_mem_op_a,
                     avm_main_mem_op_b,
                     avm_main_mem_op_c,
                     avm_main_rwa,
                     avm_main_rwb,
                     avm_main_rwc,
                     avm_main_ind_a,
                     avm_main_ind_b,
                     avm_main_ind_c,
                     avm_main_ind_op_a,
                     avm_main_ind_op_b,
                     avm_main_ind_op_c,
                     avm_main_mem_idx_a,
                     avm_main_mem_idx_b,
                     avm_main_mem_idx_c,
                     avm_main_last,
                     avm_main_bin_op_id,
                     perm_main_alu,
                     perm_main_bin,
                     perm_main_mem_a,
                     perm_main_mem_b,
                     perm_main_mem_c,
                     perm_main_mem_ind_a,
                     perm_main_mem_ind_b,
                     perm_main_mem_ind_c,
                     lookup_byte_lengths,
                     lookup_byte_operations,
                     incl_main_tag_err,
                     incl_mem_tag_err,
                     lookup_byte_lengths_counts,
                     lookup_byte_operations_counts,
                     incl_main_tag_err_counts,
                     incl_mem_tag_err_counts,
                     avm_mem_m_tag_shift,
                     avm_mem_m_rw_shift,
                     avm_mem_m_addr_shift,
                     avm_mem_m_val_shift,
                     avm_binary_acc_ic_shift,
                     avm_binary_op_id_shift,
                     avm_binary_acc_ib_shift,
                     avm_binary_mem_tag_ctr_shift,
                     avm_binary_acc_ia_shift,
                     avm_main_internal_return_ptr_shift,
                     avm_main_pc_shift,
                     avm_alu_alu_u16_r1_shift,
                     avm_alu_alu_u16_r3_shift,
                     avm_alu_alu_u16_r2_shift,
                     avm_alu_alu_u16_r5_shift,
                     avm_alu_alu_u16_r4_shift,
                     avm_alu_alu_u16_r6_shift,
                     avm_alu_alu_u16_r0_shift,
                     avm_alu_alu_u16_r7_shift };
        };
        RefVector<DataType> get_unshifted()
        {
            return { avm_main_clk,
                     avm_main_first,
                     avm_mem_m_clk,
                     avm_mem_m_sub_clk,
                     avm_mem_m_addr,
                     avm_mem_m_tag,
                     avm_mem_m_val,
                     avm_mem_m_lastAccess,
                     avm_mem_m_last,
                     avm_mem_m_rw,
                     avm_mem_r_in_tag,
                     avm_mem_w_in_tag,
                     avm_mem_m_op_a,
                     avm_mem_m_op_b,
                     avm_mem_m_op_c,
                     avm_mem_m_ind_op_a,
                     avm_mem_m_ind_op_b,
                     avm_mem_m_ind_op_c,
                     avm_mem_m_sel_mov,
                     avm_mem_m_tag_err,
                     avm_mem_m_one_min_inv,
                     avm_alu_alu_clk,
                     avm_alu_alu_ia,
                     avm_alu_alu_ib,
                     avm_alu_alu_ic,
                     avm_alu_alu_op_add,
                     avm_alu_alu_op_sub,
                     avm_alu_alu_op_mul,
                     avm_alu_alu_op_div,
                     avm_alu_alu_op_not,
                     avm_alu_alu_op_eq,
                     avm_alu_alu_sel,
                     avm_alu_alu_in_tag,
                     avm_alu_alu_ff_tag,
                     avm_alu_alu_u8_tag,
                     avm_alu_alu_u16_tag,
                     avm_alu_alu_u32_tag,
                     avm_alu_alu_u64_tag,
                     avm_alu_alu_u128_tag,
                     avm_alu_alu_u8_r0,
                     avm_alu_alu_u8_r1,
                     avm_alu_alu_u16_r0,
                     avm_alu_alu_u16_r1,
                     avm_alu_alu_u16_r2,
                     avm_alu_alu_u16_r3,
                     avm_alu_alu_u16_r4,
                     avm_alu_alu_u16_r5,
                     avm_alu_alu_u16_r6,
                     avm_alu_alu_u16_r7,
                     avm_alu_alu_u64_r0,
                     avm_alu_alu_cf,
                     avm_alu_alu_op_eq_diff_inv,
                     avm_byte_lookup_table_op_id,
                     avm_byte_lookup_table_input_a,
                     avm_byte_lookup_table_input_b,
                     avm_byte_lookup_table_output,
                     avm_byte_lookup_bin_sel,
                     avm_byte_lookup_table_in_tags,
                     avm_byte_lookup_table_byte_lengths,
                     avm_binary_bin_clk,
                     avm_binary_bin_sel,
                     avm_binary_acc_ia,
                     avm_binary_acc_ib,
                     avm_binary_acc_ic,
                     avm_binary_in_tag,
                     avm_binary_op_id,
                     avm_binary_bin_ia_bytes,
                     avm_binary_bin_ib_bytes,
                     avm_binary_bin_ic_bytes,
                     avm_binary_start,
                     avm_binary_mem_tag_ctr,
                     avm_binary_mem_tag_ctr_inv,
                     avm_main_sel_rng_8,
                     avm_main_sel_rng_16,
                     avm_main_pc,
                     avm_main_internal_return_ptr,
                     avm_main_sel_internal_call,
                     avm_main_sel_internal_return,
                     avm_main_sel_jump,
                     avm_main_sel_halt,
                     avm_main_sel_mov,
                     avm_main_sel_op_add,
                     avm_main_sel_op_sub,
                     avm_main_sel_op_mul,
                     avm_main_sel_op_div,
                     avm_main_sel_op_not,
                     avm_main_sel_op_eq,
                     avm_main_sel_op_and,
                     avm_main_sel_op_or,
                     avm_main_sel_op_xor,
                     avm_main_alu_sel,
                     avm_main_bin_sel,
                     avm_main_r_in_tag,
                     avm_main_w_in_tag,
                     avm_main_op_err,
                     avm_main_tag_err,
                     avm_main_inv,
                     avm_main_ia,
                     avm_main_ib,
                     avm_main_ic,
                     avm_main_mem_op_a,
                     avm_main_mem_op_b,
                     avm_main_mem_op_c,
                     avm_main_rwa,
                     avm_main_rwb,
                     avm_main_rwc,
                     avm_main_ind_a,
                     avm_main_ind_b,
                     avm_main_ind_c,
                     avm_main_ind_op_a,
                     avm_main_ind_op_b,
                     avm_main_ind_op_c,
                     avm_main_mem_idx_a,
                     avm_main_mem_idx_b,
                     avm_main_mem_idx_c,
                     avm_main_last,
                     avm_main_bin_op_id,
                     perm_main_alu,
                     perm_main_bin,
                     perm_main_mem_a,
                     perm_main_mem_b,
                     perm_main_mem_c,
                     perm_main_mem_ind_a,
                     perm_main_mem_ind_b,
                     perm_main_mem_ind_c,
                     lookup_byte_lengths,
                     lookup_byte_operations,
                     incl_main_tag_err,
                     incl_mem_tag_err,
                     lookup_byte_lengths_counts,
                     lookup_byte_operations_counts,
                     incl_main_tag_err_counts,
                     incl_mem_tag_err_counts };
        };
        RefVector<DataType> get_to_be_shifted()
        {
            return { avm_mem_m_tag,      avm_mem_m_rw,
                     avm_mem_m_addr,     avm_mem_m_val,
                     avm_binary_acc_ic,  avm_binary_op_id,
                     avm_binary_acc_ib,  avm_binary_mem_tag_ctr,
                     avm_binary_acc_ia,  avm_main_internal_return_ptr,
                     avm_main_pc,        avm_alu_alu_u16_r1,
                     avm_alu_alu_u16_r3, avm_alu_alu_u16_r2,
                     avm_alu_alu_u16_r5, avm_alu_alu_u16_r4,
                     avm_alu_alu_u16_r6, avm_alu_alu_u16_r0,
                     avm_alu_alu_u16_r7 };
        };
        RefVector<DataType> get_shifted()
        {
            return { avm_mem_m_tag_shift,      avm_mem_m_rw_shift,
                     avm_mem_m_addr_shift,     avm_mem_m_val_shift,
                     avm_binary_acc_ic_shift,  avm_binary_op_id_shift,
                     avm_binary_acc_ib_shift,  avm_binary_mem_tag_ctr_shift,
                     avm_binary_acc_ia_shift,  avm_main_internal_return_ptr_shift,
                     avm_main_pc_shift,        avm_alu_alu_u16_r1_shift,
                     avm_alu_alu_u16_r3_shift, avm_alu_alu_u16_r2_shift,
                     avm_alu_alu_u16_r5_shift, avm_alu_alu_u16_r4_shift,
                     avm_alu_alu_u16_r6_shift, avm_alu_alu_u16_r0_shift,
                     avm_alu_alu_u16_r7_shift };
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
            return { avm_mem_m_tag,      avm_mem_m_rw,
                     avm_mem_m_addr,     avm_mem_m_val,
                     avm_binary_acc_ic,  avm_binary_op_id,
                     avm_binary_acc_ib,  avm_binary_mem_tag_ctr,
                     avm_binary_acc_ia,  avm_main_internal_return_ptr,
                     avm_main_pc,        avm_alu_alu_u16_r1,
                     avm_alu_alu_u16_r3, avm_alu_alu_u16_r2,
                     avm_alu_alu_u16_r5, avm_alu_alu_u16_r4,
                     avm_alu_alu_u16_r6, avm_alu_alu_u16_r0,
                     avm_alu_alu_u16_r7 };
        };

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 0> get_table_column_wires() { return {}; };
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
        [[nodiscard]] size_t get_polynomial_size() const { return avm_mem_m_clk.size(); }
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

    class CommitmentLabels : public AllEntities<std::string> {
      private:
        using Base = AllEntities<std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string>()
        {
            Base::avm_main_clk = "AVM_MAIN_CLK";
            Base::avm_main_first = "AVM_MAIN_FIRST";
            Base::avm_mem_m_clk = "AVM_MEM_M_CLK";
            Base::avm_mem_m_sub_clk = "AVM_MEM_M_SUB_CLK";
            Base::avm_mem_m_addr = "AVM_MEM_M_ADDR";
            Base::avm_mem_m_tag = "AVM_MEM_M_TAG";
            Base::avm_mem_m_val = "AVM_MEM_M_VAL";
            Base::avm_mem_m_lastAccess = "AVM_MEM_M_LASTACCESS";
            Base::avm_mem_m_last = "AVM_MEM_M_LAST";
            Base::avm_mem_m_rw = "AVM_MEM_M_RW";
            Base::avm_mem_r_in_tag = "AVM_MEM_R_IN_TAG";
            Base::avm_mem_w_in_tag = "AVM_MEM_W_IN_TAG";
            Base::avm_mem_m_op_a = "AVM_MEM_M_OP_A";
            Base::avm_mem_m_op_b = "AVM_MEM_M_OP_B";
            Base::avm_mem_m_op_c = "AVM_MEM_M_OP_C";
            Base::avm_mem_m_ind_op_a = "AVM_MEM_M_IND_OP_A";
            Base::avm_mem_m_ind_op_b = "AVM_MEM_M_IND_OP_B";
            Base::avm_mem_m_ind_op_c = "AVM_MEM_M_IND_OP_C";
            Base::avm_mem_m_sel_mov = "AVM_MEM_M_SEL_MOV";
            Base::avm_mem_m_tag_err = "AVM_MEM_M_TAG_ERR";
            Base::avm_mem_m_one_min_inv = "AVM_MEM_M_ONE_MIN_INV";
            Base::avm_alu_alu_clk = "AVM_ALU_ALU_CLK";
            Base::avm_alu_alu_ia = "AVM_ALU_ALU_IA";
            Base::avm_alu_alu_ib = "AVM_ALU_ALU_IB";
            Base::avm_alu_alu_ic = "AVM_ALU_ALU_IC";
            Base::avm_alu_alu_op_add = "AVM_ALU_ALU_OP_ADD";
            Base::avm_alu_alu_op_sub = "AVM_ALU_ALU_OP_SUB";
            Base::avm_alu_alu_op_mul = "AVM_ALU_ALU_OP_MUL";
            Base::avm_alu_alu_op_div = "AVM_ALU_ALU_OP_DIV";
            Base::avm_alu_alu_op_not = "AVM_ALU_ALU_OP_NOT";
            Base::avm_alu_alu_op_eq = "AVM_ALU_ALU_OP_EQ";
            Base::avm_alu_alu_sel = "AVM_ALU_ALU_SEL";
            Base::avm_alu_alu_in_tag = "AVM_ALU_ALU_IN_TAG";
            Base::avm_alu_alu_ff_tag = "AVM_ALU_ALU_FF_TAG";
            Base::avm_alu_alu_u8_tag = "AVM_ALU_ALU_U8_TAG";
            Base::avm_alu_alu_u16_tag = "AVM_ALU_ALU_U16_TAG";
            Base::avm_alu_alu_u32_tag = "AVM_ALU_ALU_U32_TAG";
            Base::avm_alu_alu_u64_tag = "AVM_ALU_ALU_U64_TAG";
            Base::avm_alu_alu_u128_tag = "AVM_ALU_ALU_U128_TAG";
            Base::avm_alu_alu_u8_r0 = "AVM_ALU_ALU_U8_R0";
            Base::avm_alu_alu_u8_r1 = "AVM_ALU_ALU_U8_R1";
            Base::avm_alu_alu_u16_r0 = "AVM_ALU_ALU_U16_R0";
            Base::avm_alu_alu_u16_r1 = "AVM_ALU_ALU_U16_R1";
            Base::avm_alu_alu_u16_r2 = "AVM_ALU_ALU_U16_R2";
            Base::avm_alu_alu_u16_r3 = "AVM_ALU_ALU_U16_R3";
            Base::avm_alu_alu_u16_r4 = "AVM_ALU_ALU_U16_R4";
            Base::avm_alu_alu_u16_r5 = "AVM_ALU_ALU_U16_R5";
            Base::avm_alu_alu_u16_r6 = "AVM_ALU_ALU_U16_R6";
            Base::avm_alu_alu_u16_r7 = "AVM_ALU_ALU_U16_R7";
            Base::avm_alu_alu_u64_r0 = "AVM_ALU_ALU_U64_R0";
            Base::avm_alu_alu_cf = "AVM_ALU_ALU_CF";
            Base::avm_alu_alu_op_eq_diff_inv = "AVM_ALU_ALU_OP_EQ_DIFF_INV";
            Base::avm_byte_lookup_table_op_id = "AVM_BYTE_LOOKUP_TABLE_OP_ID";
            Base::avm_byte_lookup_table_input_a = "AVM_BYTE_LOOKUP_TABLE_INPUT_A";
            Base::avm_byte_lookup_table_input_b = "AVM_BYTE_LOOKUP_TABLE_INPUT_B";
            Base::avm_byte_lookup_table_output = "AVM_BYTE_LOOKUP_TABLE_OUTPUT";
            Base::avm_byte_lookup_bin_sel = "AVM_BYTE_LOOKUP_BIN_SEL";
            Base::avm_byte_lookup_table_in_tags = "AVM_BYTE_LOOKUP_TABLE_IN_TAGS";
            Base::avm_byte_lookup_table_byte_lengths = "AVM_BYTE_LOOKUP_TABLE_BYTE_LENGTHS";
            Base::avm_binary_bin_clk = "AVM_BINARY_BIN_CLK";
            Base::avm_binary_bin_sel = "AVM_BINARY_BIN_SEL";
            Base::avm_binary_acc_ia = "AVM_BINARY_ACC_IA";
            Base::avm_binary_acc_ib = "AVM_BINARY_ACC_IB";
            Base::avm_binary_acc_ic = "AVM_BINARY_ACC_IC";
            Base::avm_binary_in_tag = "AVM_BINARY_IN_TAG";
            Base::avm_binary_op_id = "AVM_BINARY_OP_ID";
            Base::avm_binary_bin_ia_bytes = "AVM_BINARY_BIN_IA_BYTES";
            Base::avm_binary_bin_ib_bytes = "AVM_BINARY_BIN_IB_BYTES";
            Base::avm_binary_bin_ic_bytes = "AVM_BINARY_BIN_IC_BYTES";
            Base::avm_binary_start = "AVM_BINARY_START";
            Base::avm_binary_mem_tag_ctr = "AVM_BINARY_MEM_TAG_CTR";
            Base::avm_binary_mem_tag_ctr_inv = "AVM_BINARY_MEM_TAG_CTR_INV";
            Base::avm_main_sel_rng_8 = "AVM_MAIN_SEL_RNG_8";
            Base::avm_main_sel_rng_16 = "AVM_MAIN_SEL_RNG_16";
            Base::avm_main_pc = "AVM_MAIN_PC";
            Base::avm_main_internal_return_ptr = "AVM_MAIN_INTERNAL_RETURN_PTR";
            Base::avm_main_sel_internal_call = "AVM_MAIN_SEL_INTERNAL_CALL";
            Base::avm_main_sel_internal_return = "AVM_MAIN_SEL_INTERNAL_RETURN";
            Base::avm_main_sel_jump = "AVM_MAIN_SEL_JUMP";
            Base::avm_main_sel_halt = "AVM_MAIN_SEL_HALT";
            Base::avm_main_sel_mov = "AVM_MAIN_SEL_MOV";
            Base::avm_main_sel_op_add = "AVM_MAIN_SEL_OP_ADD";
            Base::avm_main_sel_op_sub = "AVM_MAIN_SEL_OP_SUB";
            Base::avm_main_sel_op_mul = "AVM_MAIN_SEL_OP_MUL";
            Base::avm_main_sel_op_div = "AVM_MAIN_SEL_OP_DIV";
            Base::avm_main_sel_op_not = "AVM_MAIN_SEL_OP_NOT";
            Base::avm_main_sel_op_eq = "AVM_MAIN_SEL_OP_EQ";
            Base::avm_main_sel_op_and = "AVM_MAIN_SEL_OP_AND";
            Base::avm_main_sel_op_or = "AVM_MAIN_SEL_OP_OR";
            Base::avm_main_sel_op_xor = "AVM_MAIN_SEL_OP_XOR";
            Base::avm_main_alu_sel = "AVM_MAIN_ALU_SEL";
            Base::avm_main_bin_sel = "AVM_MAIN_BIN_SEL";
            Base::avm_main_r_in_tag = "AVM_MAIN_R_IN_TAG";
            Base::avm_main_w_in_tag = "AVM_MAIN_W_IN_TAG";
            Base::avm_main_op_err = "AVM_MAIN_OP_ERR";
            Base::avm_main_tag_err = "AVM_MAIN_TAG_ERR";
            Base::avm_main_inv = "AVM_MAIN_INV";
            Base::avm_main_ia = "AVM_MAIN_IA";
            Base::avm_main_ib = "AVM_MAIN_IB";
            Base::avm_main_ic = "AVM_MAIN_IC";
            Base::avm_main_mem_op_a = "AVM_MAIN_MEM_OP_A";
            Base::avm_main_mem_op_b = "AVM_MAIN_MEM_OP_B";
            Base::avm_main_mem_op_c = "AVM_MAIN_MEM_OP_C";
            Base::avm_main_rwa = "AVM_MAIN_RWA";
            Base::avm_main_rwb = "AVM_MAIN_RWB";
            Base::avm_main_rwc = "AVM_MAIN_RWC";
            Base::avm_main_ind_a = "AVM_MAIN_IND_A";
            Base::avm_main_ind_b = "AVM_MAIN_IND_B";
            Base::avm_main_ind_c = "AVM_MAIN_IND_C";
            Base::avm_main_ind_op_a = "AVM_MAIN_IND_OP_A";
            Base::avm_main_ind_op_b = "AVM_MAIN_IND_OP_B";
            Base::avm_main_ind_op_c = "AVM_MAIN_IND_OP_C";
            Base::avm_main_mem_idx_a = "AVM_MAIN_MEM_IDX_A";
            Base::avm_main_mem_idx_b = "AVM_MAIN_MEM_IDX_B";
            Base::avm_main_mem_idx_c = "AVM_MAIN_MEM_IDX_C";
            Base::avm_main_last = "AVM_MAIN_LAST";
            Base::avm_main_bin_op_id = "AVM_MAIN_BIN_OP_ID";
            Base::perm_main_alu = "PERM_MAIN_ALU";
            Base::perm_main_bin = "PERM_MAIN_BIN";
            Base::perm_main_mem_a = "PERM_MAIN_MEM_A";
            Base::perm_main_mem_b = "PERM_MAIN_MEM_B";
            Base::perm_main_mem_c = "PERM_MAIN_MEM_C";
            Base::perm_main_mem_ind_a = "PERM_MAIN_MEM_IND_A";
            Base::perm_main_mem_ind_b = "PERM_MAIN_MEM_IND_B";
            Base::perm_main_mem_ind_c = "PERM_MAIN_MEM_IND_C";
            Base::lookup_byte_lengths = "LOOKUP_BYTE_LENGTHS";
            Base::lookup_byte_operations = "LOOKUP_BYTE_OPERATIONS";
            Base::incl_main_tag_err = "INCL_MAIN_TAG_ERR";
            Base::incl_mem_tag_err = "INCL_MEM_TAG_ERR";
            Base::lookup_byte_lengths_counts = "LOOKUP_BYTE_LENGTHS_COUNTS";
            Base::lookup_byte_operations_counts = "LOOKUP_BYTE_OPERATIONS_COUNTS";
            Base::incl_main_tag_err_counts = "INCL_MAIN_TAG_ERR_COUNTS";
            Base::incl_mem_tag_err_counts = "INCL_MEM_TAG_ERR_COUNTS";
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

        Commitment avm_mem_m_clk;
        Commitment avm_mem_m_sub_clk;
        Commitment avm_mem_m_addr;
        Commitment avm_mem_m_tag;
        Commitment avm_mem_m_val;
        Commitment avm_mem_m_lastAccess;
        Commitment avm_mem_m_last;
        Commitment avm_mem_m_rw;
        Commitment avm_mem_r_in_tag;
        Commitment avm_mem_w_in_tag;
        Commitment avm_mem_m_op_a;
        Commitment avm_mem_m_op_b;
        Commitment avm_mem_m_op_c;
        Commitment avm_mem_m_ind_op_a;
        Commitment avm_mem_m_ind_op_b;
        Commitment avm_mem_m_ind_op_c;
        Commitment avm_mem_m_sel_mov;
        Commitment avm_mem_m_tag_err;
        Commitment avm_mem_m_one_min_inv;
        Commitment avm_alu_alu_clk;
        Commitment avm_alu_alu_ia;
        Commitment avm_alu_alu_ib;
        Commitment avm_alu_alu_ic;
        Commitment avm_alu_alu_op_add;
        Commitment avm_alu_alu_op_sub;
        Commitment avm_alu_alu_op_mul;
        Commitment avm_alu_alu_op_div;
        Commitment avm_alu_alu_op_not;
        Commitment avm_alu_alu_op_eq;
        Commitment avm_alu_alu_sel;
        Commitment avm_alu_alu_in_tag;
        Commitment avm_alu_alu_ff_tag;
        Commitment avm_alu_alu_u8_tag;
        Commitment avm_alu_alu_u16_tag;
        Commitment avm_alu_alu_u32_tag;
        Commitment avm_alu_alu_u64_tag;
        Commitment avm_alu_alu_u128_tag;
        Commitment avm_alu_alu_u8_r0;
        Commitment avm_alu_alu_u8_r1;
        Commitment avm_alu_alu_u16_r0;
        Commitment avm_alu_alu_u16_r1;
        Commitment avm_alu_alu_u16_r2;
        Commitment avm_alu_alu_u16_r3;
        Commitment avm_alu_alu_u16_r4;
        Commitment avm_alu_alu_u16_r5;
        Commitment avm_alu_alu_u16_r6;
        Commitment avm_alu_alu_u16_r7;
        Commitment avm_alu_alu_u64_r0;
        Commitment avm_alu_alu_cf;
        Commitment avm_alu_alu_op_eq_diff_inv;
        Commitment avm_byte_lookup_table_op_id;
        Commitment avm_byte_lookup_table_input_a;
        Commitment avm_byte_lookup_table_input_b;
        Commitment avm_byte_lookup_table_output;
        Commitment avm_byte_lookup_bin_sel;
        Commitment avm_byte_lookup_table_in_tags;
        Commitment avm_byte_lookup_table_byte_lengths;
        Commitment avm_binary_bin_clk;
        Commitment avm_binary_bin_sel;
        Commitment avm_binary_acc_ia;
        Commitment avm_binary_acc_ib;
        Commitment avm_binary_acc_ic;
        Commitment avm_binary_in_tag;
        Commitment avm_binary_op_id;
        Commitment avm_binary_bin_ia_bytes;
        Commitment avm_binary_bin_ib_bytes;
        Commitment avm_binary_bin_ic_bytes;
        Commitment avm_binary_start;
        Commitment avm_binary_mem_tag_ctr;
        Commitment avm_binary_mem_tag_ctr_inv;
        Commitment avm_main_sel_rng_8;
        Commitment avm_main_sel_rng_16;
        Commitment avm_main_pc;
        Commitment avm_main_internal_return_ptr;
        Commitment avm_main_sel_internal_call;
        Commitment avm_main_sel_internal_return;
        Commitment avm_main_sel_jump;
        Commitment avm_main_sel_halt;
        Commitment avm_main_sel_mov;
        Commitment avm_main_sel_op_add;
        Commitment avm_main_sel_op_sub;
        Commitment avm_main_sel_op_mul;
        Commitment avm_main_sel_op_div;
        Commitment avm_main_sel_op_not;
        Commitment avm_main_sel_op_eq;
        Commitment avm_main_sel_op_and;
        Commitment avm_main_sel_op_or;
        Commitment avm_main_sel_op_xor;
        Commitment avm_main_alu_sel;
        Commitment avm_main_bin_sel;
        Commitment avm_main_r_in_tag;
        Commitment avm_main_w_in_tag;
        Commitment avm_main_op_err;
        Commitment avm_main_tag_err;
        Commitment avm_main_inv;
        Commitment avm_main_ia;
        Commitment avm_main_ib;
        Commitment avm_main_ic;
        Commitment avm_main_mem_op_a;
        Commitment avm_main_mem_op_b;
        Commitment avm_main_mem_op_c;
        Commitment avm_main_rwa;
        Commitment avm_main_rwb;
        Commitment avm_main_rwc;
        Commitment avm_main_ind_a;
        Commitment avm_main_ind_b;
        Commitment avm_main_ind_c;
        Commitment avm_main_ind_op_a;
        Commitment avm_main_ind_op_b;
        Commitment avm_main_ind_op_c;
        Commitment avm_main_mem_idx_a;
        Commitment avm_main_mem_idx_b;
        Commitment avm_main_mem_idx_c;
        Commitment avm_main_last;
        Commitment avm_main_bin_op_id;
        Commitment perm_main_alu;
        Commitment perm_main_bin;
        Commitment perm_main_mem_a;
        Commitment perm_main_mem_b;
        Commitment perm_main_mem_c;
        Commitment perm_main_mem_ind_a;
        Commitment perm_main_mem_ind_b;
        Commitment perm_main_mem_ind_c;
        Commitment lookup_byte_lengths;
        Commitment lookup_byte_operations;
        Commitment incl_main_tag_err;
        Commitment incl_mem_tag_err;
        Commitment lookup_byte_lengths_counts;
        Commitment lookup_byte_operations_counts;
        Commitment incl_main_tag_err_counts;
        Commitment incl_mem_tag_err_counts;

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

            avm_mem_m_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_sub_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_addr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_val = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_lastAccess = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_last = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_rw = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_r_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_w_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_ind_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_ind_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_ind_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_sel_mov = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_mem_m_one_min_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_add = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_sub = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_mul = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_div = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_not = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_eq = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_ff_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u8_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u32_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u64_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u128_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u8_r0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u8_r1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r2 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r3 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r4 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r5 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r6 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u16_r7 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_u64_r0 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_cf = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_alu_alu_op_eq_diff_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_op_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_input_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_input_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_output = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_bin_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_in_tags = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_byte_lookup_table_byte_lengths =
                deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_bin_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_bin_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_acc_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_acc_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_acc_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_op_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_bin_ia_bytes = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_bin_ib_bytes = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_bin_ic_bytes = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_start = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_mem_tag_ctr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_binary_mem_tag_ctr_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_rng_8 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_rng_16 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_pc = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_internal_return_ptr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_internal_call = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_internal_return = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_jump = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_halt = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_mov = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_add = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_sub = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_mul = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_div = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_not = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_eq = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_and = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_or = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_sel_op_xor = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_alu_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_bin_sel = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_r_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_w_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_op_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwa = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwb = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_rwc = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_ind_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_mem_idx_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_last = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            avm_main_bin_op_id = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_alu = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_bin = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            perm_main_mem_ind_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_lengths = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_operations = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_main_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_mem_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_lengths_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            lookup_byte_operations_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_main_tag_err_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);
            incl_mem_tag_err_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);

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

            serialize_to_buffer<Commitment>(avm_mem_m_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_sub_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_addr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_val, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_lastAccess, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_last, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_rw, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_r_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_w_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_ind_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_ind_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_ind_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_sel_mov, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_mem_m_one_min_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_add, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_sub, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_mul, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_div, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_not, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_eq, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_ff_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u8_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u32_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u64_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u128_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u8_r0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u8_r1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r2, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r3, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r4, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r5, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r6, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u16_r7, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_u64_r0, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_cf, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_alu_alu_op_eq_diff_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_op_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_input_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_input_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_output, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_bin_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_in_tags, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_byte_lookup_table_byte_lengths, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_bin_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_bin_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_acc_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_acc_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_acc_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_op_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_bin_ia_bytes, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_bin_ib_bytes, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_bin_ic_bytes, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_start, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_mem_tag_ctr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_binary_mem_tag_ctr_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_rng_8, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_rng_16, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_pc, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_internal_return_ptr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_internal_call, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_internal_return, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_jump, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_halt, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_mov, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_add, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_sub, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_mul, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_div, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_not, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_eq, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_and, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_or, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_sel_op_xor, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_alu_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_bin_sel, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_r_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_w_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_op_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwa, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwb, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_rwc, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_ind_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_mem_idx_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_last, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avm_main_bin_op_id, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_alu, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_bin, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(perm_main_mem_ind_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_lengths, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_operations, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_main_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_mem_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_lengths_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_byte_operations_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_main_tag_err_counts, Transcript::proof_data);
            serialize_to_buffer<Commitment>(incl_mem_tag_err_counts, Transcript::proof_data);

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
