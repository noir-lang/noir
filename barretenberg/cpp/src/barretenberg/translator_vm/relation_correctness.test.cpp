#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/translator_vm/goblin_translator_flavor.hpp"

#include <gtest/gtest.h>
using namespace bb;

/**
 * @brief Check that a given relation is satified for a set of polynomials
 *
 * @tparam relation_idx Index into a tuple of provided relations
 * @tparam Flavor
 */
template <typename Flavor, typename Relation> void check_relation(auto circuit_size, auto& polynomials, auto params)
{
    for (size_t i = 0; i < circuit_size; i++) {
        // Define the appropriate SumcheckArrayOfValuesOverSubrelations type for this relation and initialize to zero
        using SumcheckArrayOfValuesOverSubrelations = typename Relation::SumcheckArrayOfValuesOverSubrelations;
        SumcheckArrayOfValuesOverSubrelations result;
        for (auto& element : result) {
            element = 0;
        }

        // Evaluate each constraint in the relation and check that each is satisfied
        Relation::accumulate(result, polynomials.get_row(i), params, 1);
        for (auto& element : result) {
            ASSERT_EQ(element, 0);
        }
    }
}

class GoblinTranslatorRelationCorrectnessTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

/**
 * @brief Test the correctness of GolbinTranslator's Permutation Relation
 *
 */
TEST_F(GoblinTranslatorRelationCorrectnessTests, Permutation)
{
    using Flavor = GoblinTranslatorFlavor;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = bb::Polynomial<FF>;
    auto& engine = numeric::get_debug_randomness();
    const size_t mini_circuit_size = 2048;
    auto full_circuit_size = mini_circuit_size * Flavor::CONCATENATION_GROUP_SIZE;

    // We only need gamma, because permutationr elation only uses gamma
    FF gamma = FF::random_element();

    // Fill relation parameters
    RelationParameters<FF> params;
    params.gamma = gamma;

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    for (Polynomial& prover_poly : prover_polynomials.get_all()) {
        prover_poly = Polynomial{ full_circuit_size };
    }

    // Fill in lagrange polynomials used in the permutation relation
    prover_polynomials.lagrange_first[0] = 1;
    prover_polynomials.lagrange_last[full_circuit_size - 1] = 1;

    // Put random values in all the non-concatenated constraint polynomials used to range constrain the values
    auto fill_polynomial_with_random_14_bit_values = [&](auto& polynomial) {
        for (size_t i = 0; i < mini_circuit_size; i++) {
            polynomial[i] = engine.get_random_uint16() & ((1 << Flavor::MICRO_LIMB_BITS) - 1);
        }
    };
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_3);

    // Compute ordered range constraint polynomials that go in the denominator of the grand product polynomial
    compute_goblin_translator_range_constraint_ordered_polynomials<Flavor>(prover_polynomials, mini_circuit_size);

    // Compute the fixed numerator (part of verification key)
    prover_polynomials.compute_extra_range_constraint_numerator();

    // Compute concatenated polynomials (4 polynomials produced from other constraint polynomials by concatenation)
    compute_concatenated_polynomials<Flavor>(prover_polynomials);

    // Compute the grand product polynomial
    compute_grand_product<Flavor, bb::GoblinTranslatorPermutationRelation<FF>>(prover_polynomials, params);
    prover_polynomials.z_perm_shift = prover_polynomials.z_perm.shifted();

    using Relations = typename Flavor::Relations;

    // Check that permutation relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<0, Relations>>(full_circuit_size, prover_polynomials, params);
}

TEST_F(GoblinTranslatorRelationCorrectnessTests, DeltaRangeConstraint)
{
    using Flavor = GoblinTranslatorFlavor;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = bb::Polynomial<FF>;
    auto& engine = numeric::get_debug_randomness();
    const size_t mini_circuit_size = 2048;
    const auto circuit_size = Flavor::CONCATENATION_GROUP_SIZE * mini_circuit_size;
    const auto sort_step = Flavor::SORT_STEP;
    const auto max_value = (1 << Flavor::MICRO_LIMB_BITS) - 1;

    // No relation parameters are used in this relation
    RelationParameters<FF> params;

    ProverPolynomials prover_polynomials;
    // Allocate polynomials
    for (Polynomial& polynomial : prover_polynomials.get_all()) {
        polynomial = Polynomial{ circuit_size };
    }

    // Construct lagrange polynomials that are needed for Goblin Translator's DeltaRangeConstraint Relation
    prover_polynomials.lagrange_first[0] = 1;
    prover_polynomials.lagrange_last[circuit_size - 1] = 1;

    // Create a vector and fill with necessary steps for the DeltaRangeConstraint relation
    auto sorted_elements_count = (max_value / sort_step) + 1;
    std::vector<uint64_t> vector_for_sorting(circuit_size);
    for (size_t i = 0; i < sorted_elements_count - 1; i++) {
        vector_for_sorting[i] = i * sort_step;
    }
    vector_for_sorting[sorted_elements_count - 1] = max_value;

    // Add random values to fill the leftover space
    for (size_t i = sorted_elements_count; i < circuit_size; i++) {
        vector_for_sorting[i] = engine.get_random_uint16() & ((1 << Flavor::MICRO_LIMB_BITS) - 1);
    }

    // Get ordered polynomials
    auto polynomial_pointers = std::vector{ &prover_polynomials.ordered_range_constraints_0,
                                            &prover_polynomials.ordered_range_constraints_1,
                                            &prover_polynomials.ordered_range_constraints_2,
                                            &prover_polynomials.ordered_range_constraints_3,
                                            &prover_polynomials.ordered_range_constraints_4 };

    // Sort the vector
    std::sort(vector_for_sorting.begin(), vector_for_sorting.end());

    // Copy values, transforming them into Finite Field elements
    std::transform(vector_for_sorting.cbegin(),
                   vector_for_sorting.cend(),
                   prover_polynomials.ordered_range_constraints_0.begin(),
                   [](uint64_t in) { return FF(in); });

    // Copy the same polynomial into the 4 other ordered polynomials (they are not the same in an actual proof, but we
    // only need to check the correctness of the relation and it acts independently on each polynomial)
    parallel_for(4, [&](size_t i) {
        std::copy(prover_polynomials.ordered_range_constraints_0.begin(),
                  prover_polynomials.ordered_range_constraints_0.end(),
                  polynomial_pointers[i + 1]->begin());
    });

    // Get shifted polynomials
    prover_polynomials.ordered_range_constraints_0_shift = prover_polynomials.ordered_range_constraints_0.shifted();
    prover_polynomials.ordered_range_constraints_1_shift = prover_polynomials.ordered_range_constraints_1.shifted();
    prover_polynomials.ordered_range_constraints_2_shift = prover_polynomials.ordered_range_constraints_2.shifted();
    prover_polynomials.ordered_range_constraints_3_shift = prover_polynomials.ordered_range_constraints_3.shifted();
    prover_polynomials.ordered_range_constraints_4_shift = prover_polynomials.ordered_range_constraints_4.shifted();

    using Relations = typename Flavor::Relations;

    // Check that DeltaRangeConstraint relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<1, Relations>>(circuit_size, prover_polynomials, params);
}

/**
 * @brief Test the correctness of GoblinTranslatorFlavor's  extra relations (GoblinTranslatorOpcodeConstraintRelation
 * and GoblinTranslatorAccumulatorTransferRelation)
 *
 */
TEST_F(GoblinTranslatorRelationCorrectnessTests, GoblinTranslatorExtraRelationsCorrectness)
{
    using Flavor = GoblinTranslatorFlavor;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using ProverPolynomialIds = typename Flavor::ProverPolynomialIds;
    using Polynomial = bb::Polynomial<FF>;

    auto& engine = numeric::get_debug_randomness();

    const size_t mini_circuit_size = 2048;
    const auto circuit_size = Flavor::CONCATENATION_GROUP_SIZE * mini_circuit_size;

    // We only use accumulated_result from relation parameters in this relation
    RelationParameters<FF> params;
    params.accumulated_result = {
        FF::random_element(), FF::random_element(), FF::random_element(), FF::random_element()
    };

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    // We use polynomial ids to make shifting the polynomials easier
    ProverPolynomialIds prover_polynomial_ids;
    auto polynomial_id_get_all = prover_polynomial_ids.get_all();
    std::vector<Polynomial> polynomial_container;
    std::vector<size_t> polynomial_ids;
    for (size_t i = 0; i < polynomial_id_get_all.size(); i++) {
        Polynomial temporary_polynomial(circuit_size);
        // Allocate polynomials
        polynomial_container.push_back(temporary_polynomial);
        // Push sequential ids to polynomial ids
        polynomial_ids.push_back(i);
        polynomial_id_get_all[i] = polynomial_ids[i];
    }
    // Get ids of shifted polynomials and put them in a set
    auto shifted_ids = prover_polynomial_ids.get_shifted();
    std::unordered_set<size_t> shifted_id_set;
    for (auto& id : shifted_ids) {
        shifted_id_set.emplace(id);
    }
    // Assign to non-shifted prover polynomials
    auto polynomial_get_all = prover_polynomials.get_all();
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        if (!shifted_id_set.contains(i)) {
            polynomial_get_all[i] = polynomial_container[i].share();
        }
    }

    // Assign to shifted prover polynomials using ids
    for (size_t i = 0; i < shifted_ids.size(); i++) {
        auto shifted_id = shifted_ids[i];
        auto to_be_shifted_id = prover_polynomial_ids.get_to_be_shifted()[i];
        polynomial_get_all[shifted_id] = polynomial_container[to_be_shifted_id].shifted();
    }

    // Fill in lagrange even polynomial
    for (size_t i = 2; i < mini_circuit_size; i += 2) {
        prover_polynomials.lagrange_even_in_minicircuit[i] = 1;
    }
    constexpr size_t NUMBER_OF_POSSIBLE_OPCODES = 6;
    constexpr std::array<uint64_t, NUMBER_OF_POSSIBLE_OPCODES> possible_opcode_values = { 0, 1, 2, 3, 4, 8 };

    // Assign random opcode values
    for (size_t i = 1; i < mini_circuit_size - 1; i += 2) {
        prover_polynomials.op[i] =
            possible_opcode_values[static_cast<size_t>(engine.get_random_uint8() % NUMBER_OF_POSSIBLE_OPCODES)];
    }

    // Initialize used lagrange polynomials
    prover_polynomials.lagrange_second[1] = 1;
    prover_polynomials.lagrange_second_to_last_in_minicircuit[mini_circuit_size - 2] = 1;

    // Put random values in accumulator binary limbs (values should be preserved across even->next odd shift)
    for (size_t i = 2; i < mini_circuit_size - 2; i += 2) {
        prover_polynomials.accumulators_binary_limbs_0[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_1[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_2[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_3[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_0[i + 1] = prover_polynomials.accumulators_binary_limbs_0[i];
        prover_polynomials.accumulators_binary_limbs_1[i + 1] = prover_polynomials.accumulators_binary_limbs_1[i];
        prover_polynomials.accumulators_binary_limbs_2[i + 1] = prover_polynomials.accumulators_binary_limbs_2[i];
        prover_polynomials.accumulators_binary_limbs_3[i + 1] = prover_polynomials.accumulators_binary_limbs_3[i];
    }

    // The values of accumulator binary limbs at index 1 should equal the accumulated result from relation parameters
    prover_polynomials.accumulators_binary_limbs_0[1] = params.accumulated_result[0];
    prover_polynomials.accumulators_binary_limbs_1[1] = params.accumulated_result[1];
    prover_polynomials.accumulators_binary_limbs_2[1] = params.accumulated_result[2];
    prover_polynomials.accumulators_binary_limbs_3[1] = params.accumulated_result[3];

    using Relations = typename Flavor::Relations;

    // Check that Opcode Constraint relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<2, Relations>>(circuit_size, prover_polynomials, params);

    // Check that Accumulator Transfer relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<3, Relations>>(circuit_size, prover_polynomials, params);
}
/**
 * @brief Test the correctness of GoblinTranslatorFlavor's Decomposition Relation
 *
 */
TEST_F(GoblinTranslatorRelationCorrectnessTests, Decomposition)
{
    using Flavor = GoblinTranslatorFlavor;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using ProverPolynomialIds = typename Flavor::ProverPolynomialIds;
    using Polynomial = bb::Polynomial<FF>;
    auto& engine = numeric::get_debug_randomness();

    constexpr size_t mini_circuit_size = 2048;
    const auto circuit_size = Flavor::CONCATENATION_GROUP_SIZE * mini_circuit_size;

    // Decomposition relation doesn't use any relation parameters
    RelationParameters<FF> params;

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    // We use polynomial ids to make shifting the polynomials easier
    ProverPolynomialIds prover_polynomial_ids;
    std::vector<Polynomial> polynomial_container;
    std::vector<size_t> polynomial_ids;
    auto polynomial_id_get_all = prover_polynomial_ids.get_all();
    auto polynomial_get_all = prover_polynomials.get_all();
    for (size_t i = 0; i < polynomial_id_get_all.size(); i++) {
        Polynomial temporary_polynomial(circuit_size);
        // Allocate polynomials
        polynomial_container.push_back(temporary_polynomial);
        // Push sequential ids to polynomial ids
        polynomial_ids.push_back(i);
        polynomial_id_get_all[i] = polynomial_ids[i];
    }
    // Get ids of shifted polynomials and put them in a set
    auto shifted_ids = prover_polynomial_ids.get_shifted();
    std::unordered_set<size_t> shifted_id_set;
    for (auto& id : shifted_ids) {
        shifted_id_set.emplace(id);
    }
    // Assign spans to non-shifted prover polynomials
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        if (!shifted_id_set.contains(i)) {
            polynomial_get_all[i] = polynomial_container[i].share();
        }
    }

    // Assign shifted spans to shifted prover polynomials using ids
    for (size_t i = 0; i < shifted_ids.size(); i++) {
        auto shifted_id = shifted_ids[i];
        auto to_be_shifted_id = prover_polynomial_ids.get_to_be_shifted()[i];
        polynomial_get_all[shifted_id] = polynomial_container[to_be_shifted_id].shifted();
    }

    // Fill in lagrange odd polynomial (the only non-witness one we are using)
    for (size_t i = 1; i < mini_circuit_size - 1; i += 2) {
        prover_polynomials.lagrange_odd_in_minicircuit[i] = 1;
    }

    constexpr size_t NUM_LIMB_BITS = Flavor::CircuitBuilder::NUM_LIMB_BITS;
    constexpr size_t HIGH_WIDE_LIMB_WIDTH =
        Flavor::CircuitBuilder::NUM_LIMB_BITS + Flavor::CircuitBuilder::NUM_LAST_LIMB_BITS;
    constexpr size_t LOW_WIDE_LIMB_WIDTH = Flavor::CircuitBuilder::NUM_LIMB_BITS * 2;
    constexpr size_t Z_LIMB_WIDTH = 128;
    constexpr size_t MICRO_LIMB_WIDTH = Flavor::MICRO_LIMB_BITS;
    constexpr size_t SHIFT_12_TO_14 = 4;
    constexpr size_t SHIFT_10_TO_14 = 16;
    constexpr size_t SHIFT_8_TO_14 = 64;
    constexpr size_t SHIFT_4_TO_14 = 1024;

    /**
     * @brief Decompose a standard 68-bit limb of binary into 5 14-bit limbs and the 6th limb that is the same as the
     * 5th but shifted by 2 bits
     *
     */
    auto decompose_standard_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& limb_4, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            limb_4 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 4, MICRO_LIMB_WIDTH * 5);
            shifted_limb = limb_4 * SHIFT_12_TO_14;
        };

    /**
     * @brief Decompose a standard 50-bit top limb into 4 14-bit limbs and the 5th limb that is the same as 5th, but
     * shifted by 6 bits
     *
     */
    auto decompose_standard_top_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            shifted_limb = limb_3 * SHIFT_8_TO_14;
        };

    /**
     * @brief Decompose the 60-bit top limb of z1 or z2 into 5 14-bit limbs and a 6th limb which is equal to the 5th,
     * but shifted by 10 bits.
     *
     */
    auto decompose_standard_top_z_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& limb_4, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            limb_4 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 4, MICRO_LIMB_WIDTH * 5);
            shifted_limb = limb_4 * SHIFT_4_TO_14;
        };

    /**
     * @brief Decompose the 52-bit top limb of quotient into 4 14-bit limbs and the 5th limb that is the same as 5th,
     * but shifted by 4 bits
     *
     */
    auto decompose_top_quotient_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            shifted_limb = limb_3 * SHIFT_10_TO_14;
        };

    /**
     * @brief Decompose relation wide limb into 6 14-bit limbs
     *
     */
    auto decompose_relation_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& limb_4, auto& limb_5) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            limb_4 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 4, MICRO_LIMB_WIDTH * 5);
            limb_5 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 5, MICRO_LIMB_WIDTH * 6);
        };

    // Put random values in all the non-concatenated constraint polynomials used to range constrain the values
    for (size_t i = 1; i < mini_circuit_size - 1; i += 2) {
        // P.x
        prover_polynomials.x_lo_y_hi[i] = FF(engine.get_random_uint256() & ((uint256_t(1) << LOW_WIDE_LIMB_WIDTH) - 1));
        prover_polynomials.x_hi_z_1[i] = FF(engine.get_random_uint256() & ((uint256_t(1) << HIGH_WIDE_LIMB_WIDTH) - 1));

        // P.y
        prover_polynomials.y_lo_z_2[i] = FF(engine.get_random_uint256() & ((uint256_t(1) << LOW_WIDE_LIMB_WIDTH) - 1));
        prover_polynomials.x_lo_y_hi[i + 1] =
            FF(engine.get_random_uint256() & ((uint256_t(1) << HIGH_WIDE_LIMB_WIDTH) - 1));

        // z1 and z2
        prover_polynomials.x_hi_z_1[i + 1] = FF(engine.get_random_uint256() & ((uint256_t(1) << Z_LIMB_WIDTH) - 1));
        prover_polynomials.y_lo_z_2[i + 1] = FF(engine.get_random_uint256() & ((uint256_t(1) << Z_LIMB_WIDTH) - 1));

        // Slice P.x into chunks
        prover_polynomials.p_x_low_limbs[i] = uint256_t(prover_polynomials.x_lo_y_hi[i]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_x_low_limbs[i + 1] =
            uint256_t(prover_polynomials.x_lo_y_hi[i]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        prover_polynomials.p_x_high_limbs[i] = uint256_t(prover_polynomials.x_hi_z_1[i]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_x_high_limbs[i + 1] =
            uint256_t(prover_polynomials.x_hi_z_1[i]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);

        // Slice P.y into chunks
        prover_polynomials.p_y_low_limbs[i] = uint256_t(prover_polynomials.y_lo_z_2[i]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_y_low_limbs[i + 1] =
            uint256_t(prover_polynomials.y_lo_z_2[i]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        prover_polynomials.p_y_high_limbs[i] = uint256_t(prover_polynomials.x_lo_y_hi[i + 1]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_y_high_limbs[i + 1] =
            uint256_t(prover_polynomials.x_lo_y_hi[i + 1]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);

        // Slice z1 and z2 into chunks
        prover_polynomials.z_low_limbs[i] = uint256_t(prover_polynomials.x_hi_z_1[i + 1]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.z_low_limbs[i + 1] = uint256_t(prover_polynomials.y_lo_z_2[i + 1]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.z_high_limbs[i] =
            uint256_t(prover_polynomials.x_hi_z_1[i + 1]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        prover_polynomials.z_high_limbs[i + 1] =
            uint256_t(prover_polynomials.y_lo_z_2[i + 1]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);

        // Slice accumulator
        auto tmp = uint256_t(BF::random_element(&engine));
        prover_polynomials.accumulators_binary_limbs_0[i] = tmp.slice(0, NUM_LIMB_BITS);
        prover_polynomials.accumulators_binary_limbs_1[i] = tmp.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2);
        prover_polynomials.accumulators_binary_limbs_2[i] = tmp.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3);
        prover_polynomials.accumulators_binary_limbs_3[i] = tmp.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4);

        // Slice low limbs of P.x into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_x_low_limbs[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_0[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_1[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_2[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_3[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_4[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_tail[i]);

        decompose_standard_limb(prover_polynomials.p_x_low_limbs[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_tail[i + 1]);

        // Slice high limbs of P.x into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_x_high_limbs[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_0[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_1[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_2[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_3[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_4[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_tail[i]);

        decompose_standard_top_limb(prover_polynomials.p_x_high_limbs[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_4[i + 1]);

        // Slice low limbs of P.y into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_y_low_limbs[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_0[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_1[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_2[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_3[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_4[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_tail[i]);

        decompose_standard_limb(prover_polynomials.p_y_low_limbs[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_tail[i + 1]);

        // Slice high limbs of P.y into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_y_high_limbs[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_0[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_1[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_2[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_3[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_4[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_tail[i]);

        decompose_standard_top_limb(prover_polynomials.p_y_high_limbs[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_4[i + 1]);

        // Slice low limb of of z1 and z2 into range constraints
        decompose_standard_limb(prover_polynomials.z_low_limbs[i],
                                prover_polynomials.z_low_limbs_range_constraint_0[i],
                                prover_polynomials.z_low_limbs_range_constraint_1[i],
                                prover_polynomials.z_low_limbs_range_constraint_2[i],
                                prover_polynomials.z_low_limbs_range_constraint_3[i],
                                prover_polynomials.z_low_limbs_range_constraint_4[i],
                                prover_polynomials.z_low_limbs_range_constraint_tail[i]);

        decompose_standard_limb(prover_polynomials.z_low_limbs[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_tail[i + 1]);

        // Slice high limb of of z1 and z2 into range constraints
        decompose_standard_top_z_limb(prover_polynomials.z_high_limbs[i],
                                      prover_polynomials.z_high_limbs_range_constraint_0[i],
                                      prover_polynomials.z_high_limbs_range_constraint_1[i],
                                      prover_polynomials.z_high_limbs_range_constraint_2[i],
                                      prover_polynomials.z_high_limbs_range_constraint_3[i],
                                      prover_polynomials.z_high_limbs_range_constraint_4[i],
                                      prover_polynomials.z_high_limbs_range_constraint_tail[i]);

        decompose_standard_top_z_limb(prover_polynomials.z_high_limbs[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_0[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_1[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_2[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_3[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_4[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_tail[i + 1]);

        // Slice accumulator limbs into range constraints
        decompose_standard_limb(prover_polynomials.accumulators_binary_limbs_0[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_0[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_1[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_2[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_3[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_4[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_tail[i]);
        decompose_standard_limb(prover_polynomials.accumulators_binary_limbs_1[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_tail[i + 1]);

        decompose_standard_limb(prover_polynomials.accumulators_binary_limbs_2[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_0[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_1[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_2[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_3[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_4[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_tail[i]);
        decompose_standard_top_limb(prover_polynomials.accumulators_binary_limbs_3[i],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_4[i + 1]);

        // Slice quotient limbs into range constraints
        decompose_standard_limb(prover_polynomials.quotient_low_binary_limbs[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_0[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_1[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_2[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_3[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_4[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_tail[i]);
        decompose_standard_limb(prover_polynomials.quotient_low_binary_limbs_shift[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_tail[i + 1]);

        decompose_standard_limb(prover_polynomials.quotient_high_binary_limbs[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_0[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_1[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_2[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_3[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_4[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_tail[i]);

        decompose_top_quotient_limb(prover_polynomials.quotient_high_binary_limbs_shift[i],
                                    prover_polynomials.quotient_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_4[i + 1]);

        // Decompose wide relation limbs into range constraints
        decompose_relation_limb(prover_polynomials.relation_wide_limbs[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_0[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_1[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_2[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_3[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_tail[i + 1],
                                prover_polynomials.accumulator_high_limbs_range_constraint_tail[i + 1]);

        decompose_relation_limb(prover_polynomials.relation_wide_limbs[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_0[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_1[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_2[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_3[i + 1],
                                prover_polynomials.p_y_high_limbs_range_constraint_tail[i + 1],
                                prover_polynomials.quotient_high_limbs_range_constraint_tail[i + 1]);
    }

    using Relations = Flavor::Relations;
    // Check that Decomposition relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<4, Relations>>(circuit_size, prover_polynomials, params);
}

/**
 * @brief Test the correctness of GoblinTranslatorFlavor's  NonNativeField Relation
 *
 */
TEST_F(GoblinTranslatorRelationCorrectnessTests, NonNative)
{
    using Flavor = GoblinTranslatorFlavor;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using ProverPolynomialIds = typename Flavor::ProverPolynomialIds;
    using GroupElement = typename Flavor::GroupElement;
    using Polynomial = bb::Polynomial<FF>;

    constexpr size_t NUM_LIMB_BITS = Flavor::NUM_LIMB_BITS;
    constexpr auto mini_circuit_size = 2048;
    constexpr auto circuit_size = Flavor::CONCATENATION_GROUP_SIZE * mini_circuit_size;

    auto& engine = numeric::get_debug_randomness();

    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    // Generate random EccOpQueue actions
    for (size_t i = 0; i < ((mini_circuit_size >> 1) - 1); i++) {
        switch (engine.get_random_uint8() & 3) {
        case 0:
            op_queue->empty_row_for_testing();
            break;
        case 1:
            op_queue->eq_and_reset();
            break;
        case 2:
            op_queue->add_accumulate(GroupElement::random_element(&engine));
            break;
        case 3:
            op_queue->mul_accumulate(GroupElement::random_element(&engine), FF::random_element(&engine));
            break;
        }
    }
    const auto batching_challenge_v = BF::random_element(&engine);
    const auto evaluation_input_x = BF::random_element(&engine);

    // Generating all the values is pretty tedious, so just use CircuitBuilder
    auto circuit_builder = GoblinTranslatorCircuitBuilder(batching_challenge_v, evaluation_input_x, op_queue);

    // The non-native field relation uses limbs of evaluation_input_x and powers of batching_challenge_v as inputs
    RelationParameters<FF> params;
    auto v_power = BF::one();
    for (size_t i = 0; i < 4 /*Number of powers of v that we need {1,2,3,4}*/; i++) {
        v_power *= batching_challenge_v;
        auto uint_v_power = uint256_t(v_power);
        params.batching_challenge_v[i] = { uint_v_power.slice(0, NUM_LIMB_BITS),
                                           uint_v_power.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                           uint_v_power.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                           uint_v_power.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                           uint_v_power };
    }
    auto uint_input_x = uint256_t(evaluation_input_x);
    params.evaluation_input_x = { uint_input_x.slice(0, NUM_LIMB_BITS),
                                  uint_input_x.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                  uint_input_x.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                  uint_input_x.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                  uint_input_x };

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    // We use polynomial ids to make shifting the polynomials easier
    ProverPolynomialIds prover_polynomial_ids;
    std::vector<Polynomial> polynomial_container;
    std::vector<size_t> polynomial_ids;
    auto polynomial_get_all = prover_polynomials.get_all();
    auto polynomial_id_get_all = prover_polynomial_ids.get_all();
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        Polynomial temporary_polynomial(circuit_size);
        // Allocate polynomials
        polynomial_container.push_back(temporary_polynomial);
        // Push sequential ids to polynomial ids
        polynomial_ids.push_back(i);
        polynomial_id_get_all[i] = polynomial_ids[i];
    }
    // Get ids of shifted polynomials and put them in a set
    auto shifted_ids = prover_polynomial_ids.get_shifted();
    std::unordered_set<size_t> shifted_id_set;
    for (auto& id : shifted_ids) {
        shifted_id_set.emplace(id);
    }
    // Assign to non-shifted prover polynomials
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        if (!shifted_id_set.contains(i)) {
            polynomial_get_all[i] = polynomial_container[i].share();
        }
    }

    // Assign to shifted prover polynomials using ids
    for (size_t i = 0; i < shifted_ids.size(); i++) {
        auto shifted_id = shifted_ids[i];
        auto to_be_shifted_id = prover_polynomial_ids.get_to_be_shifted()[i];
        polynomial_get_all[shifted_id] = polynomial_container[to_be_shifted_id].shifted();
    }

    // Copy values of wires used in the non-native field relation from the circuit builder
    for (size_t i = 1; i < circuit_builder.get_num_gates(); i++) {
        prover_polynomials.op[i] = circuit_builder.get_variable(circuit_builder.wires[circuit_builder.OP][i]);
        prover_polynomials.p_x_low_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_X_LOW_LIMBS][i]);
        prover_polynomials.p_x_high_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_X_HIGH_LIMBS][i]);
        prover_polynomials.p_y_low_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_Y_LOW_LIMBS][i]);
        prover_polynomials.p_y_high_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_Y_HIGH_LIMBS][i]);
        prover_polynomials.z_low_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.Z_LOW_LIMBS][i]);
        prover_polynomials.z_high_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.Z_HIGH_LIMBS][i]);
        prover_polynomials.accumulators_binary_limbs_0[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_0][i]);
        prover_polynomials.accumulators_binary_limbs_1[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_1][i]);
        prover_polynomials.accumulators_binary_limbs_2[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_2][i]);
        prover_polynomials.accumulators_binary_limbs_3[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_3][i]);
        prover_polynomials.quotient_low_binary_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.QUOTIENT_LOW_BINARY_LIMBS][i]);
        prover_polynomials.quotient_high_binary_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.QUOTIENT_HIGH_BINARY_LIMBS][i]);
        prover_polynomials.relation_wide_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.RELATION_WIDE_LIMBS][i]);
    }

    // Fill in lagrange odd polynomial
    for (size_t i = 1; i < mini_circuit_size - 1; i += 2) {
        prover_polynomials.lagrange_odd_in_minicircuit[i] = 1;
    }

    using Relations = Flavor::Relations;
    // Check that Non-Native Field relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<5, Relations>>(circuit_size, prover_polynomials, params);
}
