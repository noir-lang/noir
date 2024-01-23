/**
 * @file goblin_translator_relation_consistency.test.cpp
 * @brief Show that relation arithmetic has a simple form.
 * @details The purpose of this test suite is to show that the identity arithmetic implemented in the Relations is
 * equivalent to a simpler unoptimized version implemented in the tests themselves. This is useful 1) as documentation
 * since the simple implementations here should make the underlying arithmetic easier to see, and 2) as a check that
 * optimizations introduced into the Relations have not changed the result.
 *
 * For this purpose, we simply feed (the same) random inputs into each of the two implementations and confirm that
 * the outputs match. This does not confirm the correctness of the identity arithmetic (the identities will not be
 * satisfied in general by random inputs) only that the two implementations are equivalent.
 *
 */
#include "barretenberg/flavor/goblin_translator.hpp"
#include <gtest/gtest.h>

using namespace bb;

using Flavor = honk::flavor::GoblinTranslator;
using FF = typename Flavor::FF;
using InputElements = typename Flavor::AllValues;

InputElements get_random_input()
{
    InputElements result;
    for (FF& element : result.get_all()) {
        element = FF::random_element();
    }
    return result;
}

InputElements get_special_input() // use non-random values
{
    InputElements result;
    FF idx = 0;
    for (FF& element : result.get_all()) {
        idx += FF(1);
        element = idx;
    }
    return result;
}

class GoblinTranslatorRelationConsistency : public testing::Test {
  public:
    template <typename Relation>
    static void validate_relation_execution(const auto& expected_values,
                                            const InputElements& input_elements,
                                            const auto& parameters)
    {
        typename Relation::SumcheckArrayOfValuesOverSubrelations accumulator;
        std::fill(accumulator.begin(), accumulator.end(), FF(0));
        Relation::accumulate(accumulator, input_elements, parameters, 1);
        EXPECT_EQ(accumulator, expected_values);
    };
};

TEST_F(GoblinTranslatorRelationConsistency, PermutationRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GoblinTranslatorPermutationRelation<FF>;
        using RelationValues = typename Relation::SumcheckArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? get_random_input() : get_special_input();
        const auto& concatenated_range_constraints_0 = input_elements.concatenated_range_constraints_0;
        const auto& concatenated_range_constraints_1 = input_elements.concatenated_range_constraints_1;
        const auto& concatenated_range_constraints_2 = input_elements.concatenated_range_constraints_2;
        const auto& concatenated_range_constraints_3 = input_elements.concatenated_range_constraints_3;
        const auto& ordered_range_constraints_0 = input_elements.ordered_range_constraints_0;
        const auto& ordered_range_constraints_1 = input_elements.ordered_range_constraints_1;
        const auto& ordered_range_constraints_2 = input_elements.ordered_range_constraints_2;
        const auto& ordered_range_constraints_3 = input_elements.ordered_range_constraints_3;
        const auto& ordered_range_constraints_4 = input_elements.ordered_range_constraints_4;
        const auto& ordered_extra_range_constraints_numerator =
            input_elements.ordered_extra_range_constraints_numerator;
        const auto& z_perm = input_elements.z_perm;
        const auto& z_perm_shift = input_elements.z_perm_shift;
        const auto& lagrange_first = input_elements.lagrange_first;
        const auto& lagrange_last = input_elements.lagrange_last;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();
        const auto& gamma = parameters.gamma;

        // (Contribution 1)
        auto contribution_1 =
            (z_perm + lagrange_first) * (concatenated_range_constraints_0 + gamma) *
                (concatenated_range_constraints_1 + gamma) * (concatenated_range_constraints_2 + gamma) *
                (concatenated_range_constraints_3 + gamma) * (ordered_extra_range_constraints_numerator + gamma) -
            (z_perm_shift + lagrange_last) * (ordered_range_constraints_0 + gamma) *
                (ordered_range_constraints_1 + gamma) * (ordered_range_constraints_2 + gamma) *
                (ordered_range_constraints_3 + gamma) * (ordered_range_constraints_4 + gamma);
        expected_values[0] = contribution_1;

        // (Contribution 2)
        auto contribution_2 = z_perm_shift * lagrange_last;
        expected_values[1] = contribution_2;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(GoblinTranslatorRelationConsistency, GenPermSortRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GoblinTranslatorGenPermSortRelation<FF>;
        using RelationValues = typename Relation::SumcheckArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? get_random_input() : get_special_input();

        const auto& ordered_range_constraints_0 = input_elements.ordered_range_constraints_0;
        const auto& ordered_range_constraints_1 = input_elements.ordered_range_constraints_1;
        const auto& ordered_range_constraints_2 = input_elements.ordered_range_constraints_2;
        const auto& ordered_range_constraints_3 = input_elements.ordered_range_constraints_3;
        const auto& ordered_range_constraints_4 = input_elements.ordered_range_constraints_4;
        const auto& ordered_range_constraints_0_shift = input_elements.ordered_range_constraints_0_shift;
        const auto& ordered_range_constraints_1_shift = input_elements.ordered_range_constraints_1_shift;
        const auto& ordered_range_constraints_2_shift = input_elements.ordered_range_constraints_2_shift;
        const auto& ordered_range_constraints_3_shift = input_elements.ordered_range_constraints_3_shift;
        const auto& ordered_range_constraints_4_shift = input_elements.ordered_range_constraints_4_shift;
        const auto& lagrange_last = input_elements.lagrange_last;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();

        const size_t MICRO_LIMB_BITS = 14;
        const auto minus_one = FF(-1);
        const auto minus_two = FF(-2);
        const auto minus_three = FF(-3);
        const auto maximum_value = -FF((1 << MICRO_LIMB_BITS) - 1);

        // First compute individual deltas
        const auto delta_1 = ordered_range_constraints_0_shift - ordered_range_constraints_0;
        const auto delta_2 = ordered_range_constraints_1_shift - ordered_range_constraints_1;
        const auto delta_3 = ordered_range_constraints_2_shift - ordered_range_constraints_2;
        const auto delta_4 = ordered_range_constraints_3_shift - ordered_range_constraints_3;
        const auto delta_5 = ordered_range_constraints_4_shift - ordered_range_constraints_4;

        const auto not_last = lagrange_last + minus_one;

        // Check the delta is {0,1,2,3}
        auto delta_in_range = [not_last, minus_one, minus_two, minus_three](auto delta) {
            return not_last * delta * (delta + minus_one) * (delta + minus_two) * (delta + minus_three);
        };

        // Check delta correctness
        expected_values[0] = delta_in_range(delta_1);
        expected_values[1] = delta_in_range(delta_2);
        expected_values[2] = delta_in_range(delta_3);
        expected_values[3] = delta_in_range(delta_4);
        expected_values[4] = delta_in_range(delta_5);
        // Check that the last value is maximum allowed
        expected_values[5] = lagrange_last * (ordered_range_constraints_0 + maximum_value);
        expected_values[6] = lagrange_last * (ordered_range_constraints_1 + maximum_value);
        expected_values[7] = lagrange_last * (ordered_range_constraints_2 + maximum_value);
        expected_values[8] = lagrange_last * (ordered_range_constraints_3 + maximum_value);
        expected_values[9] = lagrange_last * (ordered_range_constraints_4 + maximum_value);
        // We don't check that the first value is zero, because the shift mechanism already ensures it

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(GoblinTranslatorRelationConsistency, DecompositionRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GoblinTranslatorDecompositionRelation<FF>;
        using RelationValues = typename Relation::SumcheckArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? get_random_input() : get_special_input();

        // Get all the wires
        const auto& p_x_low_limbs_range_constraint_0 = input_elements.p_x_low_limbs_range_constraint_0;
        const auto& p_x_low_limbs_range_constraint_1 = input_elements.p_x_low_limbs_range_constraint_1;
        const auto& p_x_low_limbs_range_constraint_2 = input_elements.p_x_low_limbs_range_constraint_2;
        const auto& p_x_low_limbs_range_constraint_3 = input_elements.p_x_low_limbs_range_constraint_3;
        const auto& p_x_low_limbs_range_constraint_4 = input_elements.p_x_low_limbs_range_constraint_4;
        const auto& p_x_low_limbs_range_constraint_tail = input_elements.p_x_low_limbs_range_constraint_tail;
        const auto& p_x_low_limbs = input_elements.p_x_low_limbs;
        const auto& p_x_high_limbs_range_constraint_0 = input_elements.p_x_high_limbs_range_constraint_0;
        const auto& p_x_high_limbs_range_constraint_1 = input_elements.p_x_high_limbs_range_constraint_1;
        const auto& p_x_high_limbs_range_constraint_2 = input_elements.p_x_high_limbs_range_constraint_2;
        const auto& p_x_high_limbs_range_constraint_3 = input_elements.p_x_high_limbs_range_constraint_3;
        const auto& p_x_high_limbs_range_constraint_4 = input_elements.p_x_high_limbs_range_constraint_4;
        const auto& p_x_high_limbs_range_constraint_tail = input_elements.p_x_high_limbs_range_constraint_tail;
        const auto& p_x_high_limbs = input_elements.p_x_high_limbs;
        const auto& p_x_low_limbs_range_constraint_0_shift = input_elements.p_x_low_limbs_range_constraint_0_shift;
        const auto& p_x_low_limbs_range_constraint_1_shift = input_elements.p_x_low_limbs_range_constraint_1_shift;
        const auto& p_x_low_limbs_range_constraint_2_shift = input_elements.p_x_low_limbs_range_constraint_2_shift;
        const auto& p_x_low_limbs_range_constraint_3_shift = input_elements.p_x_low_limbs_range_constraint_3_shift;
        const auto& p_x_low_limbs_range_constraint_4_shift = input_elements.p_x_low_limbs_range_constraint_4_shift;
        const auto& p_x_low_limbs_range_constraint_tail_shift =
            input_elements.p_x_low_limbs_range_constraint_tail_shift;
        const auto& p_x_low_limbs_shift = input_elements.p_x_low_limbs_shift;
        const auto& p_x_high_limbs_range_constraint_0_shift = input_elements.p_x_high_limbs_range_constraint_0_shift;
        const auto& p_x_high_limbs_range_constraint_1_shift = input_elements.p_x_high_limbs_range_constraint_1_shift;
        const auto& p_x_high_limbs_range_constraint_2_shift = input_elements.p_x_high_limbs_range_constraint_2_shift;
        const auto& p_x_high_limbs_range_constraint_3_shift = input_elements.p_x_high_limbs_range_constraint_3_shift;
        const auto& p_x_high_limbs_range_constraint_4_shift = input_elements.p_x_high_limbs_range_constraint_4_shift;
        const auto& p_x_high_limbs_range_constraint_tail_shift =
            input_elements.p_x_high_limbs_range_constraint_tail_shift;
        const auto& p_x_high_limbs_shift = input_elements.p_x_high_limbs_shift;
        const auto& p_y_low_limbs_range_constraint_0 = input_elements.p_y_low_limbs_range_constraint_0;
        const auto& p_y_low_limbs_range_constraint_1 = input_elements.p_y_low_limbs_range_constraint_1;
        const auto& p_y_low_limbs_range_constraint_2 = input_elements.p_y_low_limbs_range_constraint_2;
        const auto& p_y_low_limbs_range_constraint_3 = input_elements.p_y_low_limbs_range_constraint_3;
        const auto& p_y_low_limbs_range_constraint_4 = input_elements.p_y_low_limbs_range_constraint_4;
        const auto& p_y_low_limbs_range_constraint_tail = input_elements.p_y_low_limbs_range_constraint_tail;
        const auto& p_y_low_limbs = input_elements.p_y_low_limbs;
        const auto& p_y_high_limbs_range_constraint_0 = input_elements.p_y_high_limbs_range_constraint_0;
        const auto& p_y_high_limbs_range_constraint_1 = input_elements.p_y_high_limbs_range_constraint_1;
        const auto& p_y_high_limbs_range_constraint_2 = input_elements.p_y_high_limbs_range_constraint_2;
        const auto& p_y_high_limbs_range_constraint_3 = input_elements.p_y_high_limbs_range_constraint_3;
        const auto& p_y_high_limbs_range_constraint_4 = input_elements.p_y_high_limbs_range_constraint_4;
        const auto& p_y_high_limbs_range_constraint_tail = input_elements.p_y_high_limbs_range_constraint_tail;
        const auto& p_y_high_limbs = input_elements.p_y_high_limbs;
        const auto& p_y_low_limbs_range_constraint_0_shift = input_elements.p_y_low_limbs_range_constraint_0_shift;
        const auto& p_y_low_limbs_range_constraint_1_shift = input_elements.p_y_low_limbs_range_constraint_1_shift;
        const auto& p_y_low_limbs_range_constraint_2_shift = input_elements.p_y_low_limbs_range_constraint_2_shift;
        const auto& p_y_low_limbs_range_constraint_3_shift = input_elements.p_y_low_limbs_range_constraint_3_shift;
        const auto& p_y_low_limbs_range_constraint_4_shift = input_elements.p_y_low_limbs_range_constraint_4_shift;
        const auto& p_y_low_limbs_range_constraint_tail_shift =
            input_elements.p_y_low_limbs_range_constraint_tail_shift;
        const auto& p_y_low_limbs_shift = input_elements.p_y_low_limbs_shift;
        const auto& p_y_high_limbs_range_constraint_0_shift = input_elements.p_y_high_limbs_range_constraint_0_shift;
        const auto& p_y_high_limbs_range_constraint_1_shift = input_elements.p_y_high_limbs_range_constraint_1_shift;
        const auto& p_y_high_limbs_range_constraint_2_shift = input_elements.p_y_high_limbs_range_constraint_2_shift;
        const auto& p_y_high_limbs_range_constraint_3_shift = input_elements.p_y_high_limbs_range_constraint_3_shift;
        const auto& p_y_high_limbs_range_constraint_4_shift = input_elements.p_y_high_limbs_range_constraint_4_shift;
        const auto& p_y_high_limbs_range_constraint_tail_shift =
            input_elements.p_y_high_limbs_range_constraint_tail_shift;
        const auto& p_y_high_limbs_shift = input_elements.p_y_high_limbs_shift;
        const auto& z_low_limbs_range_constraint_0 = input_elements.z_low_limbs_range_constraint_0;
        const auto& z_low_limbs_range_constraint_1 = input_elements.z_low_limbs_range_constraint_1;
        const auto& z_low_limbs_range_constraint_2 = input_elements.z_low_limbs_range_constraint_2;
        const auto& z_low_limbs_range_constraint_3 = input_elements.z_low_limbs_range_constraint_3;
        const auto& z_low_limbs_range_constraint_4 = input_elements.z_low_limbs_range_constraint_4;
        const auto& z_low_limbs_range_constraint_tail = input_elements.z_low_limbs_range_constraint_tail;
        const auto& z_low_limbs = input_elements.z_low_limbs;
        const auto& z_low_limbs_range_constraint_0_shift = input_elements.z_low_limbs_range_constraint_0_shift;
        const auto& z_low_limbs_range_constraint_1_shift = input_elements.z_low_limbs_range_constraint_1_shift;
        const auto& z_low_limbs_range_constraint_2_shift = input_elements.z_low_limbs_range_constraint_2_shift;
        const auto& z_low_limbs_range_constraint_3_shift = input_elements.z_low_limbs_range_constraint_3_shift;
        const auto& z_low_limbs_range_constraint_4_shift = input_elements.z_low_limbs_range_constraint_4_shift;
        const auto& z_low_limbs_range_constraint_tail_shift = input_elements.z_low_limbs_range_constraint_tail_shift;
        const auto& z_low_limbs_shift = input_elements.z_low_limbs_shift;
        const auto& z_high_limbs_range_constraint_0 = input_elements.z_high_limbs_range_constraint_0;
        const auto& z_high_limbs_range_constraint_1 = input_elements.z_high_limbs_range_constraint_1;
        const auto& z_high_limbs_range_constraint_2 = input_elements.z_high_limbs_range_constraint_2;
        const auto& z_high_limbs_range_constraint_3 = input_elements.z_high_limbs_range_constraint_3;
        const auto& z_high_limbs_range_constraint_4 = input_elements.z_high_limbs_range_constraint_4;
        const auto& z_high_limbs_range_constraint_tail = input_elements.z_high_limbs_range_constraint_tail;
        const auto& z_high_limbs = input_elements.z_high_limbs;
        const auto& z_high_limbs_range_constraint_0_shift = input_elements.z_high_limbs_range_constraint_0_shift;
        const auto& z_high_limbs_range_constraint_1_shift = input_elements.z_high_limbs_range_constraint_1_shift;
        const auto& z_high_limbs_range_constraint_2_shift = input_elements.z_high_limbs_range_constraint_2_shift;
        const auto& z_high_limbs_range_constraint_3_shift = input_elements.z_high_limbs_range_constraint_3_shift;
        const auto& z_high_limbs_range_constraint_4_shift = input_elements.z_high_limbs_range_constraint_4_shift;
        const auto& z_high_limbs_range_constraint_tail_shift = input_elements.z_high_limbs_range_constraint_tail_shift;
        const auto& z_high_limbs_shift = input_elements.z_high_limbs_shift;
        const auto& accumulator_low_limbs_range_constraint_0 = input_elements.accumulator_low_limbs_range_constraint_0;
        const auto& accumulator_low_limbs_range_constraint_1 = input_elements.accumulator_low_limbs_range_constraint_1;
        const auto& accumulator_low_limbs_range_constraint_2 = input_elements.accumulator_low_limbs_range_constraint_2;
        const auto& accumulator_low_limbs_range_constraint_3 = input_elements.accumulator_low_limbs_range_constraint_3;
        const auto& accumulator_low_limbs_range_constraint_4 = input_elements.accumulator_low_limbs_range_constraint_4;
        const auto& accumulator_low_limbs_range_constraint_tail =
            input_elements.accumulator_low_limbs_range_constraint_tail;
        const auto& accumulator_low_limbs_range_constraint_0_shift =
            input_elements.accumulator_low_limbs_range_constraint_0_shift;
        const auto& accumulator_low_limbs_range_constraint_1_shift =
            input_elements.accumulator_low_limbs_range_constraint_1_shift;
        const auto& accumulator_low_limbs_range_constraint_2_shift =
            input_elements.accumulator_low_limbs_range_constraint_2_shift;
        const auto& accumulator_low_limbs_range_constraint_3_shift =
            input_elements.accumulator_low_limbs_range_constraint_3_shift;
        const auto& accumulator_low_limbs_range_constraint_4_shift =
            input_elements.accumulator_low_limbs_range_constraint_4_shift;
        const auto& accumulator_low_limbs_range_constraint_tail_shift =
            input_elements.accumulator_low_limbs_range_constraint_tail_shift;
        const auto& accumulator_high_limbs_range_constraint_0 =
            input_elements.accumulator_high_limbs_range_constraint_0;
        const auto& accumulator_high_limbs_range_constraint_1 =
            input_elements.accumulator_high_limbs_range_constraint_1;
        const auto& accumulator_high_limbs_range_constraint_2 =
            input_elements.accumulator_high_limbs_range_constraint_2;
        const auto& accumulator_high_limbs_range_constraint_3 =
            input_elements.accumulator_high_limbs_range_constraint_3;
        const auto& accumulator_high_limbs_range_constraint_4 =
            input_elements.accumulator_high_limbs_range_constraint_4;
        const auto& accumulator_high_limbs_range_constraint_tail =
            input_elements.accumulator_high_limbs_range_constraint_tail;
        const auto& accumulator_high_limbs_range_constraint_0_shift =
            input_elements.accumulator_high_limbs_range_constraint_0_shift;
        const auto& accumulator_high_limbs_range_constraint_1_shift =
            input_elements.accumulator_high_limbs_range_constraint_1_shift;
        const auto& accumulator_high_limbs_range_constraint_2_shift =
            input_elements.accumulator_high_limbs_range_constraint_2_shift;
        const auto& accumulator_high_limbs_range_constraint_3_shift =
            input_elements.accumulator_high_limbs_range_constraint_3_shift;
        const auto& accumulator_high_limbs_range_constraint_4_shift =
            input_elements.accumulator_high_limbs_range_constraint_4_shift;
        const auto& accumulator_high_limbs_range_constraint_tail_shift =
            input_elements.accumulator_high_limbs_range_constraint_tail_shift;
        const auto& accumulators_binary_limbs_0 = input_elements.accumulators_binary_limbs_0;
        const auto& accumulators_binary_limbs_1 = input_elements.accumulators_binary_limbs_1;
        const auto& accumulators_binary_limbs_2 = input_elements.accumulators_binary_limbs_2;
        const auto& accumulators_binary_limbs_3 = input_elements.accumulators_binary_limbs_3;
        const auto& quotient_low_limbs_range_constraint_0 = input_elements.quotient_low_limbs_range_constraint_0;
        const auto& quotient_low_limbs_range_constraint_1 = input_elements.quotient_low_limbs_range_constraint_1;
        const auto& quotient_low_limbs_range_constraint_2 = input_elements.quotient_low_limbs_range_constraint_2;
        const auto& quotient_low_limbs_range_constraint_3 = input_elements.quotient_low_limbs_range_constraint_3;
        const auto& quotient_low_limbs_range_constraint_4 = input_elements.quotient_low_limbs_range_constraint_4;
        const auto& quotient_low_limbs_range_constraint_tail = input_elements.quotient_low_limbs_range_constraint_tail;
        const auto& quotient_low_limbs_range_constraint_0_shift =
            input_elements.quotient_low_limbs_range_constraint_0_shift;
        const auto& quotient_low_limbs_range_constraint_1_shift =
            input_elements.quotient_low_limbs_range_constraint_1_shift;
        const auto& quotient_low_limbs_range_constraint_2_shift =
            input_elements.quotient_low_limbs_range_constraint_2_shift;
        const auto& quotient_low_limbs_range_constraint_3_shift =
            input_elements.quotient_low_limbs_range_constraint_3_shift;
        const auto& quotient_low_limbs_range_constraint_4_shift =
            input_elements.quotient_low_limbs_range_constraint_4_shift;
        const auto& quotient_low_limbs_range_constraint_tail_shift =
            input_elements.quotient_low_limbs_range_constraint_tail_shift;
        const auto& quotient_high_limbs_range_constraint_0 = input_elements.quotient_high_limbs_range_constraint_0;
        const auto& quotient_high_limbs_range_constraint_1 = input_elements.quotient_high_limbs_range_constraint_1;
        const auto& quotient_high_limbs_range_constraint_2 = input_elements.quotient_high_limbs_range_constraint_2;
        const auto& quotient_high_limbs_range_constraint_3 = input_elements.quotient_high_limbs_range_constraint_3;
        const auto& quotient_high_limbs_range_constraint_4 = input_elements.quotient_high_limbs_range_constraint_4;
        const auto& quotient_high_limbs_range_constraint_tail =
            input_elements.quotient_high_limbs_range_constraint_tail;
        const auto& quotient_high_limbs_range_constraint_0_shift =
            input_elements.quotient_high_limbs_range_constraint_0_shift;
        const auto& quotient_high_limbs_range_constraint_1_shift =
            input_elements.quotient_high_limbs_range_constraint_1_shift;
        const auto& quotient_high_limbs_range_constraint_2_shift =
            input_elements.quotient_high_limbs_range_constraint_2_shift;
        const auto& quotient_high_limbs_range_constraint_3_shift =
            input_elements.quotient_high_limbs_range_constraint_3_shift;
        const auto& quotient_high_limbs_range_constraint_4_shift =
            input_elements.quotient_high_limbs_range_constraint_4_shift;
        const auto& quotient_high_limbs_range_constraint_tail_shift =
            input_elements.quotient_high_limbs_range_constraint_tail_shift;
        const auto& quotient_low_binary_limbs = input_elements.quotient_low_binary_limbs;
        const auto& quotient_low_binary_limbs_shift = input_elements.quotient_low_binary_limbs_shift;
        const auto& quotient_high_binary_limbs = input_elements.quotient_high_binary_limbs;
        const auto& quotient_high_binary_limbs_shift = input_elements.quotient_high_binary_limbs_shift;
        const auto& relation_wide_limbs_range_constraint_0 = input_elements.relation_wide_limbs_range_constraint_0;
        const auto& relation_wide_limbs_range_constraint_1 = input_elements.relation_wide_limbs_range_constraint_1;
        const auto& relation_wide_limbs_range_constraint_2 = input_elements.relation_wide_limbs_range_constraint_2;
        const auto& relation_wide_limbs_range_constraint_3 = input_elements.relation_wide_limbs_range_constraint_3;
        const auto& relation_wide_limbs_range_constraint_0_shift =
            input_elements.relation_wide_limbs_range_constraint_0_shift;
        const auto& relation_wide_limbs_range_constraint_1_shift =
            input_elements.relation_wide_limbs_range_constraint_1_shift;
        const auto& relation_wide_limbs_range_constraint_2_shift =
            input_elements.relation_wide_limbs_range_constraint_2_shift;
        const auto& relation_wide_limbs_range_constraint_3_shift =
            input_elements.relation_wide_limbs_range_constraint_3_shift;
        const auto& relation_wide_limbs = input_elements.relation_wide_limbs;
        const auto& relation_wide_limbs_shift = input_elements.relation_wide_limbs_shift;

        const auto& x_lo_y_hi = input_elements.x_lo_y_hi;
        const auto& x_hi_z_1 = input_elements.x_hi_z_1;
        const auto& y_lo_z_2 = input_elements.y_lo_z_2;
        const auto& x_lo_y_hi_shift = input_elements.x_lo_y_hi_shift;
        const auto& x_hi_z_1_shift = input_elements.x_hi_z_1_shift;
        const auto& y_lo_z_2_shift = input_elements.y_lo_z_2_shift;

        const auto& lagrange_odd_in_minicircuit = input_elements.lagrange_odd_in_minicircuit;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();

        const size_t NUM_MICRO_LIMB_BITS = 14;
        const size_t NUM_LIMB_BITS = 68;
        const auto MICRO_LIMB_SHIFT = FF(uint256_t(1) << NUM_MICRO_LIMB_BITS);
        const auto MICRO_LIMB_SHIFTx2 = MICRO_LIMB_SHIFT * MICRO_LIMB_SHIFT;
        const auto MICRO_LIMB_SHIFTx3 = MICRO_LIMB_SHIFTx2 * MICRO_LIMB_SHIFT;
        const auto MICRO_LIMB_SHIFTx4 = MICRO_LIMB_SHIFTx3 * MICRO_LIMB_SHIFT;
        const auto MICRO_LIMB_SHIFTx5 = MICRO_LIMB_SHIFTx4 * MICRO_LIMB_SHIFT;

        const auto SHIFT_10_TO_14 = FF(1 << 4);
        const auto SHIFT_12_TO_14 = FF(1 << 2);
        const auto SHIFT_4_TO_14 = FF(1 << 10);
        const auto SHIFT_8_TO_14 = FF(1 << 6);
        const auto LIMB_SHIFT = FF(uint256_t(1) << NUM_LIMB_BITS);

        // All decomposition happen only at odd indices, so we use lagrange odd
        /**
         * @brief Check decomposition of a relation limb. Relation limbs are 84 bits, so the decompositon takes 6
         * 14-bit microlimbs
         *
         */
        auto check_relation_limb_decomposition = [MICRO_LIMB_SHIFT,
                                                  MICRO_LIMB_SHIFTx2,
                                                  MICRO_LIMB_SHIFTx3,
                                                  MICRO_LIMB_SHIFTx4,
                                                  MICRO_LIMB_SHIFTx5,
                                                  lagrange_odd_in_minicircuit](auto& micro_limb_0,
                                                                               auto& micro_limb_1,
                                                                               auto& micro_limb_2,
                                                                               auto& micro_limb_3,
                                                                               auto& micro_limb_4,
                                                                               auto& micro_limb_5,
                                                                               auto& decomposed_limb) {
            return (micro_limb_0 + micro_limb_1 * MICRO_LIMB_SHIFT + micro_limb_2 * MICRO_LIMB_SHIFTx2 +
                    micro_limb_3 * MICRO_LIMB_SHIFTx3 + micro_limb_4 * MICRO_LIMB_SHIFTx4 +
                    micro_limb_5 * MICRO_LIMB_SHIFTx5 - decomposed_limb) *
                   lagrange_odd_in_minicircuit;
        };

        /**
         * @brief Check the decomposition of a standard limb. Standard limbs are 68 bits, so we decompose them into
         * 5 14-bit microlimbs
         *
         */
        auto check_standard_limb_decomposition =
            [MICRO_LIMB_SHIFT, MICRO_LIMB_SHIFTx2, MICRO_LIMB_SHIFTx3, MICRO_LIMB_SHIFTx4, lagrange_odd_in_minicircuit](
                auto& micro_limb_0,
                auto& micro_limb_1,
                auto& micro_limb_2,
                auto& micro_limb_3,
                auto& micro_limb_4,
                auto& decomposed_limb) {
                return (micro_limb_0 + micro_limb_1 * MICRO_LIMB_SHIFT + micro_limb_2 * MICRO_LIMB_SHIFTx2 +
                        micro_limb_3 * MICRO_LIMB_SHIFTx3 + micro_limb_4 * MICRO_LIMB_SHIFTx4 - decomposed_limb) *
                       lagrange_odd_in_minicircuit;
            };

        /**
         * @brief Check the decomposition of a standard top limb. Standard top limb is 50 bits (254 = 68 * 3 + 50)
         *
         */
        auto check_standard_top_limb_decomposition =
            [MICRO_LIMB_SHIFT, MICRO_LIMB_SHIFTx2, MICRO_LIMB_SHIFTx3, lagrange_odd_in_minicircuit](
                auto& micro_limb_0, auto& micro_limb_1, auto& micro_limb_2, auto& micro_limb_3, auto& decomposed_limb) {
                return (micro_limb_0 + micro_limb_1 * MICRO_LIMB_SHIFT + micro_limb_2 * MICRO_LIMB_SHIFTx2 +
                        micro_limb_3 * MICRO_LIMB_SHIFTx3 - decomposed_limb) *
                       lagrange_odd_in_minicircuit;
            };

        /**
         * @brief Ensure that the last microlimb of a standard limb decomposition is 12 bits by checking a shifted
         * version.
         *
         */
        auto check_standard_tail_micro_limb_correctness =
            [SHIFT_12_TO_14, lagrange_odd_in_minicircuit](auto& nonshifted_micro_limb, auto shifted_micro_limb) {
                return (nonshifted_micro_limb * SHIFT_12_TO_14 - shifted_micro_limb) * lagrange_odd_in_minicircuit;
            };

        /**
         * @brief Ensure that the last microlimb of a standard top limb decomposition is 8 bits by checking a
         * shifted version.
         *
         */
        auto check_top_tail_micro_limb_correctness =
            [SHIFT_8_TO_14, lagrange_odd_in_minicircuit](auto& nonshifted_micro_limb, auto shifted_micro_limb) {
                return (nonshifted_micro_limb * SHIFT_8_TO_14 - shifted_micro_limb) * lagrange_odd_in_minicircuit;
            };

        /**
         * @brief Ensure that the last microlimb of z top limb decomposition is 4 bits by checking a shifted
         * version.
         *
         */
        auto check_z_top_tail_micro_limb_correctness =
            [SHIFT_4_TO_14, lagrange_odd_in_minicircuit](auto& nonshifted_micro_limb, auto shifted_micro_limb) {
                return (nonshifted_micro_limb * SHIFT_4_TO_14 - shifted_micro_limb) * lagrange_odd_in_minicircuit;
            };

        /**
         * @brief Ensure that the last microlimb of quotient top limb decomposition is 10 bits by checking a shifted
         * version.
         *
         */
        auto check_quotient_top_tail_micro_limb_correctness =
            [SHIFT_10_TO_14, lagrange_odd_in_minicircuit](auto& nonshifted_micro_limb, auto shifted_micro_limb) {
                return (nonshifted_micro_limb * SHIFT_10_TO_14 - shifted_micro_limb) * lagrange_odd_in_minicircuit;
            };

        /**
         * @brief Check decomposition of wide 128-bit limbs into two 68-bit limbs.
         *
         */
        auto check_wide_limb_into_regular_limb_correctness =
            [LIMB_SHIFT, lagrange_odd_in_minicircuit](auto& low_limb, auto& high_limb, auto& wide_limb) {
                return (low_limb + high_limb * LIMB_SHIFT - wide_limb) * lagrange_odd_in_minicircuit;
            };

        // Check decomposition 50-72 bit limbs into microlimbs
        expected_values[0] = check_standard_limb_decomposition(p_x_low_limbs_range_constraint_0,
                                                               p_x_low_limbs_range_constraint_1,
                                                               p_x_low_limbs_range_constraint_2,
                                                               p_x_low_limbs_range_constraint_3,
                                                               p_x_low_limbs_range_constraint_4,
                                                               p_x_low_limbs);
        expected_values[1] = check_standard_limb_decomposition(p_x_low_limbs_range_constraint_0_shift,
                                                               p_x_low_limbs_range_constraint_1_shift,
                                                               p_x_low_limbs_range_constraint_2_shift,
                                                               p_x_low_limbs_range_constraint_3_shift,
                                                               p_x_low_limbs_range_constraint_4_shift,
                                                               p_x_low_limbs_shift);
        expected_values[2] = check_standard_limb_decomposition(p_x_high_limbs_range_constraint_0,
                                                               p_x_high_limbs_range_constraint_1,
                                                               p_x_high_limbs_range_constraint_2,
                                                               p_x_high_limbs_range_constraint_3,
                                                               p_x_high_limbs_range_constraint_4,
                                                               p_x_high_limbs);
        expected_values[3] = check_standard_top_limb_decomposition(p_x_high_limbs_range_constraint_0_shift,
                                                                   p_x_high_limbs_range_constraint_1_shift,
                                                                   p_x_high_limbs_range_constraint_2_shift,
                                                                   p_x_high_limbs_range_constraint_3_shift,
                                                                   p_x_high_limbs_shift);

        expected_values[4] = check_standard_limb_decomposition(p_y_low_limbs_range_constraint_0,
                                                               p_y_low_limbs_range_constraint_1,
                                                               p_y_low_limbs_range_constraint_2,
                                                               p_y_low_limbs_range_constraint_3,
                                                               p_y_low_limbs_range_constraint_4,
                                                               p_y_low_limbs);
        expected_values[5] = check_standard_limb_decomposition(p_y_low_limbs_range_constraint_0_shift,
                                                               p_y_low_limbs_range_constraint_1_shift,
                                                               p_y_low_limbs_range_constraint_2_shift,
                                                               p_y_low_limbs_range_constraint_3_shift,
                                                               p_y_low_limbs_range_constraint_4_shift,
                                                               p_y_low_limbs_shift);
        expected_values[6] = check_standard_limb_decomposition(p_y_high_limbs_range_constraint_0,
                                                               p_y_high_limbs_range_constraint_1,
                                                               p_y_high_limbs_range_constraint_2,
                                                               p_y_high_limbs_range_constraint_3,
                                                               p_y_high_limbs_range_constraint_4,
                                                               p_y_high_limbs);
        expected_values[7] = check_standard_top_limb_decomposition(p_y_high_limbs_range_constraint_0_shift,
                                                                   p_y_high_limbs_range_constraint_1_shift,
                                                                   p_y_high_limbs_range_constraint_2_shift,
                                                                   p_y_high_limbs_range_constraint_3_shift,
                                                                   p_y_high_limbs_shift);
        expected_values[8] = check_standard_limb_decomposition(z_low_limbs_range_constraint_0,
                                                               z_low_limbs_range_constraint_1,
                                                               z_low_limbs_range_constraint_2,
                                                               z_low_limbs_range_constraint_3,
                                                               z_low_limbs_range_constraint_4,
                                                               z_low_limbs);
        expected_values[9] = check_standard_limb_decomposition(z_low_limbs_range_constraint_0_shift,
                                                               z_low_limbs_range_constraint_1_shift,
                                                               z_low_limbs_range_constraint_2_shift,
                                                               z_low_limbs_range_constraint_3_shift,
                                                               z_low_limbs_range_constraint_4_shift,
                                                               z_low_limbs_shift);
        expected_values[10] = check_standard_limb_decomposition(z_high_limbs_range_constraint_0,
                                                                z_high_limbs_range_constraint_1,
                                                                z_high_limbs_range_constraint_2,
                                                                z_high_limbs_range_constraint_3,
                                                                z_high_limbs_range_constraint_4,
                                                                z_high_limbs);
        expected_values[11] = check_standard_limb_decomposition(z_high_limbs_range_constraint_0_shift,
                                                                z_high_limbs_range_constraint_1_shift,
                                                                z_high_limbs_range_constraint_2_shift,
                                                                z_high_limbs_range_constraint_3_shift,
                                                                z_high_limbs_range_constraint_4_shift,
                                                                z_high_limbs_shift);
        expected_values[12] = check_standard_limb_decomposition(accumulator_low_limbs_range_constraint_0,
                                                                accumulator_low_limbs_range_constraint_1,
                                                                accumulator_low_limbs_range_constraint_2,
                                                                accumulator_low_limbs_range_constraint_3,
                                                                accumulator_low_limbs_range_constraint_4,
                                                                accumulators_binary_limbs_0);
        expected_values[13] = check_standard_limb_decomposition(accumulator_low_limbs_range_constraint_0_shift,
                                                                accumulator_low_limbs_range_constraint_1_shift,
                                                                accumulator_low_limbs_range_constraint_2_shift,
                                                                accumulator_low_limbs_range_constraint_3_shift,
                                                                accumulator_low_limbs_range_constraint_4_shift,
                                                                accumulators_binary_limbs_1);
        expected_values[14] = check_standard_limb_decomposition(accumulator_high_limbs_range_constraint_0,
                                                                accumulator_high_limbs_range_constraint_1,
                                                                accumulator_high_limbs_range_constraint_2,
                                                                accumulator_high_limbs_range_constraint_3,
                                                                accumulator_high_limbs_range_constraint_4,
                                                                accumulators_binary_limbs_2);
        expected_values[15] = check_standard_top_limb_decomposition(accumulator_high_limbs_range_constraint_0_shift,
                                                                    accumulator_high_limbs_range_constraint_1_shift,
                                                                    accumulator_high_limbs_range_constraint_2_shift,
                                                                    accumulator_high_limbs_range_constraint_3_shift,
                                                                    accumulators_binary_limbs_3);
        expected_values[16] = check_standard_limb_decomposition(quotient_low_limbs_range_constraint_0,
                                                                quotient_low_limbs_range_constraint_1,
                                                                quotient_low_limbs_range_constraint_2,
                                                                quotient_low_limbs_range_constraint_3,
                                                                quotient_low_limbs_range_constraint_4,
                                                                quotient_low_binary_limbs);
        expected_values[17] = check_standard_limb_decomposition(quotient_low_limbs_range_constraint_0_shift,
                                                                quotient_low_limbs_range_constraint_1_shift,
                                                                quotient_low_limbs_range_constraint_2_shift,
                                                                quotient_low_limbs_range_constraint_3_shift,
                                                                quotient_low_limbs_range_constraint_4_shift,
                                                                quotient_low_binary_limbs_shift);
        expected_values[18] = check_standard_limb_decomposition(quotient_high_limbs_range_constraint_0,
                                                                quotient_high_limbs_range_constraint_1,
                                                                quotient_high_limbs_range_constraint_2,
                                                                quotient_high_limbs_range_constraint_3,
                                                                quotient_high_limbs_range_constraint_4,
                                                                quotient_high_binary_limbs);
        expected_values[19] = check_standard_top_limb_decomposition(quotient_high_limbs_range_constraint_0_shift,
                                                                    quotient_high_limbs_range_constraint_1_shift,
                                                                    quotient_high_limbs_range_constraint_2_shift,
                                                                    quotient_high_limbs_range_constraint_3_shift,
                                                                    quotient_high_binary_limbs_shift);

        expected_values[20] = check_relation_limb_decomposition(relation_wide_limbs_range_constraint_0,
                                                                relation_wide_limbs_range_constraint_1,
                                                                relation_wide_limbs_range_constraint_2,
                                                                relation_wide_limbs_range_constraint_3,
                                                                p_x_high_limbs_range_constraint_tail_shift,
                                                                accumulator_high_limbs_range_constraint_tail_shift,
                                                                relation_wide_limbs);
        expected_values[21] = check_relation_limb_decomposition(relation_wide_limbs_range_constraint_0_shift,
                                                                relation_wide_limbs_range_constraint_1_shift,
                                                                relation_wide_limbs_range_constraint_2_shift,
                                                                relation_wide_limbs_range_constraint_3_shift,
                                                                p_y_high_limbs_range_constraint_tail_shift,
                                                                quotient_high_limbs_range_constraint_tail_shift,
                                                                relation_wide_limbs_shift);

        // Contributions enforcing tail range constraints (range constraints less than 14 bits)
        expected_values[22] = check_standard_tail_micro_limb_correctness(p_x_low_limbs_range_constraint_4,
                                                                         p_x_low_limbs_range_constraint_tail);

        expected_values[23] = check_standard_tail_micro_limb_correctness(p_x_low_limbs_range_constraint_4_shift,
                                                                         p_x_low_limbs_range_constraint_tail_shift);

        expected_values[24] = check_standard_tail_micro_limb_correctness(p_x_high_limbs_range_constraint_4,
                                                                         p_x_high_limbs_range_constraint_tail);

        expected_values[25] = check_top_tail_micro_limb_correctness(p_x_high_limbs_range_constraint_3_shift,
                                                                    p_x_high_limbs_range_constraint_4_shift);

        expected_values[26] = check_standard_tail_micro_limb_correctness(p_y_low_limbs_range_constraint_4,
                                                                         p_y_low_limbs_range_constraint_tail);

        expected_values[27] = check_standard_tail_micro_limb_correctness(p_y_low_limbs_range_constraint_4_shift,
                                                                         p_y_low_limbs_range_constraint_tail_shift);

        expected_values[28] = check_standard_tail_micro_limb_correctness(p_y_high_limbs_range_constraint_4,
                                                                         p_y_high_limbs_range_constraint_tail);

        expected_values[29] = check_top_tail_micro_limb_correctness(p_y_high_limbs_range_constraint_3_shift,
                                                                    p_y_high_limbs_range_constraint_4_shift);

        expected_values[30] = check_standard_tail_micro_limb_correctness(z_low_limbs_range_constraint_4,
                                                                         z_low_limbs_range_constraint_tail);

        expected_values[31] = check_standard_tail_micro_limb_correctness(z_low_limbs_range_constraint_4_shift,
                                                                         z_low_limbs_range_constraint_tail_shift);

        expected_values[32] = check_z_top_tail_micro_limb_correctness(z_high_limbs_range_constraint_4,
                                                                      z_high_limbs_range_constraint_tail);

        expected_values[33] = check_z_top_tail_micro_limb_correctness(z_high_limbs_range_constraint_4_shift,
                                                                      z_high_limbs_range_constraint_tail_shift);

        expected_values[34] = check_standard_tail_micro_limb_correctness(accumulator_low_limbs_range_constraint_4,
                                                                         accumulator_low_limbs_range_constraint_tail);
        expected_values[35] = check_standard_tail_micro_limb_correctness(
            accumulator_low_limbs_range_constraint_4_shift, accumulator_low_limbs_range_constraint_tail_shift);

        expected_values[36] = check_standard_tail_micro_limb_correctness(accumulator_high_limbs_range_constraint_4,
                                                                         accumulator_high_limbs_range_constraint_tail);

        expected_values[37] = check_top_tail_micro_limb_correctness(accumulator_high_limbs_range_constraint_3_shift,
                                                                    accumulator_high_limbs_range_constraint_4_shift);

        expected_values[38] = check_standard_tail_micro_limb_correctness(quotient_low_limbs_range_constraint_4,
                                                                         quotient_low_limbs_range_constraint_tail);

        expected_values[39] = check_standard_tail_micro_limb_correctness(
            quotient_low_limbs_range_constraint_4_shift, quotient_low_limbs_range_constraint_tail_shift);

        expected_values[40] = check_standard_tail_micro_limb_correctness(quotient_high_limbs_range_constraint_4,
                                                                         quotient_high_limbs_range_constraint_tail);

        expected_values[41] = check_quotient_top_tail_micro_limb_correctness(
            quotient_high_limbs_range_constraint_3_shift, quotient_high_limbs_range_constraint_4_shift);

        // Constraints for decomposition of EccOpQueue values

        expected_values[42] =
            check_wide_limb_into_regular_limb_correctness(p_x_low_limbs, p_x_low_limbs_shift, x_lo_y_hi);

        expected_values[43] =
            check_wide_limb_into_regular_limb_correctness(p_x_high_limbs, p_x_high_limbs_shift, x_hi_z_1);

        expected_values[44] =
            check_wide_limb_into_regular_limb_correctness(p_y_low_limbs, p_y_low_limbs_shift, y_lo_z_2);

        expected_values[45] =
            check_wide_limb_into_regular_limb_correctness(p_y_high_limbs, p_y_high_limbs_shift, x_lo_y_hi_shift);

        expected_values[46] = check_wide_limb_into_regular_limb_correctness(z_low_limbs, z_high_limbs, x_hi_z_1_shift);

        expected_values[47] =
            check_wide_limb_into_regular_limb_correctness(z_low_limbs_shift, z_high_limbs_shift, y_lo_z_2_shift);

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(GoblinTranslatorRelationConsistency, OpcodeConstraintRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GoblinTranslatorOpcodeConstraintRelation<FF>;
        using RelationValues = typename Relation::SumcheckArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? get_random_input() : get_special_input();
        const auto& op = input_elements.op;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();

        // (Contribution 1)
        auto contribution_1 = op * (op - FF(1)) * (op - FF(2)) * (op - FF(3)) * (op - FF(4)) * (op - FF(8));
        expected_values[0] = contribution_1;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(GoblinTranslatorRelationConsistency, AccumulatorTransferRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GoblinTranslatorAccumulatorTransferRelation<FF>;
        using RelationValues = typename Relation::SumcheckArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? get_random_input() : get_special_input();

        const auto& lagrange_even_in_minicircuit = input_elements.lagrange_even_in_minicircuit;
        const auto& lagrange_second = input_elements.lagrange_second;
        const auto& lagrange_second_to_last_in_minicircuit = input_elements.lagrange_second_to_last_in_minicircuit;
        const auto& accumulators_binary_limbs_0 = input_elements.accumulators_binary_limbs_0;
        const auto& accumulators_binary_limbs_0_shift = input_elements.accumulators_binary_limbs_0_shift;
        const auto& accumulators_binary_limbs_1 = input_elements.accumulators_binary_limbs_1;
        const auto& accumulators_binary_limbs_1_shift = input_elements.accumulators_binary_limbs_1_shift;
        const auto& accumulators_binary_limbs_2 = input_elements.accumulators_binary_limbs_2;
        const auto& accumulators_binary_limbs_2_shift = input_elements.accumulators_binary_limbs_2_shift;
        const auto& accumulators_binary_limbs_3 = input_elements.accumulators_binary_limbs_3;
        const auto& accumulators_binary_limbs_3_shift = input_elements.accumulators_binary_limbs_3_shift;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();

        const auto [accumulated_result_0, accumulated_result_1, accumulated_result_2, accumulated_result_3] =
            parameters.accumulated_result;

        // Check transfer of accumulator at even indices
        expected_values[0] =
            lagrange_even_in_minicircuit * (accumulators_binary_limbs_0 - accumulators_binary_limbs_0_shift);
        expected_values[1] =
            lagrange_even_in_minicircuit * (accumulators_binary_limbs_1 - accumulators_binary_limbs_1_shift);
        expected_values[2] =
            lagrange_even_in_minicircuit * (accumulators_binary_limbs_2 - accumulators_binary_limbs_2_shift);
        expected_values[3] =
            lagrange_even_in_minicircuit * (accumulators_binary_limbs_3 - accumulators_binary_limbs_3_shift);

        // Check the accumulator starts as zero
        expected_values[4] = accumulators_binary_limbs_0 * lagrange_second_to_last_in_minicircuit;
        expected_values[5] = accumulators_binary_limbs_1 * lagrange_second_to_last_in_minicircuit;
        expected_values[6] = accumulators_binary_limbs_2 * lagrange_second_to_last_in_minicircuit;
        expected_values[7] = accumulators_binary_limbs_3 * lagrange_second_to_last_in_minicircuit;

        // Check the accumulator results in submitted value
        expected_values[8] = (accumulators_binary_limbs_0 - accumulated_result_0) * lagrange_second;
        expected_values[9] = (accumulators_binary_limbs_1 - accumulated_result_1) * lagrange_second;
        expected_values[10] = (accumulators_binary_limbs_2 - accumulated_result_2) * lagrange_second;
        expected_values[11] = (accumulators_binary_limbs_3 - accumulated_result_3) * lagrange_second;
        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(GoblinTranslatorRelationConsistency, NonNativeFieldRelation)
{
    const auto run_test = [](bool random_inputs) {
        constexpr size_t NUM_LIMB_BITS = 68;
        constexpr FF shift = FF(uint256_t(1) << NUM_LIMB_BITS);
        constexpr FF shiftx2 = FF(uint256_t(1) << (NUM_LIMB_BITS * 2));
        constexpr FF shiftx3 = FF(uint256_t(1) << (NUM_LIMB_BITS * 3));
        constexpr uint512_t MODULUS_U512 = uint512_t(curve::BN254::BaseField::modulus);
        constexpr uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (NUM_LIMB_BITS << 2);
        constexpr uint512_t NEGATIVE_PRIME_MODULUS = BINARY_BASIS_MODULUS - MODULUS_U512;
        constexpr std::array<FF, 5> NEGATIVE_MODULUS_LIMBS = {
            FF(NEGATIVE_PRIME_MODULUS.slice(0, NUM_LIMB_BITS).lo),
            FF(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo),
            FF(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo),
            FF(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo),
            -FF(curve::BN254::BaseField::modulus)
        };

        using Relation = GoblinTranslatorNonNativeFieldRelation<FF>;
        using RelationValues = typename Relation::SumcheckArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? get_random_input() : get_special_input();

        auto& op = input_elements.op;
        auto& p_x_low_limbs = input_elements.p_x_low_limbs;
        auto& p_y_low_limbs = input_elements.p_y_low_limbs;
        auto& p_x_high_limbs = input_elements.p_x_high_limbs;
        auto& p_y_high_limbs = input_elements.p_y_high_limbs;
        auto& accumulators_binary_limbs_0 = input_elements.accumulators_binary_limbs_0;
        auto& accumulators_binary_limbs_1 = input_elements.accumulators_binary_limbs_1;
        auto& accumulators_binary_limbs_2 = input_elements.accumulators_binary_limbs_2;
        auto& accumulators_binary_limbs_3 = input_elements.accumulators_binary_limbs_3;
        auto& z_low_limbs = input_elements.z_low_limbs;
        auto& z_high_limbs = input_elements.z_high_limbs;
        auto& quotient_low_binary_limbs = input_elements.quotient_low_binary_limbs;
        auto& quotient_high_binary_limbs = input_elements.quotient_high_binary_limbs;
        auto& p_x_low_limbs_shift = input_elements.p_x_low_limbs_shift;
        auto& p_y_low_limbs_shift = input_elements.p_y_low_limbs_shift;
        auto& p_x_high_limbs_shift = input_elements.p_x_high_limbs_shift;
        auto& p_y_high_limbs_shift = input_elements.p_y_high_limbs_shift;
        auto& accumulators_binary_limbs_0_shift = input_elements.accumulators_binary_limbs_0_shift;
        auto& accumulators_binary_limbs_1_shift = input_elements.accumulators_binary_limbs_1_shift;
        auto& accumulators_binary_limbs_2_shift = input_elements.accumulators_binary_limbs_2_shift;
        auto& accumulators_binary_limbs_3_shift = input_elements.accumulators_binary_limbs_3_shift;
        auto& z_low_limbs_shift = input_elements.z_low_limbs_shift;
        auto& z_high_limbs_shift = input_elements.z_high_limbs_shift;
        auto& quotient_low_binary_limbs_shift = input_elements.quotient_low_binary_limbs_shift;
        auto& quotient_high_binary_limbs_shift = input_elements.quotient_high_binary_limbs_shift;
        auto& relation_wide_limbs = input_elements.relation_wide_limbs;
        auto& relation_wide_limbs_shift = input_elements.relation_wide_limbs_shift;
        auto& lagrange_odd_in_minicircuit = input_elements.lagrange_odd_in_minicircuit;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();

        // A detailed description of these subrelations is located in the relation's documentation

        // Lower wide limb (lower 136 bits) subrelation
        expected_values[0] =
            (accumulators_binary_limbs_0_shift * parameters.evaluation_input_x[0] + op +
             p_x_low_limbs * parameters.batching_challenge_v[0][0] +
             p_y_low_limbs * parameters.batching_challenge_v[1][0] +
             z_low_limbs * parameters.batching_challenge_v[2][0] +
             z_low_limbs_shift * parameters.batching_challenge_v[3][0] +
             quotient_low_binary_limbs * NEGATIVE_MODULUS_LIMBS[0] - accumulators_binary_limbs_0 +
             (accumulators_binary_limbs_1_shift * parameters.evaluation_input_x[0] +
              accumulators_binary_limbs_0_shift * parameters.evaluation_input_x[1] +
              p_x_low_limbs * parameters.batching_challenge_v[0][1] +
              p_x_low_limbs_shift * parameters.batching_challenge_v[0][0] +
              p_y_low_limbs * parameters.batching_challenge_v[1][1] +
              p_y_low_limbs_shift * parameters.batching_challenge_v[1][0] +
              z_low_limbs * parameters.batching_challenge_v[2][1] +
              z_high_limbs * parameters.batching_challenge_v[2][0] +
              z_low_limbs_shift * parameters.batching_challenge_v[3][1] +
              z_high_limbs_shift * parameters.batching_challenge_v[3][0] +
              quotient_low_binary_limbs * NEGATIVE_MODULUS_LIMBS[1] +
              quotient_low_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[0] - accumulators_binary_limbs_1) *
                 shift -
             relation_wide_limbs * shiftx2) *
            lagrange_odd_in_minicircuit;

        // Higher wide limb subrelation
        expected_values[1] =
            (relation_wide_limbs + accumulators_binary_limbs_2_shift * parameters.evaluation_input_x[0] +
             accumulators_binary_limbs_1_shift * parameters.evaluation_input_x[1] +
             accumulators_binary_limbs_0_shift * parameters.evaluation_input_x[2] +
             p_x_high_limbs * parameters.batching_challenge_v[0][0] +
             p_x_low_limbs_shift * parameters.batching_challenge_v[0][1] +
             p_x_low_limbs * parameters.batching_challenge_v[0][2] +
             p_y_high_limbs * parameters.batching_challenge_v[1][0] +
             p_y_low_limbs_shift * parameters.batching_challenge_v[1][1] +
             p_y_low_limbs * parameters.batching_challenge_v[1][2] +
             z_high_limbs * parameters.batching_challenge_v[2][1] +
             z_low_limbs * parameters.batching_challenge_v[2][2] +
             z_high_limbs_shift * parameters.batching_challenge_v[3][1] +
             z_low_limbs_shift * parameters.batching_challenge_v[3][2] +
             quotient_high_binary_limbs * NEGATIVE_MODULUS_LIMBS[0] +
             quotient_low_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[1] +
             quotient_low_binary_limbs * NEGATIVE_MODULUS_LIMBS[2] - accumulators_binary_limbs_2 +
             (accumulators_binary_limbs_3_shift * parameters.evaluation_input_x[0] +
              accumulators_binary_limbs_2_shift * parameters.evaluation_input_x[1] +
              accumulators_binary_limbs_1_shift * parameters.evaluation_input_x[2] +
              accumulators_binary_limbs_0_shift * parameters.evaluation_input_x[3] +
              p_x_high_limbs_shift * parameters.batching_challenge_v[0][0] +
              p_x_high_limbs * parameters.batching_challenge_v[0][1] +
              p_x_low_limbs_shift * parameters.batching_challenge_v[0][2] +
              p_x_low_limbs * parameters.batching_challenge_v[0][3] +
              p_y_high_limbs_shift * parameters.batching_challenge_v[1][0] +
              p_y_high_limbs * parameters.batching_challenge_v[1][1] +
              p_y_low_limbs_shift * parameters.batching_challenge_v[1][2] +
              p_y_low_limbs * parameters.batching_challenge_v[1][3] +
              z_high_limbs * parameters.batching_challenge_v[2][2] +
              z_low_limbs * parameters.batching_challenge_v[2][3] +
              z_high_limbs_shift * parameters.batching_challenge_v[3][2] +
              z_low_limbs_shift * parameters.batching_challenge_v[3][3] +
              quotient_high_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[0] +
              quotient_high_binary_limbs * NEGATIVE_MODULUS_LIMBS[1] +
              quotient_low_binary_limbs_shift * NEGATIVE_MODULUS_LIMBS[2] +
              quotient_low_binary_limbs * NEGATIVE_MODULUS_LIMBS[3] - accumulators_binary_limbs_3) *
                 shift -
             relation_wide_limbs_shift * shiftx2) *
            lagrange_odd_in_minicircuit;
        auto reconstructed_p_x =
            (p_x_low_limbs + p_x_low_limbs_shift * shift + p_x_high_limbs * shiftx2 + p_x_high_limbs_shift * shiftx3);
        auto reconstructed_p_y =
            (p_y_low_limbs + p_y_low_limbs_shift * shift + p_y_high_limbs * shiftx2 + p_y_high_limbs_shift * shiftx3);
        auto reconstructed_previous_accumulator =
            (accumulators_binary_limbs_0_shift + accumulators_binary_limbs_1_shift * shift +
             accumulators_binary_limbs_2_shift * shiftx2 + accumulators_binary_limbs_3_shift * shiftx3);
        auto reconstructed_current_accumulator =
            (accumulators_binary_limbs_0 + accumulators_binary_limbs_1 * shift + accumulators_binary_limbs_2 * shiftx2 +
             accumulators_binary_limbs_3 * shiftx3);
        auto reconstructed_z1 = (z_low_limbs + z_high_limbs * shift);
        auto reconstructed_z2 = (z_low_limbs_shift + z_high_limbs_shift * shift);
        auto reconstructed_quotient =
            (quotient_low_binary_limbs + quotient_low_binary_limbs_shift * shift +
             quotient_high_binary_limbs * shiftx2 + quotient_high_binary_limbs_shift * shiftx3);

        // Native field relation
        expected_values[2] = (reconstructed_previous_accumulator * parameters.evaluation_input_x[4] + op +
                              reconstructed_p_x * parameters.batching_challenge_v[0][4] +
                              reconstructed_p_y * parameters.batching_challenge_v[1][4] +
                              reconstructed_z1 * parameters.batching_challenge_v[2][4] +
                              reconstructed_z2 * parameters.batching_challenge_v[3][4] +
                              reconstructed_quotient * NEGATIVE_MODULUS_LIMBS[4] - reconstructed_current_accumulator) *
                             lagrange_odd_in_minicircuit;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};
