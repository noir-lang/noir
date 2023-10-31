#pragma once
#include "../sumcheck/relation_definitions_fwd.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_translator_circuit_builder.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/relations/translator_vm/translator_decomposition_relation.hpp"
#include "barretenberg/proof_system/relations/translator_vm/translator_extra_relations.hpp"
#include "barretenberg/proof_system/relations/translator_vm/translator_gen_perm_sort_relation.hpp"
#include "barretenberg/proof_system/relations/translator_vm/translator_non_native_field_relation.hpp"
#include "barretenberg/proof_system/relations/translator_vm/translator_permutation_relation.hpp"
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
        DataType& lagrange_first = std::get<0>(this->_data);
        DataType& lagrange_last = std::get<1>(this->_data);
        // TODO(#758): Check if one of these can be replaced by shifts
        DataType& lagrange_odd_in_minicircuit = std::get<2>(this->_data);
        DataType& lagrange_even_in_minicircuit = std::get<3>(this->_data);
        DataType& lagrange_second = std::get<4>(this->_data);
        DataType& lagrange_second_to_last_in_minicircuit = std::get<5>(this->_data);
        DataType& ordered_extra_range_constraints_numerator = std::get<6>(this->_data);
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
        DataType& op = std::get<0>(this->_data);
        DataType& x_lo_y_hi = std::get<1>(this->_data);
        DataType& x_hi_z_1 = std::get<2>(this->_data);
        DataType& y_lo_z_2 = std::get<3>(this->_data);
        DataType& p_x_low_limbs = std::get<4>(this->_data);
        DataType& p_x_low_limbs_range_constraint_0 = std::get<5>(this->_data);
        DataType& p_x_low_limbs_range_constraint_1 = std::get<6>(this->_data);
        DataType& p_x_low_limbs_range_constraint_2 = std::get<7>(this->_data);
        DataType& p_x_low_limbs_range_constraint_3 = std::get<8>(this->_data);
        DataType& p_x_low_limbs_range_constraint_4 = std::get<9>(this->_data);
        DataType& p_x_low_limbs_range_constraint_tail = std::get<10>(this->_data);
        DataType& p_x_high_limbs = std::get<11>(this->_data);
        DataType& p_x_high_limbs_range_constraint_0 = std::get<12>(this->_data);
        DataType& p_x_high_limbs_range_constraint_1 = std::get<13>(this->_data);
        DataType& p_x_high_limbs_range_constraint_2 = std::get<14>(this->_data);
        DataType& p_x_high_limbs_range_constraint_3 = std::get<15>(this->_data);
        DataType& p_x_high_limbs_range_constraint_4 = std::get<16>(this->_data);
        DataType& p_x_high_limbs_range_constraint_tail = std::get<17>(this->_data);
        DataType& p_y_low_limbs = std::get<18>(this->_data);
        DataType& p_y_low_limbs_range_constraint_0 = std::get<19>(this->_data);
        DataType& p_y_low_limbs_range_constraint_1 = std::get<20>(this->_data);
        DataType& p_y_low_limbs_range_constraint_2 = std::get<21>(this->_data);
        DataType& p_y_low_limbs_range_constraint_3 = std::get<22>(this->_data);
        DataType& p_y_low_limbs_range_constraint_4 = std::get<23>(this->_data);
        DataType& p_y_low_limbs_range_constraint_tail = std::get<24>(this->_data);
        DataType& p_y_high_limbs = std::get<25>(this->_data);
        DataType& p_y_high_limbs_range_constraint_0 = std::get<26>(this->_data);
        DataType& p_y_high_limbs_range_constraint_1 = std::get<27>(this->_data);
        DataType& p_y_high_limbs_range_constraint_2 = std::get<28>(this->_data);
        DataType& p_y_high_limbs_range_constraint_3 = std::get<29>(this->_data);
        DataType& p_y_high_limbs_range_constraint_4 = std::get<30>(this->_data);
        DataType& p_y_high_limbs_range_constraint_tail = std::get<31>(this->_data);
        DataType& z_low_limbs = std::get<32>(this->_data);
        DataType& z_low_limbs_range_constraint_0 = std::get<33>(this->_data);
        DataType& z_low_limbs_range_constraint_1 = std::get<34>(this->_data);
        DataType& z_low_limbs_range_constraint_2 = std::get<35>(this->_data);
        DataType& z_low_limbs_range_constraint_3 = std::get<36>(this->_data);
        DataType& z_low_limbs_range_constraint_4 = std::get<37>(this->_data);
        DataType& z_low_limbs_range_constraint_tail = std::get<38>(this->_data);
        DataType& z_high_limbs = std::get<39>(this->_data);
        DataType& z_high_limbs_range_constraint_0 = std::get<40>(this->_data);
        DataType& z_high_limbs_range_constraint_1 = std::get<41>(this->_data);
        DataType& z_high_limbs_range_constraint_2 = std::get<42>(this->_data);
        DataType& z_high_limbs_range_constraint_3 = std::get<43>(this->_data);
        DataType& z_high_limbs_range_constraint_4 = std::get<44>(this->_data);
        DataType& z_high_limbs_range_constraint_tail = std::get<45>(this->_data);
        DataType& accumulators_binary_limbs_0 = std::get<46>(this->_data);
        DataType& accumulators_binary_limbs_1 = std::get<47>(this->_data);
        DataType& accumulators_binary_limbs_2 = std::get<48>(this->_data);
        DataType& accumulators_binary_limbs_3 = std::get<49>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_0 = std::get<50>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_1 = std::get<51>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_2 = std::get<52>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_3 = std::get<53>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_4 = std::get<54>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_tail = std::get<55>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_0 = std::get<56>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_1 = std::get<57>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_2 = std::get<58>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_3 = std::get<59>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_4 = std::get<60>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_tail = std::get<61>(this->_data);
        DataType& quotient_low_binary_limbs = std::get<62>(this->_data);
        DataType& quotient_high_binary_limbs = std::get<63>(this->_data);
        DataType& quotient_low_limbs_range_constraint_0 = std::get<64>(this->_data);
        DataType& quotient_low_limbs_range_constraint_1 = std::get<65>(this->_data);
        DataType& quotient_low_limbs_range_constraint_2 = std::get<66>(this->_data);
        DataType& quotient_low_limbs_range_constraint_3 = std::get<67>(this->_data);
        DataType& quotient_low_limbs_range_constraint_4 = std::get<68>(this->_data);
        DataType& quotient_low_limbs_range_constraint_tail = std::get<69>(this->_data);
        DataType& quotient_high_limbs_range_constraint_0 = std::get<70>(this->_data);
        DataType& quotient_high_limbs_range_constraint_1 = std::get<71>(this->_data);
        DataType& quotient_high_limbs_range_constraint_2 = std::get<72>(this->_data);
        DataType& quotient_high_limbs_range_constraint_3 = std::get<73>(this->_data);
        DataType& quotient_high_limbs_range_constraint_4 = std::get<74>(this->_data);
        DataType& quotient_high_limbs_range_constraint_tail = std::get<75>(this->_data);
        DataType& relation_wide_limbs = std::get<76>(this->_data);
        DataType& relation_wide_limbs_range_constraint_0 = std::get<77>(this->_data);
        DataType& relation_wide_limbs_range_constraint_1 = std::get<78>(this->_data);
        DataType& relation_wide_limbs_range_constraint_2 = std::get<79>(this->_data);
        DataType& relation_wide_limbs_range_constraint_3 = std::get<80>(this->_data);
        DataType& concatenated_range_constraints_0 = std::get<81>(this->_data);
        DataType& concatenated_range_constraints_1 = std::get<82>(this->_data);
        DataType& concatenated_range_constraints_2 = std::get<83>(this->_data);
        DataType& concatenated_range_constraints_3 = std::get<84>(this->_data);
        DataType& ordered_range_constraints_0 = std::get<85>(this->_data);
        DataType& ordered_range_constraints_1 = std::get<86>(this->_data);
        DataType& ordered_range_constraints_2 = std::get<87>(this->_data);
        DataType& ordered_range_constraints_3 = std::get<88>(this->_data);
        DataType& ordered_range_constraints_4 = std::get<89>(this->_data);
        DataType& z_perm = std::get<90>(this->_data);

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
        DataType& op = std::get<0>(this->_data);
        DataType& x_lo_y_hi = std::get<1>(this->_data);
        DataType& x_hi_z_1 = std::get<2>(this->_data);
        DataType& y_lo_z_2 = std::get<3>(this->_data);
        DataType& p_x_low_limbs = std::get<4>(this->_data);
        DataType& p_x_low_limbs_range_constraint_0 = std::get<5>(this->_data);
        DataType& p_x_low_limbs_range_constraint_1 = std::get<6>(this->_data);
        DataType& p_x_low_limbs_range_constraint_2 = std::get<7>(this->_data);
        DataType& p_x_low_limbs_range_constraint_3 = std::get<8>(this->_data);
        DataType& p_x_low_limbs_range_constraint_4 = std::get<9>(this->_data);
        DataType& p_x_low_limbs_range_constraint_tail = std::get<10>(this->_data);
        DataType& p_x_high_limbs = std::get<11>(this->_data);
        DataType& p_x_high_limbs_range_constraint_0 = std::get<12>(this->_data);
        DataType& p_x_high_limbs_range_constraint_1 = std::get<13>(this->_data);
        DataType& p_x_high_limbs_range_constraint_2 = std::get<14>(this->_data);
        DataType& p_x_high_limbs_range_constraint_3 = std::get<15>(this->_data);
        DataType& p_x_high_limbs_range_constraint_4 = std::get<16>(this->_data);
        DataType& p_x_high_limbs_range_constraint_tail = std::get<17>(this->_data);
        DataType& p_y_low_limbs = std::get<18>(this->_data);
        DataType& p_y_low_limbs_range_constraint_0 = std::get<19>(this->_data);
        DataType& p_y_low_limbs_range_constraint_1 = std::get<20>(this->_data);
        DataType& p_y_low_limbs_range_constraint_2 = std::get<21>(this->_data);
        DataType& p_y_low_limbs_range_constraint_3 = std::get<22>(this->_data);
        DataType& p_y_low_limbs_range_constraint_4 = std::get<23>(this->_data);
        DataType& p_y_low_limbs_range_constraint_tail = std::get<24>(this->_data);
        DataType& p_y_high_limbs = std::get<25>(this->_data);
        DataType& p_y_high_limbs_range_constraint_0 = std::get<26>(this->_data);
        DataType& p_y_high_limbs_range_constraint_1 = std::get<27>(this->_data);
        DataType& p_y_high_limbs_range_constraint_2 = std::get<28>(this->_data);
        DataType& p_y_high_limbs_range_constraint_3 = std::get<29>(this->_data);
        DataType& p_y_high_limbs_range_constraint_4 = std::get<30>(this->_data);
        DataType& p_y_high_limbs_range_constraint_tail = std::get<31>(this->_data);
        DataType& z_low_limbs = std::get<32>(this->_data);
        DataType& z_low_limbs_range_constraint_0 = std::get<33>(this->_data);
        DataType& z_low_limbs_range_constraint_1 = std::get<34>(this->_data);
        DataType& z_low_limbs_range_constraint_2 = std::get<35>(this->_data);
        DataType& z_low_limbs_range_constraint_3 = std::get<36>(this->_data);
        DataType& z_low_limbs_range_constraint_4 = std::get<37>(this->_data);
        DataType& z_low_limbs_range_constraint_tail = std::get<38>(this->_data);
        DataType& z_high_limbs = std::get<39>(this->_data);
        DataType& z_high_limbs_range_constraint_0 = std::get<40>(this->_data);
        DataType& z_high_limbs_range_constraint_1 = std::get<41>(this->_data);
        DataType& z_high_limbs_range_constraint_2 = std::get<42>(this->_data);
        DataType& z_high_limbs_range_constraint_3 = std::get<43>(this->_data);
        DataType& z_high_limbs_range_constraint_4 = std::get<44>(this->_data);
        DataType& z_high_limbs_range_constraint_tail = std::get<45>(this->_data);
        DataType& accumulators_binary_limbs_0 = std::get<46>(this->_data);
        DataType& accumulators_binary_limbs_1 = std::get<47>(this->_data);
        DataType& accumulators_binary_limbs_2 = std::get<48>(this->_data);
        DataType& accumulators_binary_limbs_3 = std::get<49>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_0 = std::get<50>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_1 = std::get<51>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_2 = std::get<52>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_3 = std::get<53>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_4 = std::get<54>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_tail = std::get<55>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_0 = std::get<56>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_1 = std::get<57>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_2 = std::get<58>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_3 = std::get<59>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_4 = std::get<60>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_tail = std::get<61>(this->_data);
        DataType& quotient_low_binary_limbs = std::get<62>(this->_data);
        DataType& quotient_high_binary_limbs = std::get<63>(this->_data);
        DataType& quotient_low_limbs_range_constraint_0 = std::get<64>(this->_data);
        DataType& quotient_low_limbs_range_constraint_1 = std::get<65>(this->_data);
        DataType& quotient_low_limbs_range_constraint_2 = std::get<66>(this->_data);
        DataType& quotient_low_limbs_range_constraint_3 = std::get<67>(this->_data);
        DataType& quotient_low_limbs_range_constraint_4 = std::get<68>(this->_data);
        DataType& quotient_low_limbs_range_constraint_tail = std::get<69>(this->_data);
        DataType& quotient_high_limbs_range_constraint_0 = std::get<70>(this->_data);
        DataType& quotient_high_limbs_range_constraint_1 = std::get<71>(this->_data);
        DataType& quotient_high_limbs_range_constraint_2 = std::get<72>(this->_data);
        DataType& quotient_high_limbs_range_constraint_3 = std::get<73>(this->_data);
        DataType& quotient_high_limbs_range_constraint_4 = std::get<74>(this->_data);
        DataType& quotient_high_limbs_range_constraint_tail = std::get<75>(this->_data);
        DataType& relation_wide_limbs = std::get<76>(this->_data);
        DataType& relation_wide_limbs_range_constraint_0 = std::get<77>(this->_data);
        DataType& relation_wide_limbs_range_constraint_1 = std::get<78>(this->_data);
        DataType& relation_wide_limbs_range_constraint_2 = std::get<79>(this->_data);
        DataType& relation_wide_limbs_range_constraint_3 = std::get<80>(this->_data);
        DataType& concatenated_range_constraints_0 = std::get<81>(this->_data);
        DataType& concatenated_range_constraints_1 = std::get<82>(this->_data);
        DataType& concatenated_range_constraints_2 = std::get<83>(this->_data);
        DataType& concatenated_range_constraints_3 = std::get<84>(this->_data);
        DataType& ordered_range_constraints_0 = std::get<85>(this->_data);
        DataType& ordered_range_constraints_1 = std::get<86>(this->_data);
        DataType& ordered_range_constraints_2 = std::get<87>(this->_data);
        DataType& ordered_range_constraints_3 = std::get<88>(this->_data);
        DataType& ordered_range_constraints_4 = std::get<89>(this->_data);
        DataType& z_perm = std::get<90>(this->_data);
        DataType& x_lo_y_hi_shift = std::get<91>(this->_data);
        DataType& x_hi_z_1_shift = std::get<92>(this->_data);
        DataType& y_lo_z_2_shift = std::get<93>(this->_data);
        DataType& p_x_low_limbs_shift = std::get<94>(this->_data);
        DataType& p_x_low_limbs_range_constraint_0_shift = std::get<95>(this->_data);
        DataType& p_x_low_limbs_range_constraint_1_shift = std::get<96>(this->_data);
        DataType& p_x_low_limbs_range_constraint_2_shift = std::get<97>(this->_data);
        DataType& p_x_low_limbs_range_constraint_3_shift = std::get<98>(this->_data);
        DataType& p_x_low_limbs_range_constraint_4_shift = std::get<99>(this->_data);
        DataType& p_x_low_limbs_range_constraint_tail_shift = std::get<100>(this->_data);
        DataType& p_x_high_limbs_shift = std::get<101>(this->_data);
        DataType& p_x_high_limbs_range_constraint_0_shift = std::get<102>(this->_data);
        DataType& p_x_high_limbs_range_constraint_1_shift = std::get<103>(this->_data);
        DataType& p_x_high_limbs_range_constraint_2_shift = std::get<104>(this->_data);
        DataType& p_x_high_limbs_range_constraint_3_shift = std::get<105>(this->_data);
        DataType& p_x_high_limbs_range_constraint_4_shift = std::get<106>(this->_data);
        DataType& p_x_high_limbs_range_constraint_tail_shift = std::get<107>(this->_data);
        DataType& p_y_low_limbs_shift = std::get<108>(this->_data);
        DataType& p_y_low_limbs_range_constraint_0_shift = std::get<109>(this->_data);
        DataType& p_y_low_limbs_range_constraint_1_shift = std::get<110>(this->_data);
        DataType& p_y_low_limbs_range_constraint_2_shift = std::get<111>(this->_data);
        DataType& p_y_low_limbs_range_constraint_3_shift = std::get<112>(this->_data);
        DataType& p_y_low_limbs_range_constraint_4_shift = std::get<113>(this->_data);
        DataType& p_y_low_limbs_range_constraint_tail_shift = std::get<114>(this->_data);
        DataType& p_y_high_limbs_shift = std::get<115>(this->_data);
        DataType& p_y_high_limbs_range_constraint_0_shift = std::get<116>(this->_data);
        DataType& p_y_high_limbs_range_constraint_1_shift = std::get<117>(this->_data);
        DataType& p_y_high_limbs_range_constraint_2_shift = std::get<118>(this->_data);
        DataType& p_y_high_limbs_range_constraint_3_shift = std::get<119>(this->_data);
        DataType& p_y_high_limbs_range_constraint_4_shift = std::get<120>(this->_data);
        DataType& p_y_high_limbs_range_constraint_tail_shift = std::get<121>(this->_data);
        DataType& z_low_limbs_shift = std::get<122>(this->_data);
        DataType& z_low_limbs_range_constraint_0_shift = std::get<123>(this->_data);
        DataType& z_low_limbs_range_constraint_1_shift = std::get<124>(this->_data);
        DataType& z_low_limbs_range_constraint_2_shift = std::get<125>(this->_data);
        DataType& z_low_limbs_range_constraint_3_shift = std::get<126>(this->_data);
        DataType& z_low_limbs_range_constraint_4_shift = std::get<127>(this->_data);
        DataType& z_low_limbs_range_constraint_tail_shift = std::get<128>(this->_data);
        DataType& z_high_limbs_shift = std::get<129>(this->_data);
        DataType& z_high_limbs_range_constraint_0_shift = std::get<130>(this->_data);
        DataType& z_high_limbs_range_constraint_1_shift = std::get<131>(this->_data);
        DataType& z_high_limbs_range_constraint_2_shift = std::get<132>(this->_data);
        DataType& z_high_limbs_range_constraint_3_shift = std::get<133>(this->_data);
        DataType& z_high_limbs_range_constraint_4_shift = std::get<134>(this->_data);
        DataType& z_high_limbs_range_constraint_tail_shift = std::get<135>(this->_data);
        DataType& accumulators_binary_limbs_0_shift = std::get<136>(this->_data);
        DataType& accumulators_binary_limbs_1_shift = std::get<137>(this->_data);
        DataType& accumulators_binary_limbs_2_shift = std::get<138>(this->_data);
        DataType& accumulators_binary_limbs_3_shift = std::get<139>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_0_shift = std::get<140>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_1_shift = std::get<141>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_2_shift = std::get<142>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_3_shift = std::get<143>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_4_shift = std::get<144>(this->_data);
        DataType& accumulator_low_limbs_range_constraint_tail_shift = std::get<145>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_0_shift = std::get<146>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_1_shift = std::get<147>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_2_shift = std::get<148>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_3_shift = std::get<149>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_4_shift = std::get<150>(this->_data);
        DataType& accumulator_high_limbs_range_constraint_tail_shift = std::get<151>(this->_data);
        DataType& quotient_low_binary_limbs_shift = std::get<152>(this->_data);
        DataType& quotient_high_binary_limbs_shift = std::get<153>(this->_data);
        DataType& quotient_low_limbs_range_constraint_0_shift = std::get<154>(this->_data);
        DataType& quotient_low_limbs_range_constraint_1_shift = std::get<155>(this->_data);
        DataType& quotient_low_limbs_range_constraint_2_shift = std::get<156>(this->_data);
        DataType& quotient_low_limbs_range_constraint_3_shift = std::get<157>(this->_data);
        DataType& quotient_low_limbs_range_constraint_4_shift = std::get<158>(this->_data);
        DataType& quotient_low_limbs_range_constraint_tail_shift = std::get<159>(this->_data);
        DataType& quotient_high_limbs_range_constraint_0_shift = std::get<160>(this->_data);
        DataType& quotient_high_limbs_range_constraint_1_shift = std::get<161>(this->_data);
        DataType& quotient_high_limbs_range_constraint_2_shift = std::get<162>(this->_data);
        DataType& quotient_high_limbs_range_constraint_3_shift = std::get<163>(this->_data);
        DataType& quotient_high_limbs_range_constraint_4_shift = std::get<164>(this->_data);
        DataType& quotient_high_limbs_range_constraint_tail_shift = std::get<165>(this->_data);
        DataType& relation_wide_limbs_shift = std::get<166>(this->_data);
        DataType& relation_wide_limbs_range_constraint_0_shift = std::get<167>(this->_data);
        DataType& relation_wide_limbs_range_constraint_1_shift = std::get<168>(this->_data);
        DataType& relation_wide_limbs_range_constraint_2_shift = std::get<169>(this->_data);
        DataType& relation_wide_limbs_range_constraint_3_shift = std::get<170>(this->_data);
        DataType& ordered_range_constraints_0_shift = std::get<171>(this->_data);
        DataType& ordered_range_constraints_1_shift = std::get<172>(this->_data);
        DataType& ordered_range_constraints_2_shift = std::get<173>(this->_data);
        DataType& ordered_range_constraints_3_shift = std::get<174>(this->_data);
        DataType& ordered_range_constraints_4_shift = std::get<175>(this->_data);
        DataType& z_perm_shift = std::get<176>(this->_data);
        DataType& lagrange_first = std::get<177>(this->_data);
        DataType& lagrange_last = std::get<178>(this->_data);
        DataType& lagrange_odd_in_minicircuit = std::get<179>(this->_data);
        DataType& lagrange_even_in_minicircuit = std::get<180>(this->_data);
        DataType& lagrange_second = std::get<181>(this->_data);
        DataType& lagrange_second_to_last_in_minicircuit = std::get<182>(this->_data);
        DataType& ordered_extra_range_constraints_numerator = std::get<183>(this->_data);

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

        AllEntities() = default;

        AllEntities(const AllEntities& other)
            : AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>(other){};

        AllEntities(AllEntities&& other) noexcept
            : AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>(other){};

        AllEntities& operator=(const AllEntities& other)
        {
            if (this == &other) {
                return *this;
            }
            AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>::operator=(other);
            return *this;
        }

        AllEntities& operator=(AllEntities&& other) noexcept
        {
            AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>::operator=(other);
            return *this;
        }

        ~AllEntities() = default;
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
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        AllValues get_row(const size_t row_idx)
        {
            AllValues result;
            size_t column_idx = 0; // TODO(https://github.com/AztecProtocol/barretenberg/issues/391) zip
            for (auto& column : this->_data) {
                result[column_idx] = column[row_idx];
                column_idx++;
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

using GoblinTranslatorBasic = GoblinTranslator_<2048>;
} // namespace proof_system::honk::flavor
