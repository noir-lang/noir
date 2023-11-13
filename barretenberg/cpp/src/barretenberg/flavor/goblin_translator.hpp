#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_translator_circuit_builder.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/translator_vm/translator_decomposition_relation.hpp"
#include "barretenberg/relations/translator_vm/translator_extra_relations.hpp"
#include "barretenberg/relations/translator_vm/translator_gen_perm_sort_relation.hpp"
#include "barretenberg/relations/translator_vm/translator_non_native_field_relation.hpp"
#include "barretenberg/relations/translator_vm/translator_permutation_relation.hpp"
#include "relation_definitions_fwd.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

namespace proof_system::honk::flavor {

template <size_t mini_circuit_size> class GoblinTranslator_ {

  public:
    /**
     * @brief Enum containing IDs of all the polynomials used in Goblin Translator
     *
     * @details We use the enum for easier updates of structure sizes and for cases where we need to get a particular
     * polynomial programmatically
     */
    enum ALL_ENTITIES_IDS : size_t {
        /*The first 4 wires contain the standard values from the EccOpQueue*/
        OP,
        X_LO_Y_HI,
        X_HI_Z_1,
        Y_LO_Z_2,
        /*P.xₗₒ split into 2 NUM_LIMB_BITS bit limbs*/
        P_X_LOW_LIMBS,
        /*Low limbs split further into smaller chunks for range constraints*/
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_0,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_1,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_2,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_3,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_4,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        /*P.xₕᵢ split into 2 NUM_LIMB_BITS bit limbs*/
        P_X_HIGH_LIMBS,
        /*High limbs split into chunks for range constraints*/
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL, // The tail also contains some leftover values from  relation wide limb
                                              // range cosntraints
        /*P.yₗₒ split into 2 NUM_LIMB_BITS bit limbs*/
        P_Y_LOW_LIMBS,
        /*Low limbs split into chunks for range constraints*/
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_1,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_2,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_3,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_4,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        /*P.yₕᵢ split into 2 NUM_LIMB_BITS bit limbs*/
        P_Y_HIGH_LIMBS,
        /*High limbs split into chunks for range constraints*/
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL, // The tail also contains some leftover values from  relation wide limb
                                              // range cosntraints
        /*Low limbs of z_1 and z_2*/
        Z_LOW_LIMBS,
        /*Range constraints for low limbs of z_1 and z_2*/
        Z_LOW_LIMBS_RANGE_CONSTRAINT_0,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_1,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_2,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_3,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_4,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        /*High Limbs of z_1 and z_2*/
        Z_HIGH_LIMBS,
        /*Range constraints for high limbs of z_1 and z_2*/
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_0,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL,
        /* Contain NUM_LIMB_BITS-bit limbs of current and previous accumulator (previous at higher indices because of
           the nuances of KZG commitment) */
        ACCUMULATORS_BINARY_LIMBS_0,
        ACCUMULATORS_BINARY_LIMBS_1,
        ACCUMULATORS_BINARY_LIMBS_2,
        ACCUMULATORS_BINARY_LIMBS_3,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for the current accumulator limbs (no need to
                                                  // redo previous accumulator)
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_1,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_2,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_3,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_4,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL, // The tail also contains some leftover values from  relation wide
                                                      // limb range constraints

        /* Quotient limbs*/
        QUOTIENT_LOW_BINARY_LIMBS,
        QUOTIENT_HIGH_BINARY_LIMBS,
        /* Range constraints for quotient */
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_0,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_1,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_2,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_3,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_4,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_0,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL, // The tail also contains some leftover values from  relation wide
                                                   // limb range constraints

        /* Limbs for checking the correctness of  mod 2²⁷² relations*/
        RELATION_WIDE_LIMBS,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_0,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_1,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_2,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_3,
        /*Concatenations of various range constraint wires*/
        CONCATENATED_RANGE_CONSTRAINTS_0,
        CONCATENATED_RANGE_CONSTRAINTS_1,
        CONCATENATED_RANGE_CONSTRAINTS_2,
        CONCATENATED_RANGE_CONSTRAINTS_3,
        /*Values from concatenated range constraints + some additional ones*/
        ORDERED_RANGE_CONSTRAINTS_0,
        ORDERED_RANGE_CONSTRAINTS_1,
        ORDERED_RANGE_CONSTRAINTS_2,
        ORDERED_RANGE_CONSTRAINTS_3,
        ORDERED_RANGE_CONSTRAINTS_4,
        /*Grand Product Polynomial*/
        Z_PERM,
        /*Shifted versions of polynomials*/
        X_LO_Y_HI_SHIFT,
        X_HI_Z_1_SHIFT,
        Y_LO_Z_2_SHIFT,
        P_X_LOW_LIMBS_SHIFT,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        P_X_HIGH_LIMBS_SHIFT,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        P_Y_LOW_LIMBS_SHIFT,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        P_Y_HIGH_LIMBS_SHIFT,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        Z_LOW_LIMBS_SHIFT,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        Z_HIGH_LIMBS_SHIFT,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        ACCUMULATORS_BINARY_LIMBS_0_SHIFT,
        ACCUMULATORS_BINARY_LIMBS_1_SHIFT,
        ACCUMULATORS_BINARY_LIMBS_2_SHIFT,
        ACCUMULATORS_BINARY_LIMBS_3_SHIFT,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        QUOTIENT_LOW_BINARY_LIMBS_SHIFT,
        QUOTIENT_HIGH_BINARY_LIMBS_SHIFT,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_4_SHIFT,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL_SHIFT,
        RELATION_WIDE_LIMBS_SHIFT,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_0_SHIFT,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_1_SHIFT,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_2_SHIFT,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_3_SHIFT,
        ORDERED_RANGE_CONSTRAINTS_0_SHIFT,
        ORDERED_RANGE_CONSTRAINTS_1_SHIFT,
        ORDERED_RANGE_CONSTRAINTS_2_SHIFT,
        ORDERED_RANGE_CONSTRAINTS_3_SHIFT,
        ORDERED_RANGE_CONSTRAINTS_4_SHIFT,

        Z_PERM_SHIFT,
        /*All precomputed polynomials*/
        LAGRANGE_FIRST,
        LAGRANGE_LAST,
        LAGRANGE_ODD_IN_MINICIRCUIT,
        LAGRANGE_EVEN_IN_MINICIRCUIT,
        LAGRANGE_SECOND,
        LAGRANGE_SECOND_TO_LAST_IN_MINICIRCUIT,
        ORDERED_EXTRA_RANGE_CONSTRAINTS_NUMERATOR,
        /*Utility value*/
        TOTAL_COUNT

    };

    using CircuitBuilder = GoblinTranslatorCircuitBuilder;
    using PCS = pcs::kzg::KZG<curve::BN254>;
    using Curve = curve::BN254;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using CommitmentHandle = Curve::AffineElement;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;
    using FF = Curve::ScalarField;
    using BF = Curve::BaseField;
    using Polynomial = barretenberg::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;

    // The size of the circuit which is filled with non-zero values for most polynomials. Most relations (everything
    // except for Permutation and GenPermSort) can be evaluated just on the first chunk
    // It is also the only parameter that can be changed without updating relations or structures in the flavor
    static constexpr size_t MINI_CIRCUIT_SIZE = mini_circuit_size;

    // None of this parameters can be changed

    // How many mini_circuit_size polynomials are concatenated in one concatenated_*
    static constexpr size_t CONCATENATION_INDEX = 16;

    // The number of concatenated_* wires
    static constexpr size_t NUM_CONCATENATED_WIRES = 4;

    // Actual circuit size
    static constexpr size_t FULL_CIRCUIT_SIZE = MINI_CIRCUIT_SIZE * CONCATENATION_INDEX;

    // Number of wires
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;

    // The step in the GenPermSort relation
    static constexpr size_t SORT_STEP = 3;

    // The bitness of the range constraint
    static constexpr size_t MICRO_LIMB_BITS = CircuitBuilder::MICRO_LIMB_BITS;

    // The limbs of the modulus we are emulating in the goblin translator. 4 binary 68-bit limbs and the prime one
    static constexpr auto NEGATIVE_MODULUS_LIMBS = CircuitBuilder::NEGATIVE_MODULUS_LIMBS;

    // Number of bits in a binary limb
    // This is not a configurable value. Relations are sepcifically designed for it to be 68
    static constexpr size_t NUM_LIMB_BITS = CircuitBuilder::NUM_LIMB_BITS;

    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = ALL_ENTITIES_IDS::TOTAL_COUNT;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = ALL_ENTITIES_IDS::TOTAL_COUNT - ALL_ENTITIES_IDS::Z_PERM_SHIFT;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES =
        ALL_ENTITIES_IDS::TOTAL_COUNT - (ALL_ENTITIES_IDS::Z_PERM_SHIFT - ALL_ENTITIES_IDS::Z_PERM);

    using GrandProductRelations = std::tuple<GoblinTranslatorPermutationRelation<FF>>;
    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<GoblinTranslatorPermutationRelation<FF>,
                                 GoblinTranslatorGenPermSortRelation<FF>,
                                 GoblinTranslatorOpcodeConstraintRelation<FF>,
                                 GoblinTranslatorAccumulatorTransferRelation<FF>,
                                 GoblinTranslatorDecompositionRelation<FF>,
                                 GoblinTranslatorNonNativeFieldRelation<FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();
    static constexpr size_t MAX_TOTAL_RELATION_LENGTH = compute_max_total_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t BATCHED_RELATION_TOTAL_LENGTH = MAX_TOTAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size_v<Relations>;

    // define the containers for storing the contributions from each relation in Sumcheck
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

  private:
    template <typename DataType, typename HandleType>
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    class PrecomputedEntities : public PrecomputedEntities_<DataType, HandleType, NUM_PRECOMPUTED_ENTITIES> {
      public:
        DataType lagrange_first; // column 0
        DataType lagrange_last;  // column 1
        // TODO(#758): Check if one of these can be replaced by shifts
        DataType lagrange_odd_in_minicircuit;               // column 2
        DataType lagrange_even_in_minicircuit;              // column 3
        DataType lagrange_second;                           // column 4
        DataType lagrange_second_to_last_in_minicircuit;    // column 5
        DataType ordered_extra_range_constraints_numerator; // column 6
        DEFINE_POINTER_VIEW(NUM_PRECOMPUTED_ENTITIES,
                            &lagrange_first,
                            &lagrange_last,
                            &lagrange_odd_in_minicircuit,
                            &lagrange_even_in_minicircuit,
                            &lagrange_second,
                            &lagrange_second_to_last_in_minicircuit,
                            &ordered_extra_range_constraints_numerator);
        std::vector<HandleType> get_selectors() { return {}; };
        std::vector<HandleType> get_sigma_polynomials() { return {}; };
        std::vector<HandleType> get_id_polynomials() { return {}; };
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType, typename HandleType>
    class WitnessEntities : public WitnessEntities_<DataType, HandleType, NUM_WITNESS_ENTITIES> {
      public:
        DataType op;                                           // column 0
        DataType x_lo_y_hi;                                    // column 1
        DataType x_hi_z_1;                                     // column 2
        DataType y_lo_z_2;                                     // column 3
        DataType p_x_low_limbs;                                // column 4
        DataType p_x_low_limbs_range_constraint_0;             // column 5
        DataType p_x_low_limbs_range_constraint_1;             // column 6
        DataType p_x_low_limbs_range_constraint_2;             // column 7
        DataType p_x_low_limbs_range_constraint_3;             // column 8
        DataType p_x_low_limbs_range_constraint_4;             // column 9
        DataType p_x_low_limbs_range_constraint_tail;          // column 10
        DataType p_x_high_limbs;                               // column 11
        DataType p_x_high_limbs_range_constraint_0;            // column 12
        DataType p_x_high_limbs_range_constraint_1;            // column 13
        DataType p_x_high_limbs_range_constraint_2;            // column 14
        DataType p_x_high_limbs_range_constraint_3;            // column 15
        DataType p_x_high_limbs_range_constraint_4;            // column 16
        DataType p_x_high_limbs_range_constraint_tail;         // column 17
        DataType p_y_low_limbs;                                // column 18
        DataType p_y_low_limbs_range_constraint_0;             // column 19
        DataType p_y_low_limbs_range_constraint_1;             // column 20
        DataType p_y_low_limbs_range_constraint_2;             // column 21
        DataType p_y_low_limbs_range_constraint_3;             // column 22
        DataType p_y_low_limbs_range_constraint_4;             // column 23
        DataType p_y_low_limbs_range_constraint_tail;          // column 24
        DataType p_y_high_limbs;                               // column 25
        DataType p_y_high_limbs_range_constraint_0;            // column 26
        DataType p_y_high_limbs_range_constraint_1;            // column 27
        DataType p_y_high_limbs_range_constraint_2;            // column 28
        DataType p_y_high_limbs_range_constraint_3;            // column 29
        DataType p_y_high_limbs_range_constraint_4;            // column 30
        DataType p_y_high_limbs_range_constraint_tail;         // column 31
        DataType z_low_limbs;                                  // column 32
        DataType z_low_limbs_range_constraint_0;               // column 33
        DataType z_low_limbs_range_constraint_1;               // column 34
        DataType z_low_limbs_range_constraint_2;               // column 35
        DataType z_low_limbs_range_constraint_3;               // column 36
        DataType z_low_limbs_range_constraint_4;               // column 37
        DataType z_low_limbs_range_constraint_tail;            // column 38
        DataType z_high_limbs;                                 // column 39
        DataType z_high_limbs_range_constraint_0;              // column 40
        DataType z_high_limbs_range_constraint_1;              // column 41
        DataType z_high_limbs_range_constraint_2;              // column 42
        DataType z_high_limbs_range_constraint_3;              // column 43
        DataType z_high_limbs_range_constraint_4;              // column 44
        DataType z_high_limbs_range_constraint_tail;           // column 45
        DataType accumulators_binary_limbs_0;                  // column 46
        DataType accumulators_binary_limbs_1;                  // column 47
        DataType accumulators_binary_limbs_2;                  // column 48
        DataType accumulators_binary_limbs_3;                  // column 49
        DataType accumulator_low_limbs_range_constraint_0;     // column 50
        DataType accumulator_low_limbs_range_constraint_1;     // column 51
        DataType accumulator_low_limbs_range_constraint_2;     // column 52
        DataType accumulator_low_limbs_range_constraint_3;     // column 53
        DataType accumulator_low_limbs_range_constraint_4;     // column 54
        DataType accumulator_low_limbs_range_constraint_tail;  // column 55
        DataType accumulator_high_limbs_range_constraint_0;    // column 56
        DataType accumulator_high_limbs_range_constraint_1;    // column 57
        DataType accumulator_high_limbs_range_constraint_2;    // column 58
        DataType accumulator_high_limbs_range_constraint_3;    // column 59
        DataType accumulator_high_limbs_range_constraint_4;    // column 60
        DataType accumulator_high_limbs_range_constraint_tail; // column 61
        DataType quotient_low_binary_limbs;                    // column 62
        DataType quotient_high_binary_limbs;                   // column 63
        DataType quotient_low_limbs_range_constraint_0;        // column 64
        DataType quotient_low_limbs_range_constraint_1;        // column 65
        DataType quotient_low_limbs_range_constraint_2;        // column 66
        DataType quotient_low_limbs_range_constraint_3;        // column 67
        DataType quotient_low_limbs_range_constraint_4;        // column 68
        DataType quotient_low_limbs_range_constraint_tail;     // column 69
        DataType quotient_high_limbs_range_constraint_0;       // column 70
        DataType quotient_high_limbs_range_constraint_1;       // column 71
        DataType quotient_high_limbs_range_constraint_2;       // column 72
        DataType quotient_high_limbs_range_constraint_3;       // column 73
        DataType quotient_high_limbs_range_constraint_4;       // column 74
        DataType quotient_high_limbs_range_constraint_tail;    // column 75
        DataType relation_wide_limbs;                          // column 76
        DataType relation_wide_limbs_range_constraint_0;       // column 77
        DataType relation_wide_limbs_range_constraint_1;       // column 78
        DataType relation_wide_limbs_range_constraint_2;       // column 79
        DataType relation_wide_limbs_range_constraint_3;       // column 80
        DataType concatenated_range_constraints_0;             // column 81
        DataType concatenated_range_constraints_1;             // column 82
        DataType concatenated_range_constraints_2;             // column 83
        DataType concatenated_range_constraints_3;             // column 84
        DataType ordered_range_constraints_0;                  // column 85
        DataType ordered_range_constraints_1;                  // column 86
        DataType ordered_range_constraints_2;                  // column 87
        DataType ordered_range_constraints_3;                  // column 88
        DataType ordered_range_constraints_4;                  // column 89
        DataType z_perm;                                       // column 90

        DEFINE_POINTER_VIEW(NUM_WITNESS_ENTITIES,
                            &op,
                            &x_lo_y_hi,
                            &x_hi_z_1,
                            &y_lo_z_2,
                            &p_x_low_limbs,
                            &p_x_low_limbs_range_constraint_0,
                            &p_x_low_limbs_range_constraint_1,
                            &p_x_low_limbs_range_constraint_2,
                            &p_x_low_limbs_range_constraint_3,
                            &p_x_low_limbs_range_constraint_4,
                            &p_x_low_limbs_range_constraint_tail,
                            &p_x_high_limbs,
                            &p_x_high_limbs_range_constraint_0,
                            &p_x_high_limbs_range_constraint_1,
                            &p_x_high_limbs_range_constraint_2,
                            &p_x_high_limbs_range_constraint_3,
                            &p_x_high_limbs_range_constraint_4,
                            &p_x_high_limbs_range_constraint_tail,
                            &p_y_low_limbs,
                            &p_y_low_limbs_range_constraint_0,
                            &p_y_low_limbs_range_constraint_1,
                            &p_y_low_limbs_range_constraint_2,
                            &p_y_low_limbs_range_constraint_3,
                            &p_y_low_limbs_range_constraint_4,
                            &p_y_low_limbs_range_constraint_tail,
                            &p_y_high_limbs,
                            &p_y_high_limbs_range_constraint_0,
                            &p_y_high_limbs_range_constraint_1,
                            &p_y_high_limbs_range_constraint_2,
                            &p_y_high_limbs_range_constraint_3,
                            &p_y_high_limbs_range_constraint_4,
                            &p_y_high_limbs_range_constraint_tail,
                            &z_low_limbs,
                            &z_low_limbs_range_constraint_0,
                            &z_low_limbs_range_constraint_1,
                            &z_low_limbs_range_constraint_2,
                            &z_low_limbs_range_constraint_3,
                            &z_low_limbs_range_constraint_4,
                            &z_low_limbs_range_constraint_tail,
                            &z_high_limbs,
                            &z_high_limbs_range_constraint_0,
                            &z_high_limbs_range_constraint_1,
                            &z_high_limbs_range_constraint_2,
                            &z_high_limbs_range_constraint_3,
                            &z_high_limbs_range_constraint_4,
                            &z_high_limbs_range_constraint_tail,
                            &accumulators_binary_limbs_0,
                            &accumulators_binary_limbs_1,
                            &accumulators_binary_limbs_2,
                            &accumulators_binary_limbs_3,
                            &accumulator_low_limbs_range_constraint_0,
                            &accumulator_low_limbs_range_constraint_1,
                            &accumulator_low_limbs_range_constraint_2,
                            &accumulator_low_limbs_range_constraint_3,
                            &accumulator_low_limbs_range_constraint_4,
                            &accumulator_low_limbs_range_constraint_tail,
                            &accumulator_high_limbs_range_constraint_0,
                            &accumulator_high_limbs_range_constraint_1,
                            &accumulator_high_limbs_range_constraint_2,
                            &accumulator_high_limbs_range_constraint_3,
                            &accumulator_high_limbs_range_constraint_4,
                            &accumulator_high_limbs_range_constraint_tail,
                            &quotient_low_binary_limbs,
                            &quotient_high_binary_limbs,
                            &quotient_low_limbs_range_constraint_0,
                            &quotient_low_limbs_range_constraint_1,
                            &quotient_low_limbs_range_constraint_2,
                            &quotient_low_limbs_range_constraint_3,
                            &quotient_low_limbs_range_constraint_4,
                            &quotient_low_limbs_range_constraint_tail,
                            &quotient_high_limbs_range_constraint_0,
                            &quotient_high_limbs_range_constraint_1,
                            &quotient_high_limbs_range_constraint_2,
                            &quotient_high_limbs_range_constraint_3,
                            &quotient_high_limbs_range_constraint_4,
                            &quotient_high_limbs_range_constraint_tail,
                            &relation_wide_limbs,
                            &relation_wide_limbs_range_constraint_0,
                            &relation_wide_limbs_range_constraint_1,
                            &relation_wide_limbs_range_constraint_2,
                            &relation_wide_limbs_range_constraint_3,
                            &concatenated_range_constraints_0,
                            &concatenated_range_constraints_1,
                            &concatenated_range_constraints_2,
                            &concatenated_range_constraints_3,
                            &ordered_range_constraints_0,
                            &ordered_range_constraints_1,
                            &ordered_range_constraints_2,
                            &ordered_range_constraints_3,
                            &ordered_range_constraints_4,
                            &z_perm, )

        std::vector<HandleType> get_wires() override
        {
            return { op,
                     x_lo_y_hi,
                     x_hi_z_1,
                     y_lo_z_2,
                     p_x_low_limbs,
                     p_x_low_limbs_range_constraint_0,
                     p_x_low_limbs_range_constraint_1,
                     p_x_low_limbs_range_constraint_2,
                     p_x_low_limbs_range_constraint_3,
                     p_x_low_limbs_range_constraint_4,
                     p_x_low_limbs_range_constraint_tail,
                     p_x_high_limbs,
                     p_x_high_limbs_range_constraint_0,
                     p_x_high_limbs_range_constraint_1,
                     p_x_high_limbs_range_constraint_2,
                     p_x_high_limbs_range_constraint_3,
                     p_x_high_limbs_range_constraint_4,
                     p_x_high_limbs_range_constraint_tail,
                     p_y_low_limbs,
                     p_y_low_limbs_range_constraint_0,
                     p_y_low_limbs_range_constraint_1,
                     p_y_low_limbs_range_constraint_2,
                     p_y_low_limbs_range_constraint_3,
                     p_y_low_limbs_range_constraint_4,
                     p_y_low_limbs_range_constraint_tail,
                     p_y_high_limbs,
                     p_y_high_limbs_range_constraint_0,
                     p_y_high_limbs_range_constraint_1,
                     p_y_high_limbs_range_constraint_2,
                     p_y_high_limbs_range_constraint_3,
                     p_y_high_limbs_range_constraint_4,
                     p_y_high_limbs_range_constraint_tail,
                     z_low_limbs,
                     z_low_limbs_range_constraint_0,
                     z_low_limbs_range_constraint_1,
                     z_low_limbs_range_constraint_2,
                     z_low_limbs_range_constraint_3,
                     z_low_limbs_range_constraint_4,
                     z_low_limbs_range_constraint_tail,
                     z_high_limbs,
                     z_high_limbs_range_constraint_0,
                     z_high_limbs_range_constraint_1,
                     z_high_limbs_range_constraint_2,
                     z_high_limbs_range_constraint_3,
                     z_high_limbs_range_constraint_4,
                     z_high_limbs_range_constraint_tail,
                     accumulators_binary_limbs_0,
                     accumulators_binary_limbs_1,
                     accumulators_binary_limbs_2,
                     accumulators_binary_limbs_3,
                     accumulator_low_limbs_range_constraint_0,
                     accumulator_low_limbs_range_constraint_1,
                     accumulator_low_limbs_range_constraint_2,
                     accumulator_low_limbs_range_constraint_3,
                     accumulator_low_limbs_range_constraint_4,
                     accumulator_low_limbs_range_constraint_tail,
                     accumulator_high_limbs_range_constraint_0,
                     accumulator_high_limbs_range_constraint_1,
                     accumulator_high_limbs_range_constraint_2,
                     accumulator_high_limbs_range_constraint_3,
                     accumulator_high_limbs_range_constraint_4,
                     accumulator_high_limbs_range_constraint_tail,
                     quotient_low_binary_limbs,
                     quotient_high_binary_limbs,
                     quotient_low_limbs_range_constraint_0,
                     quotient_low_limbs_range_constraint_1,
                     quotient_low_limbs_range_constraint_2,
                     quotient_low_limbs_range_constraint_3,
                     quotient_low_limbs_range_constraint_4,
                     quotient_low_limbs_range_constraint_tail,
                     quotient_high_limbs_range_constraint_0,
                     quotient_high_limbs_range_constraint_1,
                     quotient_high_limbs_range_constraint_2,
                     quotient_high_limbs_range_constraint_3,
                     quotient_high_limbs_range_constraint_4,
                     quotient_high_limbs_range_constraint_tail,
                     relation_wide_limbs,
                     relation_wide_limbs_range_constraint_0,
                     relation_wide_limbs_range_constraint_1,
                     relation_wide_limbs_range_constraint_2,
                     relation_wide_limbs_range_constraint_3,
                     ordered_range_constraints_0,
                     ordered_range_constraints_1,
                     ordered_range_constraints_2,
                     ordered_range_constraints_3,
                     ordered_range_constraints_4 };
        };

        /**
         * @brief Get the polynomials that need to be constructed from other polynomials by concatenation
         *
         * @return std::vector<HandleType>
         */
        std::vector<HandleType> get_concatenated_constraints()
        {
            return { concatenated_range_constraints_0,
                     concatenated_range_constraints_1,
                     concatenated_range_constraints_2,
                     concatenated_range_constraints_3 };
        }

        /**
         * @brief Get the polynomials that are concatenated for the permutation relation
         *
         * @return std::vector<std::vector<HandleType>>
         */
        std::vector<std::vector<HandleType>> get_concatenation_groups()
        {
            return {
                {
                    p_x_low_limbs_range_constraint_0,
                    p_x_low_limbs_range_constraint_1,
                    p_x_low_limbs_range_constraint_2,
                    p_x_low_limbs_range_constraint_3,
                    p_x_low_limbs_range_constraint_4,
                    p_x_low_limbs_range_constraint_tail,
                    p_x_high_limbs_range_constraint_0,
                    p_x_high_limbs_range_constraint_1,
                    p_x_high_limbs_range_constraint_2,
                    p_x_high_limbs_range_constraint_3,
                    p_x_high_limbs_range_constraint_4,
                    p_x_high_limbs_range_constraint_tail,
                    p_y_low_limbs_range_constraint_0,
                    p_y_low_limbs_range_constraint_1,
                    p_y_low_limbs_range_constraint_2,
                    p_y_low_limbs_range_constraint_3,
                },
                {
                    p_y_low_limbs_range_constraint_4,
                    p_y_low_limbs_range_constraint_tail,
                    p_y_high_limbs_range_constraint_0,
                    p_y_high_limbs_range_constraint_1,
                    p_y_high_limbs_range_constraint_2,
                    p_y_high_limbs_range_constraint_3,
                    p_y_high_limbs_range_constraint_4,
                    p_y_high_limbs_range_constraint_tail,
                    z_low_limbs_range_constraint_0,
                    z_low_limbs_range_constraint_1,
                    z_low_limbs_range_constraint_2,
                    z_low_limbs_range_constraint_3,
                    z_low_limbs_range_constraint_4,
                    z_low_limbs_range_constraint_tail,
                    z_high_limbs_range_constraint_0,
                    z_high_limbs_range_constraint_1,
                },
                {
                    z_high_limbs_range_constraint_2,
                    z_high_limbs_range_constraint_3,
                    z_high_limbs_range_constraint_4,
                    z_high_limbs_range_constraint_tail,
                    accumulator_low_limbs_range_constraint_0,
                    accumulator_low_limbs_range_constraint_1,
                    accumulator_low_limbs_range_constraint_2,
                    accumulator_low_limbs_range_constraint_3,
                    accumulator_low_limbs_range_constraint_4,
                    accumulator_low_limbs_range_constraint_tail,
                    accumulator_high_limbs_range_constraint_0,
                    accumulator_high_limbs_range_constraint_1,
                    accumulator_high_limbs_range_constraint_2,
                    accumulator_high_limbs_range_constraint_3,
                    accumulator_high_limbs_range_constraint_4,
                    accumulator_high_limbs_range_constraint_tail,
                },
                {
                    quotient_low_limbs_range_constraint_0,
                    quotient_low_limbs_range_constraint_1,
                    quotient_low_limbs_range_constraint_2,
                    quotient_low_limbs_range_constraint_3,
                    quotient_low_limbs_range_constraint_4,
                    quotient_low_limbs_range_constraint_tail,
                    quotient_high_limbs_range_constraint_0,
                    quotient_high_limbs_range_constraint_1,
                    quotient_high_limbs_range_constraint_2,
                    quotient_high_limbs_range_constraint_3,
                    quotient_high_limbs_range_constraint_4,
                    quotient_high_limbs_range_constraint_tail,
                    relation_wide_limbs_range_constraint_0,
                    relation_wide_limbs_range_constraint_1,
                    relation_wide_limbs_range_constraint_2,
                    relation_wide_limbs_range_constraint_3,
                },
            };
        };
    };

    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + "shiftEntities". It could be
     * implemented as such, but we have this now.
     */
    template <typename DataType, typename HandleType>
    class AllEntities : public AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES> {
      public:
        DataType op;                                                 // column 0
        DataType x_lo_y_hi;                                          // column 1
        DataType x_hi_z_1;                                           // column 2
        DataType y_lo_z_2;                                           // column 3
        DataType p_x_low_limbs;                                      // column 4
        DataType p_x_low_limbs_range_constraint_0;                   // column 5
        DataType p_x_low_limbs_range_constraint_1;                   // column 6
        DataType p_x_low_limbs_range_constraint_2;                   // column 7
        DataType p_x_low_limbs_range_constraint_3;                   // column 8
        DataType p_x_low_limbs_range_constraint_4;                   // column 9
        DataType p_x_low_limbs_range_constraint_tail;                // column 10
        DataType p_x_high_limbs;                                     // column 11
        DataType p_x_high_limbs_range_constraint_0;                  // column 12
        DataType p_x_high_limbs_range_constraint_1;                  // column 13
        DataType p_x_high_limbs_range_constraint_2;                  // column 14
        DataType p_x_high_limbs_range_constraint_3;                  // column 15
        DataType p_x_high_limbs_range_constraint_4;                  // column 16
        DataType p_x_high_limbs_range_constraint_tail;               // column 17
        DataType p_y_low_limbs;                                      // column 18
        DataType p_y_low_limbs_range_constraint_0;                   // column 19
        DataType p_y_low_limbs_range_constraint_1;                   // column 20
        DataType p_y_low_limbs_range_constraint_2;                   // column 21
        DataType p_y_low_limbs_range_constraint_3;                   // column 22
        DataType p_y_low_limbs_range_constraint_4;                   // column 23
        DataType p_y_low_limbs_range_constraint_tail;                // column 24
        DataType p_y_high_limbs;                                     // column 25
        DataType p_y_high_limbs_range_constraint_0;                  // column 26
        DataType p_y_high_limbs_range_constraint_1;                  // column 27
        DataType p_y_high_limbs_range_constraint_2;                  // column 28
        DataType p_y_high_limbs_range_constraint_3;                  // column 29
        DataType p_y_high_limbs_range_constraint_4;                  // column 30
        DataType p_y_high_limbs_range_constraint_tail;               // column 31
        DataType z_low_limbs;                                        // column 32
        DataType z_low_limbs_range_constraint_0;                     // column 33
        DataType z_low_limbs_range_constraint_1;                     // column 34
        DataType z_low_limbs_range_constraint_2;                     // column 35
        DataType z_low_limbs_range_constraint_3;                     // column 36
        DataType z_low_limbs_range_constraint_4;                     // column 37
        DataType z_low_limbs_range_constraint_tail;                  // column 38
        DataType z_high_limbs;                                       // column 39
        DataType z_high_limbs_range_constraint_0;                    // column 40
        DataType z_high_limbs_range_constraint_1;                    // column 41
        DataType z_high_limbs_range_constraint_2;                    // column 42
        DataType z_high_limbs_range_constraint_3;                    // column 43
        DataType z_high_limbs_range_constraint_4;                    // column 44
        DataType z_high_limbs_range_constraint_tail;                 // column 45
        DataType accumulators_binary_limbs_0;                        // column 46
        DataType accumulators_binary_limbs_1;                        // column 47
        DataType accumulators_binary_limbs_2;                        // column 48
        DataType accumulators_binary_limbs_3;                        // column 49
        DataType accumulator_low_limbs_range_constraint_0;           // column 50
        DataType accumulator_low_limbs_range_constraint_1;           // column 51
        DataType accumulator_low_limbs_range_constraint_2;           // column 52
        DataType accumulator_low_limbs_range_constraint_3;           // column 53
        DataType accumulator_low_limbs_range_constraint_4;           // column 54
        DataType accumulator_low_limbs_range_constraint_tail;        // column 55
        DataType accumulator_high_limbs_range_constraint_0;          // column 56
        DataType accumulator_high_limbs_range_constraint_1;          // column 57
        DataType accumulator_high_limbs_range_constraint_2;          // column 58
        DataType accumulator_high_limbs_range_constraint_3;          // column 59
        DataType accumulator_high_limbs_range_constraint_4;          // column 60
        DataType accumulator_high_limbs_range_constraint_tail;       // column 61
        DataType quotient_low_binary_limbs;                          // column 62
        DataType quotient_high_binary_limbs;                         // column 63
        DataType quotient_low_limbs_range_constraint_0;              // column 64
        DataType quotient_low_limbs_range_constraint_1;              // column 65
        DataType quotient_low_limbs_range_constraint_2;              // column 66
        DataType quotient_low_limbs_range_constraint_3;              // column 67
        DataType quotient_low_limbs_range_constraint_4;              // column 68
        DataType quotient_low_limbs_range_constraint_tail;           // column 69
        DataType quotient_high_limbs_range_constraint_0;             // column 70
        DataType quotient_high_limbs_range_constraint_1;             // column 71
        DataType quotient_high_limbs_range_constraint_2;             // column 72
        DataType quotient_high_limbs_range_constraint_3;             // column 73
        DataType quotient_high_limbs_range_constraint_4;             // column 74
        DataType quotient_high_limbs_range_constraint_tail;          // column 75
        DataType relation_wide_limbs;                                // column 76
        DataType relation_wide_limbs_range_constraint_0;             // column 77
        DataType relation_wide_limbs_range_constraint_1;             // column 78
        DataType relation_wide_limbs_range_constraint_2;             // column 79
        DataType relation_wide_limbs_range_constraint_3;             // column 80
        DataType concatenated_range_constraints_0;                   // column 81
        DataType concatenated_range_constraints_1;                   // column 82
        DataType concatenated_range_constraints_2;                   // column 83
        DataType concatenated_range_constraints_3;                   // column 84
        DataType ordered_range_constraints_0;                        // column 85
        DataType ordered_range_constraints_1;                        // column 86
        DataType ordered_range_constraints_2;                        // column 87
        DataType ordered_range_constraints_3;                        // column 88
        DataType ordered_range_constraints_4;                        // column 89
        DataType z_perm;                                             // column 90
        DataType x_lo_y_hi_shift;                                    // column 91
        DataType x_hi_z_1_shift;                                     // column 92
        DataType y_lo_z_2_shift;                                     // column 93
        DataType p_x_low_limbs_shift;                                // column 94
        DataType p_x_low_limbs_range_constraint_0_shift;             // column 95
        DataType p_x_low_limbs_range_constraint_1_shift;             // column 96
        DataType p_x_low_limbs_range_constraint_2_shift;             // column 97
        DataType p_x_low_limbs_range_constraint_3_shift;             // column 98
        DataType p_x_low_limbs_range_constraint_4_shift;             // column 99
        DataType p_x_low_limbs_range_constraint_tail_shift;          // column 100
        DataType p_x_high_limbs_shift;                               // column 101
        DataType p_x_high_limbs_range_constraint_0_shift;            // column 102
        DataType p_x_high_limbs_range_constraint_1_shift;            // column 103
        DataType p_x_high_limbs_range_constraint_2_shift;            // column 104
        DataType p_x_high_limbs_range_constraint_3_shift;            // column 105
        DataType p_x_high_limbs_range_constraint_4_shift;            // column 106
        DataType p_x_high_limbs_range_constraint_tail_shift;         // column 107
        DataType p_y_low_limbs_shift;                                // column 108
        DataType p_y_low_limbs_range_constraint_0_shift;             // column 109
        DataType p_y_low_limbs_range_constraint_1_shift;             // column 110
        DataType p_y_low_limbs_range_constraint_2_shift;             // column 111
        DataType p_y_low_limbs_range_constraint_3_shift;             // column 112
        DataType p_y_low_limbs_range_constraint_4_shift;             // column 113
        DataType p_y_low_limbs_range_constraint_tail_shift;          // column 114
        DataType p_y_high_limbs_shift;                               // column 115
        DataType p_y_high_limbs_range_constraint_0_shift;            // column 116
        DataType p_y_high_limbs_range_constraint_1_shift;            // column 117
        DataType p_y_high_limbs_range_constraint_2_shift;            // column 118
        DataType p_y_high_limbs_range_constraint_3_shift;            // column 119
        DataType p_y_high_limbs_range_constraint_4_shift;            // column 120
        DataType p_y_high_limbs_range_constraint_tail_shift;         // column 121
        DataType z_low_limbs_shift;                                  // column 122
        DataType z_low_limbs_range_constraint_0_shift;               // column 123
        DataType z_low_limbs_range_constraint_1_shift;               // column 124
        DataType z_low_limbs_range_constraint_2_shift;               // column 125
        DataType z_low_limbs_range_constraint_3_shift;               // column 126
        DataType z_low_limbs_range_constraint_4_shift;               // column 127
        DataType z_low_limbs_range_constraint_tail_shift;            // column 128
        DataType z_high_limbs_shift;                                 // column 129
        DataType z_high_limbs_range_constraint_0_shift;              // column 130
        DataType z_high_limbs_range_constraint_1_shift;              // column 131
        DataType z_high_limbs_range_constraint_2_shift;              // column 132
        DataType z_high_limbs_range_constraint_3_shift;              // column 133
        DataType z_high_limbs_range_constraint_4_shift;              // column 134
        DataType z_high_limbs_range_constraint_tail_shift;           // column 135
        DataType accumulators_binary_limbs_0_shift;                  // column 136
        DataType accumulators_binary_limbs_1_shift;                  // column 137
        DataType accumulators_binary_limbs_2_shift;                  // column 138
        DataType accumulators_binary_limbs_3_shift;                  // column 139
        DataType accumulator_low_limbs_range_constraint_0_shift;     // column 140
        DataType accumulator_low_limbs_range_constraint_1_shift;     // column 141
        DataType accumulator_low_limbs_range_constraint_2_shift;     // column 142
        DataType accumulator_low_limbs_range_constraint_3_shift;     // column 143
        DataType accumulator_low_limbs_range_constraint_4_shift;     // column 144
        DataType accumulator_low_limbs_range_constraint_tail_shift;  // column 145
        DataType accumulator_high_limbs_range_constraint_0_shift;    // column 146
        DataType accumulator_high_limbs_range_constraint_1_shift;    // column 147
        DataType accumulator_high_limbs_range_constraint_2_shift;    // column 148
        DataType accumulator_high_limbs_range_constraint_3_shift;    // column 149
        DataType accumulator_high_limbs_range_constraint_4_shift;    // column 150
        DataType accumulator_high_limbs_range_constraint_tail_shift; // column 151
        DataType quotient_low_binary_limbs_shift;                    // column 152
        DataType quotient_high_binary_limbs_shift;                   // column 153
        DataType quotient_low_limbs_range_constraint_0_shift;        // column 154
        DataType quotient_low_limbs_range_constraint_1_shift;        // column 155
        DataType quotient_low_limbs_range_constraint_2_shift;        // column 156
        DataType quotient_low_limbs_range_constraint_3_shift;        // column 157
        DataType quotient_low_limbs_range_constraint_4_shift;        // column 158
        DataType quotient_low_limbs_range_constraint_tail_shift;     // column 159
        DataType quotient_high_limbs_range_constraint_0_shift;       // column 160
        DataType quotient_high_limbs_range_constraint_1_shift;       // column 161
        DataType quotient_high_limbs_range_constraint_2_shift;       // column 162
        DataType quotient_high_limbs_range_constraint_3_shift;       // column 163
        DataType quotient_high_limbs_range_constraint_4_shift;       // column 164
        DataType quotient_high_limbs_range_constraint_tail_shift;    // column 165
        DataType relation_wide_limbs_shift;                          // column 166
        DataType relation_wide_limbs_range_constraint_0_shift;       // column 167
        DataType relation_wide_limbs_range_constraint_1_shift;       // column 168
        DataType relation_wide_limbs_range_constraint_2_shift;       // column 169
        DataType relation_wide_limbs_range_constraint_3_shift;       // column 170
        DataType ordered_range_constraints_0_shift;                  // column 171
        DataType ordered_range_constraints_1_shift;                  // column 172
        DataType ordered_range_constraints_2_shift;                  // column 173
        DataType ordered_range_constraints_3_shift;                  // column 174
        DataType ordered_range_constraints_4_shift;                  // column 175
        DataType z_perm_shift;                                       // column 176
        DataType lagrange_first;                                     // column 177
        DataType lagrange_last;                                      // column 178
        DataType lagrange_odd_in_minicircuit;                        // column 179
        DataType lagrange_even_in_minicircuit;                       // column 180
        DataType lagrange_second;                                    // column 181
        DataType lagrange_second_to_last_in_minicircuit;             // column 182
        DataType ordered_extra_range_constraints_numerator;          // column 183

        // defines a method pointer_view that returns the following, with const and non-const variants
        DEFINE_POINTER_VIEW(NUM_ALL_ENTITIES,
                            &op,
                            &x_lo_y_hi,
                            &x_hi_z_1,
                            &y_lo_z_2,
                            &p_x_low_limbs,
                            &p_x_low_limbs_range_constraint_0,
                            &p_x_low_limbs_range_constraint_1,
                            &p_x_low_limbs_range_constraint_2,
                            &p_x_low_limbs_range_constraint_3,
                            &p_x_low_limbs_range_constraint_4,
                            &p_x_low_limbs_range_constraint_tail,
                            &p_x_high_limbs,
                            &p_x_high_limbs_range_constraint_0,
                            &p_x_high_limbs_range_constraint_1,
                            &p_x_high_limbs_range_constraint_2,
                            &p_x_high_limbs_range_constraint_3,
                            &p_x_high_limbs_range_constraint_4,
                            &p_x_high_limbs_range_constraint_tail,
                            &p_y_low_limbs,
                            &p_y_low_limbs_range_constraint_0,
                            &p_y_low_limbs_range_constraint_1,
                            &p_y_low_limbs_range_constraint_2,
                            &p_y_low_limbs_range_constraint_3,
                            &p_y_low_limbs_range_constraint_4,
                            &p_y_low_limbs_range_constraint_tail,
                            &p_y_high_limbs,
                            &p_y_high_limbs_range_constraint_0,
                            &p_y_high_limbs_range_constraint_1,
                            &p_y_high_limbs_range_constraint_2,
                            &p_y_high_limbs_range_constraint_3,
                            &p_y_high_limbs_range_constraint_4,
                            &p_y_high_limbs_range_constraint_tail,
                            &z_low_limbs,
                            &z_low_limbs_range_constraint_0,
                            &z_low_limbs_range_constraint_1,
                            &z_low_limbs_range_constraint_2,
                            &z_low_limbs_range_constraint_3,
                            &z_low_limbs_range_constraint_4,
                            &z_low_limbs_range_constraint_tail,
                            &z_high_limbs,
                            &z_high_limbs_range_constraint_0,
                            &z_high_limbs_range_constraint_1,
                            &z_high_limbs_range_constraint_2,
                            &z_high_limbs_range_constraint_3,
                            &z_high_limbs_range_constraint_4,
                            &z_high_limbs_range_constraint_tail,
                            &accumulators_binary_limbs_0,
                            &accumulators_binary_limbs_1,
                            &accumulators_binary_limbs_2,
                            &accumulators_binary_limbs_3,
                            &accumulator_low_limbs_range_constraint_0,
                            &accumulator_low_limbs_range_constraint_1,
                            &accumulator_low_limbs_range_constraint_2,
                            &accumulator_low_limbs_range_constraint_3,
                            &accumulator_low_limbs_range_constraint_4,
                            &accumulator_low_limbs_range_constraint_tail,
                            &accumulator_high_limbs_range_constraint_0,
                            &accumulator_high_limbs_range_constraint_1,
                            &accumulator_high_limbs_range_constraint_2,
                            &accumulator_high_limbs_range_constraint_3,
                            &accumulator_high_limbs_range_constraint_4,
                            &accumulator_high_limbs_range_constraint_tail,
                            &quotient_low_binary_limbs,
                            &quotient_high_binary_limbs,
                            &quotient_low_limbs_range_constraint_0,
                            &quotient_low_limbs_range_constraint_1,
                            &quotient_low_limbs_range_constraint_2,
                            &quotient_low_limbs_range_constraint_3,
                            &quotient_low_limbs_range_constraint_4,
                            &quotient_low_limbs_range_constraint_tail,
                            &quotient_high_limbs_range_constraint_0,
                            &quotient_high_limbs_range_constraint_1,
                            &quotient_high_limbs_range_constraint_2,
                            &quotient_high_limbs_range_constraint_3,
                            &quotient_high_limbs_range_constraint_4,
                            &quotient_high_limbs_range_constraint_tail,
                            &relation_wide_limbs,
                            &relation_wide_limbs_range_constraint_0,
                            &relation_wide_limbs_range_constraint_1,
                            &relation_wide_limbs_range_constraint_2,
                            &relation_wide_limbs_range_constraint_3,
                            &concatenated_range_constraints_0,
                            &concatenated_range_constraints_1,
                            &concatenated_range_constraints_2,
                            &concatenated_range_constraints_3,
                            &ordered_range_constraints_0,
                            &ordered_range_constraints_1,
                            &ordered_range_constraints_2,
                            &ordered_range_constraints_3,
                            &ordered_range_constraints_4,
                            &z_perm,
                            &x_lo_y_hi_shift,
                            &x_hi_z_1_shift,
                            &y_lo_z_2_shift,
                            &p_x_low_limbs_shift,
                            &p_x_low_limbs_range_constraint_0_shift,
                            &p_x_low_limbs_range_constraint_1_shift,
                            &p_x_low_limbs_range_constraint_2_shift,
                            &p_x_low_limbs_range_constraint_3_shift,
                            &p_x_low_limbs_range_constraint_4_shift,
                            &p_x_low_limbs_range_constraint_tail_shift,
                            &p_x_high_limbs_shift,
                            &p_x_high_limbs_range_constraint_0_shift,
                            &p_x_high_limbs_range_constraint_1_shift,
                            &p_x_high_limbs_range_constraint_2_shift,
                            &p_x_high_limbs_range_constraint_3_shift,
                            &p_x_high_limbs_range_constraint_4_shift,
                            &p_x_high_limbs_range_constraint_tail_shift,
                            &p_y_low_limbs_shift,
                            &p_y_low_limbs_range_constraint_0_shift,
                            &p_y_low_limbs_range_constraint_1_shift,
                            &p_y_low_limbs_range_constraint_2_shift,
                            &p_y_low_limbs_range_constraint_3_shift,
                            &p_y_low_limbs_range_constraint_4_shift,
                            &p_y_low_limbs_range_constraint_tail_shift,
                            &p_y_high_limbs_shift,
                            &p_y_high_limbs_range_constraint_0_shift,
                            &p_y_high_limbs_range_constraint_1_shift,
                            &p_y_high_limbs_range_constraint_2_shift,
                            &p_y_high_limbs_range_constraint_3_shift,
                            &p_y_high_limbs_range_constraint_4_shift,
                            &p_y_high_limbs_range_constraint_tail_shift,
                            &z_low_limbs_shift,
                            &z_low_limbs_range_constraint_0_shift,
                            &z_low_limbs_range_constraint_1_shift,
                            &z_low_limbs_range_constraint_2_shift,
                            &z_low_limbs_range_constraint_3_shift,
                            &z_low_limbs_range_constraint_4_shift,
                            &z_low_limbs_range_constraint_tail_shift,
                            &z_high_limbs_shift,
                            &z_high_limbs_range_constraint_0_shift,
                            &z_high_limbs_range_constraint_1_shift,
                            &z_high_limbs_range_constraint_2_shift,
                            &z_high_limbs_range_constraint_3_shift,
                            &z_high_limbs_range_constraint_4_shift,
                            &z_high_limbs_range_constraint_tail_shift,
                            &accumulators_binary_limbs_0_shift,
                            &accumulators_binary_limbs_1_shift,
                            &accumulators_binary_limbs_2_shift,
                            &accumulators_binary_limbs_3_shift,
                            &accumulator_low_limbs_range_constraint_0_shift,
                            &accumulator_low_limbs_range_constraint_1_shift,
                            &accumulator_low_limbs_range_constraint_2_shift,
                            &accumulator_low_limbs_range_constraint_3_shift,
                            &accumulator_low_limbs_range_constraint_4_shift,
                            &accumulator_low_limbs_range_constraint_tail_shift,
                            &accumulator_high_limbs_range_constraint_0_shift,
                            &accumulator_high_limbs_range_constraint_1_shift,
                            &accumulator_high_limbs_range_constraint_2_shift,
                            &accumulator_high_limbs_range_constraint_3_shift,
                            &accumulator_high_limbs_range_constraint_4_shift,
                            &accumulator_high_limbs_range_constraint_tail_shift,
                            &quotient_low_binary_limbs_shift,
                            &quotient_high_binary_limbs_shift,
                            &quotient_low_limbs_range_constraint_0_shift,
                            &quotient_low_limbs_range_constraint_1_shift,
                            &quotient_low_limbs_range_constraint_2_shift,
                            &quotient_low_limbs_range_constraint_3_shift,
                            &quotient_low_limbs_range_constraint_4_shift,
                            &quotient_low_limbs_range_constraint_tail_shift,
                            &quotient_high_limbs_range_constraint_0_shift,
                            &quotient_high_limbs_range_constraint_1_shift,
                            &quotient_high_limbs_range_constraint_2_shift,
                            &quotient_high_limbs_range_constraint_3_shift,
                            &quotient_high_limbs_range_constraint_4_shift,
                            &quotient_high_limbs_range_constraint_tail_shift,
                            &relation_wide_limbs_shift,
                            &relation_wide_limbs_range_constraint_0_shift,
                            &relation_wide_limbs_range_constraint_1_shift,
                            &relation_wide_limbs_range_constraint_2_shift,
                            &relation_wide_limbs_range_constraint_3_shift,
                            &ordered_range_constraints_0_shift,
                            &ordered_range_constraints_1_shift,
                            &ordered_range_constraints_2_shift,
                            &ordered_range_constraints_3_shift,
                            &ordered_range_constraints_4_shift,
                            &z_perm_shift,
                            &lagrange_first,
                            &lagrange_last,
                            &lagrange_odd_in_minicircuit,
                            &lagrange_even_in_minicircuit,
                            &lagrange_second,
                            &lagrange_second_to_last_in_minicircuit,
                            &ordered_extra_range_constraints_numerator)

        std::vector<HandleType> get_wires() override
        {

            return { op,
                     x_lo_y_hi,
                     x_hi_z_1,
                     y_lo_z_2,
                     p_x_low_limbs,
                     p_x_low_limbs_range_constraint_0,
                     p_x_low_limbs_range_constraint_1,
                     p_x_low_limbs_range_constraint_2,
                     p_x_low_limbs_range_constraint_3,
                     p_x_low_limbs_range_constraint_4,
                     p_x_low_limbs_range_constraint_tail,
                     p_x_high_limbs,
                     p_x_high_limbs_range_constraint_0,
                     p_x_high_limbs_range_constraint_1,
                     p_x_high_limbs_range_constraint_2,
                     p_x_high_limbs_range_constraint_3,
                     p_x_high_limbs_range_constraint_4,
                     p_x_high_limbs_range_constraint_tail,
                     p_y_low_limbs,
                     p_y_low_limbs_range_constraint_0,
                     p_y_low_limbs_range_constraint_1,
                     p_y_low_limbs_range_constraint_2,
                     p_y_low_limbs_range_constraint_3,
                     p_y_low_limbs_range_constraint_4,
                     p_y_low_limbs_range_constraint_tail,
                     p_y_high_limbs,
                     p_y_high_limbs_range_constraint_0,
                     p_y_high_limbs_range_constraint_1,
                     p_y_high_limbs_range_constraint_2,
                     p_y_high_limbs_range_constraint_3,
                     p_y_high_limbs_range_constraint_4,
                     p_y_high_limbs_range_constraint_tail,
                     z_low_limbs,
                     z_low_limbs_range_constraint_0,
                     z_low_limbs_range_constraint_1,
                     z_low_limbs_range_constraint_2,
                     z_low_limbs_range_constraint_3,
                     z_low_limbs_range_constraint_4,
                     z_low_limbs_range_constraint_tail,
                     z_high_limbs,
                     z_high_limbs_range_constraint_0,
                     z_high_limbs_range_constraint_1,
                     z_high_limbs_range_constraint_2,
                     z_high_limbs_range_constraint_3,
                     z_high_limbs_range_constraint_4,
                     z_high_limbs_range_constraint_tail,
                     accumulators_binary_limbs_0,
                     accumulators_binary_limbs_1,
                     accumulators_binary_limbs_2,
                     accumulators_binary_limbs_3,
                     accumulator_low_limbs_range_constraint_0,
                     accumulator_low_limbs_range_constraint_1,
                     accumulator_low_limbs_range_constraint_2,
                     accumulator_low_limbs_range_constraint_3,
                     accumulator_low_limbs_range_constraint_4,
                     accumulator_low_limbs_range_constraint_tail,
                     accumulator_high_limbs_range_constraint_0,
                     accumulator_high_limbs_range_constraint_1,
                     accumulator_high_limbs_range_constraint_2,
                     accumulator_high_limbs_range_constraint_3,
                     accumulator_high_limbs_range_constraint_4,
                     accumulator_high_limbs_range_constraint_tail,
                     quotient_low_binary_limbs,
                     quotient_high_binary_limbs,
                     quotient_low_limbs_range_constraint_0,
                     quotient_low_limbs_range_constraint_1,
                     quotient_low_limbs_range_constraint_2,
                     quotient_low_limbs_range_constraint_3,
                     quotient_low_limbs_range_constraint_4,
                     quotient_low_limbs_range_constraint_tail,
                     quotient_high_limbs_range_constraint_0,
                     quotient_high_limbs_range_constraint_1,
                     quotient_high_limbs_range_constraint_2,
                     quotient_high_limbs_range_constraint_3,
                     quotient_high_limbs_range_constraint_4,
                     quotient_high_limbs_range_constraint_tail,
                     relation_wide_limbs,
                     relation_wide_limbs_range_constraint_0,
                     relation_wide_limbs_range_constraint_1,
                     relation_wide_limbs_range_constraint_2,
                     relation_wide_limbs_range_constraint_3,
                     ordered_range_constraints_0,
                     ordered_range_constraints_1,
                     ordered_range_constraints_2,
                     ordered_range_constraints_3,
                     ordered_range_constraints_4 };
        };

        /**
         * @brief Get the polynomials that are concatenated for the permutation relation
         *
         * @return std::vector<std::vector<HandleType>>
         */
        std::vector<std::vector<HandleType>> get_concatenation_groups()
        {
            return {
                {
                    p_x_low_limbs_range_constraint_0,
                    p_x_low_limbs_range_constraint_1,
                    p_x_low_limbs_range_constraint_2,
                    p_x_low_limbs_range_constraint_3,
                    p_x_low_limbs_range_constraint_4,
                    p_x_low_limbs_range_constraint_tail,
                    p_x_high_limbs_range_constraint_0,
                    p_x_high_limbs_range_constraint_1,
                    p_x_high_limbs_range_constraint_2,
                    p_x_high_limbs_range_constraint_3,
                    p_x_high_limbs_range_constraint_4,
                    p_x_high_limbs_range_constraint_tail,
                    p_y_low_limbs_range_constraint_0,
                    p_y_low_limbs_range_constraint_1,
                    p_y_low_limbs_range_constraint_2,
                    p_y_low_limbs_range_constraint_3,
                },
                {
                    p_y_low_limbs_range_constraint_4,
                    p_y_low_limbs_range_constraint_tail,
                    p_y_high_limbs_range_constraint_0,
                    p_y_high_limbs_range_constraint_1,
                    p_y_high_limbs_range_constraint_2,
                    p_y_high_limbs_range_constraint_3,
                    p_y_high_limbs_range_constraint_4,
                    p_y_high_limbs_range_constraint_tail,
                    z_low_limbs_range_constraint_0,
                    z_low_limbs_range_constraint_1,
                    z_low_limbs_range_constraint_2,
                    z_low_limbs_range_constraint_3,
                    z_low_limbs_range_constraint_4,
                    z_low_limbs_range_constraint_tail,
                    z_high_limbs_range_constraint_0,
                    z_high_limbs_range_constraint_1,
                },
                {
                    z_high_limbs_range_constraint_2,
                    z_high_limbs_range_constraint_3,
                    z_high_limbs_range_constraint_4,
                    z_high_limbs_range_constraint_tail,
                    accumulator_low_limbs_range_constraint_0,
                    accumulator_low_limbs_range_constraint_1,
                    accumulator_low_limbs_range_constraint_2,
                    accumulator_low_limbs_range_constraint_3,
                    accumulator_low_limbs_range_constraint_4,
                    accumulator_low_limbs_range_constraint_tail,
                    accumulator_high_limbs_range_constraint_0,
                    accumulator_high_limbs_range_constraint_1,
                    accumulator_high_limbs_range_constraint_2,
                    accumulator_high_limbs_range_constraint_3,
                    accumulator_high_limbs_range_constraint_4,
                    accumulator_high_limbs_range_constraint_tail,
                },
                {
                    quotient_low_limbs_range_constraint_0,
                    quotient_low_limbs_range_constraint_1,
                    quotient_low_limbs_range_constraint_2,
                    quotient_low_limbs_range_constraint_3,
                    quotient_low_limbs_range_constraint_4,
                    quotient_low_limbs_range_constraint_tail,
                    quotient_high_limbs_range_constraint_0,
                    quotient_high_limbs_range_constraint_1,
                    quotient_high_limbs_range_constraint_2,
                    quotient_high_limbs_range_constraint_3,
                    quotient_high_limbs_range_constraint_4,
                    quotient_high_limbs_range_constraint_tail,
                    relation_wide_limbs_range_constraint_0,
                    relation_wide_limbs_range_constraint_1,
                    relation_wide_limbs_range_constraint_2,
                    relation_wide_limbs_range_constraint_3,
                },
            };
        }
        /**
         * @brief Get the polynomials that need to be constructed from other polynomials by concatenation
         *
         * @return std::vector<HandleType>
         */
        std::vector<HandleType> get_concatenated_constraints()
        {
            return { concatenated_range_constraints_0,
                     concatenated_range_constraints_1,
                     concatenated_range_constraints_2,
                     concatenated_range_constraints_3 };
        };
        /**
         * @brief Get the polynomials from the grand product denominator
         *
         * @return std::vector<HandleType>
         */
        std::vector<HandleType> get_ordered_constraints()
        {
            return { ordered_range_constraints_0,
                     ordered_range_constraints_1,
                     ordered_range_constraints_2,
                     ordered_range_constraints_3,
                     ordered_range_constraints_4 };
        };

        // Gemini-specific getters.
        std::vector<HandleType> get_unshifted() override
        {
            return {
                op,
                x_lo_y_hi,
                x_hi_z_1,
                y_lo_z_2,
                p_x_low_limbs,
                p_x_low_limbs_range_constraint_0,
                p_x_low_limbs_range_constraint_1,
                p_x_low_limbs_range_constraint_2,
                p_x_low_limbs_range_constraint_3,
                p_x_low_limbs_range_constraint_4,
                p_x_low_limbs_range_constraint_tail,
                p_x_high_limbs,
                p_x_high_limbs_range_constraint_0,
                p_x_high_limbs_range_constraint_1,
                p_x_high_limbs_range_constraint_2,
                p_x_high_limbs_range_constraint_3,
                p_x_high_limbs_range_constraint_4,
                p_x_high_limbs_range_constraint_tail,
                p_y_low_limbs,
                p_y_low_limbs_range_constraint_0,
                p_y_low_limbs_range_constraint_1,
                p_y_low_limbs_range_constraint_2,
                p_y_low_limbs_range_constraint_3,
                p_y_low_limbs_range_constraint_4,
                p_y_low_limbs_range_constraint_tail,
                p_y_high_limbs,
                p_y_high_limbs_range_constraint_0,
                p_y_high_limbs_range_constraint_1,
                p_y_high_limbs_range_constraint_2,
                p_y_high_limbs_range_constraint_3,
                p_y_high_limbs_range_constraint_4,
                p_y_high_limbs_range_constraint_tail,
                z_low_limbs,
                z_low_limbs_range_constraint_0,
                z_low_limbs_range_constraint_1,
                z_low_limbs_range_constraint_2,
                z_low_limbs_range_constraint_3,
                z_low_limbs_range_constraint_4,
                z_low_limbs_range_constraint_tail,
                z_high_limbs,
                z_high_limbs_range_constraint_0,
                z_high_limbs_range_constraint_1,
                z_high_limbs_range_constraint_2,
                z_high_limbs_range_constraint_3,
                z_high_limbs_range_constraint_4,
                z_high_limbs_range_constraint_tail,
                accumulators_binary_limbs_0,
                accumulators_binary_limbs_1,
                accumulators_binary_limbs_2,
                accumulators_binary_limbs_3,
                accumulator_low_limbs_range_constraint_0,
                accumulator_low_limbs_range_constraint_1,
                accumulator_low_limbs_range_constraint_2,
                accumulator_low_limbs_range_constraint_3,
                accumulator_low_limbs_range_constraint_4,
                accumulator_low_limbs_range_constraint_tail,
                accumulator_high_limbs_range_constraint_0,
                accumulator_high_limbs_range_constraint_1,
                accumulator_high_limbs_range_constraint_2,
                accumulator_high_limbs_range_constraint_3,
                accumulator_high_limbs_range_constraint_4,
                accumulator_high_limbs_range_constraint_tail,
                quotient_low_binary_limbs,
                quotient_high_binary_limbs,
                quotient_low_limbs_range_constraint_0,
                quotient_low_limbs_range_constraint_1,
                quotient_low_limbs_range_constraint_2,
                quotient_low_limbs_range_constraint_3,
                quotient_low_limbs_range_constraint_4,
                quotient_low_limbs_range_constraint_tail,
                quotient_high_limbs_range_constraint_0,
                quotient_high_limbs_range_constraint_1,
                quotient_high_limbs_range_constraint_2,
                quotient_high_limbs_range_constraint_3,
                quotient_high_limbs_range_constraint_4,
                quotient_high_limbs_range_constraint_tail,
                relation_wide_limbs,
                relation_wide_limbs_range_constraint_0,
                relation_wide_limbs_range_constraint_1,
                relation_wide_limbs_range_constraint_2,
                relation_wide_limbs_range_constraint_3,
                ordered_range_constraints_0,
                ordered_range_constraints_1,
                ordered_range_constraints_2,
                ordered_range_constraints_3,
                ordered_range_constraints_4,
                z_perm,

                lagrange_first,
                lagrange_last,
                lagrange_odd_in_minicircuit,
                lagrange_even_in_minicircuit,
                lagrange_second,
                lagrange_second_to_last_in_minicircuit,
                ordered_extra_range_constraints_numerator,

            };
        };
        std::vector<HandleType> get_to_be_shifted() override
        {
            return {
                x_lo_y_hi,
                x_hi_z_1,
                y_lo_z_2,
                p_x_low_limbs,
                p_x_low_limbs_range_constraint_0,
                p_x_low_limbs_range_constraint_1,
                p_x_low_limbs_range_constraint_2,
                p_x_low_limbs_range_constraint_3,
                p_x_low_limbs_range_constraint_4,
                p_x_low_limbs_range_constraint_tail,
                p_x_high_limbs,
                p_x_high_limbs_range_constraint_0,
                p_x_high_limbs_range_constraint_1,
                p_x_high_limbs_range_constraint_2,
                p_x_high_limbs_range_constraint_3,
                p_x_high_limbs_range_constraint_4,
                p_x_high_limbs_range_constraint_tail,
                p_y_low_limbs,
                p_y_low_limbs_range_constraint_0,
                p_y_low_limbs_range_constraint_1,
                p_y_low_limbs_range_constraint_2,
                p_y_low_limbs_range_constraint_3,
                p_y_low_limbs_range_constraint_4,
                p_y_low_limbs_range_constraint_tail,
                p_y_high_limbs,
                p_y_high_limbs_range_constraint_0,
                p_y_high_limbs_range_constraint_1,
                p_y_high_limbs_range_constraint_2,
                p_y_high_limbs_range_constraint_3,
                p_y_high_limbs_range_constraint_4,
                p_y_high_limbs_range_constraint_tail,
                z_low_limbs,
                z_low_limbs_range_constraint_0,
                z_low_limbs_range_constraint_1,
                z_low_limbs_range_constraint_2,
                z_low_limbs_range_constraint_3,
                z_low_limbs_range_constraint_4,
                z_low_limbs_range_constraint_tail,
                z_high_limbs,
                z_high_limbs_range_constraint_0,
                z_high_limbs_range_constraint_1,
                z_high_limbs_range_constraint_2,
                z_high_limbs_range_constraint_3,
                z_high_limbs_range_constraint_4,
                z_high_limbs_range_constraint_tail,
                accumulators_binary_limbs_0,
                accumulators_binary_limbs_1,
                accumulators_binary_limbs_2,
                accumulators_binary_limbs_3,
                accumulator_low_limbs_range_constraint_0,
                accumulator_low_limbs_range_constraint_1,
                accumulator_low_limbs_range_constraint_2,
                accumulator_low_limbs_range_constraint_3,
                accumulator_low_limbs_range_constraint_4,
                accumulator_low_limbs_range_constraint_tail,
                accumulator_high_limbs_range_constraint_0,
                accumulator_high_limbs_range_constraint_1,
                accumulator_high_limbs_range_constraint_2,
                accumulator_high_limbs_range_constraint_3,
                accumulator_high_limbs_range_constraint_4,
                accumulator_high_limbs_range_constraint_tail,
                quotient_low_binary_limbs,
                quotient_high_binary_limbs,
                quotient_low_limbs_range_constraint_0,
                quotient_low_limbs_range_constraint_1,
                quotient_low_limbs_range_constraint_2,
                quotient_low_limbs_range_constraint_3,
                quotient_low_limbs_range_constraint_4,
                quotient_low_limbs_range_constraint_tail,
                quotient_high_limbs_range_constraint_0,
                quotient_high_limbs_range_constraint_1,
                quotient_high_limbs_range_constraint_2,
                quotient_high_limbs_range_constraint_3,
                quotient_high_limbs_range_constraint_4,
                quotient_high_limbs_range_constraint_tail,
                relation_wide_limbs,
                relation_wide_limbs_range_constraint_0,
                relation_wide_limbs_range_constraint_1,
                relation_wide_limbs_range_constraint_2,
                relation_wide_limbs_range_constraint_3,
                ordered_range_constraints_0,
                ordered_range_constraints_1,
                ordered_range_constraints_2,
                ordered_range_constraints_3,
                ordered_range_constraints_4,

                z_perm,
            };
        };
        std::vector<HandleType> get_shifted() override
        {
            return {
                x_lo_y_hi_shift,
                x_hi_z_1_shift,
                y_lo_z_2_shift,
                p_x_low_limbs_shift,
                p_x_low_limbs_range_constraint_0_shift,
                p_x_low_limbs_range_constraint_1_shift,
                p_x_low_limbs_range_constraint_2_shift,
                p_x_low_limbs_range_constraint_3_shift,
                p_x_low_limbs_range_constraint_4_shift,
                p_x_low_limbs_range_constraint_tail_shift,
                p_x_high_limbs_shift,
                p_x_high_limbs_range_constraint_0_shift,
                p_x_high_limbs_range_constraint_1_shift,
                p_x_high_limbs_range_constraint_2_shift,
                p_x_high_limbs_range_constraint_3_shift,
                p_x_high_limbs_range_constraint_4_shift,
                p_x_high_limbs_range_constraint_tail_shift,
                p_y_low_limbs_shift,
                p_y_low_limbs_range_constraint_0_shift,
                p_y_low_limbs_range_constraint_1_shift,
                p_y_low_limbs_range_constraint_2_shift,
                p_y_low_limbs_range_constraint_3_shift,
                p_y_low_limbs_range_constraint_4_shift,
                p_y_low_limbs_range_constraint_tail_shift,
                p_y_high_limbs_shift,
                p_y_high_limbs_range_constraint_0_shift,
                p_y_high_limbs_range_constraint_1_shift,
                p_y_high_limbs_range_constraint_2_shift,
                p_y_high_limbs_range_constraint_3_shift,
                p_y_high_limbs_range_constraint_4_shift,
                p_y_high_limbs_range_constraint_tail_shift,
                z_low_limbs_shift,
                z_low_limbs_range_constraint_0_shift,
                z_low_limbs_range_constraint_1_shift,
                z_low_limbs_range_constraint_2_shift,
                z_low_limbs_range_constraint_3_shift,
                z_low_limbs_range_constraint_4_shift,
                z_low_limbs_range_constraint_tail_shift,
                z_high_limbs_shift,
                z_high_limbs_range_constraint_0_shift,
                z_high_limbs_range_constraint_1_shift,
                z_high_limbs_range_constraint_2_shift,
                z_high_limbs_range_constraint_3_shift,
                z_high_limbs_range_constraint_4_shift,
                z_high_limbs_range_constraint_tail_shift,
                accumulators_binary_limbs_0_shift,
                accumulators_binary_limbs_1_shift,
                accumulators_binary_limbs_2_shift,
                accumulators_binary_limbs_3_shift,
                accumulator_low_limbs_range_constraint_0_shift,
                accumulator_low_limbs_range_constraint_1_shift,
                accumulator_low_limbs_range_constraint_2_shift,
                accumulator_low_limbs_range_constraint_3_shift,
                accumulator_low_limbs_range_constraint_4_shift,
                accumulator_low_limbs_range_constraint_tail_shift,
                accumulator_high_limbs_range_constraint_0_shift,
                accumulator_high_limbs_range_constraint_1_shift,
                accumulator_high_limbs_range_constraint_2_shift,
                accumulator_high_limbs_range_constraint_3_shift,
                accumulator_high_limbs_range_constraint_4_shift,
                accumulator_high_limbs_range_constraint_tail_shift,
                quotient_low_binary_limbs_shift,
                quotient_high_binary_limbs_shift,
                quotient_low_limbs_range_constraint_0_shift,
                quotient_low_limbs_range_constraint_1_shift,
                quotient_low_limbs_range_constraint_2_shift,
                quotient_low_limbs_range_constraint_3_shift,
                quotient_low_limbs_range_constraint_4_shift,
                quotient_low_limbs_range_constraint_tail_shift,
                quotient_high_limbs_range_constraint_0_shift,
                quotient_high_limbs_range_constraint_1_shift,
                quotient_high_limbs_range_constraint_2_shift,
                quotient_high_limbs_range_constraint_3_shift,
                quotient_high_limbs_range_constraint_4_shift,
                quotient_high_limbs_range_constraint_tail_shift,
                relation_wide_limbs_shift,
                relation_wide_limbs_range_constraint_0_shift,
                relation_wide_limbs_range_constraint_1_shift,
                relation_wide_limbs_range_constraint_2_shift,
                relation_wide_limbs_range_constraint_3_shift,
                ordered_range_constraints_0_shift,
                ordered_range_constraints_1_shift,
                ordered_range_constraints_2_shift,
                ordered_range_constraints_3_shift,
                ordered_range_constraints_4_shift,

                z_perm_shift,
            };
        };

        /**
         * @brief Polynomials/commitments, that can be constructed only after the r challenge has been received from
         * gemini
         *
         * @return std::vector<HandleType>
         */
        std::vector<HandleType> get_special() { return get_concatenated_constraints(); }

        std::vector<HandleType> get_unshifted_then_shifted_then_special()
        {
            std::vector<HandleType> result{ get_unshifted() };
            std::vector<HandleType> shifted{ get_shifted() };
            std::vector<HandleType> special{ get_special() };
            result.insert(result.end(), shifted.begin(), shifted.end());
            result.insert(result.end(), special.begin(), special.end());
            return result;
        }

        friend std::ostream& operator<<(std::ostream& os, const AllEntities& a)
        {
            os << "{ ";
            std::ios_base::fmtflags f(os.flags());
            for (size_t i = 0; i < NUM_ALL_ENTITIES - 1; i++) {
                os << "e[" << std::setw(2) << i << "] = " << (a._data[i]) << ",\n";
            }
            os << "e[" << std::setw(2) << (NUM_ALL_ENTITIES - 1) << "] = " << std::get<NUM_ALL_ENTITIES - 1>(a._data)
               << " }";

            os.flags(f);
            return os;
        }
    };

  public:
    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve
     * inherit from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                          WitnessEntities<Polynomial, PolynomialHandle>> {
      public:
        BF batching_challenge_v = { 0 };
        BF evaluation_input_x = { 0 };
        ProvingKey() = default;

        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                 WitnessEntities<Polynomial, PolynomialHandle>>;
        using Base::Base;

        ProvingKey(const size_t circuit_size)
            : ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                          WitnessEntities<Polynomial, PolynomialHandle>>(circuit_size, 0)

            , batching_challenge_v(0)
            , evaluation_input_x(0)

        {}
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to
     * resolve that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for
     * portability of our circuits.
     */
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment, CommitmentHandle>>;

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
     * @brief A container for the prover polynomials handles; only stores spans.
     */
    class ProverPolynomials : public AllEntities<PolynomialHandle, PolynomialHandle> {
      public:
        [[nodiscard]] size_t get_polynomial_size() const { return this->op.size(); }
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        [[nodiscard]] AllValues get_row(size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.pointer_view(), this->pointer_view())) {
                *result_field = (*polynomial)[row_idx];
            }
            return result;
        }
    };

    /**
     * @brief A container for easier mapping of polynomials
     */
    using ProverPolynomialIds = AllEntities<size_t, size_t>;

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
        AllValues get_row(const size_t row_idx) const
        {
            AllValues result;
            size_t column_idx = 0; // // TODO(https://github.com/AztecProtocol/barretenberg/issues/391) zip
            for (auto& column : this->_data) {
                result[column_idx] = column[row_idx];
                column_idx++;
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
            for (auto& poly : this->_data) {
                poly = Polynomial(circuit_size / 2);
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
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly
     * needed. It has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string, std::string> {
      public:
        CommitmentLabels()
        {

            this->op = "OP";
            this->x_lo_y_hi = "X_LO_Y_HI";
            this->x_hi_z_1 = "X_HI_Z_1";
            this->y_lo_z_2 = "Y_LO_Z_2";
            this->p_x_low_limbs = "P_X_LOW_LIMBS";
            this->p_x_low_limbs_range_constraint_0 = "P_X_LOW_LIMBS_RANGE_CONSTRAINT_0";
            this->p_x_low_limbs_range_constraint_1 = "P_X_LOW_LIMBS_RANGE_CONSTRAINT_1";
            this->p_x_low_limbs_range_constraint_2 = "P_X_LOW_LIMBS_RANGE_CONSTRAINT_2";
            this->p_x_low_limbs_range_constraint_3 = "P_X_LOW_LIMBS_RANGE_CONSTRAINT_3";
            this->p_x_low_limbs_range_constraint_4 = "P_X_LOW_LIMBS_RANGE_CONSTRAINT_4";
            this->p_x_low_limbs_range_constraint_tail = "P_X_LOW_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->p_x_high_limbs = "P_X_HIGH_LIMBS";
            this->p_x_high_limbs_range_constraint_0 = "P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0";
            this->p_x_high_limbs_range_constraint_1 = "P_X_HIGH_LIMBS_RANGE_CONSTRAINT_1";
            this->p_x_high_limbs_range_constraint_2 = "P_X_HIGH_LIMBS_RANGE_CONSTRAINT_2";
            this->p_x_high_limbs_range_constraint_3 = "P_X_HIGH_LIMBS_RANGE_CONSTRAINT_3";
            this->p_x_high_limbs_range_constraint_4 = "P_X_HIGH_LIMBS_RANGE_CONSTRAINT_4";
            this->p_x_high_limbs_range_constraint_tail = "P_X_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->p_y_low_limbs = "P_Y_LOW_LIMBS";
            this->p_y_low_limbs_range_constraint_0 = "P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0";
            this->p_y_low_limbs_range_constraint_1 = "P_Y_LOW_LIMBS_RANGE_CONSTRAINT_1";
            this->p_y_low_limbs_range_constraint_2 = "P_Y_LOW_LIMBS_RANGE_CONSTRAINT_2";
            this->p_y_low_limbs_range_constraint_3 = "P_Y_LOW_LIMBS_RANGE_CONSTRAINT_3";
            this->p_y_low_limbs_range_constraint_4 = "P_Y_LOW_LIMBS_RANGE_CONSTRAINT_4";
            this->p_y_low_limbs_range_constraint_tail = "P_Y_LOW_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->p_y_high_limbs = "P_Y_HIGH_LIMBS";
            this->p_y_high_limbs_range_constraint_0 = "P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0";
            this->p_y_high_limbs_range_constraint_1 = "P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_1";
            this->p_y_high_limbs_range_constraint_2 = "P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_2";
            this->p_y_high_limbs_range_constraint_3 = "P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_3";
            this->p_y_high_limbs_range_constraint_4 = "P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_4";
            this->p_y_high_limbs_range_constraint_tail = "P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->z_low_limbs = "Z_LOw_LIMBS";
            this->z_low_limbs_range_constraint_0 = "Z_LOW_LIMBS_RANGE_CONSTRAINT_0";
            this->z_low_limbs_range_constraint_1 = "Z_LOW_LIMBS_RANGE_CONSTRAINT_1";
            this->z_low_limbs_range_constraint_2 = "Z_LOW_LIMBS_RANGE_CONSTRAINT_2";
            this->z_low_limbs_range_constraint_3 = "Z_LOW_LIMBS_RANGE_CONSTRAINT_3";
            this->z_low_limbs_range_constraint_4 = "Z_LOW_LIMBS_RANGE_CONSTRAINT_4";
            this->z_low_limbs_range_constraint_tail = "Z_LOW_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->z_high_limbs = "Z_HIGH_LIMBS";
            this->z_high_limbs_range_constraint_0 = "Z_HIGH_LIMBS_RANGE_CONSTRAINT_0";
            this->z_high_limbs_range_constraint_1 = "Z_HIGH_LIMBS_RANGE_CONSTRAINT_1";
            this->z_high_limbs_range_constraint_2 = "Z_HIGH_LIMBS_RANGE_CONSTRAINT_2";
            this->z_high_limbs_range_constraint_3 = "Z_HIGH_LIMBS_RANGE_CONSTRAINT_3";
            this->z_high_limbs_range_constraint_4 = "Z_HIGH_LIMBS_RANGE_CONSTRAINT_4";
            this->z_high_limbs_range_constraint_tail = "Z_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->accumulators_binary_limbs_0 = "ACCUMULATORS_BINARY_LIMBS_0";
            this->accumulators_binary_limbs_1 = "ACCUMULATORS_BINARY_LIMBS_1";
            this->accumulators_binary_limbs_2 = "ACCUMULATORS_BINARY_LIMBS_2";
            this->accumulators_binary_limbs_3 = "ACCUMULATORS_BINARY_LIMBS_3";
            this->accumulator_low_limbs_range_constraint_0 = "ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0";
            this->accumulator_low_limbs_range_constraint_1 = "ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_1";
            this->accumulator_low_limbs_range_constraint_2 = "ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_2";
            this->accumulator_low_limbs_range_constraint_3 = "ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_3";
            this->accumulator_low_limbs_range_constraint_4 = "ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_4";
            this->accumulator_low_limbs_range_constraint_tail = "ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->accumulator_high_limbs_range_constraint_0 = "ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0";
            this->accumulator_high_limbs_range_constraint_1 = "ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_1";
            this->accumulator_high_limbs_range_constraint_2 = "ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_2";
            this->accumulator_high_limbs_range_constraint_3 = "ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_3";
            this->accumulator_high_limbs_range_constraint_4 = "ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_4";
            this->accumulator_high_limbs_range_constraint_tail = "ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->quotient_low_binary_limbs = "QUOTIENT_LOW_BINARY_LIMBS";
            this->quotient_high_binary_limbs = "QUOTIENT_HIGH_BINARY_LIMBS";
            this->quotient_low_limbs_range_constraint_0 = "QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_0";
            this->quotient_low_limbs_range_constraint_1 = "QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_1";
            this->quotient_low_limbs_range_constraint_2 = "QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_2";
            this->quotient_low_limbs_range_constraint_3 = "QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_3";
            this->quotient_low_limbs_range_constraint_4 = "QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_4";
            this->quotient_low_limbs_range_constraint_tail = "QUOTIENT_LOW_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->quotient_high_limbs_range_constraint_0 = "QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_0";
            this->quotient_high_limbs_range_constraint_1 = "QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_1";
            this->quotient_high_limbs_range_constraint_2 = "QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_2";
            this->quotient_high_limbs_range_constraint_3 = "QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_3";
            this->quotient_high_limbs_range_constraint_4 = "QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_4";
            this->quotient_high_limbs_range_constraint_tail = "QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL";
            this->relation_wide_limbs = "RELATION_WIDE_LIMBS";
            this->relation_wide_limbs_range_constraint_0 = "RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_0";
            this->relation_wide_limbs_range_constraint_1 = "RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_1";
            this->relation_wide_limbs_range_constraint_2 = "RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_2";
            this->relation_wide_limbs_range_constraint_3 = "RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_2";
            this->concatenated_range_constraints_0 = "CONCATENATED_RANGE_CONSTRAINTS_0";
            this->concatenated_range_constraints_1 = "CONCATENATED_RANGE_CONSTRAINTS_1";
            this->concatenated_range_constraints_2 = "CONCATENATED_RANGE_CONSTRAINTS_2";
            this->concatenated_range_constraints_3 = "CONCATENATED_RANGE_CONSTRAINTS_3";
            this->z_perm = "Z_PERM";
            // "__" are only used for debugging
            this->lagrange_first = "__LAGRANGE_FIRST";
            this->lagrange_last = "__LAGRANGE_LAST";
            this->lagrange_odd_in_minicircuit = "__LAGRANGE_ODD_IN_MINICIRCUIT";
            this->lagrange_even_in_minicircuit = "__LAGRANGE_EVEN_IN_MINICIRCUIT";
            this->lagrange_second = "__LAGRANGE_SECOND";
            this->lagrange_second_to_last_in_minicircuit = "__LAGRANGE_SECOND_TO_LAST_IN_MINICIRCUIT";
            this->ordered_extra_range_constraints_numerator = "__ORDERED_EXTRA_RANGE_CONSTRAINTS_NUMERATOR";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment, CommitmentHandle> {
      public:
        VerifierCommitments(std::shared_ptr<VerificationKey> verification_key,
                            [[maybe_unused]] const BaseTranscript<FF>& transcript)
        {
            static_cast<void>(transcript);
            this->lagrange_first = verification_key->lagrange_first;
            this->lagrange_last = verification_key->lagrange_last;
            this->lagrange_odd_in_minicircuit = verification_key->lagrange_odd_in_minicircuit;
            this->lagrange_even_in_minicircuit = verification_key->lagrange_even_in_minicircuit;
            this->lagrange_second = verification_key->lagrange_second;
            this->lagrange_second_to_last_in_minicircuit = verification_key->lagrange_second_to_last_in_minicircuit;
            this->ordered_extra_range_constraints_numerator =
                verification_key->ordered_extra_range_constraints_numerator;
        }
    };
};

using GoblinTranslator = GoblinTranslator_<2048>;

} // namespace proof_system::honk::flavor

namespace proof_system {

extern template class GoblinTranslatorPermutationRelationImpl<barretenberg::fr>;
extern template class GoblinTranslatorGenPermSortRelationImpl<barretenberg::fr>;
extern template class GoblinTranslatorOpcodeConstraintRelationImpl<barretenberg::fr>;
extern template class GoblinTranslatorAccumulatorTransferRelationImpl<barretenberg::fr>;
extern template class GoblinTranslatorDecompositionRelationImpl<barretenberg::fr>;
extern template class GoblinTranslatorNonNativeFieldRelationImpl<barretenberg::fr>;

DECLARE_SUMCHECK_RELATION_CLASS(GoblinTranslatorPermutationRelationImpl, honk::flavor::GoblinTranslator);
DECLARE_SUMCHECK_RELATION_CLASS(GoblinTranslatorGenPermSortRelationImpl, honk::flavor::GoblinTranslator);
DECLARE_SUMCHECK_RELATION_CLASS(GoblinTranslatorOpcodeConstraintRelationImpl, honk::flavor::GoblinTranslator);
DECLARE_SUMCHECK_RELATION_CLASS(GoblinTranslatorAccumulatorTransferRelationImpl, honk::flavor::GoblinTranslator);
DECLARE_SUMCHECK_RELATION_CLASS(GoblinTranslatorDecompositionRelationImpl, honk::flavor::GoblinTranslator);
DECLARE_SUMCHECK_RELATION_CLASS(GoblinTranslatorNonNativeFieldRelationImpl, honk::flavor::GoblinTranslator);

} // namespace proof_system
