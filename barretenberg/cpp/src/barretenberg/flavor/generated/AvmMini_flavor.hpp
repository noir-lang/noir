

#pragma once
#include "../relation_definitions.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/generated/AvmMini/avm_mini.hpp"
#include "barretenberg/relations/generated/AvmMini/mem_trace.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::honk::flavor {

class AvmMiniFlavor {
  public:
    using Curve = curve::BN254;
    using G1 = Curve::Group;
    using PCS = pcs::kzg::KZG<Curve>;

    using FF = G1::subgroup_field;
    using Polynomial = bb::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = G1::element;
    using Commitment = G1::affine_element;
    using CommitmentHandle = G1::affine_element;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;
    using RelationSeparator = FF;

    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 2;
    static constexpr size_t NUM_WITNESS_ENTITIES = 38;
    static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
    // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for
    // the unshifted and one for the shifted
    static constexpr size_t NUM_ALL_ENTITIES = 46;

    using Relations = std::tuple<AvmMini_vm::mem_trace<FF>, AvmMini_vm::avm_mini<FF>>;

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

        DEFINE_FLAVOR_MEMBERS(DataType, avmMini_clk, avmMini_first)

        RefVector<DataType> get_selectors() { return { avmMini_clk, avmMini_first }; };
        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
        RefVector<DataType> get_table_polynomials() { return {}; };
    };

    template <typename DataType> class WitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              memTrace_m_clk,
                              memTrace_m_sub_clk,
                              memTrace_m_addr,
                              memTrace_m_tag,
                              memTrace_m_val,
                              memTrace_m_lastAccess,
                              memTrace_m_last,
                              memTrace_m_rw,
                              memTrace_m_in_tag,
                              memTrace_m_tag_err,
                              memTrace_m_one_min_inv,
                              avmMini_pc,
                              avmMini_internal_return_ptr,
                              avmMini_sel_internal_call,
                              avmMini_sel_internal_return,
                              avmMini_sel_jump,
                              avmMini_sel_halt,
                              avmMini_sel_op_add,
                              avmMini_sel_op_sub,
                              avmMini_sel_op_mul,
                              avmMini_sel_op_div,
                              avmMini_in_tag,
                              avmMini_op_err,
                              avmMini_tag_err,
                              avmMini_inv,
                              avmMini_ia,
                              avmMini_ib,
                              avmMini_ic,
                              avmMini_mem_op_a,
                              avmMini_mem_op_b,
                              avmMini_mem_op_c,
                              avmMini_rwa,
                              avmMini_rwb,
                              avmMini_rwc,
                              avmMini_mem_idx_a,
                              avmMini_mem_idx_b,
                              avmMini_mem_idx_c,
                              avmMini_last)

        RefVector<DataType> get_wires()
        {
            return { memTrace_m_clk,
                     memTrace_m_sub_clk,
                     memTrace_m_addr,
                     memTrace_m_tag,
                     memTrace_m_val,
                     memTrace_m_lastAccess,
                     memTrace_m_last,
                     memTrace_m_rw,
                     memTrace_m_in_tag,
                     memTrace_m_tag_err,
                     memTrace_m_one_min_inv,
                     avmMini_pc,
                     avmMini_internal_return_ptr,
                     avmMini_sel_internal_call,
                     avmMini_sel_internal_return,
                     avmMini_sel_jump,
                     avmMini_sel_halt,
                     avmMini_sel_op_add,
                     avmMini_sel_op_sub,
                     avmMini_sel_op_mul,
                     avmMini_sel_op_div,
                     avmMini_in_tag,
                     avmMini_op_err,
                     avmMini_tag_err,
                     avmMini_inv,
                     avmMini_ia,
                     avmMini_ib,
                     avmMini_ic,
                     avmMini_mem_op_a,
                     avmMini_mem_op_b,
                     avmMini_mem_op_c,
                     avmMini_rwa,
                     avmMini_rwb,
                     avmMini_rwc,
                     avmMini_mem_idx_a,
                     avmMini_mem_idx_b,
                     avmMini_mem_idx_c,
                     avmMini_last };
        };
        RefVector<DataType> get_sorted_polynomials() { return {}; };
    };

    template <typename DataType> class AllEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              avmMini_clk,
                              avmMini_first,
                              memTrace_m_clk,
                              memTrace_m_sub_clk,
                              memTrace_m_addr,
                              memTrace_m_tag,
                              memTrace_m_val,
                              memTrace_m_lastAccess,
                              memTrace_m_last,
                              memTrace_m_rw,
                              memTrace_m_in_tag,
                              memTrace_m_tag_err,
                              memTrace_m_one_min_inv,
                              avmMini_pc,
                              avmMini_internal_return_ptr,
                              avmMini_sel_internal_call,
                              avmMini_sel_internal_return,
                              avmMini_sel_jump,
                              avmMini_sel_halt,
                              avmMini_sel_op_add,
                              avmMini_sel_op_sub,
                              avmMini_sel_op_mul,
                              avmMini_sel_op_div,
                              avmMini_in_tag,
                              avmMini_op_err,
                              avmMini_tag_err,
                              avmMini_inv,
                              avmMini_ia,
                              avmMini_ib,
                              avmMini_ic,
                              avmMini_mem_op_a,
                              avmMini_mem_op_b,
                              avmMini_mem_op_c,
                              avmMini_rwa,
                              avmMini_rwb,
                              avmMini_rwc,
                              avmMini_mem_idx_a,
                              avmMini_mem_idx_b,
                              avmMini_mem_idx_c,
                              avmMini_last,
                              memTrace_m_rw_shift,
                              memTrace_m_tag_shift,
                              memTrace_m_addr_shift,
                              memTrace_m_val_shift,
                              avmMini_internal_return_ptr_shift,
                              avmMini_pc_shift)

        RefVector<DataType> get_wires()
        {
            return { avmMini_clk,
                     avmMini_first,
                     memTrace_m_clk,
                     memTrace_m_sub_clk,
                     memTrace_m_addr,
                     memTrace_m_tag,
                     memTrace_m_val,
                     memTrace_m_lastAccess,
                     memTrace_m_last,
                     memTrace_m_rw,
                     memTrace_m_in_tag,
                     memTrace_m_tag_err,
                     memTrace_m_one_min_inv,
                     avmMini_pc,
                     avmMini_internal_return_ptr,
                     avmMini_sel_internal_call,
                     avmMini_sel_internal_return,
                     avmMini_sel_jump,
                     avmMini_sel_halt,
                     avmMini_sel_op_add,
                     avmMini_sel_op_sub,
                     avmMini_sel_op_mul,
                     avmMini_sel_op_div,
                     avmMini_in_tag,
                     avmMini_op_err,
                     avmMini_tag_err,
                     avmMini_inv,
                     avmMini_ia,
                     avmMini_ib,
                     avmMini_ic,
                     avmMini_mem_op_a,
                     avmMini_mem_op_b,
                     avmMini_mem_op_c,
                     avmMini_rwa,
                     avmMini_rwb,
                     avmMini_rwc,
                     avmMini_mem_idx_a,
                     avmMini_mem_idx_b,
                     avmMini_mem_idx_c,
                     avmMini_last,
                     memTrace_m_rw_shift,
                     memTrace_m_tag_shift,
                     memTrace_m_addr_shift,
                     memTrace_m_val_shift,
                     avmMini_internal_return_ptr_shift,
                     avmMini_pc_shift };
        };
        RefVector<DataType> get_unshifted()
        {
            return { avmMini_clk,
                     avmMini_first,
                     memTrace_m_clk,
                     memTrace_m_sub_clk,
                     memTrace_m_addr,
                     memTrace_m_tag,
                     memTrace_m_val,
                     memTrace_m_lastAccess,
                     memTrace_m_last,
                     memTrace_m_rw,
                     memTrace_m_in_tag,
                     memTrace_m_tag_err,
                     memTrace_m_one_min_inv,
                     avmMini_pc,
                     avmMini_internal_return_ptr,
                     avmMini_sel_internal_call,
                     avmMini_sel_internal_return,
                     avmMini_sel_jump,
                     avmMini_sel_halt,
                     avmMini_sel_op_add,
                     avmMini_sel_op_sub,
                     avmMini_sel_op_mul,
                     avmMini_sel_op_div,
                     avmMini_in_tag,
                     avmMini_op_err,
                     avmMini_tag_err,
                     avmMini_inv,
                     avmMini_ia,
                     avmMini_ib,
                     avmMini_ic,
                     avmMini_mem_op_a,
                     avmMini_mem_op_b,
                     avmMini_mem_op_c,
                     avmMini_rwa,
                     avmMini_rwb,
                     avmMini_rwc,
                     avmMini_mem_idx_a,
                     avmMini_mem_idx_b,
                     avmMini_mem_idx_c,
                     avmMini_last };
        };
        RefVector<DataType> get_to_be_shifted()
        {
            return { memTrace_m_rw, memTrace_m_tag, memTrace_m_addr, memTrace_m_val, avmMini_internal_return_ptr,
                     avmMini_pc };
        };
        RefVector<DataType> get_shifted()
        {
            return { memTrace_m_rw_shift,
                     memTrace_m_tag_shift,
                     memTrace_m_addr_shift,
                     memTrace_m_val_shift,
                     avmMini_internal_return_ptr_shift,
                     avmMini_pc_shift };
        };
    };

  public:
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>;
        using Base::Base;

        RefVector<DataType> get_to_be_shifted()
        {
            return { memTrace_m_rw, memTrace_m_tag, memTrace_m_addr, memTrace_m_val, avmMini_internal_return_ptr,
                     avmMini_pc };
        };

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 0> get_table_column_wires() { return {}; };
    };

    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

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
        // Define all operations as default, except move construction/assignment
        ProverPolynomials() = default;
        ProverPolynomials& operator=(const ProverPolynomials&) = delete;
        ProverPolynomials(const ProverPolynomials& o) = delete;
        ProverPolynomials(ProverPolynomials&& o) noexcept = default;
        ProverPolynomials& operator=(ProverPolynomials&& o) noexcept = default;
        ~ProverPolynomials() = default;
        [[nodiscard]] size_t get_polynomial_size() const { return memTrace_m_clk.size(); }
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
            Base::avmMini_clk = "AVMMINI_CLK";
            Base::avmMini_first = "AVMMINI_FIRST";
            Base::memTrace_m_clk = "MEMTRACE_M_CLK";
            Base::memTrace_m_sub_clk = "MEMTRACE_M_SUB_CLK";
            Base::memTrace_m_addr = "MEMTRACE_M_ADDR";
            Base::memTrace_m_tag = "MEMTRACE_M_TAG";
            Base::memTrace_m_val = "MEMTRACE_M_VAL";
            Base::memTrace_m_lastAccess = "MEMTRACE_M_LASTACCESS";
            Base::memTrace_m_last = "MEMTRACE_M_LAST";
            Base::memTrace_m_rw = "MEMTRACE_M_RW";
            Base::memTrace_m_in_tag = "MEMTRACE_M_IN_TAG";
            Base::memTrace_m_tag_err = "MEMTRACE_M_TAG_ERR";
            Base::memTrace_m_one_min_inv = "MEMTRACE_M_ONE_MIN_INV";
            Base::avmMini_pc = "AVMMINI_PC";
            Base::avmMini_internal_return_ptr = "AVMMINI_INTERNAL_RETURN_PTR";
            Base::avmMini_sel_internal_call = "AVMMINI_SEL_INTERNAL_CALL";
            Base::avmMini_sel_internal_return = "AVMMINI_SEL_INTERNAL_RETURN";
            Base::avmMini_sel_jump = "AVMMINI_SEL_JUMP";
            Base::avmMini_sel_halt = "AVMMINI_SEL_HALT";
            Base::avmMini_sel_op_add = "AVMMINI_SEL_OP_ADD";
            Base::avmMini_sel_op_sub = "AVMMINI_SEL_OP_SUB";
            Base::avmMini_sel_op_mul = "AVMMINI_SEL_OP_MUL";
            Base::avmMini_sel_op_div = "AVMMINI_SEL_OP_DIV";
            Base::avmMini_in_tag = "AVMMINI_IN_TAG";
            Base::avmMini_op_err = "AVMMINI_OP_ERR";
            Base::avmMini_tag_err = "AVMMINI_TAG_ERR";
            Base::avmMini_inv = "AVMMINI_INV";
            Base::avmMini_ia = "AVMMINI_IA";
            Base::avmMini_ib = "AVMMINI_IB";
            Base::avmMini_ic = "AVMMINI_IC";
            Base::avmMini_mem_op_a = "AVMMINI_MEM_OP_A";
            Base::avmMini_mem_op_b = "AVMMINI_MEM_OP_B";
            Base::avmMini_mem_op_c = "AVMMINI_MEM_OP_C";
            Base::avmMini_rwa = "AVMMINI_RWA";
            Base::avmMini_rwb = "AVMMINI_RWB";
            Base::avmMini_rwc = "AVMMINI_RWC";
            Base::avmMini_mem_idx_a = "AVMMINI_MEM_IDX_A";
            Base::avmMini_mem_idx_b = "AVMMINI_MEM_IDX_B";
            Base::avmMini_mem_idx_c = "AVMMINI_MEM_IDX_C";
            Base::avmMini_last = "AVMMINI_LAST";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      private:
        using Base = AllEntities<Commitment>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            avmMini_clk = verification_key->avmMini_clk;
            avmMini_first = verification_key->avmMini_first;
        }
    };

    class Transcript : public BaseTranscript {
      public:
        uint32_t circuit_size;

        Commitment memTrace_m_clk;
        Commitment memTrace_m_sub_clk;
        Commitment memTrace_m_addr;
        Commitment memTrace_m_tag;
        Commitment memTrace_m_val;
        Commitment memTrace_m_lastAccess;
        Commitment memTrace_m_last;
        Commitment memTrace_m_rw;
        Commitment memTrace_m_in_tag;
        Commitment memTrace_m_tag_err;
        Commitment memTrace_m_one_min_inv;
        Commitment avmMini_pc;
        Commitment avmMini_internal_return_ptr;
        Commitment avmMini_sel_internal_call;
        Commitment avmMini_sel_internal_return;
        Commitment avmMini_sel_jump;
        Commitment avmMini_sel_halt;
        Commitment avmMini_sel_op_add;
        Commitment avmMini_sel_op_sub;
        Commitment avmMini_sel_op_mul;
        Commitment avmMini_sel_op_div;
        Commitment avmMini_in_tag;
        Commitment avmMini_op_err;
        Commitment avmMini_tag_err;
        Commitment avmMini_inv;
        Commitment avmMini_ia;
        Commitment avmMini_ib;
        Commitment avmMini_ic;
        Commitment avmMini_mem_op_a;
        Commitment avmMini_mem_op_b;
        Commitment avmMini_mem_op_c;
        Commitment avmMini_rwa;
        Commitment avmMini_rwb;
        Commitment avmMini_rwc;
        Commitment avmMini_mem_idx_a;
        Commitment avmMini_mem_idx_b;
        Commitment avmMini_mem_idx_c;
        Commitment avmMini_last;

        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript() = default;

        Transcript(const std::vector<uint8_t>& proof)
            : BaseTranscript(proof)
        {}

        void deserialize_full_transcript()
        {
            size_t num_bytes_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(proof_data, num_bytes_read);
            size_t log_n = numeric::get_msb(circuit_size);

            memTrace_m_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_sub_clk = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_addr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_val = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_lastAccess = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_last = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_rw = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            memTrace_m_one_min_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_pc = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_internal_return_ptr = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_internal_call = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_internal_return = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_jump = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_halt = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_op_add = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_op_sub = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_op_mul = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_sel_op_div = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_in_tag = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_op_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_tag_err = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_inv = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_ia = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_ib = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_ic = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_mem_op_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_mem_op_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_mem_op_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_rwa = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_rwb = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_rwc = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_mem_idx_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_mem_idx_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_mem_idx_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            avmMini_last = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);

            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(
                    deserialize_from_buffer<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(Transcript::proof_data,
                                                                                                 num_bytes_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(Transcript::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_bytes_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
        }

        void serialize_full_transcript()
        {
            size_t old_proof_length = proof_data.size();
            Transcript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            serialize_to_buffer(circuit_size, Transcript::proof_data);

            serialize_to_buffer<Commitment>(memTrace_m_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_sub_clk, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_addr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_val, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_lastAccess, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_last, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_rw, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(memTrace_m_one_min_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_pc, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_internal_return_ptr, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_internal_call, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_internal_return, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_jump, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_halt, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_op_add, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_op_sub, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_op_mul, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_sel_op_div, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_in_tag, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_op_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_tag_err, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_inv, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_ia, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_ib, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_ic, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_mem_op_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_mem_op_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_mem_op_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_rwa, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_rwb, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_rwc, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_mem_idx_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_mem_idx_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_mem_idx_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(avmMini_last, Transcript::proof_data);

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

} // namespace bb::honk::flavor
