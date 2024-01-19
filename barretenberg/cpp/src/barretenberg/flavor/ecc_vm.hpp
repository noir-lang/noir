#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/ipa/ipa.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/common/std_array.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/ecc_vm/ecc_lookup_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_msm_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_point_table_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_set_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_transcript_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_wnaf_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"
#include "relation_definitions.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members)

namespace bb::honk {
namespace flavor {

template <typename CycleGroup_T, typename Curve_T, typename PCS_T> class ECCVMBase {
  public:
    // forward template params into the ECCVMBase namespace
    using CycleGroup = CycleGroup_T;
    using Curve = Curve_T;
    using G1 = typename Curve::Group;
    using PCS = PCS_T;

    using FF = typename G1::subgroup_field;
    using Polynomial = bb::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = typename G1::element;
    using Commitment = typename G1::affine_element;
    using CommitmentHandle = typename G1::affine_element;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;
    using RelationSeparator = FF;

    static constexpr size_t NUM_WIRES = 74;

    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 105;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 3;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 76;

    using GrandProductRelations = std::tuple<sumcheck::ECCVMSetRelation<FF>>;
    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<sumcheck::ECCVMTranscriptRelation<FF>,
                                 sumcheck::ECCVMPointTableRelation<FF>,
                                 sumcheck::ECCVMWnafRelation<FF>,
                                 sumcheck::ECCVMMSMRelation<FF>,
                                 sumcheck::ECCVMSetRelation<FF>,
                                 sumcheck::ECCVMLookupRelation<FF>>;

    using LookupRelation = sumcheck::ECCVMLookupRelation<FF>;
    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // Instantiate the BarycentricData needed to extend each Relation Univariate

    // define the containers for storing the contributions from each relation in Sumcheck
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

  private:
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    template <typename DataType_> class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        using DataType = DataType_;
        DEFINE_FLAVOR_MEMBERS(DataType,
                              lagrange_first,  // column 0
                              lagrange_second, // column 1
                              lagrange_last);  // column 2

        DataType get_selectors() { return get_all(); };
        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
        RefVector<DataType> get_table_polynomials() { return {}; };
    };

    /**
     * @brief Container for all derived witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType> struct DerivedWitnessEntities {
        DEFINE_FLAVOR_MEMBERS(DataType,
                              z_perm,           // column 0
                              lookup_inverses); // column 1
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType> class WireEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              transcript_add,               // column 0
                              transcript_mul,               // column 1
                              transcript_eq,                // column 2
                              transcript_collision_check,   // column 3
                              transcript_msm_transition,    // column 4
                              transcript_pc,                // column 5
                              transcript_msm_count,         // column 6
                              transcript_Px,                // column 7
                              transcript_Py,                // column 8
                              transcript_z1,                // column 9
                              transcript_z2,                // column 10
                              transcript_z1zero,            // column 11
                              transcript_z2zero,            // column 12
                              transcript_op,                // column 13
                              transcript_accumulator_x,     // column 14
                              transcript_accumulator_y,     // column 15
                              transcript_msm_x,             // column 16
                              transcript_msm_y,             // column 17
                              precompute_pc,                // column 18
                              precompute_point_transition,  // column 19
                              precompute_round,             // column 20
                              precompute_scalar_sum,        // column 21
                              precompute_s1hi,              // column 22
                              precompute_s1lo,              // column 23
                              precompute_s2hi,              // column 24
                              precompute_s2lo,              // column 25
                              precompute_s3hi,              // column 26
                              precompute_s3lo,              // column 27
                              precompute_s4hi,              // column 28
                              precompute_s4lo,              // column 29
                              precompute_skew,              // column 30
                              precompute_dx,                // column 31
                              precompute_dy,                // column 32
                              precompute_tx,                // column 33
                              precompute_ty,                // column 34
                              msm_transition,               // column 35
                              msm_add,                      // column 36
                              msm_double,                   // column 37
                              msm_skew,                     // column 38
                              msm_accumulator_x,            // column 39
                              msm_accumulator_y,            // column 40
                              msm_pc,                       // column 41
                              msm_size_of_msm,              // column 42
                              msm_count,                    // column 43
                              msm_round,                    // column 44
                              msm_add1,                     // column 45
                              msm_add2,                     // column 46
                              msm_add3,                     // column 47
                              msm_add4,                     // column 48
                              msm_x1,                       // column 49
                              msm_y1,                       // column 50
                              msm_x2,                       // column 51
                              msm_y2,                       // column 52
                              msm_x3,                       // column 53
                              msm_y3,                       // column 54
                              msm_x4,                       // column 55
                              msm_y4,                       // column 56
                              msm_collision_x1,             // column 57
                              msm_collision_x2,             // column 58
                              msm_collision_x3,             // column 59
                              msm_collision_x4,             // column 60
                              msm_lambda1,                  // column 61
                              msm_lambda2,                  // column 62
                              msm_lambda3,                  // column 63
                              msm_lambda4,                  // column 64
                              msm_slice1,                   // column 65
                              msm_slice2,                   // column 66
                              msm_slice3,                   // column 67
                              msm_slice4,                   // column 68
                              transcript_accumulator_empty, // column 69
                              transcript_reset_accumulator, // column 70
                              precompute_select,            // column 71
                              lookup_read_counts_0,         // column 72
                              lookup_read_counts_1);        // column 73
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType>
    class WitnessEntities : public WireEntities<DataType>, public DerivedWitnessEntities<DataType> {
      public:
        DEFINE_COMPOUND_GET_ALL(WireEntities<DataType>, DerivedWitnessEntities<DataType>)
        RefVector<DataType> get_wires() { return WireEntities<DataType>::get_all(); };
        // The sorted concatenations of table and witness data needed for plookup.
        RefVector<DataType> get_sorted_polynomials() { return {}; };
    };

    /**
     * @brief Represents polynomials shifted by 1 or their evaluations, defined relative to WitnessEntities.
     */
    template <typename DataType> class ShiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              transcript_mul_shift,               // column 0
                              transcript_msm_count_shift,         // column 1
                              transcript_accumulator_x_shift,     // column 2
                              transcript_accumulator_y_shift,     // column 3
                              precompute_scalar_sum_shift,        // column 4
                              precompute_s1hi_shift,              // column 5
                              precompute_dx_shift,                // column 6
                              precompute_dy_shift,                // column 7
                              precompute_tx_shift,                // column 8
                              precompute_ty_shift,                // column 9
                              msm_transition_shift,               // column 10
                              msm_add_shift,                      // column 11
                              msm_double_shift,                   // column 12
                              msm_skew_shift,                     // column 13
                              msm_accumulator_x_shift,            // column 14
                              msm_accumulator_y_shift,            // column 15
                              msm_count_shift,                    // column 16
                              msm_round_shift,                    // column 17
                              msm_add1_shift,                     // column 18
                              msm_pc_shift,                       // column 19
                              precompute_pc_shift,                // column 20
                              transcript_pc_shift,                // column 21
                              precompute_round_shift,             // column 22
                              transcript_accumulator_empty_shift, // column 23
                              precompute_select_shift,            // column 24
                              z_perm_shift);                      // column 25
    };

    template <typename DataType, typename PrecomputedAndWitnessEntitiesSuperset>
    static RefVector<DataType> get_to_be_shifted(PrecomputedAndWitnessEntitiesSuperset& entities)
    {
        // NOTE: must match order of ShiftedEntities above!
        return { entities.transcript_mul,
                 entities.transcript_msm_count,
                 entities.transcript_accumulator_x,
                 entities.transcript_accumulator_y,
                 entities.precompute_scalar_sum,
                 entities.precompute_s1hi,
                 entities.precompute_dx,
                 entities.precompute_dy,
                 entities.precompute_tx,
                 entities.precompute_ty,
                 entities.msm_transition,
                 entities.msm_add,
                 entities.msm_double,
                 entities.msm_skew,
                 entities.msm_accumulator_x,
                 entities.msm_accumulator_y,
                 entities.msm_count,
                 entities.msm_round,
                 entities.msm_add1,
                 entities.msm_pc,
                 entities.precompute_pc,
                 entities.transcript_pc,
                 entities.precompute_round,
                 entities.transcript_accumulator_empty,
                 entities.precompute_select,
                 entities.z_perm };
    }
    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + ShiftedEntities.
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/788): Move to normal composition once comfortable
     * updating usage sites.
     */
    template <typename DataType>
    class AllEntities : public PrecomputedEntities<DataType>,
                        public WitnessEntities<DataType>,
                        public ShiftedEntities<DataType> {
      public:
        // Initialize members
        AllEntities()
            : PrecomputedEntities<DataType>{}
            , WitnessEntities<DataType>{}
            , ShiftedEntities<DataType>{}
        {}
        // get_wires is inherited

        DEFINE_COMPOUND_GET_ALL(PrecomputedEntities<DataType>, WitnessEntities<DataType>, ShiftedEntities<DataType>)
        // Gemini-specific getters.
        RefVector<DataType> get_unshifted()
        {
            return concatenate(PrecomputedEntities<DataType>::get_all(), WitnessEntities<DataType>::get_all());
        };

        RefVector<DataType> get_to_be_shifted() { return ECCVMBase::get_to_be_shifted<DataType>(*this); }
        RefVector<DataType> get_shifted() { return ShiftedEntities<DataType>::get_all(); };
    };

  public:
    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve
     * inherit from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>;
        using Base::Base;

        RefVector<Polynomial> get_to_be_shifted() { return ECCVMBase::get_to_be_shifted<Polynomial>(*this); }
        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 3> get_table_column_wires() { return {}; };
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to
     * resolve that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for
     * portability of our circuits.
     */
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using FoldedPolynomials = AllEntities<std::vector<FF>>;

    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials
     * evaluated at one point.
     */
    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
        AllValues(std::array<FF, NUM_ALL_ENTITIES> _data_in) { this->_data = _data_in; }
    };

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using RowPolynomials = AllEntities<FF>;

    /**
     * @brief A container for storing the partially evaluated multivariates produced by sumcheck.
     */
    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial> {

      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto& poly : this->get_all()) {
                poly = Polynomial(circuit_size / 2);
            }
        }
    };

    /**
     * @brief A container for univariates used during sumcheck.
     */
    template <size_t LENGTH> using ProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A container for the prover polynomials.
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
        [[nodiscard]] size_t get_polynomial_size() const { return this->lagrange_first.size(); }
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        AllValues get_row(const size_t row_idx)
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }
    };

    /**
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly
     * needed. It has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string> {
      private:
        using Base = AllEntities<std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string>()
        {
            Base::transcript_add = "TRANSCRIPT_ADD";
            Base::transcript_mul = "TRANSCRIPT_MUL";
            Base::transcript_eq = "TRANSCRIPT_EQ";
            Base::transcript_collision_check = "TRANSCRIPT_COLLISION_CHECK";
            Base::transcript_msm_transition = "TRANSCRIPT_MSM_TRANSITION";
            Base::transcript_pc = "TRANSCRIPT_PC";
            Base::transcript_msm_count = "TRANSCRIPT_MSM_COUNT";
            Base::transcript_Px = "TRANSCRIPT_PX";
            Base::transcript_Py = "TRANSCRIPT_PY";
            Base::transcript_z1 = "TRANSCRIPT_Z1";
            Base::transcript_z2 = "TRANSCRIPT_Z2";
            Base::transcript_z1zero = "TRANSCRIPT_Z1ZERO";
            Base::transcript_z2zero = "TRANSCRIPT_Z2ZERO";
            Base::transcript_op = "TRANSCRIPT_OP";
            Base::transcript_accumulator_x = "TRANSCRIPT_ACCUMULATOR_X";
            Base::transcript_accumulator_y = "TRANSCRIPT_ACCUMULATOR_Y";
            Base::transcript_msm_x = "TRANSCRIPT_MSM_X";
            Base::transcript_msm_y = "TRANSCRIPT_MSM_Y";
            Base::precompute_pc = "PRECOMPUTE_PC";
            Base::precompute_point_transition = "PRECOMPUTE_POINT_TRANSITION";
            Base::precompute_round = "PRECOMPUTE_ROUND";
            Base::precompute_scalar_sum = "PRECOMPUTE_SCALAR_SUM";
            Base::precompute_s1hi = "PRECOMPUTE_S1HI";
            Base::precompute_s1lo = "PRECOMPUTE_S1LO";
            Base::precompute_s2hi = "PRECOMPUTE_S2HI";
            Base::precompute_s2lo = "PRECOMPUTE_S2LO";
            Base::precompute_s3hi = "PRECOMPUTE_S3HI";
            Base::precompute_s3lo = "PRECOMPUTE_S3LO";
            Base::precompute_s4hi = "PRECOMPUTE_S4HI";
            Base::precompute_s4lo = "PRECOMPUTE_S4LO";
            Base::precompute_skew = "PRECOMPUTE_SKEW";
            Base::precompute_dx = "PRECOMPUTE_DX";
            Base::precompute_dy = "PRECOMPUTE_DY";
            Base::precompute_tx = "PRECOMPUTE_TX";
            Base::precompute_ty = "PRECOMPUTE_TY";
            Base::msm_transition = "MSM_TRANSITION";
            Base::msm_add = "MSM_ADD";
            Base::msm_double = "MSM_DOUBLE";
            Base::msm_skew = "MSM_SKEW";
            Base::msm_accumulator_x = "MSM_ACCUMULATOR_X";
            Base::msm_accumulator_y = "MSM_ACCUMULATOR_Y";
            Base::msm_pc = "MSM_PC";
            Base::msm_size_of_msm = "MSM_SIZE_OF_MSM";
            Base::msm_count = "MSM_COUNT";
            Base::msm_round = "MSM_ROUND";
            Base::msm_add1 = "MSM_ADD1";
            Base::msm_add2 = "MSM_ADD2";
            Base::msm_add3 = "MSM_ADD3";
            Base::msm_add4 = "MSM_ADD4";
            Base::msm_x1 = "MSM_X1";
            Base::msm_y1 = "MSM_Y1";
            Base::msm_x2 = "MSM_X2";
            Base::msm_y2 = "MSM_Y2";
            Base::msm_x3 = "MSM_X3";
            Base::msm_y3 = "MSM_Y3";
            Base::msm_x4 = "MSM_X4";
            Base::msm_y4 = "MSM_Y4";
            Base::msm_collision_x1 = "MSM_COLLISION_X1";
            Base::msm_collision_x2 = "MSM_COLLISION_X2";
            Base::msm_collision_x3 = "MSM_COLLISION_X3";
            Base::msm_collision_x4 = "MSM_COLLISION_X4";
            Base::msm_lambda1 = "MSM_LAMBDA1";
            Base::msm_lambda2 = "MSM_LAMBDA2";
            Base::msm_lambda3 = "MSM_LAMBDA3";
            Base::msm_lambda4 = "MSM_LAMBDA4";
            Base::msm_slice1 = "MSM_SLICE1";
            Base::msm_slice2 = "MSM_SLICE2";
            Base::msm_slice3 = "MSM_SLICE3";
            Base::msm_slice4 = "MSM_SLICE4";
            Base::transcript_accumulator_empty = "TRANSCRIPT_ACCUMULATOR_EMPTY";
            Base::transcript_reset_accumulator = "TRANSCRIPT_RESET_ACCUMULATOR";
            Base::precompute_select = "PRECOMPUTE_SELECT";
            Base::lookup_read_counts_0 = "LOOKUP_READ_COUNTS_0";
            Base::lookup_read_counts_1 = "LOOKUP_READ_COUNTS_1";
            Base::z_perm = "Z_PERM";
            Base::lookup_inverses = "LOOKUP_INVERSES";
            // The ones beginning with "__" are only used for debugging
            Base::lagrange_first = "__LAGRANGE_FIRST";
            Base::lagrange_second = "__LAGRANGE_SECOND";
            Base::lagrange_last = "__LAGRANGE_LAST";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            this->lagrange_first = verification_key->lagrange_first;
            this->lagrange_second = verification_key->lagrange_second;
            this->lagrange_last = verification_key->lagrange_last;
        }
    };

    /**
     * @brief Derived class that defines proof structure for ECCVM proofs, as well as supporting functions.
     *
     */
    class Transcript : public BaseTranscript {
      public:
        uint32_t circuit_size;
        Commitment transcript_add_comm;
        Commitment transcript_mul_comm;
        Commitment transcript_eq_comm;
        Commitment transcript_collision_check_comm;
        Commitment transcript_msm_transition_comm;
        Commitment transcript_pc_comm;
        Commitment transcript_msm_count_comm;
        Commitment transcript_Px_comm;
        Commitment transcript_Py_comm;
        Commitment transcript_z1_comm;
        Commitment transcript_z2_comm;
        Commitment transcript_z1zero_comm;
        Commitment transcript_z2zero_comm;
        Commitment transcript_op_comm;
        Commitment transcript_accumulator_x_comm;
        Commitment transcript_accumulator_y_comm;
        Commitment transcript_msm_x_comm;
        Commitment transcript_msm_y_comm;
        Commitment precompute_pc_comm;
        Commitment precompute_point_transition_comm;
        Commitment precompute_round_comm;
        Commitment precompute_scalar_sum_comm;
        Commitment precompute_s1hi_comm;
        Commitment precompute_s1lo_comm;
        Commitment precompute_s2hi_comm;
        Commitment precompute_s2lo_comm;
        Commitment precompute_s3hi_comm;
        Commitment precompute_s3lo_comm;
        Commitment precompute_s4hi_comm;
        Commitment precompute_s4lo_comm;
        Commitment precompute_skew_comm;
        Commitment precompute_dx_comm;
        Commitment precompute_dy_comm;
        Commitment precompute_tx_comm;
        Commitment precompute_ty_comm;
        Commitment msm_transition_comm;
        Commitment msm_add_comm;
        Commitment msm_double_comm;
        Commitment msm_skew_comm;
        Commitment msm_accumulator_x_comm;
        Commitment msm_accumulator_y_comm;
        Commitment msm_pc_comm;
        Commitment msm_size_of_msm_comm;
        Commitment msm_count_comm;
        Commitment msm_round_comm;
        Commitment msm_add1_comm;
        Commitment msm_add2_comm;
        Commitment msm_add3_comm;
        Commitment msm_add4_comm;
        Commitment msm_x1_comm;
        Commitment msm_y1_comm;
        Commitment msm_x2_comm;
        Commitment msm_y2_comm;
        Commitment msm_x3_comm;
        Commitment msm_y3_comm;
        Commitment msm_x4_comm;
        Commitment msm_y4_comm;
        Commitment msm_collision_x1_comm;
        Commitment msm_collision_x2_comm;
        Commitment msm_collision_x3_comm;
        Commitment msm_collision_x4_comm;
        Commitment msm_lambda1_comm;
        Commitment msm_lambda2_comm;
        Commitment msm_lambda3_comm;
        Commitment msm_lambda4_comm;
        Commitment msm_slice1_comm;
        Commitment msm_slice2_comm;
        Commitment msm_slice3_comm;
        Commitment msm_slice4_comm;
        Commitment transcript_accumulator_empty_comm;
        Commitment transcript_reset_accumulator_comm;
        Commitment precompute_select_comm;
        Commitment lookup_read_counts_0_comm;
        Commitment lookup_read_counts_1_comm;
        Commitment z_perm_comm;
        Commitment lookup_inverses_comm;
        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> gemini_univariate_comms;
        std::vector<FF> gemini_a_evals;
        Commitment shplonk_q_comm;
        Commitment kzg_w_comm;
        // the rest are only for Grumpkin
        uint64_t ipa_poly_degree;
        std::vector<Commitment> ipa_l_comms;
        std::vector<Commitment> ipa_r_comms;
        FF ipa_a_0_eval;

        Transcript() = default;

        Transcript(const std::vector<uint8_t>& proof)
            : BaseTranscript(proof)
        {}

        void deserialize_full_transcript()
        {
            // take current proof and put them into the struct
            size_t num_bytes_read = 0;
            circuit_size =
                BaseTranscript::template deserialize_from_buffer<uint32_t>(BaseTranscript::proof_data, num_bytes_read);
            size_t log_n = numeric::get_msb(circuit_size);
            transcript_add_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_mul_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_eq_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_collision_check_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_msm_transition_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_pc_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_msm_count_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_Px_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_Py_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_z1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_z2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_z1zero_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_z2zero_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_op_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_accumulator_x_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_accumulator_y_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_msm_x_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_msm_y_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_pc_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_point_transition_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_round_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_scalar_sum_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s1hi_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s1lo_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s2hi_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s2lo_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s3hi_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s3lo_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s4hi_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_s4lo_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_skew_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_dx_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_dy_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_tx_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_ty_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_transition_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_add_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                        num_bytes_read);
            msm_double_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                           num_bytes_read);
            msm_skew_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                         num_bytes_read);
            msm_accumulator_x_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_accumulator_y_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_pc_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_size_of_msm_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_count_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                          num_bytes_read);
            msm_round_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                          num_bytes_read);
            msm_add1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                         num_bytes_read);
            msm_add2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                         num_bytes_read);
            msm_add3_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                         num_bytes_read);
            msm_add4_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                         num_bytes_read);
            msm_x1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_y1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_x2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_y2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_x3_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_y3_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_x4_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_y4_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            msm_collision_x1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_collision_x2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_collision_x3_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_collision_x4_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            msm_lambda1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                            num_bytes_read);
            msm_lambda2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                            num_bytes_read);
            msm_lambda3_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                            num_bytes_read);
            msm_lambda4_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                            num_bytes_read);
            msm_slice1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                           num_bytes_read);
            msm_slice2_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                           num_bytes_read);
            msm_slice3_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                           num_bytes_read);
            msm_slice4_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                           num_bytes_read);
            transcript_accumulator_empty_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            transcript_reset_accumulator_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            precompute_select_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            lookup_read_counts_0_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            lookup_read_counts_1_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            lookup_inverses_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(
                BaseTranscript::proof_data, num_bytes_read);
            z_perm_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                       num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(BaseTranscript::template deserialize_from_buffer<
                                                  bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                    BaseTranscript::proof_data, num_bytes_read));
            }
            sumcheck_evaluations = BaseTranscript::template deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(
                BaseTranscript::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n - 1; ++i) {
                gemini_univariate_comms.emplace_back(BaseTranscript::template deserialize_from_buffer<Commitment>(
                    BaseTranscript::proof_data, num_bytes_read));
            }
            for (size_t i = 0; i < log_n; ++i) {
                gemini_a_evals.emplace_back(
                    BaseTranscript::template deserialize_from_buffer<FF>(BaseTranscript::proof_data, num_bytes_read));
            }
            shplonk_q_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                          num_bytes_read);
            if (std::is_same<PCS, pcs::kzg::KZG<curve::BN254>>::value) {
                kzg_w_comm = BaseTranscript::template deserialize_from_buffer<Commitment>(BaseTranscript::proof_data,
                                                                                          num_bytes_read);
            } else if (std::is_same<PCS, pcs::ipa::IPA<curve::Grumpkin>>::value) {
                ipa_poly_degree = BaseTranscript::template deserialize_from_buffer<uint64_t>(BaseTranscript::proof_data,
                                                                                             num_bytes_read);
                auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
                for (size_t i = 0; i < log_poly_degree; ++i) {
                    ipa_l_comms.emplace_back(BaseTranscript::template deserialize_from_buffer<Commitment>(
                        BaseTranscript::proof_data, num_bytes_read));
                    ipa_r_comms.emplace_back(BaseTranscript::template deserialize_from_buffer<Commitment>(
                        BaseTranscript::proof_data, num_bytes_read));
                }
                ipa_a_0_eval =
                    BaseTranscript::template deserialize_from_buffer<FF>(BaseTranscript::proof_data, num_bytes_read);
            } else {
                throw_or_abort("Unsupported PCS");
            }
        }

        void serialize_full_transcript()
        {
            size_t old_proof_length = BaseTranscript::proof_data.size();
            BaseTranscript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            BaseTranscript::template serialize_to_buffer(circuit_size, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_add_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_mul_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_eq_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_collision_check_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_msm_transition_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_pc_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_msm_count_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_Px_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_Py_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_z1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_z2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_z1zero_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_z2zero_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_op_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_accumulator_x_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_accumulator_y_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_msm_x_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_msm_y_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_pc_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_point_transition_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_round_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_scalar_sum_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s1hi_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s1lo_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s2hi_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s2lo_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s3hi_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s3lo_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s4hi_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_s4lo_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_skew_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_dx_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_dy_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_tx_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_ty_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_transition_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_add_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_double_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_skew_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_accumulator_x_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_accumulator_y_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_pc_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_size_of_msm_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_count_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_round_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_add1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_add2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_add3_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_add4_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_x1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_y1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_x2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_y2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_x3_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_y3_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_x4_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_y4_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_collision_x1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_collision_x2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_collision_x3_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_collision_x4_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_lambda1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_lambda2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_lambda3_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_lambda4_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_slice1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_slice2_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_slice3_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(msm_slice4_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_accumulator_empty_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(transcript_reset_accumulator_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(precompute_select_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(lookup_read_counts_0_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(lookup_read_counts_1_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(lookup_inverses_comm, BaseTranscript::proof_data);
            BaseTranscript::template serialize_to_buffer(z_perm_comm, BaseTranscript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                BaseTranscript::template serialize_to_buffer(sumcheck_univariates[i], BaseTranscript::proof_data);
            }
            BaseTranscript::template serialize_to_buffer(sumcheck_evaluations, BaseTranscript::proof_data);
            for (size_t i = 0; i < log_n - 1; ++i) {
                BaseTranscript::template serialize_to_buffer(gemini_univariate_comms[i], BaseTranscript::proof_data);
            }
            for (size_t i = 0; i < log_n; ++i) {
                BaseTranscript::template serialize_to_buffer(gemini_a_evals[i], BaseTranscript::proof_data);
            }
            BaseTranscript::template serialize_to_buffer(shplonk_q_comm, BaseTranscript::proof_data);
            if (std::is_same<PCS, pcs::kzg::KZG<curve::BN254>>::value) {
                BaseTranscript::template serialize_to_buffer(kzg_w_comm, BaseTranscript::proof_data);
            } else if (std::is_same<PCS, pcs::ipa::IPA<curve::Grumpkin>>::value) {
                BaseTranscript::template serialize_to_buffer(ipa_poly_degree, BaseTranscript::proof_data);
                auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
                for (size_t i = 0; i < log_poly_degree; ++i) {
                    BaseTranscript::template serialize_to_buffer(ipa_l_comms[i], BaseTranscript::proof_data);
                    BaseTranscript::template serialize_to_buffer(ipa_r_comms[i], BaseTranscript::proof_data);
                }

                BaseTranscript::template serialize_to_buffer(ipa_a_0_eval, BaseTranscript::proof_data);
            }
            ASSERT(BaseTranscript::proof_data.size() == old_proof_length);
        }
    };
};

class ECCVM : public ECCVMBase<bb::g1, curve::Grumpkin, pcs::ipa::IPA<curve::Grumpkin>> {};

// NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace flavor
} // namespace bb::honk
