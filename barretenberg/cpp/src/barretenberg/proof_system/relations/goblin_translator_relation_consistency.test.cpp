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
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include <gtest/gtest.h>

using namespace proof_system;

namespace proof_system::ultra_relation_consistency_tests {

using FF = barretenberg::fr;
struct InputElements {
    static constexpr size_t NUM_ELEMENTS = 184;
    std::array<FF, NUM_ELEMENTS> _data;

    static InputElements get_random()
    {
        InputElements result;
        std::generate(result._data.begin(), result._data.end(), [] { return FF::random_element(); });
        return result;
    }

    static InputElements get_special() // use non-random values
    {
        InputElements result;
        FF idx = 0;
        std::generate(result._data.begin(), result._data.end(), [&] {
            idx += FF(1);
            return idx;
        });
        return result;
    }
    FF& op = std::get<0>(this->_data);
    FF& x_lo_y_hi = std::get<1>(this->_data);
    FF& x_hi_z_1 = std::get<2>(this->_data);
    FF& y_lo_z_2 = std::get<3>(this->_data);
    FF& p_x_low_limbs = std::get<4>(this->_data);
    FF& p_x_low_limbs_range_constraint_0 = std::get<5>(this->_data);
    FF& p_x_low_limbs_range_constraint_1 = std::get<6>(this->_data);
    FF& p_x_low_limbs_range_constraint_2 = std::get<7>(this->_data);
    FF& p_x_low_limbs_range_constraint_3 = std::get<8>(this->_data);
    FF& p_x_low_limbs_range_constraint_4 = std::get<9>(this->_data);
    FF& p_x_low_limbs_range_constraint_tail = std::get<10>(this->_data);
    FF& p_x_high_limbs = std::get<11>(this->_data);
    FF& p_x_high_limbs_range_constraint_0 = std::get<12>(this->_data);
    FF& p_x_high_limbs_range_constraint_1 = std::get<13>(this->_data);
    FF& p_x_high_limbs_range_constraint_2 = std::get<14>(this->_data);
    FF& p_x_high_limbs_range_constraint_3 = std::get<15>(this->_data);
    FF& p_x_high_limbs_range_constraint_4 = std::get<16>(this->_data);
    FF& p_x_high_limbs_range_constraint_tail = std::get<17>(this->_data);
    FF& p_y_low_limbs = std::get<18>(this->_data);
    FF& p_y_low_limbs_range_constraint_0 = std::get<19>(this->_data);
    FF& p_y_low_limbs_range_constraint_1 = std::get<20>(this->_data);
    FF& p_y_low_limbs_range_constraint_2 = std::get<21>(this->_data);
    FF& p_y_low_limbs_range_constraint_3 = std::get<22>(this->_data);
    FF& p_y_low_limbs_range_constraint_4 = std::get<23>(this->_data);
    FF& p_y_low_limbs_range_constraint_tail = std::get<24>(this->_data);
    FF& p_y_high_limbs = std::get<25>(this->_data);
    FF& p_y_high_limbs_range_constraint_0 = std::get<26>(this->_data);
    FF& p_y_high_limbs_range_constraint_1 = std::get<27>(this->_data);
    FF& p_y_high_limbs_range_constraint_2 = std::get<28>(this->_data);
    FF& p_y_high_limbs_range_constraint_3 = std::get<29>(this->_data);
    FF& p_y_high_limbs_range_constraint_4 = std::get<30>(this->_data);
    FF& p_y_high_limbs_range_constraint_tail = std::get<31>(this->_data);
    FF& z_lo_limbs = std::get<32>(this->_data);
    FF& z_lo_limbs_range_constraint_0 = std::get<33>(this->_data);
    FF& z_lo_limbs_range_constraint_1 = std::get<34>(this->_data);
    FF& z_lo_limbs_range_constraint_2 = std::get<35>(this->_data);
    FF& z_lo_limbs_range_constraint_3 = std::get<36>(this->_data);
    FF& z_lo_limbs_range_constraint_4 = std::get<37>(this->_data);
    FF& z_lo_limbs_range_constraint_tail = std::get<38>(this->_data);
    FF& z_hi_limbs = std::get<39>(this->_data);
    FF& z_hi_limbs_range_constraint_0 = std::get<40>(this->_data);
    FF& z_hi_limbs_range_constraint_1 = std::get<41>(this->_data);
    FF& z_hi_limbs_range_constraint_2 = std::get<42>(this->_data);
    FF& z_hi_limbs_range_constraint_3 = std::get<43>(this->_data);
    FF& z_hi_limbs_range_constraint_4 = std::get<44>(this->_data);
    FF& z_hi_limbs_range_constraint_tail = std::get<45>(this->_data);
    FF& accumulators_binary_limbs_0 = std::get<46>(this->_data);
    FF& accumulators_binary_limbs_1 = std::get<47>(this->_data);
    FF& accumulators_binary_limbs_2 = std::get<48>(this->_data);
    FF& accumulators_binary_limbs_3 = std::get<49>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_0 = std::get<50>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_1 = std::get<51>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_2 = std::get<52>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_3 = std::get<53>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_4 = std::get<54>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_tail = std::get<55>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_0 = std::get<56>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_1 = std::get<57>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_2 = std::get<58>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_3 = std::get<59>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_4 = std::get<60>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_tail = std::get<61>(this->_data);
    FF& quotient_lo_binary_limbs = std::get<62>(this->_data);
    FF& quotient_hi_binary_limbs = std::get<63>(this->_data);
    FF& quotient_lo_limbs_range_constraint_0 = std::get<64>(this->_data);
    FF& quotient_lo_limbs_range_constraint_1 = std::get<65>(this->_data);
    FF& quotient_lo_limbs_range_constraint_2 = std::get<66>(this->_data);
    FF& quotient_lo_limbs_range_constraint_3 = std::get<67>(this->_data);
    FF& quotient_lo_limbs_range_constraint_4 = std::get<68>(this->_data);
    FF& quotient_lo_limbs_range_constraint_tail = std::get<69>(this->_data);
    FF& quotient_hi_limbs_range_constraint_0 = std::get<70>(this->_data);
    FF& quotient_hi_limbs_range_constraint_1 = std::get<71>(this->_data);
    FF& quotient_hi_limbs_range_constraint_2 = std::get<72>(this->_data);
    FF& quotient_hi_limbs_range_constraint_3 = std::get<73>(this->_data);
    FF& quotient_hi_limbs_range_constraint_4 = std::get<74>(this->_data);
    FF& quotient_hi_limbs_range_constraint_tail = std::get<75>(this->_data);
    FF& relation_wide_limbs = std::get<76>(this->_data);
    FF& relation_wide_limbs_range_constraint_0 = std::get<77>(this->_data);
    FF& relation_wide_limbs_range_constraint_1 = std::get<78>(this->_data);
    FF& relation_wide_limbs_range_constraint_2 = std::get<79>(this->_data);
    FF& relation_wide_limbs_range_constraint_3 = std::get<80>(this->_data);
    FF& concatenated_range_constraints_0 = std::get<81>(this->_data);
    FF& concatenated_range_constraints_1 = std::get<82>(this->_data);
    FF& concatenated_range_constraints_2 = std::get<83>(this->_data);
    FF& concatenated_range_constraints_3 = std::get<84>(this->_data);
    FF& ordered_range_constraints_0 = std::get<85>(this->_data);
    FF& ordered_range_constraints_1 = std::get<86>(this->_data);
    FF& ordered_range_constraints_2 = std::get<87>(this->_data);
    FF& ordered_range_constraints_3 = std::get<88>(this->_data);
    FF& ordered_range_constraints_4 = std::get<89>(this->_data);
    FF& z_perm = std::get<90>(this->_data);
    FF& x_lo_y_hi_shift = std::get<91>(this->_data);
    FF& x_hi_z_1_shift = std::get<92>(this->_data);
    FF& y_lo_z_2_shift = std::get<93>(this->_data);
    FF& p_x_low_limbs_shift = std::get<94>(this->_data);
    FF& p_x_low_limbs_range_constraint_0_shift = std::get<95>(this->_data);
    FF& p_x_low_limbs_range_constraint_1_shift = std::get<96>(this->_data);
    FF& p_x_low_limbs_range_constraint_2_shift = std::get<97>(this->_data);
    FF& p_x_low_limbs_range_constraint_3_shift = std::get<98>(this->_data);
    FF& p_x_low_limbs_range_constraint_4_shift = std::get<99>(this->_data);
    FF& p_x_low_limbs_range_constraint_tail_shift = std::get<100>(this->_data);
    FF& p_x_high_limbs_shift = std::get<101>(this->_data);
    FF& p_x_high_limbs_range_constraint_0_shift = std::get<102>(this->_data);
    FF& p_x_high_limbs_range_constraint_1_shift = std::get<103>(this->_data);
    FF& p_x_high_limbs_range_constraint_2_shift = std::get<104>(this->_data);
    FF& p_x_high_limbs_range_constraint_3_shift = std::get<105>(this->_data);
    FF& p_x_high_limbs_range_constraint_4_shift = std::get<106>(this->_data);
    FF& p_x_high_limbs_range_constraint_tail_shift = std::get<107>(this->_data);
    FF& p_y_low_limbs_shift = std::get<108>(this->_data);
    FF& p_y_low_limbs_range_constraint_0_shift = std::get<109>(this->_data);
    FF& p_y_low_limbs_range_constraint_1_shift = std::get<110>(this->_data);
    FF& p_y_low_limbs_range_constraint_2_shift = std::get<111>(this->_data);
    FF& p_y_low_limbs_range_constraint_3_shift = std::get<112>(this->_data);
    FF& p_y_low_limbs_range_constraint_4_shift = std::get<113>(this->_data);
    FF& p_y_low_limbs_range_constraint_tail_shift = std::get<114>(this->_data);
    FF& p_y_high_limbs_shift = std::get<115>(this->_data);
    FF& p_y_high_limbs_range_constraint_0_shift = std::get<116>(this->_data);
    FF& p_y_high_limbs_range_constraint_1_shift = std::get<117>(this->_data);
    FF& p_y_high_limbs_range_constraint_2_shift = std::get<118>(this->_data);
    FF& p_y_high_limbs_range_constraint_3_shift = std::get<119>(this->_data);
    FF& p_y_high_limbs_range_constraint_4_shift = std::get<120>(this->_data);
    FF& p_y_high_limbs_range_constraint_tail_shift = std::get<121>(this->_data);
    FF& z_lo_limbs_shift = std::get<122>(this->_data);
    FF& z_lo_limbs_range_constraint_0_shift = std::get<123>(this->_data);
    FF& z_lo_limbs_range_constraint_1_shift = std::get<124>(this->_data);
    FF& z_lo_limbs_range_constraint_2_shift = std::get<125>(this->_data);
    FF& z_lo_limbs_range_constraint_3_shift = std::get<126>(this->_data);
    FF& z_lo_limbs_range_constraint_4_shift = std::get<127>(this->_data);
    FF& z_lo_limbs_range_constraint_tail_shift = std::get<128>(this->_data);
    FF& z_hi_limbs_shift = std::get<129>(this->_data);
    FF& z_hi_limbs_range_constraint_0_shift = std::get<130>(this->_data);
    FF& z_hi_limbs_range_constraint_1_shift = std::get<131>(this->_data);
    FF& z_hi_limbs_range_constraint_2_shift = std::get<132>(this->_data);
    FF& z_hi_limbs_range_constraint_3_shift = std::get<133>(this->_data);
    FF& z_hi_limbs_range_constraint_4_shift = std::get<134>(this->_data);
    FF& z_hi_limbs_range_constraint_tail_shift = std::get<135>(this->_data);
    FF& accumulators_binary_limbs_0_shift = std::get<136>(this->_data);
    FF& accumulators_binary_limbs_1_shift = std::get<137>(this->_data);
    FF& accumulators_binary_limbs_2_shift = std::get<138>(this->_data);
    FF& accumulators_binary_limbs_3_shift = std::get<139>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_0_shift = std::get<140>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_1_shift = std::get<141>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_2_shift = std::get<142>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_3_shift = std::get<143>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_4_shift = std::get<144>(this->_data);
    FF& accumulator_lo_limbs_range_constraint_tail_shift = std::get<145>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_0_shift = std::get<146>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_1_shift = std::get<147>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_2_shift = std::get<148>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_3_shift = std::get<149>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_4_shift = std::get<150>(this->_data);
    FF& accumulator_hi_limbs_range_constraint_tail_shift = std::get<151>(this->_data);
    FF& quotient_lo_binary_limbs_shift = std::get<152>(this->_data);
    FF& quotient_hi_binary_limbs_shift = std::get<153>(this->_data);
    FF& quotient_lo_limbs_range_constraint_0_shift = std::get<154>(this->_data);
    FF& quotient_lo_limbs_range_constraint_1_shift = std::get<155>(this->_data);
    FF& quotient_lo_limbs_range_constraint_2_shift = std::get<156>(this->_data);
    FF& quotient_lo_limbs_range_constraint_3_shift = std::get<157>(this->_data);
    FF& quotient_lo_limbs_range_constraint_4_shift = std::get<158>(this->_data);
    FF& quotient_lo_limbs_range_constraint_tail_shift = std::get<159>(this->_data);
    FF& quotient_hi_limbs_range_constraint_0_shift = std::get<160>(this->_data);
    FF& quotient_hi_limbs_range_constraint_1_shift = std::get<161>(this->_data);
    FF& quotient_hi_limbs_range_constraint_2_shift = std::get<162>(this->_data);
    FF& quotient_hi_limbs_range_constraint_3_shift = std::get<163>(this->_data);
    FF& quotient_hi_limbs_range_constraint_4_shift = std::get<164>(this->_data);
    FF& quotient_hi_limbs_range_constraint_tail_shift = std::get<165>(this->_data);
    FF& relation_wide_limbs_shift = std::get<166>(this->_data);
    FF& relation_wide_limbs_range_constraint_0_shift = std::get<167>(this->_data);
    FF& relation_wide_limbs_range_constraint_1_shift = std::get<168>(this->_data);
    FF& relation_wide_limbs_range_constraint_2_shift = std::get<169>(this->_data);
    FF& relation_wide_limbs_range_constraint_3_shift = std::get<170>(this->_data);
    FF& ordered_range_constraints_0_shift = std::get<171>(this->_data);
    FF& ordered_range_constraints_1_shift = std::get<172>(this->_data);
    FF& ordered_range_constraints_2_shift = std::get<173>(this->_data);
    FF& ordered_range_constraints_3_shift = std::get<174>(this->_data);
    FF& ordered_range_constraints_4_shift = std::get<175>(this->_data);
    FF& z_perm_shift = std::get<176>(this->_data);
    FF& lagrange_first = std::get<177>(this->_data);
    FF& lagrange_last = std::get<178>(this->_data);
    FF& lagrange_odd = std::get<179>(this->_data);
    FF& lagrange_even = std::get<180>(this->_data);
    FF& lagrange_second = std::get<181>(this->_data);
    FF& lagrange_second_to_last_in_minicircuit = std::get<182>(this->_data);
    FF& ordered_extra_range_constraints_numerator = std::get<183>(this->_data);
};

class GoblinTranslatorRelationConsistency : public testing::Test {
  public:
    template <typename Relation>
    static void validate_relation_execution(const auto& expected_values,
                                            const InputElements& input_elements,
                                            const auto& parameters)
    {
        typename Relation::ArrayOfValuesOverSubrelations accumulator;
        std::fill(accumulator.begin(), accumulator.end(), FF(0));
        Relation::accumulate(accumulator, input_elements, parameters, 1);
        EXPECT_EQ(accumulator, expected_values);
    };
};

TEST_F(GoblinTranslatorRelationConsistency, PermutationRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GoblinTranslatorPermutationRelation<FF>;
        using RelationValues = typename Relation::ArrayOfValuesOverSubrelations;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
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

} // namespace proof_system::ultra_relation_consistency_tests
