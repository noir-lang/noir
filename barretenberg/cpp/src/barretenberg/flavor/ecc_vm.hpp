#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/ipa/ipa.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/ecc_vm/ecc_lookup_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_msm_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_point_table_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_set_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_transcript_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_wnaf_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"
#include "relation_definitions_fwd.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members)

namespace proof_system::honk {
namespace flavor {

template <typename CycleGroup_T, typename Curve_T, typename PCS_T> class ECCVMBase {
  public:
    // forward template params into the ECCVMBase namespace
    using CycleGroup = CycleGroup_T;
    using Curve = Curve_T;
    using G1 = typename Curve::Group;
    using PCS = PCS_T;

    using FF = typename G1::subgroup_field;
    using Polynomial = barretenberg::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = typename G1::element;
    using Commitment = typename G1::affine_element;
    using CommitmentHandle = typename G1::affine_element;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

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
    template <typename DataType, typename HandleType>
    class PrecomputedEntities : public PrecomputedEntities_<DataType, HandleType, NUM_PRECOMPUTED_ENTITIES> {
      public:
        DataType lagrange_first;  // column 0
        DataType lagrange_second; // column 1
        DataType lagrange_last;   // column 2

        DEFINE_POINTER_VIEW(NUM_PRECOMPUTED_ENTITIES, &lagrange_first, &lagrange_second, &lagrange_last)

        std::vector<HandleType> get_selectors() override { return { lagrange_first, lagrange_second, lagrange_last }; };
        std::vector<HandleType> get_sigma_polynomials() override { return {}; };
        std::vector<HandleType> get_id_polynomials() override { return {}; };
        std::vector<HandleType> get_table_polynomials() { return {}; };
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType, typename HandleType>
    class WitnessEntities : public WitnessEntities_<DataType, HandleType, NUM_WITNESS_ENTITIES> {
      public:
        DataType transcript_add;               // column 0
        DataType transcript_mul;               // column 1
        DataType transcript_eq;                // column 2
        DataType transcript_collision_check;   // column 3
        DataType transcript_msm_transition;    // column 4
        DataType transcript_pc;                // column 5
        DataType transcript_msm_count;         // column 6
        DataType transcript_x;                 // column 7
        DataType transcript_y;                 // column 8
        DataType transcript_z1;                // column 9
        DataType transcript_z2;                // column 10
        DataType transcript_z1zero;            // column 11
        DataType transcript_z2zero;            // column 12
        DataType transcript_op;                // column 13
        DataType transcript_accumulator_x;     // column 14
        DataType transcript_accumulator_y;     // column 15
        DataType transcript_msm_x;             // column 16
        DataType transcript_msm_y;             // column 17
        DataType precompute_pc;                // column 18
        DataType precompute_point_transition;  // column 19
        DataType precompute_round;             // column 20
        DataType precompute_scalar_sum;        // column 21
        DataType precompute_s1hi;              // column 22
        DataType precompute_s1lo;              // column 23
        DataType precompute_s2hi;              // column 24
        DataType precompute_s2lo;              // column 25
        DataType precompute_s3hi;              // column 26
        DataType precompute_s3lo;              // column 27
        DataType precompute_s4hi;              // column 28
        DataType precompute_s4lo;              // column 29
        DataType precompute_skew;              // column 30
        DataType precompute_dx;                // column 31
        DataType precompute_dy;                // column 32
        DataType precompute_tx;                // column 33
        DataType precompute_ty;                // column 34
        DataType msm_transition;               // column 35
        DataType msm_add;                      // column 36
        DataType msm_double;                   // column 37
        DataType msm_skew;                     // column 38
        DataType msm_accumulator_x;            // column 39
        DataType msm_accumulator_y;            // column 40
        DataType msm_pc;                       // column 41
        DataType msm_size_of_msm;              // column 42
        DataType msm_count;                    // column 43
        DataType msm_round;                    // column 44
        DataType msm_add1;                     // column 45
        DataType msm_add2;                     // column 46
        DataType msm_add3;                     // column 47
        DataType msm_add4;                     // column 48
        DataType msm_x1;                       // column 49
        DataType msm_y1;                       // column 50
        DataType msm_x2;                       // column 51
        DataType msm_y2;                       // column 52
        DataType msm_x3;                       // column 53
        DataType msm_y3;                       // column 54
        DataType msm_x4;                       // column 55
        DataType msm_y4;                       // column 56
        DataType msm_collision_x1;             // column 57
        DataType msm_collision_x2;             // column 58
        DataType msm_collision_x3;             // column 59
        DataType msm_collision_x4;             // column 60
        DataType msm_lambda1;                  // column 61
        DataType msm_lambda2;                  // column 62
        DataType msm_lambda3;                  // column 63
        DataType msm_lambda4;                  // column 64
        DataType msm_slice1;                   // column 65
        DataType msm_slice2;                   // column 66
        DataType msm_slice3;                   // column 67
        DataType msm_slice4;                   // column 68
        DataType transcript_accumulator_empty; // column 69
        DataType transcript_reset_accumulator; // column 70
        DataType precompute_select;            // column 71
        DataType lookup_read_counts_0;         // column 72
        DataType lookup_read_counts_1;         // column 73
        DataType z_perm;                       // column 74
        DataType lookup_inverses;              // column 75

        DEFINE_POINTER_VIEW(NUM_WITNESS_ENTITIES,
                            &transcript_add,
                            &transcript_mul,
                            &transcript_eq,
                            &transcript_collision_check,
                            &transcript_msm_transition,
                            &transcript_pc,
                            &transcript_msm_count,
                            &transcript_x,
                            &transcript_y,
                            &transcript_z1,
                            &transcript_z2,
                            &transcript_z1zero,
                            &transcript_z2zero,
                            &transcript_op,
                            &transcript_accumulator_x,
                            &transcript_accumulator_y,
                            &transcript_msm_x,
                            &transcript_msm_y,
                            &precompute_pc,
                            &precompute_point_transition,
                            &precompute_round,
                            &precompute_scalar_sum,
                            &precompute_s1hi,
                            &precompute_s1lo,
                            &precompute_s2hi,
                            &precompute_s2lo,
                            &precompute_s3hi,
                            &precompute_s3lo,
                            &precompute_s4hi,
                            &precompute_s4lo,
                            &precompute_skew,
                            &precompute_dx,
                            &precompute_dy,
                            &precompute_tx,
                            &precompute_ty,
                            &msm_transition,
                            &msm_add,
                            &msm_double,
                            &msm_skew,
                            &msm_accumulator_x,
                            &msm_accumulator_y,
                            &msm_pc,
                            &msm_size_of_msm,
                            &msm_count,
                            &msm_round,
                            &msm_add1,
                            &msm_add2,
                            &msm_add3,
                            &msm_add4,
                            &msm_x1,
                            &msm_y1,
                            &msm_x2,
                            &msm_y2,
                            &msm_x3,
                            &msm_y3,
                            &msm_x4,
                            &msm_y4,
                            &msm_collision_x1,
                            &msm_collision_x2,
                            &msm_collision_x3,
                            &msm_collision_x4,
                            &msm_lambda1,
                            &msm_lambda2,
                            &msm_lambda3,
                            &msm_lambda4,
                            &msm_slice1,
                            &msm_slice2,
                            &msm_slice3,
                            &msm_slice4,
                            &transcript_accumulator_empty,
                            &transcript_reset_accumulator,
                            &precompute_select,
                            &lookup_read_counts_0,
                            &lookup_read_counts_1,
                            &z_perm,
                            &lookup_inverses)
        std::vector<HandleType> get_wires() override
        {
            return {
                transcript_add,
                transcript_mul,
                transcript_eq,
                transcript_collision_check,
                transcript_msm_transition,
                transcript_pc,
                transcript_msm_count,
                transcript_x,
                transcript_y,
                transcript_z1,
                transcript_z2,
                transcript_z1zero,
                transcript_z2zero,
                transcript_op,
                transcript_accumulator_x,
                transcript_accumulator_y,
                transcript_msm_x,
                transcript_msm_y,
                precompute_pc,
                precompute_point_transition,
                precompute_round,
                precompute_scalar_sum,
                precompute_s1hi,
                precompute_s1lo,
                precompute_s2hi,
                precompute_s2lo,
                precompute_s3hi,
                precompute_s3lo,
                precompute_s4hi,
                precompute_s4lo,
                precompute_skew,
                precompute_dx,
                precompute_dy,
                precompute_tx,
                precompute_ty,
                msm_transition,
                msm_add,
                msm_double,
                msm_skew,
                msm_accumulator_x,
                msm_accumulator_y,
                msm_pc,
                msm_size_of_msm,
                msm_count,
                msm_round,
                msm_add1,
                msm_add2,
                msm_add3,
                msm_add4,
                msm_x1,
                msm_y1,
                msm_x2,
                msm_y2,
                msm_x3,
                msm_y3,
                msm_x4,
                msm_y4,
                msm_collision_x1,
                msm_collision_x2,
                msm_collision_x3,
                msm_collision_x4,
                msm_lambda1,
                msm_lambda2,
                msm_lambda3,
                msm_lambda4,
                msm_slice1,
                msm_slice2,
                msm_slice3,
                msm_slice4,
                transcript_accumulator_empty,
                transcript_reset_accumulator,
                precompute_select,
                lookup_read_counts_0,
                lookup_read_counts_1,
            };
        };
        // The sorted concatenations of table and witness data needed for plookup.
        std::vector<HandleType> get_sorted_polynomials() { return {}; };
    };

    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + "ShiftedEntities". It could be
     * implemented as such, but we have this now.
     */
    template <typename DataType, typename HandleType>
    class AllEntities : public AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES> {
      public:
        DataType lagrange_first;                     // column 0
        DataType lagrange_second;                    // column 1
        DataType lagrange_last;                      // column 2
        DataType transcript_add;                     // column 3
        DataType transcript_mul;                     // column 4
        DataType transcript_eq;                      // column 5
        DataType transcript_collision_check;         // column 6
        DataType transcript_msm_transition;          // column 7
        DataType transcript_pc;                      // column 8
        DataType transcript_msm_count;               // column 9
        DataType transcript_x;                       // column 10
        DataType transcript_y;                       // column 11
        DataType transcript_z1;                      // column 12
        DataType transcript_z2;                      // column 13
        DataType transcript_z1zero;                  // column 14
        DataType transcript_z2zero;                  // column 15
        DataType transcript_op;                      // column 16
        DataType transcript_accumulator_x;           // column 17
        DataType transcript_accumulator_y;           // column 18
        DataType transcript_msm_x;                   // column 19
        DataType transcript_msm_y;                   // column 20
        DataType precompute_pc;                      // column 21
        DataType precompute_point_transition;        // column 22
        DataType precompute_round;                   // column 23
        DataType precompute_scalar_sum;              // column 24
        DataType precompute_s1hi;                    // column 25
        DataType precompute_s1lo;                    // column 26
        DataType precompute_s2hi;                    // column 27
        DataType precompute_s2lo;                    // column 28
        DataType precompute_s3hi;                    // column 29
        DataType precompute_s3lo;                    // column 30
        DataType precompute_s4hi;                    // column 31
        DataType precompute_s4lo;                    // column 32
        DataType precompute_skew;                    // column 33
        DataType precompute_dx;                      // column 34
        DataType precompute_dy;                      // column 35
        DataType precompute_tx;                      // column 36
        DataType precompute_ty;                      // column 37
        DataType msm_transition;                     // column 38
        DataType msm_add;                            // column 39
        DataType msm_double;                         // column 40
        DataType msm_skew;                           // column 41
        DataType msm_accumulator_x;                  // column 42
        DataType msm_accumulator_y;                  // column 43
        DataType msm_pc;                             // column 44
        DataType msm_size_of_msm;                    // column 45
        DataType msm_count;                          // column 46
        DataType msm_round;                          // column 47
        DataType msm_add1;                           // column 48
        DataType msm_add2;                           // column 49
        DataType msm_add3;                           // column 50
        DataType msm_add4;                           // column 51
        DataType msm_x1;                             // column 52
        DataType msm_y1;                             // column 53
        DataType msm_x2;                             // column 54
        DataType msm_y2;                             // column 55
        DataType msm_x3;                             // column 56
        DataType msm_y3;                             // column 57
        DataType msm_x4;                             // column 58
        DataType msm_y4;                             // column 59
        DataType msm_collision_x1;                   // column 60
        DataType msm_collision_x2;                   // column 61
        DataType msm_collision_x3;                   // column 62
        DataType msm_collision_x4;                   // column 63
        DataType msm_lambda1;                        // column 64
        DataType msm_lambda2;                        // column 65
        DataType msm_lambda3;                        // column 66
        DataType msm_lambda4;                        // column 67
        DataType msm_slice1;                         // column 68
        DataType msm_slice2;                         // column 69
        DataType msm_slice3;                         // column 70
        DataType msm_slice4;                         // column 71
        DataType transcript_accumulator_empty;       // column 72
        DataType transcript_reset_accumulator;       // column 73
        DataType precompute_select;                  // column 74
        DataType lookup_read_counts_0;               // column 75
        DataType lookup_read_counts_1;               // column 76
        DataType z_perm;                             // column 77
        DataType lookup_inverses;                    // column 78
        DataType transcript_mul_shift;               // column 79
        DataType transcript_msm_count_shift;         // column 80
        DataType transcript_accumulator_x_shift;     // column 81
        DataType transcript_accumulator_y_shift;     // column 82
        DataType precompute_scalar_sum_shift;        // column 83
        DataType precompute_s1hi_shift;              // column 84
        DataType precompute_dx_shift;                // column 85
        DataType precompute_dy_shift;                // column 86
        DataType precompute_tx_shift;                // column 87
        DataType precompute_ty_shift;                // column 88
        DataType msm_transition_shift;               // column 89
        DataType msm_add_shift;                      // column 90
        DataType msm_double_shift;                   // column 91
        DataType msm_skew_shift;                     // column 92
        DataType msm_accumulator_x_shift;            // column 93
        DataType msm_accumulator_y_shift;            // column 94
        DataType msm_count_shift;                    // column 95
        DataType msm_round_shift;                    // column 96
        DataType msm_add1_shift;                     // column 97
        DataType msm_pc_shift;                       // column 98
        DataType precompute_pc_shift;                // column 99
        DataType transcript_pc_shift;                // column 100
        DataType precompute_round_shift;             // column 101
        DataType transcript_accumulator_empty_shift; // column 102
        DataType precompute_select_shift;            // column 103
        DataType z_perm_shift;                       // column 104

        template <size_t index> [[nodiscard]] const DataType& lookup_read_counts() const
        {
            if constexpr (index == 0) {
                return lookup_read_counts_0;
            } else {
                static_assert(index == 1);
                return lookup_read_counts_1;
            }
        }

        // defines a method pointer_view that returns the following, with const and non-const variants
        DEFINE_POINTER_VIEW(NUM_ALL_ENTITIES,
                            &lagrange_first,
                            &lagrange_second,
                            &lagrange_last,
                            &transcript_add,
                            &transcript_mul,
                            &transcript_eq,
                            &transcript_collision_check,
                            &transcript_msm_transition,
                            &transcript_pc,
                            &transcript_msm_count,
                            &transcript_x,
                            &transcript_y,
                            &transcript_z1,
                            &transcript_z2,
                            &transcript_z1zero,
                            &transcript_z2zero,
                            &transcript_op,
                            &transcript_accumulator_x,
                            &transcript_accumulator_y,
                            &transcript_msm_x,
                            &transcript_msm_y,
                            &precompute_pc,
                            &precompute_point_transition,
                            &precompute_round,
                            &precompute_scalar_sum,
                            &precompute_s1hi,
                            &precompute_s1lo,
                            &precompute_s2hi,
                            &precompute_s2lo,
                            &precompute_s3hi,
                            &precompute_s3lo,
                            &precompute_s4hi,
                            &precompute_s4lo,
                            &precompute_skew,
                            &precompute_dx,
                            &precompute_dy,
                            &precompute_tx,
                            &precompute_ty,
                            &msm_transition,
                            &msm_add,
                            &msm_double,
                            &msm_skew,
                            &msm_accumulator_x,
                            &msm_accumulator_y,
                            &msm_pc,
                            &msm_size_of_msm,
                            &msm_count,
                            &msm_round,
                            &msm_add1,
                            &msm_add2,
                            &msm_add3,
                            &msm_add4,
                            &msm_x1,
                            &msm_y1,
                            &msm_x2,
                            &msm_y2,
                            &msm_x3,
                            &msm_y3,
                            &msm_x4,
                            &msm_y4,
                            &msm_collision_x1,
                            &msm_collision_x2,
                            &msm_collision_x3,
                            &msm_collision_x4,
                            &msm_lambda1,
                            &msm_lambda2,
                            &msm_lambda3,
                            &msm_lambda4,
                            &msm_slice1,
                            &msm_slice2,
                            &msm_slice3,
                            &msm_slice4,
                            &transcript_accumulator_empty,
                            &transcript_reset_accumulator,
                            &precompute_select,
                            &lookup_read_counts_0,
                            &lookup_read_counts_1,
                            &z_perm,
                            &lookup_inverses,
                            &transcript_mul_shift,
                            &transcript_msm_count_shift,
                            &transcript_accumulator_x_shift,
                            &transcript_accumulator_y_shift,
                            &precompute_scalar_sum_shift,
                            &precompute_s1hi_shift,
                            &precompute_dx_shift,
                            &precompute_dy_shift,
                            &precompute_tx_shift,
                            &precompute_ty_shift,
                            &msm_transition_shift,
                            &msm_add_shift,
                            &msm_double_shift,
                            &msm_skew_shift,
                            &msm_accumulator_x_shift,
                            &msm_accumulator_y_shift,
                            &msm_count_shift,
                            &msm_round_shift,
                            &msm_add1_shift,
                            &msm_pc_shift,
                            &precompute_pc_shift,
                            &transcript_pc_shift,
                            &precompute_round_shift,
                            &transcript_accumulator_empty_shift,
                            &precompute_select_shift,
                            &z_perm_shift)
        std::vector<HandleType> get_wires() override
        {
            return {
                transcript_add,
                transcript_mul,
                transcript_eq,
                transcript_collision_check,
                transcript_msm_transition,
                transcript_pc,
                transcript_msm_count,
                transcript_x,
                transcript_y,
                transcript_z1,
                transcript_z2,
                transcript_z1zero,
                transcript_z2zero,
                transcript_op,
                transcript_accumulator_x,
                transcript_accumulator_y,
                transcript_msm_x,
                transcript_msm_y,
                precompute_pc,
                precompute_point_transition,
                precompute_round,
                precompute_scalar_sum,
                precompute_s1hi,
                precompute_s1lo,
                precompute_s2hi,
                precompute_s2lo,
                precompute_s3hi,
                precompute_s3lo,
                precompute_s4hi,
                precompute_s4lo,
                precompute_skew,
                precompute_dx,
                precompute_dy,
                precompute_tx,
                precompute_ty,
                msm_transition,
                msm_add,
                msm_double,
                msm_skew,
                msm_accumulator_x,
                msm_accumulator_y,
                msm_pc,
                msm_size_of_msm,
                msm_count,
                msm_round,
                msm_add1,
                msm_add2,
                msm_add3,
                msm_add4,
                msm_x1,
                msm_y1,
                msm_x2,
                msm_y2,
                msm_x3,
                msm_y3,
                msm_x4,
                msm_y4,
                msm_collision_x1,
                msm_collision_x2,
                msm_collision_x3,
                msm_collision_x4,
                msm_lambda1,
                msm_lambda2,
                msm_lambda3,
                msm_lambda4,
                msm_slice1,
                msm_slice2,
                msm_slice3,
                msm_slice4,
                transcript_accumulator_empty,
                transcript_reset_accumulator,
                precompute_select,
                lookup_read_counts_0,
                lookup_read_counts_1,
            };
        };
        // Gemini-specific getters.
        std::vector<HandleType> get_unshifted() override
        {
            return {
                lagrange_first,
                lagrange_second,
                lagrange_last,
                transcript_add,
                transcript_eq,
                transcript_collision_check,
                transcript_msm_transition,
                transcript_x,
                transcript_y,
                transcript_z1,
                transcript_z2,
                transcript_z1zero,
                transcript_z2zero,
                transcript_op,
                transcript_msm_x,
                transcript_msm_y,
                precompute_point_transition,
                precompute_s1hi,
                precompute_s2hi,
                precompute_s2lo,
                precompute_s3hi,
                precompute_s3lo,
                precompute_s4hi,
                precompute_s4lo,
                precompute_skew,
                msm_size_of_msm,
                msm_add2,
                msm_add3,
                msm_add4,
                msm_x1,
                msm_y1,
                msm_x2,
                msm_y2,
                msm_x3,
                msm_y3,
                msm_x4,
                msm_y4,
                msm_collision_x1,
                msm_collision_x2,
                msm_collision_x3,
                msm_collision_x4,
                msm_lambda1,
                msm_lambda2,
                msm_lambda3,
                msm_lambda4,
                msm_slice1,
                msm_slice2,
                msm_slice3,
                msm_slice4,
                transcript_reset_accumulator,
                lookup_read_counts_0,
                lookup_read_counts_1,
                lookup_inverses,
            };
        };

        std::vector<HandleType> get_to_be_shifted() override
        {
            return {
                transcript_mul,
                transcript_msm_count,
                transcript_accumulator_x,
                transcript_accumulator_y,
                precompute_scalar_sum,
                precompute_s1hi,
                precompute_dx,
                precompute_dy,
                precompute_tx,
                precompute_ty,
                msm_transition,
                msm_add,
                msm_double,
                msm_skew,
                msm_accumulator_x,
                msm_accumulator_y,
                msm_count,
                msm_round,
                msm_add1,
                msm_pc,
                precompute_pc,
                transcript_pc,
                precompute_round,
                transcript_accumulator_empty,
                precompute_select,
                z_perm,
            };
        };
        std::vector<HandleType> get_shifted() override
        {
            return {
                transcript_mul_shift,
                transcript_msm_count_shift,
                transcript_accumulator_x_shift,
                transcript_accumulator_y_shift,
                precompute_scalar_sum_shift,
                precompute_s1hi_shift,
                precompute_dx_shift,
                precompute_dy_shift,
                precompute_tx_shift,
                precompute_ty_shift,
                msm_transition_shift,
                msm_add_shift,
                msm_double_shift,
                msm_skew_shift,
                msm_accumulator_x_shift,
                msm_accumulator_y_shift,
                msm_count_shift,
                msm_round_shift,
                msm_add1_shift,
                msm_pc_shift,
                precompute_pc_shift,
                transcript_pc_shift,
                precompute_round_shift,
                transcript_accumulator_empty_shift,
                precompute_select_shift,
                z_perm_shift,
            };
        };
    };

  public:
    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve inherit
     * from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                          WitnessEntities<Polynomial, PolynomialHandle>> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                 WitnessEntities<Polynomial, PolynomialHandle>>;
        using Base::Base;

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 3> get_table_column_wires() { return {}; };
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     */
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment, CommitmentHandle>>;

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using FoldedPolynomials = AllEntities<std::vector<FF>, PolynomialHandle>;

    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials evaluated
     * at one point.
     */
    class AllValues : public AllEntities<FF, FF> {
      public:
        using Base = AllEntities<FF, FF>;
        using Base::Base;
        AllValues(std::array<FF, NUM_ALL_ENTITIES> _data_in) { this->_data = _data_in; }
    };

    /**
     * @brief An owning container of polynomials.
     * @warning When this was introduced it broke some of our design principles.
     *   - Execution trace builders don't handle "polynomials" because the interpretation of the execution trace columns
     *     as polynomials is a detail of the proving system, and trace builders are (sometimes in practice, always in
     *     principle) reusable for different proving protocols (e.g., Plonk and Honk).
     *   - Polynomial storage is handled by key classes. Polynomials aren't moved, but are accessed elsewhere by
     * std::spans.
     *
     *  We will consider revising this data model: TODO(https://github.com/AztecProtocol/barretenberg/issues/743)
     */
    class AllPolynomials : public AllEntities<Polynomial, PolynomialHandle> {
      public:
        [[nodiscard]] size_t get_polynomial_size() const { return this->lagrange_first.size(); }
        AllValues get_row(const size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.pointer_view(), this->pointer_view())) {
                *result_field = (*polynomial)[row_idx];
            }
            return result;
        }
    };
    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using RowPolynomials = AllEntities<FF, FF>;

    /**
     * @brief A container for storing the partially evaluated multivariates produced by sumcheck.
     */
    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial, PolynomialHandle> {

      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto* poly : this->pointer_view()) {
                *poly = Polynomial(circuit_size / 2);
            }
        }
    };

    /**
     * @brief A container for univariates used during sumcheck.
     */
    template <size_t LENGTH>
    using ProverUnivariates = AllEntities<barretenberg::Univariate<FF, LENGTH>, barretenberg::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A container for the prover polynomials handles; only stores spans.
     */
    class ProverPolynomials : public AllEntities<PolynomialHandle, PolynomialHandle> {
      public:
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        AllValues get_row(const size_t row_idx)
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.pointer_view(), this->pointer_view())) {
                *result_field = (*polynomial)[row_idx];
            }
            return result;
        }
    };

    /**
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly needed. It
     * has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string, std::string> {
      private:
        using Base = AllEntities<std::string, std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string, std::string>()
        {
            Base::transcript_add = "TRANSCRIPT_ADD";
            Base::transcript_mul = "TRANSCRIPT_MUL";
            Base::transcript_eq = "TRANSCRIPT_EQ";
            Base::transcript_collision_check = "TRANSCRIPT_COLLISION_CHECK";
            Base::transcript_msm_transition = "TRANSCRIPT_MSM_TRANSITION";
            Base::transcript_pc = "TRANSCRIPT_PC";
            Base::transcript_msm_count = "TRANSCRIPT_MSM_COUNT";
            Base::transcript_x = "TRANSCRIPT_X";
            Base::transcript_y = "TRANSCRIPT_Y";
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

    class VerifierCommitments : public AllEntities<Commitment, CommitmentHandle> {
      private:
        using Base = AllEntities<Commitment, CommitmentHandle>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key,
                            [[maybe_unused]] const BaseTranscript<FF>& transcript)
        {
            static_cast<void>(transcript);
            Base::lagrange_first = verification_key->lagrange_first;
            Base::lagrange_second = verification_key->lagrange_second;
            Base::lagrange_last = verification_key->lagrange_last;
        }
    };

    /**
     * @brief Derived class that defines proof structure for ECCVM proofs, as well as supporting functions.
     *
     */
    class Transcript : public BaseTranscript<FF> {
      public:
        uint32_t circuit_size;
        Commitment transcript_add_comm;
        Commitment transcript_mul_comm;
        Commitment transcript_eq_comm;
        Commitment transcript_collision_check_comm;
        Commitment transcript_msm_transition_comm;
        Commitment transcript_pc_comm;
        Commitment transcript_msm_count_comm;
        Commitment transcript_x_comm;
        Commitment transcript_y_comm;
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
        std::vector<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
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
            : BaseTranscript<FF>(proof)
        {}

        void deserialize_full_transcript() override
        {
            // take current proof and put them into the struct
            size_t num_bytes_read = 0;
            circuit_size = BaseTranscript<FF>::template deserialize_from_buffer<uint32_t>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            size_t log_n = numeric::get_msb(circuit_size);
            transcript_add_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_mul_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_eq_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_collision_check_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_msm_transition_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_pc_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_msm_count_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_x_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_y_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_z1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_z2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_z1zero_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_z2zero_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_op_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_accumulator_x_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_accumulator_y_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_msm_x_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_msm_y_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_pc_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_point_transition_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_round_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_scalar_sum_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s1hi_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s1lo_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s2hi_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s2lo_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s3hi_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s3lo_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s4hi_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_s4lo_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_skew_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_dx_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_dy_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_tx_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_ty_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_transition_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_add_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_double_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_skew_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_accumulator_x_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_accumulator_y_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_pc_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_size_of_msm_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_count_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_round_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_add1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_add2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_add3_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_add4_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_x1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_y1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_x2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_y2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_x3_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_y3_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_x4_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_y4_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_collision_x1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_collision_x2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_collision_x3_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_collision_x4_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_lambda1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_lambda2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_lambda3_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_lambda4_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_slice1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_slice2_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_slice3_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            msm_slice4_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_accumulator_empty_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            transcript_reset_accumulator_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            precompute_select_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            lookup_read_counts_0_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            lookup_read_counts_1_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            lookup_inverses_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            z_perm_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(BaseTranscript<FF>::template deserialize_from_buffer<
                                                  barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                    BaseTranscript<FF>::proof_data, num_bytes_read));
            }
            sumcheck_evaluations =
                BaseTranscript<FF>::template deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(
                    BaseTranscript<FF>::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n - 1; ++i) {
                gemini_univariate_comms.emplace_back(BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                    BaseTranscript<FF>::proof_data, num_bytes_read));
            }
            for (size_t i = 0; i < log_n; ++i) {
                gemini_a_evals.emplace_back(BaseTranscript<FF>::template deserialize_from_buffer<FF>(
                    BaseTranscript<FF>::proof_data, num_bytes_read));
            }
            shplonk_q_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                BaseTranscript<FF>::proof_data, num_bytes_read);
            if (std::is_same<PCS, pcs::kzg::KZG<curve::BN254>>::value) {
                kzg_w_comm = BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                    BaseTranscript<FF>::proof_data, num_bytes_read);
            } else if (std::is_same<PCS, pcs::ipa::IPA<curve::Grumpkin>>::value) {
                ipa_poly_degree = BaseTranscript<FF>::template deserialize_from_buffer<uint64_t>(
                    BaseTranscript<FF>::proof_data, num_bytes_read);
                auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
                for (size_t i = 0; i < log_poly_degree; ++i) {
                    ipa_l_comms.emplace_back(BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                        BaseTranscript<FF>::proof_data, num_bytes_read));
                    ipa_r_comms.emplace_back(BaseTranscript<FF>::template deserialize_from_buffer<Commitment>(
                        BaseTranscript<FF>::proof_data, num_bytes_read));
                }
                ipa_a_0_eval = BaseTranscript<FF>::template deserialize_from_buffer<FF>(BaseTranscript<FF>::proof_data,
                                                                                        num_bytes_read);
            } else {
                throw_or_abort("Unsupported PCS");
            }
        }

        void serialize_full_transcript() override
        {
            size_t old_proof_length = BaseTranscript<FF>::proof_data.size();
            BaseTranscript<FF>::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            BaseTranscript<FF>::template serialize_to_buffer(circuit_size, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_add_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_mul_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_eq_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_collision_check_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_msm_transition_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_pc_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_msm_count_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_x_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_y_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_z1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_z2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_z1zero_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_z2zero_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_op_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_accumulator_x_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_accumulator_y_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_msm_x_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_msm_y_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_pc_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_point_transition_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_round_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_scalar_sum_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s1hi_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s1lo_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s2hi_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s2lo_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s3hi_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s3lo_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s4hi_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_s4lo_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_skew_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_dx_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_dy_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_tx_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_ty_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_transition_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_add_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_double_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_skew_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_accumulator_x_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_accumulator_y_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_pc_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_size_of_msm_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_count_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_round_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_add1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_add2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_add3_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_add4_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_x1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_y1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_x2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_y2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_x3_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_y3_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_x4_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_y4_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_collision_x1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_collision_x2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_collision_x3_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_collision_x4_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_lambda1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_lambda2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_lambda3_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_lambda4_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_slice1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_slice2_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_slice3_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(msm_slice4_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_accumulator_empty_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(transcript_reset_accumulator_comm,
                                                             BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(precompute_select_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(lookup_read_counts_0_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(lookup_read_counts_1_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(lookup_inverses_comm, BaseTranscript<FF>::proof_data);
            BaseTranscript<FF>::template serialize_to_buffer(z_perm_comm, BaseTranscript<FF>::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                BaseTranscript<FF>::template serialize_to_buffer(sumcheck_univariates[i],
                                                                 BaseTranscript<FF>::proof_data);
            }
            BaseTranscript<FF>::template serialize_to_buffer(sumcheck_evaluations, BaseTranscript<FF>::proof_data);
            for (size_t i = 0; i < log_n - 1; ++i) {
                BaseTranscript<FF>::template serialize_to_buffer(gemini_univariate_comms[i],
                                                                 BaseTranscript<FF>::proof_data);
            }
            for (size_t i = 0; i < log_n; ++i) {
                BaseTranscript<FF>::template serialize_to_buffer(gemini_a_evals[i], BaseTranscript<FF>::proof_data);
            }
            BaseTranscript<FF>::template serialize_to_buffer(shplonk_q_comm, BaseTranscript<FF>::proof_data);
            if (std::is_same<PCS, pcs::kzg::KZG<curve::BN254>>::value) {
                BaseTranscript<FF>::template serialize_to_buffer(kzg_w_comm, BaseTranscript<FF>::proof_data);
            } else if (std::is_same<PCS, pcs::ipa::IPA<curve::Grumpkin>>::value) {
                BaseTranscript<FF>::template serialize_to_buffer(ipa_poly_degree, BaseTranscript<FF>::proof_data);
                auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
                for (size_t i = 0; i < log_poly_degree; ++i) {
                    BaseTranscript<FF>::template serialize_to_buffer(ipa_l_comms[i], BaseTranscript<FF>::proof_data);
                    BaseTranscript<FF>::template serialize_to_buffer(ipa_r_comms[i], BaseTranscript<FF>::proof_data);
                }

                BaseTranscript<FF>::template serialize_to_buffer(ipa_a_0_eval, BaseTranscript<FF>::proof_data);
            }
            ASSERT(BaseTranscript<FF>::proof_data.size() == old_proof_length);
        }
    };
};

class ECCVM : public ECCVMBase<grumpkin::g1, curve::BN254, pcs::kzg::KZG<curve::BN254>> {};
class ECCVMGrumpkin : public ECCVMBase<barretenberg::g1, curve::Grumpkin, pcs::ipa::IPA<curve::Grumpkin>> {};

// NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace flavor
namespace sumcheck {

extern template class ECCVMTranscriptRelationImpl<barretenberg::fr>;
extern template class ECCVMWnafRelationImpl<barretenberg::fr>;
extern template class ECCVMPointTableRelationImpl<barretenberg::fr>;
extern template class ECCVMMSMRelationImpl<barretenberg::fr>;
extern template class ECCVMSetRelationImpl<barretenberg::fr>;
extern template class ECCVMLookupRelationImpl<barretenberg::fr>;

DECLARE_SUMCHECK_RELATION_CLASS(ECCVMTranscriptRelationImpl, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMWnafRelationImpl, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMPointTableRelationImpl, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMMSMRelationImpl, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMSetRelationImpl, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationImpl, flavor::ECCVM);

DECLARE_SUMCHECK_RELATION_CLASS(ECCVMTranscriptRelationImpl, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMWnafRelationImpl, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMPointTableRelationImpl, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMMSMRelationImpl, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMSetRelationImpl, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationImpl, flavor::ECCVMGrumpkin);

DECLARE_SUMCHECK_PERMUTATION_CLASS(ECCVMSetRelationImpl, flavor::ECCVM);
DECLARE_SUMCHECK_PERMUTATION_CLASS(ECCVMSetRelationImpl, flavor::ECCVMGrumpkin);
} // namespace sumcheck
} // namespace proof_system::honk
