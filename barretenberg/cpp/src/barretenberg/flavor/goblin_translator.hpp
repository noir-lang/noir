#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_translator_circuit_builder.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/translator_vm/translator_decomposition_relation.hpp"
#include "barretenberg/relations/translator_vm/translator_extra_relations.hpp"
#include "barretenberg/relations/translator_vm/translator_gen_perm_sort_relation.hpp"
#include "barretenberg/relations/translator_vm/translator_non_native_field_relation.hpp"
#include "barretenberg/relations/translator_vm/translator_permutation_relation.hpp"
#include "relation_definitions.hpp"

namespace bb::honk::flavor {

class GoblinTranslator {

  public:
    static constexpr size_t mini_circuit_size = 2048;
    using CircuitBuilder = GoblinTranslatorCircuitBuilder;
    using Curve = curve::BN254;
    using PCS = pcs::kzg::KZG<Curve>;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using CommitmentHandle = Curve::AffineElement;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;
    using FF = Curve::ScalarField;
    using BF = Curve::BaseField;
    using Polynomial = bb::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using RelationSeparator = FF;

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

    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We
    // often need containers of this size to hold related data, so we choose a name more agnostic than
    // `NUM_POLYNOMIALS`. Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 184;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 7;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 91;

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
    using SumcheckTupleOfTuplesOfUnivariates =
        std::tuple<typename GoblinTranslatorPermutationRelation<FF>::SumcheckTupleOfUnivariatesOverSubrelations,
                   typename GoblinTranslatorGenPermSortRelation<FF>::SumcheckTupleOfUnivariatesOverSubrelations,
                   typename GoblinTranslatorOpcodeConstraintRelation<FF>::SumcheckTupleOfUnivariatesOverSubrelations,
                   typename GoblinTranslatorAccumulatorTransferRelation<FF>::SumcheckTupleOfUnivariatesOverSubrelations,
                   typename GoblinTranslatorDecompositionRelation<FF>::SumcheckTupleOfUnivariatesOverSubrelations,
                   typename GoblinTranslatorNonNativeFieldRelation<FF>::SumcheckTupleOfUnivariatesOverSubrelations>;
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
                              lagrange_first, // column 0
                              lagrange_last,  // column 1
                              // TODO(#758): Check if one of these can be replaced by shifts
                              lagrange_odd_in_minicircuit,                // column 2
                              lagrange_even_in_minicircuit,               // column 3
                              lagrange_second,                            // column 4
                              lagrange_second_to_last_in_minicircuit,     // column 5
                              ordered_extra_range_constraints_numerator); // column 6
        RefVector<DataType> get_selectors() { return {}; };
        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
    };

    template <typename DataType> class ConcatenatedRangeConstraints {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              concatenated_range_constraints_0, // column 0
                              concatenated_range_constraints_1, // column 1
                              concatenated_range_constraints_2, // column 2
                              concatenated_range_constraints_3) // column 3
    };
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/790) dedupe with shifted?
    template <typename DataType> class WireToBeShiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              x_lo_y_hi,                                    // column 0
                              x_hi_z_1,                                     // column 1
                              y_lo_z_2,                                     // column 2
                              p_x_low_limbs,                                // column 3
                              p_x_low_limbs_range_constraint_0,             // column 4
                              p_x_low_limbs_range_constraint_1,             // column 5
                              p_x_low_limbs_range_constraint_2,             // column 6
                              p_x_low_limbs_range_constraint_3,             // column 7
                              p_x_low_limbs_range_constraint_4,             // column 8
                              p_x_low_limbs_range_constraint_tail,          // column 9
                              p_x_high_limbs,                               // column 10
                              p_x_high_limbs_range_constraint_0,            // column 11
                              p_x_high_limbs_range_constraint_1,            // column 12
                              p_x_high_limbs_range_constraint_2,            // column 13
                              p_x_high_limbs_range_constraint_3,            // column 14
                              p_x_high_limbs_range_constraint_4,            // column 15
                              p_x_high_limbs_range_constraint_tail,         // column 16
                              p_y_low_limbs,                                // column 17
                              p_y_low_limbs_range_constraint_0,             // column 18
                              p_y_low_limbs_range_constraint_1,             // column 19
                              p_y_low_limbs_range_constraint_2,             // column 20
                              p_y_low_limbs_range_constraint_3,             // column 21
                              p_y_low_limbs_range_constraint_4,             // column 22
                              p_y_low_limbs_range_constraint_tail,          // column 23
                              p_y_high_limbs,                               // column 24
                              p_y_high_limbs_range_constraint_0,            // column 25
                              p_y_high_limbs_range_constraint_1,            // column 26
                              p_y_high_limbs_range_constraint_2,            // column 27
                              p_y_high_limbs_range_constraint_3,            // column 28
                              p_y_high_limbs_range_constraint_4,            // column 29
                              p_y_high_limbs_range_constraint_tail,         // column 30
                              z_low_limbs,                                  // column 31
                              z_low_limbs_range_constraint_0,               // column 32
                              z_low_limbs_range_constraint_1,               // column 33
                              z_low_limbs_range_constraint_2,               // column 34
                              z_low_limbs_range_constraint_3,               // column 35
                              z_low_limbs_range_constraint_4,               // column 36
                              z_low_limbs_range_constraint_tail,            // column 37
                              z_high_limbs,                                 // column 38
                              z_high_limbs_range_constraint_0,              // column 39
                              z_high_limbs_range_constraint_1,              // column 40
                              z_high_limbs_range_constraint_2,              // column 41
                              z_high_limbs_range_constraint_3,              // column 42
                              z_high_limbs_range_constraint_4,              // column 43
                              z_high_limbs_range_constraint_tail,           // column 44
                              accumulators_binary_limbs_0,                  // column 45
                              accumulators_binary_limbs_1,                  // column 46
                              accumulators_binary_limbs_2,                  // column 47
                              accumulators_binary_limbs_3,                  // column 48
                              accumulator_low_limbs_range_constraint_0,     // column 49
                              accumulator_low_limbs_range_constraint_1,     // column 50
                              accumulator_low_limbs_range_constraint_2,     // column 51
                              accumulator_low_limbs_range_constraint_3,     // column 52
                              accumulator_low_limbs_range_constraint_4,     // column 53
                              accumulator_low_limbs_range_constraint_tail,  // column 54
                              accumulator_high_limbs_range_constraint_0,    // column 55
                              accumulator_high_limbs_range_constraint_1,    // column 56
                              accumulator_high_limbs_range_constraint_2,    // column 57
                              accumulator_high_limbs_range_constraint_3,    // column 58
                              accumulator_high_limbs_range_constraint_4,    // column 59
                              accumulator_high_limbs_range_constraint_tail, // column 60
                              quotient_low_binary_limbs,                    // column 61
                              quotient_high_binary_limbs,                   // column 62
                              quotient_low_limbs_range_constraint_0,        // column 63
                              quotient_low_limbs_range_constraint_1,        // column 64
                              quotient_low_limbs_range_constraint_2,        // column 65
                              quotient_low_limbs_range_constraint_3,        // column 66
                              quotient_low_limbs_range_constraint_4,        // column 67
                              quotient_low_limbs_range_constraint_tail,     // column 68
                              quotient_high_limbs_range_constraint_0,       // column 69
                              quotient_high_limbs_range_constraint_1,       // column 70
                              quotient_high_limbs_range_constraint_2,       // column 71
                              quotient_high_limbs_range_constraint_3,       // column 72
                              quotient_high_limbs_range_constraint_4,       // column 73
                              quotient_high_limbs_range_constraint_tail,    // column 74
                              relation_wide_limbs,                          // column 75
                              relation_wide_limbs_range_constraint_0,       // column 76
                              relation_wide_limbs_range_constraint_1,       // column 77
                              relation_wide_limbs_range_constraint_2,       // column 78
                              relation_wide_limbs_range_constraint_3,       // column 79
                              ordered_range_constraints_0,                  // column 80
                              ordered_range_constraints_1,                  // column 81
                              ordered_range_constraints_2,                  // column 82
                              ordered_range_constraints_3,                  // column 83
                              ordered_range_constraints_4);                 // column 84
    };
    template <typename DataType> class WireNonshiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              op // column 0
        );
    };
    template <typename DataType> class DerivedWitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              z_perm); // column 0
    };
    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     */
    template <typename DataType>
    class WitnessEntities : public WireNonshiftedEntities<DataType>,
                            public WireToBeShiftedEntities<DataType>,
                            public DerivedWitnessEntities<DataType>,
                            public ConcatenatedRangeConstraints<DataType> {
      public:
        DEFINE_COMPOUND_GET_ALL(WireNonshiftedEntities<DataType>,
                                WireToBeShiftedEntities<DataType>,
                                DerivedWitnessEntities<DataType>,
                                ConcatenatedRangeConstraints<DataType>)

        RefVector<DataType> get_wires()
        {
            return { this->op,
                     this->x_lo_y_hi,
                     this->x_hi_z_1,
                     this->y_lo_z_2,
                     this->p_x_low_limbs,
                     this->p_x_low_limbs_range_constraint_0,
                     this->p_x_low_limbs_range_constraint_1,
                     this->p_x_low_limbs_range_constraint_2,
                     this->p_x_low_limbs_range_constraint_3,
                     this->p_x_low_limbs_range_constraint_4,
                     this->p_x_low_limbs_range_constraint_tail,
                     this->p_x_high_limbs,
                     this->p_x_high_limbs_range_constraint_0,
                     this->p_x_high_limbs_range_constraint_1,
                     this->p_x_high_limbs_range_constraint_2,
                     this->p_x_high_limbs_range_constraint_3,
                     this->p_x_high_limbs_range_constraint_4,
                     this->p_x_high_limbs_range_constraint_tail,
                     this->p_y_low_limbs,
                     this->p_y_low_limbs_range_constraint_0,
                     this->p_y_low_limbs_range_constraint_1,
                     this->p_y_low_limbs_range_constraint_2,
                     this->p_y_low_limbs_range_constraint_3,
                     this->p_y_low_limbs_range_constraint_4,
                     this->p_y_low_limbs_range_constraint_tail,
                     this->p_y_high_limbs,
                     this->p_y_high_limbs_range_constraint_0,
                     this->p_y_high_limbs_range_constraint_1,
                     this->p_y_high_limbs_range_constraint_2,
                     this->p_y_high_limbs_range_constraint_3,
                     this->p_y_high_limbs_range_constraint_4,
                     this->p_y_high_limbs_range_constraint_tail,
                     this->z_low_limbs,
                     this->z_low_limbs_range_constraint_0,
                     this->z_low_limbs_range_constraint_1,
                     this->z_low_limbs_range_constraint_2,
                     this->z_low_limbs_range_constraint_3,
                     this->z_low_limbs_range_constraint_4,
                     this->z_low_limbs_range_constraint_tail,
                     this->z_high_limbs,
                     this->z_high_limbs_range_constraint_0,
                     this->z_high_limbs_range_constraint_1,
                     this->z_high_limbs_range_constraint_2,
                     this->z_high_limbs_range_constraint_3,
                     this->z_high_limbs_range_constraint_4,
                     this->z_high_limbs_range_constraint_tail,
                     this->accumulators_binary_limbs_0,
                     this->accumulators_binary_limbs_1,
                     this->accumulators_binary_limbs_2,
                     this->accumulators_binary_limbs_3,
                     this->accumulator_low_limbs_range_constraint_0,
                     this->accumulator_low_limbs_range_constraint_1,
                     this->accumulator_low_limbs_range_constraint_2,
                     this->accumulator_low_limbs_range_constraint_3,
                     this->accumulator_low_limbs_range_constraint_4,
                     this->accumulator_low_limbs_range_constraint_tail,
                     this->accumulator_high_limbs_range_constraint_0,
                     this->accumulator_high_limbs_range_constraint_1,
                     this->accumulator_high_limbs_range_constraint_2,
                     this->accumulator_high_limbs_range_constraint_3,
                     this->accumulator_high_limbs_range_constraint_4,
                     this->accumulator_high_limbs_range_constraint_tail,
                     this->quotient_low_binary_limbs,
                     this->quotient_high_binary_limbs,
                     this->quotient_low_limbs_range_constraint_0,
                     this->quotient_low_limbs_range_constraint_1,
                     this->quotient_low_limbs_range_constraint_2,
                     this->quotient_low_limbs_range_constraint_3,
                     this->quotient_low_limbs_range_constraint_4,
                     this->quotient_low_limbs_range_constraint_tail,
                     this->quotient_high_limbs_range_constraint_0,
                     this->quotient_high_limbs_range_constraint_1,
                     this->quotient_high_limbs_range_constraint_2,
                     this->quotient_high_limbs_range_constraint_3,
                     this->quotient_high_limbs_range_constraint_4,
                     this->quotient_high_limbs_range_constraint_tail,
                     this->relation_wide_limbs,
                     this->relation_wide_limbs_range_constraint_0,
                     this->relation_wide_limbs_range_constraint_1,
                     this->relation_wide_limbs_range_constraint_2,
                     this->relation_wide_limbs_range_constraint_3,
                     this->ordered_range_constraints_0,
                     this->ordered_range_constraints_1,
                     this->ordered_range_constraints_2,
                     this->ordered_range_constraints_3,
                     this->ordered_range_constraints_4 };
        };

        // everything but ConcatenatedRangeConstraints
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/810)
        RefVector<DataType> get_unshifted_wires()
        {
            return concatenate(WireNonshiftedEntities<DataType>::get_all(),
                               WireToBeShiftedEntities<DataType>::get_all(),
                               DerivedWitnessEntities<DataType>::get_all());
        }
        // everything but ConcatenatedRangeConstraints
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/810)
        std::vector<std::string> get_unshifted_wire_labels()
        {
            return concatenate(WireNonshiftedEntities<DataType>::get_labels(),
                               WireToBeShiftedEntities<DataType>::get_labels(),
                               DerivedWitnessEntities<DataType>::get_labels());
        }
        RefVector<DataType> get_to_be_shifted()
        {
            return concatenate(WireToBeShiftedEntities<DataType>::get_all(),
                               DerivedWitnessEntities<DataType>::get_all());
        };

        /**
         * @brief Get the polynomials that need to be constructed from other polynomials by concatenation
         *
         * @return RefVector<DataType>
         */
        auto get_concatenated_constraints() { return ConcatenatedRangeConstraints<DataType>::get_all(); }

        /**
         * @brief Get the polynomials that are concatenated for the permutation relation
         *
         * @return std::vector<RefVector<DataType>>
         */
        std::vector<RefVector<DataType>> get_concatenation_groups()
        {
            return {
                {
                    this->p_x_low_limbs_range_constraint_0,
                    this->p_x_low_limbs_range_constraint_1,
                    this->p_x_low_limbs_range_constraint_2,
                    this->p_x_low_limbs_range_constraint_3,
                    this->p_x_low_limbs_range_constraint_4,
                    this->p_x_low_limbs_range_constraint_tail,
                    this->p_x_high_limbs_range_constraint_0,
                    this->p_x_high_limbs_range_constraint_1,
                    this->p_x_high_limbs_range_constraint_2,
                    this->p_x_high_limbs_range_constraint_3,
                    this->p_x_high_limbs_range_constraint_4,
                    this->p_x_high_limbs_range_constraint_tail,
                    this->p_y_low_limbs_range_constraint_0,
                    this->p_y_low_limbs_range_constraint_1,
                    this->p_y_low_limbs_range_constraint_2,
                    this->p_y_low_limbs_range_constraint_3,
                },
                {
                    this->p_y_low_limbs_range_constraint_4,
                    this->p_y_low_limbs_range_constraint_tail,
                    this->p_y_high_limbs_range_constraint_0,
                    this->p_y_high_limbs_range_constraint_1,
                    this->p_y_high_limbs_range_constraint_2,
                    this->p_y_high_limbs_range_constraint_3,
                    this->p_y_high_limbs_range_constraint_4,
                    this->p_y_high_limbs_range_constraint_tail,
                    this->z_low_limbs_range_constraint_0,
                    this->z_low_limbs_range_constraint_1,
                    this->z_low_limbs_range_constraint_2,
                    this->z_low_limbs_range_constraint_3,
                    this->z_low_limbs_range_constraint_4,
                    this->z_low_limbs_range_constraint_tail,
                    this->z_high_limbs_range_constraint_0,
                    this->z_high_limbs_range_constraint_1,
                },
                {
                    this->z_high_limbs_range_constraint_2,
                    this->z_high_limbs_range_constraint_3,
                    this->z_high_limbs_range_constraint_4,
                    this->z_high_limbs_range_constraint_tail,
                    this->accumulator_low_limbs_range_constraint_0,
                    this->accumulator_low_limbs_range_constraint_1,
                    this->accumulator_low_limbs_range_constraint_2,
                    this->accumulator_low_limbs_range_constraint_3,
                    this->accumulator_low_limbs_range_constraint_4,
                    this->accumulator_low_limbs_range_constraint_tail,
                    this->accumulator_high_limbs_range_constraint_0,
                    this->accumulator_high_limbs_range_constraint_1,
                    this->accumulator_high_limbs_range_constraint_2,
                    this->accumulator_high_limbs_range_constraint_3,
                    this->accumulator_high_limbs_range_constraint_4,
                    this->accumulator_high_limbs_range_constraint_tail,
                },
                {
                    this->quotient_low_limbs_range_constraint_0,
                    this->quotient_low_limbs_range_constraint_1,
                    this->quotient_low_limbs_range_constraint_2,
                    this->quotient_low_limbs_range_constraint_3,
                    this->quotient_low_limbs_range_constraint_4,
                    this->quotient_low_limbs_range_constraint_tail,
                    this->quotient_high_limbs_range_constraint_0,
                    this->quotient_high_limbs_range_constraint_1,
                    this->quotient_high_limbs_range_constraint_2,
                    this->quotient_high_limbs_range_constraint_3,
                    this->quotient_high_limbs_range_constraint_4,
                    this->quotient_high_limbs_range_constraint_tail,
                    this->relation_wide_limbs_range_constraint_0,
                    this->relation_wide_limbs_range_constraint_1,
                    this->relation_wide_limbs_range_constraint_2,
                    this->relation_wide_limbs_range_constraint_3,
                },
            };
        };
    };

    /**
     * @brief Represents polynomials shifted by 1 or their evaluations, defined relative to WireToBeShiftedEntities.
     */
    template <typename DataType> class ShiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              x_lo_y_hi_shift,                                    // column 0
                              x_hi_z_1_shift,                                     // column 1
                              y_lo_z_2_shift,                                     // column 2
                              p_x_low_limbs_shift,                                // column 3
                              p_x_low_limbs_range_constraint_0_shift,             // column 4
                              p_x_low_limbs_range_constraint_1_shift,             // column 5
                              p_x_low_limbs_range_constraint_2_shift,             // column 6
                              p_x_low_limbs_range_constraint_3_shift,             // column 7
                              p_x_low_limbs_range_constraint_4_shift,             // column 8
                              p_x_low_limbs_range_constraint_tail_shift,          // column 9
                              p_x_high_limbs_shift,                               // column 10
                              p_x_high_limbs_range_constraint_0_shift,            // column 11
                              p_x_high_limbs_range_constraint_1_shift,            // column 12
                              p_x_high_limbs_range_constraint_2_shift,            // column 13
                              p_x_high_limbs_range_constraint_3_shift,            // column 14
                              p_x_high_limbs_range_constraint_4_shift,            // column 15
                              p_x_high_limbs_range_constraint_tail_shift,         // column 16
                              p_y_low_limbs_shift,                                // column 17
                              p_y_low_limbs_range_constraint_0_shift,             // column 18
                              p_y_low_limbs_range_constraint_1_shift,             // column 19
                              p_y_low_limbs_range_constraint_2_shift,             // column 20
                              p_y_low_limbs_range_constraint_3_shift,             // column 21
                              p_y_low_limbs_range_constraint_4_shift,             // column 22
                              p_y_low_limbs_range_constraint_tail_shift,          // column 23
                              p_y_high_limbs_shift,                               // column 24
                              p_y_high_limbs_range_constraint_0_shift,            // column 25
                              p_y_high_limbs_range_constraint_1_shift,            // column 26
                              p_y_high_limbs_range_constraint_2_shift,            // column 27
                              p_y_high_limbs_range_constraint_3_shift,            // column 28
                              p_y_high_limbs_range_constraint_4_shift,            // column 29
                              p_y_high_limbs_range_constraint_tail_shift,         // column 30
                              z_low_limbs_shift,                                  // column 31
                              z_low_limbs_range_constraint_0_shift,               // column 32
                              z_low_limbs_range_constraint_1_shift,               // column 33
                              z_low_limbs_range_constraint_2_shift,               // column 34
                              z_low_limbs_range_constraint_3_shift,               // column 35
                              z_low_limbs_range_constraint_4_shift,               // column 36
                              z_low_limbs_range_constraint_tail_shift,            // column 37
                              z_high_limbs_shift,                                 // column 38
                              z_high_limbs_range_constraint_0_shift,              // column 39
                              z_high_limbs_range_constraint_1_shift,              // column 40
                              z_high_limbs_range_constraint_2_shift,              // column 41
                              z_high_limbs_range_constraint_3_shift,              // column 42
                              z_high_limbs_range_constraint_4_shift,              // column 43
                              z_high_limbs_range_constraint_tail_shift,           // column 44
                              accumulators_binary_limbs_0_shift,                  // column 45
                              accumulators_binary_limbs_1_shift,                  // column 46
                              accumulators_binary_limbs_2_shift,                  // column 47
                              accumulators_binary_limbs_3_shift,                  // column 48
                              accumulator_low_limbs_range_constraint_0_shift,     // column 49
                              accumulator_low_limbs_range_constraint_1_shift,     // column 50
                              accumulator_low_limbs_range_constraint_2_shift,     // column 51
                              accumulator_low_limbs_range_constraint_3_shift,     // column 52
                              accumulator_low_limbs_range_constraint_4_shift,     // column 53
                              accumulator_low_limbs_range_constraint_tail_shift,  // column 54
                              accumulator_high_limbs_range_constraint_0_shift,    // column 55
                              accumulator_high_limbs_range_constraint_1_shift,    // column 56
                              accumulator_high_limbs_range_constraint_2_shift,    // column 57
                              accumulator_high_limbs_range_constraint_3_shift,    // column 58
                              accumulator_high_limbs_range_constraint_4_shift,    // column 59
                              accumulator_high_limbs_range_constraint_tail_shift, // column 60
                              quotient_low_binary_limbs_shift,                    // column 61
                              quotient_high_binary_limbs_shift,                   // column 62
                              quotient_low_limbs_range_constraint_0_shift,        // column 63
                              quotient_low_limbs_range_constraint_1_shift,        // column 64
                              quotient_low_limbs_range_constraint_2_shift,        // column 65
                              quotient_low_limbs_range_constraint_3_shift,        // column 66
                              quotient_low_limbs_range_constraint_4_shift,        // column 67
                              quotient_low_limbs_range_constraint_tail_shift,     // column 68
                              quotient_high_limbs_range_constraint_0_shift,       // column 69
                              quotient_high_limbs_range_constraint_1_shift,       // column 70
                              quotient_high_limbs_range_constraint_2_shift,       // column 71
                              quotient_high_limbs_range_constraint_3_shift,       // column 72
                              quotient_high_limbs_range_constraint_4_shift,       // column 73
                              quotient_high_limbs_range_constraint_tail_shift,    // column 74
                              relation_wide_limbs_shift,                          // column 75
                              relation_wide_limbs_range_constraint_0_shift,       // column 76
                              relation_wide_limbs_range_constraint_1_shift,       // column 77
                              relation_wide_limbs_range_constraint_2_shift,       // column 78
                              relation_wide_limbs_range_constraint_3_shift,       // column 79
                              ordered_range_constraints_0_shift,                  // column 80
                              ordered_range_constraints_1_shift,                  // column 81
                              ordered_range_constraints_2_shift,                  // column 82
                              ordered_range_constraints_3_shift,                  // column 83
                              ordered_range_constraints_4_shift,                  // column 84
                              z_perm_shift)                                       // column 85
    };
    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + ShiftedEntities.
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
        RefVector<DataType> get_wires()
        {
            return { this->op,
                     this->x_lo_y_hi,
                     this->x_hi_z_1,
                     this->y_lo_z_2,
                     this->p_x_low_limbs,
                     this->p_x_low_limbs_range_constraint_0,
                     this->p_x_low_limbs_range_constraint_1,
                     this->p_x_low_limbs_range_constraint_2,
                     this->p_x_low_limbs_range_constraint_3,
                     this->p_x_low_limbs_range_constraint_4,
                     this->p_x_low_limbs_range_constraint_tail,
                     this->p_x_high_limbs,
                     this->p_x_high_limbs_range_constraint_0,
                     this->p_x_high_limbs_range_constraint_1,
                     this->p_x_high_limbs_range_constraint_2,
                     this->p_x_high_limbs_range_constraint_3,
                     this->p_x_high_limbs_range_constraint_4,
                     this->p_x_high_limbs_range_constraint_tail,
                     this->p_y_low_limbs,
                     this->p_y_low_limbs_range_constraint_0,
                     this->p_y_low_limbs_range_constraint_1,
                     this->p_y_low_limbs_range_constraint_2,
                     this->p_y_low_limbs_range_constraint_3,
                     this->p_y_low_limbs_range_constraint_4,
                     this->p_y_low_limbs_range_constraint_tail,
                     this->p_y_high_limbs,
                     this->p_y_high_limbs_range_constraint_0,
                     this->p_y_high_limbs_range_constraint_1,
                     this->p_y_high_limbs_range_constraint_2,
                     this->p_y_high_limbs_range_constraint_3,
                     this->p_y_high_limbs_range_constraint_4,
                     this->p_y_high_limbs_range_constraint_tail,
                     this->z_low_limbs,
                     this->z_low_limbs_range_constraint_0,
                     this->z_low_limbs_range_constraint_1,
                     this->z_low_limbs_range_constraint_2,
                     this->z_low_limbs_range_constraint_3,
                     this->z_low_limbs_range_constraint_4,
                     this->z_low_limbs_range_constraint_tail,
                     this->z_high_limbs,
                     this->z_high_limbs_range_constraint_0,
                     this->z_high_limbs_range_constraint_1,
                     this->z_high_limbs_range_constraint_2,
                     this->z_high_limbs_range_constraint_3,
                     this->z_high_limbs_range_constraint_4,
                     this->z_high_limbs_range_constraint_tail,
                     this->accumulators_binary_limbs_0,
                     this->accumulators_binary_limbs_1,
                     this->accumulators_binary_limbs_2,
                     this->accumulators_binary_limbs_3,
                     this->accumulator_low_limbs_range_constraint_0,
                     this->accumulator_low_limbs_range_constraint_1,
                     this->accumulator_low_limbs_range_constraint_2,
                     this->accumulator_low_limbs_range_constraint_3,
                     this->accumulator_low_limbs_range_constraint_4,
                     this->accumulator_low_limbs_range_constraint_tail,
                     this->accumulator_high_limbs_range_constraint_0,
                     this->accumulator_high_limbs_range_constraint_1,
                     this->accumulator_high_limbs_range_constraint_2,
                     this->accumulator_high_limbs_range_constraint_3,
                     this->accumulator_high_limbs_range_constraint_4,
                     this->accumulator_high_limbs_range_constraint_tail,
                     this->quotient_low_binary_limbs,
                     this->quotient_high_binary_limbs,
                     this->quotient_low_limbs_range_constraint_0,
                     this->quotient_low_limbs_range_constraint_1,
                     this->quotient_low_limbs_range_constraint_2,
                     this->quotient_low_limbs_range_constraint_3,
                     this->quotient_low_limbs_range_constraint_4,
                     this->quotient_low_limbs_range_constraint_tail,
                     this->quotient_high_limbs_range_constraint_0,
                     this->quotient_high_limbs_range_constraint_1,
                     this->quotient_high_limbs_range_constraint_2,
                     this->quotient_high_limbs_range_constraint_3,
                     this->quotient_high_limbs_range_constraint_4,
                     this->quotient_high_limbs_range_constraint_tail,
                     this->relation_wide_limbs,
                     this->relation_wide_limbs_range_constraint_0,
                     this->relation_wide_limbs_range_constraint_1,
                     this->relation_wide_limbs_range_constraint_2,
                     this->relation_wide_limbs_range_constraint_3,
                     this->ordered_range_constraints_0,
                     this->ordered_range_constraints_1,
                     this->ordered_range_constraints_2,
                     this->ordered_range_constraints_3,
                     this->ordered_range_constraints_4 };
        }

        DEFINE_COMPOUND_GET_ALL(PrecomputedEntities<DataType>, WitnessEntities<DataType>, ShiftedEntities<DataType>)
        /**
         * @brief Get the polynomials that are concatenated for the permutation relation
         *
         * @return std::vector<RefVector<DataType>>
         */
        std::vector<RefVector<DataType>> get_concatenation_groups()
        {
            return {
                {
                    this->p_x_low_limbs_range_constraint_0,
                    this->p_x_low_limbs_range_constraint_1,
                    this->p_x_low_limbs_range_constraint_2,
                    this->p_x_low_limbs_range_constraint_3,
                    this->p_x_low_limbs_range_constraint_4,
                    this->p_x_low_limbs_range_constraint_tail,
                    this->p_x_high_limbs_range_constraint_0,
                    this->p_x_high_limbs_range_constraint_1,
                    this->p_x_high_limbs_range_constraint_2,
                    this->p_x_high_limbs_range_constraint_3,
                    this->p_x_high_limbs_range_constraint_4,
                    this->p_x_high_limbs_range_constraint_tail,
                    this->p_y_low_limbs_range_constraint_0,
                    this->p_y_low_limbs_range_constraint_1,
                    this->p_y_low_limbs_range_constraint_2,
                    this->p_y_low_limbs_range_constraint_3,
                },
                {
                    this->p_y_low_limbs_range_constraint_4,
                    this->p_y_low_limbs_range_constraint_tail,
                    this->p_y_high_limbs_range_constraint_0,
                    this->p_y_high_limbs_range_constraint_1,
                    this->p_y_high_limbs_range_constraint_2,
                    this->p_y_high_limbs_range_constraint_3,
                    this->p_y_high_limbs_range_constraint_4,
                    this->p_y_high_limbs_range_constraint_tail,
                    this->z_low_limbs_range_constraint_0,
                    this->z_low_limbs_range_constraint_1,
                    this->z_low_limbs_range_constraint_2,
                    this->z_low_limbs_range_constraint_3,
                    this->z_low_limbs_range_constraint_4,
                    this->z_low_limbs_range_constraint_tail,
                    this->z_high_limbs_range_constraint_0,
                    this->z_high_limbs_range_constraint_1,
                },
                {
                    this->z_high_limbs_range_constraint_2,
                    this->z_high_limbs_range_constraint_3,
                    this->z_high_limbs_range_constraint_4,
                    this->z_high_limbs_range_constraint_tail,
                    this->accumulator_low_limbs_range_constraint_0,
                    this->accumulator_low_limbs_range_constraint_1,
                    this->accumulator_low_limbs_range_constraint_2,
                    this->accumulator_low_limbs_range_constraint_3,
                    this->accumulator_low_limbs_range_constraint_4,
                    this->accumulator_low_limbs_range_constraint_tail,
                    this->accumulator_high_limbs_range_constraint_0,
                    this->accumulator_high_limbs_range_constraint_1,
                    this->accumulator_high_limbs_range_constraint_2,
                    this->accumulator_high_limbs_range_constraint_3,
                    this->accumulator_high_limbs_range_constraint_4,
                    this->accumulator_high_limbs_range_constraint_tail,
                },
                {
                    this->quotient_low_limbs_range_constraint_0,
                    this->quotient_low_limbs_range_constraint_1,
                    this->quotient_low_limbs_range_constraint_2,
                    this->quotient_low_limbs_range_constraint_3,
                    this->quotient_low_limbs_range_constraint_4,
                    this->quotient_low_limbs_range_constraint_tail,
                    this->quotient_high_limbs_range_constraint_0,
                    this->quotient_high_limbs_range_constraint_1,
                    this->quotient_high_limbs_range_constraint_2,
                    this->quotient_high_limbs_range_constraint_3,
                    this->quotient_high_limbs_range_constraint_4,
                    this->quotient_high_limbs_range_constraint_tail,
                    this->relation_wide_limbs_range_constraint_0,
                    this->relation_wide_limbs_range_constraint_1,
                    this->relation_wide_limbs_range_constraint_2,
                    this->relation_wide_limbs_range_constraint_3,
                },
            };
        }
        /**
         * @brief Get the polynomials that need to be constructed from other polynomials by concatenation
         *
         * @return RefVector<DataType>
         */
        RefVector<DataType> get_concatenated_constraints()
        {
            return ConcatenatedRangeConstraints<DataType>::get_all();
        };
        /**
         * @brief Get the polynomials from the grand product denominator
         *
         * @return RefVector<DataType>
         */
        RefVector<DataType> get_ordered_constraints()
        {
            return { this->ordered_range_constraints_0,
                     this->ordered_range_constraints_1,
                     this->ordered_range_constraints_2,
                     this->ordered_range_constraints_3,
                     this->ordered_range_constraints_4 };
        };

        // Gemini-specific getters.
        RefVector<DataType> get_unshifted()
        {
            return concatenate(PrecomputedEntities<DataType>::get_all(),
                               WitnessEntities<DataType>::get_unshifted_wires());
        }
        // get_to_be_shifted is inherited
        RefVector<DataType> get_shifted()
        {
            return { this->x_lo_y_hi_shift,
                     this->x_hi_z_1_shift,
                     this->y_lo_z_2_shift,
                     this->p_x_low_limbs_shift,
                     this->p_x_low_limbs_range_constraint_0_shift,
                     this->p_x_low_limbs_range_constraint_1_shift,
                     this->p_x_low_limbs_range_constraint_2_shift,
                     this->p_x_low_limbs_range_constraint_3_shift,
                     this->p_x_low_limbs_range_constraint_4_shift,
                     this->p_x_low_limbs_range_constraint_tail_shift,
                     this->p_x_high_limbs_shift,
                     this->p_x_high_limbs_range_constraint_0_shift,
                     this->p_x_high_limbs_range_constraint_1_shift,
                     this->p_x_high_limbs_range_constraint_2_shift,
                     this->p_x_high_limbs_range_constraint_3_shift,
                     this->p_x_high_limbs_range_constraint_4_shift,
                     this->p_x_high_limbs_range_constraint_tail_shift,
                     this->p_y_low_limbs_shift,
                     this->p_y_low_limbs_range_constraint_0_shift,
                     this->p_y_low_limbs_range_constraint_1_shift,
                     this->p_y_low_limbs_range_constraint_2_shift,
                     this->p_y_low_limbs_range_constraint_3_shift,
                     this->p_y_low_limbs_range_constraint_4_shift,
                     this->p_y_low_limbs_range_constraint_tail_shift,
                     this->p_y_high_limbs_shift,
                     this->p_y_high_limbs_range_constraint_0_shift,
                     this->p_y_high_limbs_range_constraint_1_shift,
                     this->p_y_high_limbs_range_constraint_2_shift,
                     this->p_y_high_limbs_range_constraint_3_shift,
                     this->p_y_high_limbs_range_constraint_4_shift,
                     this->p_y_high_limbs_range_constraint_tail_shift,
                     this->z_low_limbs_shift,
                     this->z_low_limbs_range_constraint_0_shift,
                     this->z_low_limbs_range_constraint_1_shift,
                     this->z_low_limbs_range_constraint_2_shift,
                     this->z_low_limbs_range_constraint_3_shift,
                     this->z_low_limbs_range_constraint_4_shift,
                     this->z_low_limbs_range_constraint_tail_shift,
                     this->z_high_limbs_shift,
                     this->z_high_limbs_range_constraint_0_shift,
                     this->z_high_limbs_range_constraint_1_shift,
                     this->z_high_limbs_range_constraint_2_shift,
                     this->z_high_limbs_range_constraint_3_shift,
                     this->z_high_limbs_range_constraint_4_shift,
                     this->z_high_limbs_range_constraint_tail_shift,
                     this->accumulators_binary_limbs_0_shift,
                     this->accumulators_binary_limbs_1_shift,
                     this->accumulators_binary_limbs_2_shift,
                     this->accumulators_binary_limbs_3_shift,
                     this->accumulator_low_limbs_range_constraint_0_shift,
                     this->accumulator_low_limbs_range_constraint_1_shift,
                     this->accumulator_low_limbs_range_constraint_2_shift,
                     this->accumulator_low_limbs_range_constraint_3_shift,
                     this->accumulator_low_limbs_range_constraint_4_shift,
                     this->accumulator_low_limbs_range_constraint_tail_shift,
                     this->accumulator_high_limbs_range_constraint_0_shift,
                     this->accumulator_high_limbs_range_constraint_1_shift,
                     this->accumulator_high_limbs_range_constraint_2_shift,
                     this->accumulator_high_limbs_range_constraint_3_shift,
                     this->accumulator_high_limbs_range_constraint_4_shift,
                     this->accumulator_high_limbs_range_constraint_tail_shift,
                     this->quotient_low_binary_limbs_shift,
                     this->quotient_high_binary_limbs_shift,
                     this->quotient_low_limbs_range_constraint_0_shift,
                     this->quotient_low_limbs_range_constraint_1_shift,
                     this->quotient_low_limbs_range_constraint_2_shift,
                     this->quotient_low_limbs_range_constraint_3_shift,
                     this->quotient_low_limbs_range_constraint_4_shift,
                     this->quotient_low_limbs_range_constraint_tail_shift,
                     this->quotient_high_limbs_range_constraint_0_shift,
                     this->quotient_high_limbs_range_constraint_1_shift,
                     this->quotient_high_limbs_range_constraint_2_shift,
                     this->quotient_high_limbs_range_constraint_3_shift,
                     this->quotient_high_limbs_range_constraint_4_shift,
                     this->quotient_high_limbs_range_constraint_tail_shift,
                     this->relation_wide_limbs_shift,
                     this->relation_wide_limbs_range_constraint_0_shift,
                     this->relation_wide_limbs_range_constraint_1_shift,
                     this->relation_wide_limbs_range_constraint_2_shift,
                     this->relation_wide_limbs_range_constraint_3_shift,
                     this->ordered_range_constraints_0_shift,
                     this->ordered_range_constraints_1_shift,
                     this->ordered_range_constraints_2_shift,
                     this->ordered_range_constraints_3_shift,
                     this->ordered_range_constraints_4_shift,

                     this->z_perm_shift };
        };

        /**
         * @brief Polynomials/commitments, that can be constructed only after the r challenge has been received from
         * gemini
         *
         * @return RefVector<DataType>
         */
        RefVector<DataType> get_special() { return get_concatenated_constraints(); }

        RefVector<DataType> get_unshifted_then_shifted_then_special()
        {
            RefVector<DataType> result{ this->get_unshifted() };
            RefVector<DataType> shifted{ get_shifted() };
            RefVector<DataType> special{ get_special() };
            result.insert(result.end(), shifted.begin(), shifted.end());
            result.insert(result.end(), special.begin(), special.end());
            return result;
        }

        friend std::ostream& operator<<(std::ostream& os, const AllEntities& a)
        {
            os << "{ ";
            std::ios_base::fmtflags f(os.flags());
            auto entities = a.get_all();
            for (size_t i = 0; i < entities.size() - 1; i++) {
                os << "e[" << std::setw(2) << i << "] = " << (entities[i]) << ",\n";
            }
            os << "e[" << std::setw(2) << (entities.size() - 1) << "] = " << entities[entities.size() - 1] << " }";

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
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>> {
      public:
        BF batching_challenge_v = { 0 };
        BF evaluation_input_x = { 0 };
        ProvingKey() = default;

        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>;
        using Base::Base;

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/810): get around this by properly having
        // concatenated range be a concept outside of witnessentities
        std::vector<std::string> get_labels()
        {
            return concatenate(PrecomputedEntities<Polynomial>::get_labels(),
                               WitnessEntities<Polynomial>::get_unshifted_wire_labels());
        }
        RefVector<Polynomial> get_all()
        {
            return concatenate(PrecomputedEntities<Polynomial>::get_all(),
                               WitnessEntities<Polynomial>::get_unshifted_wires());
        }

        ProvingKey(const size_t circuit_size)
            : ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>(circuit_size, 0)

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
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials
     * evaluated at one point.
     */
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
        [[nodiscard]] size_t get_polynomial_size() const { return this->op.size(); }
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

    /**
     * @brief A container for easier mapping of polynomials
     */
    using ProverPolynomialIds = AllEntities<size_t>;

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
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly
     * needed. It has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string> {
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

    class VerifierCommitments : public AllEntities<Commitment> {
      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
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

    using Transcript = BaseTranscript;
};
} // namespace bb::honk::flavor